use crate::{
    AttachedComment, CommentPlacement, Error, comments::CommentCache, spanned, tagged,
    with::yaml_tag,
};
use astral_yaml::{
    Entry, Node, NodeId, NodeKind, Scalar, ScalarStyle, Span, Stream,
    schema::{ScalarType, classify},
};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use std::{borrow::Cow, cell::Cell, collections::HashSet};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Eq, PartialEq)]
enum YamlType {
    Scalar(ScalarType),
    Sequence,
    Mapping,
}

impl YamlType {
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "tag:yaml.org,2002:str" => Some(Self::Scalar(ScalarType::String)),
            "tag:yaml.org,2002:null" => Some(Self::Scalar(ScalarType::Null)),
            "tag:yaml.org,2002:bool" => Some(Self::Scalar(ScalarType::Bool)),
            "tag:yaml.org,2002:int" => Some(Self::Scalar(ScalarType::Int)),
            "tag:yaml.org,2002:float" => Some(Self::Scalar(ScalarType::Float)),
            "tag:yaml.org,2002:seq" => Some(Self::Sequence),
            "tag:yaml.org,2002:map" => Some(Self::Mapping),
            _ => None,
        }
    }

    fn accepts(self, declared: Self) -> bool {
        // Floating-point destinations also accept YAML integers.
        self == declared
            || matches!(
                (self, declared),
                (
                    Self::Scalar(ScalarType::Float),
                    Self::Scalar(ScalarType::Int)
                )
            )
    }
}

fn is_core_tag(tag: &str) -> bool {
    tag == "!" || YamlType::from_tag(tag).is_some()
}

/// Deserializes exactly one YAML document using YAML 1.2 core-schema resolution.
///
/// Plain and quoted scalars without escapes can be borrowed from the input.
/// Aliases are visited with depth and work budgets, including cyclic aliases.
/// Non-core YAML tags require [`crate::Tagged`] or [`crate::with::yaml_tag`].
/// Explicit core tags must be compatible with the destination type. Integer
/// tags also allow floating-point destinations. Strings, characters, scalar
/// bytes, and enum names require string tags, but accept untagged scalar text.
///
/// # Errors
/// Returns a located syntax, type, or resource-limit error.
pub fn from_str<'de, T: Deserialize<'de>>(source: &'de str) -> Result<T> {
    let stream = astral_yaml::parse(source)?;
    from_stream(&stream)
}

/// Deserializes a previously parsed, single-document stream.
///
/// # Errors
/// Returns a type error, a document-count error, or a resource-limit error.
pub fn from_stream<'de, T: Deserialize<'de>>(stream: &Stream<'de>) -> Result<T> {
    if stream.documents().len() != 1 {
        return Err(Error::at("expected exactly one document", stream.span()));
    }

    from_document(stream, 0)
}

/// Deserializes a document by its zero-based index in a parsed stream.
///
/// # Errors
/// Returns an error for an invalid index, a type mismatch, or an exceeded limit.
pub fn from_document<'de, T: Deserialize<'de>>(stream: &Stream<'de>, index: usize) -> Result<T> {
    let doc = stream
        .documents()
        .get(index)
        .ok_or_else(|| Error::at("document index out of bounds", stream.span()))?;

    let context = Context {
        stream,
        budget: Cell::new(1_000_000),
        scalar_budget: Cell::new(64 * 1024 * 1024),
        comments: CommentCache::new(index),
    };

    T::deserialize(Deserializer {
        context: &context,
        id: doc.root,
        depth: 0,
        tag_state: TagState::Unconsumed,
        yaml_tag: false,
    })
    .map_err(|error| error.located(stream.node(doc.root).map_or(doc.span, |node| node.span)))
}

struct Context<'s, 'de> {
    stream: &'s Stream<'de>,
    budget: Cell<usize>,
    scalar_budget: Cell<usize>,
    comments: CommentCache<'de>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TagState {
    Unconsumed,
    // Core tags still determine the payload's type after capture.
    ConsumedCore,
    // Application tags are metadata; resolve the payload without its tag.
    ConsumedApplication,
}

#[derive(Clone, Copy)]
struct Deserializer<'s, 'de> {
    context: &'s Context<'s, 'de>,
    id: NodeId,
    depth: usize,
    tag_state: TagState,
    // Enable local tags as enum names for this value only.
    yaml_tag: bool,
}

impl<'s, 'de> Deserializer<'s, 'de> {
    #[inline]
    fn raw_node(self) -> Result<&'s Node<'de>> {
        let node = self
            .context
            .stream
            .node(self.id)
            .expect("ID belongs to this stream");
        if self.depth >= 64 || self.context.budget.get() == 0 {
            return Err(Error::at(
                "deserialization resource limit exceeded",
                node.span,
            ));
        }

        self.context.budget.set(self.context.budget.get() - 1);
        if let NodeKind::Scalar(scalar) = &node.kind {
            let remaining = self
                .context
                .scalar_budget
                .get()
                .checked_sub(scalar.value.len())
                .ok_or_else(|| Error::at("expanded scalar byte limit exceeded", node.span))?;
            self.context.scalar_budget.set(remaining);
        }

        Ok(node)
    }

    fn node(self) -> Result<&'s Node<'de>> {
        let node = self.raw_node()?;
        if let NodeKind::Alias { target, .. } = node.kind {
            Self {
                id: target,
                depth: self.depth + 1,
                ..self
            }
            .node()
        } else {
            Ok(node)
        }
    }

    fn child(self, id: NodeId) -> Self {
        Self {
            id,
            depth: self.depth + 1,
            tag_state: TagState::Unconsumed,
            yaml_tag: false,
            ..self
        }
    }

    fn spanned<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let span = self.raw_node()?.span;
        let mut access = SpannedAccess {
            de: Self {
                depth: self.depth + 1,
                ..self
            },
            span,
            index: 0,
        };

        let value = visitor
            .visit_seq(&mut access)
            .map_err(|error| error.located(span))?;
        if access.index != 4 {
            return Err(Error::at("spanned value was not fully consumed", span));
        }

        Ok(value)
    }

    fn tagged<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.node()?;
        let tag = node
            .tag
            .as_ref()
            .filter(|_| self.tag_state == TagState::Unconsumed)
            .ok_or_else(|| Error::at("expected an explicit YAML tag", node.span))?;

        visitor.visit_enum(Enumeration {
            de: self,
            variant: &tag.value,
            value: VariantValue::Tagged(self.id),
        })
    }

    fn tagged_mapping<V: Visitor<'de>>(
        self,
        node: &'s Node<'de>,
        variant: &'s str,
        visitor: V,
    ) -> Result<V::Value> {
        // Serde's untagged-enum buffer accepts maps but rejects enum input.
        // Keep this conversion confined to the explicitly opted-in node.
        let mut map = TagMapping {
            enumeration: Enumeration {
                de: self,
                variant,
                value: VariantValue::Tagged(self.id),
            },
            state: TagMappingState::Key,
        };
        let value = visitor
            .visit_map(&mut map)
            .map_err(|error: Error| error.located(node.span))?;
        if map.state != TagMappingState::End {
            return Err(Error::at(
                "tagged mapping was not fully consumed",
                node.span,
            ));
        }

        Ok(value)
    }

    fn comments<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let comments =
            self.context
                .comments
                .get(self.context.stream, self.id, &self.context.budget)?;
        let remaining = self
            .context
            .budget
            .get()
            .checked_sub(comments.len())
            .ok_or_else(|| {
                Error::at(
                    "deserialization resource limit exceeded",
                    self.context.stream.node(self.id).unwrap().span,
                )
            })?;
        self.context.budget.set(remaining);

        let mut ordered: Vec<_> = comments.iter().collect();
        ordered.sort_unstable_by_key(|comment| comment.comment.span.start());
        let records = ordered.into_iter().map(|comment| {
            de::value::SeqAccessDeserializer::new(CommentAccess { comment, index: 0 })
        });

        seed.deserialize(de::value::SeqDeserializer::new(records))
    }

    fn scalar(self, expected: ScalarType) -> Result<(&'s Scalar<'de>, ScalarType)> {
        let node = self.node()?;
        self.reject_tag(node)?;
        let value = self.scalar_value(node)?;
        self.check_core_tag(node, YamlType::Scalar(expected))?;

        Ok(value)
    }

    fn reject_tag(self, node: &Node<'_>) -> Result<()> {
        if self.tag_state == TagState::Unconsumed
            && node
                .tag
                .as_ref()
                .is_some_and(|tag| !is_core_tag(&tag.value))
        {
            return Err(Error::at(
                "non-core YAML tags require Tagged<T> or with::yaml_tag",
                node.span,
            ));
        }

        Ok(())
    }

    fn enum_tag(self, node: &'s Node<'de>) -> Option<&'s str> {
        node.tag
            .as_ref()
            .filter(|_| self.yaml_tag && self.tag_state == TagState::Unconsumed)
            .and_then(|tag| tag.value.strip_prefix('!'))
            .filter(|name| !name.is_empty())
    }

    fn variant_name(self, name: &'s str) -> Result<de::value::StrDeserializer<'s, Error>> {
        let remaining = self
            .context
            .scalar_budget
            .get()
            .checked_sub(name.len())
            .ok_or_else(|| {
                let node = self.context.stream.node(self.id).unwrap();
                Error::at("expanded scalar byte limit exceeded", node.span)
            })?;
        self.context.scalar_budget.set(remaining);

        Ok(de::value::StrDeserializer::new(name))
    }

    #[inline]
    fn scalar_value(self, node: &'s Node<'de>) -> Result<(&'s Scalar<'de>, ScalarType)> {
        let NodeKind::Scalar(s) = &node.kind else {
            return Err(Error::at("expected a scalar", node.span));
        };

        let tag = if self.tag_state == TagState::ConsumedApplication {
            None
        } else {
            node.tag.as_ref().map(|t| t.value.as_ref())
        };

        let kind = match tag {
            Some("!") => ScalarType::String,
            Some(tag) => match YamlType::from_tag(tag) {
                Some(YamlType::Scalar(kind)) => kind,
                _ => return Err(Error::at("unsupported scalar tag", node.span)),
            },
            None if s.style == ScalarStyle::Plain => classify(&s.value),
            None => ScalarType::String,
        };

        if tag.is_some() && kind != ScalarType::String {
            let actual = classify(&s.value);
            if actual != kind && !(kind == ScalarType::Float && actual == ScalarType::Int) {
                return Err(Error::at("scalar does not match its tag", s.span));
            }
        }

        Ok((s, kind))
    }

    fn text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (scalar, _) = self.scalar(ScalarType::String)?;
        Self::visit_text(scalar, visitor)
    }

    fn visit_text<V: Visitor<'de>>(scalar: &Scalar<'de>, visitor: V) -> Result<V::Value> {
        match &scalar.value {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_str(s),
        }
    }

    fn integer(self) -> Result<(&'s str, u32, Span)> {
        let (scalar, kind) = self.scalar(ScalarType::Int)?;
        Self::integer_value(scalar, kind)
    }

    fn integer_value<'a>(s: &'a Scalar<'_>, kind: ScalarType) -> Result<(&'a str, u32, Span)> {
        if kind != ScalarType::Int {
            return Err(Error::at("expected an integer", s.span));
        }

        let (value, radix) = if let Some(s) = s.value.strip_prefix("0x") {
            (s, 16)
        } else if let Some(s) = s.value.strip_prefix("0o") {
            (s, 8)
        } else {
            (s.value.as_ref(), 10)
        };

        Ok((value, radix, s.span))
    }

    fn float(self) -> Result<f64> {
        let (scalar, kind) = self.scalar(ScalarType::Float)?;
        Self::float_value(scalar, kind)
    }

    fn radix_integer_value(s: &Scalar<'_>, kind: ScalarType) -> Result<Option<u128>> {
        if kind != ScalarType::Int || !(s.value.starts_with("0x") || s.value.starts_with("0o")) {
            return Ok(None);
        }

        let (value, radix, span) = Self::integer_value(s, kind)?;
        u128::from_str_radix(value, radix)
            .map(Some)
            .map_err(|_| Error::at("integer out of range", span))
    }

    fn float_value(s: &Scalar<'_>, kind: ScalarType) -> Result<f64> {
        if !matches!(kind, ScalarType::Float | ScalarType::Int) {
            return Err(Error::at("expected a number", s.span));
        }

        let unsigned = s.value.trim_start_matches(['+', '-']);
        if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
            return Ok(if s.value.starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }

        if matches!(unsigned, ".nan" | ".NaN" | ".NAN") {
            return Ok(f64::NAN);
        }

        if let Some(value) = Self::radix_integer_value(s, kind)? {
            // YAML integer-to-float conversion rounds when the mantissa cannot hold every bit.
            #[allow(clippy::cast_precision_loss)]
            return Ok(value as f64);
        }

        let value: f64 = s
            .value
            .parse()
            .map_err(|_| Error::at("invalid float", s.span))?;
        if !value.is_finite() {
            return Err(Error::at("float out of range", s.span));
        }

        Ok(value)
    }

    fn check_core_tag(self, node: &Node<'_>, expected: YamlType) -> Result<()> {
        if self.tag_state == TagState::ConsumedApplication {
            return Ok(());
        }

        // Non-specific `!` follows the node's shape (and forces scalar text).
        // Application tags are checked separately by reject_tag.
        if node
            .tag
            .as_ref()
            .and_then(|tag| YamlType::from_tag(&tag.value))
            .is_some_and(|declared| !expected.accepts(declared))
        {
            return Err(Error::at("tag does not match requested type", node.span));
        }

        Ok(())
    }

    fn key(self) -> Result<Key<'s>> {
        let node = self.raw_node()?;

        if let Some(tag) = node.tag.as_ref().filter(|tag| {
            self.tag_state != TagState::ConsumedApplication && !is_core_tag(&tag.value)
        }) {
            // Application tags define their own scalar types. Without an
            // application-specific canonical form, compare the scalar text.
            let value = if let NodeKind::Scalar(scalar) = &node.kind {
                Key::String(&scalar.value)
            } else {
                Self {
                    tag_state: TagState::ConsumedApplication,
                    ..self
                }
                .key()?
            };

            return Ok(Key::Tagged(&tag.value, Box::new(value)));
        }

        match &node.kind {
            NodeKind::Scalar(_) => {
                let (scalar, kind) = self.scalar_value(node)?;
                Ok(match kind {
                    ScalarType::Null => Key::Null,
                    ScalarType::Bool => {
                        Key::Bool(matches!(scalar.value.as_ref(), "true" | "True" | "TRUE"))
                    }
                    ScalarType::String => Key::String(&scalar.value),
                    ScalarType::Int => {
                        let (value, radix, span) = Self::integer_value(scalar, kind)?;
                        let magnitude =
                            u128::from_str_radix(value.trim_start_matches(['+', '-']), radix)
                                .map_err(|_| Error::at("integer key out of range", span))?;
                        Key::Integer(value.starts_with('-') && magnitude != 0, magnitude)
                    }
                    ScalarType::Float => {
                        let value = Self::float_value(scalar, kind)?;
                        let bits = value.to_bits();
                        Key::Float(if value.is_nan() {
                            f64::NAN.to_bits()
                        } else if bits << 1 == 0 {
                            0
                        } else {
                            bits
                        })
                    }
                })
            }
            NodeKind::Sequence { items, .. } => {
                self.check_core_tag(node, YamlType::Sequence)?;
                items
                    .iter()
                    .map(|&id| self.child(id).key())
                    .collect::<Result<Vec<_>>>()
                    .map(Key::Sequence)
            }
            NodeKind::Mapping { entries, .. } => {
                self.check_core_tag(node, YamlType::Mapping)?;
                let mut pairs = entries
                    .iter()
                    .map(|entry| Ok((self.child(entry.key).key()?, self.child(entry.value).key()?)))
                    .collect::<Result<Vec<_>>>()?;
                pairs.sort();
                if pairs.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(Error::at("duplicate mapping key", node.span));
                }

                Ok(Key::Mapping(pairs))
            }
            NodeKind::Alias { target, .. } => self.child(*target).key(),
        }
    }

    fn check_mapping_keys(self, entries: &[Entry], span: Span) -> Result<()> {
        // Avoid constructing canonical keys for small string-key mappings.
        // Other key types fall back to check_general_keys.
        if entries.len() > 8 {
            return self.check_general_keys(entries, span);
        }

        let mut strings = [""; 8];

        for (index, entry) in entries.iter().enumerate() {
            let de = self.child(entry.key);
            let node = de.raw_node()?;
            if !matches!(node.kind, NodeKind::Scalar(_)) {
                return self.check_general_keys(entries, span);
            }

            if node
                .tag
                .as_ref()
                .is_some_and(|tag| !is_core_tag(&tag.value))
            {
                return self.check_general_keys(entries, span);
            }

            let (scalar, kind) = de.scalar_value(node)?;
            if kind != ScalarType::String {
                return self.check_general_keys(entries, span);
            }

            let key = scalar.value.as_ref();
            if strings[..index].contains(&key) {
                return Err(Error::at("duplicate mapping key", node.span));
            }

            strings[index] = key;
        }

        Ok(())
    }

    fn check_general_keys(self, entries: &[Entry], span: Span) -> Result<()> {
        // Keep up to eight canonical keys on the stack. Use a hash set beyond
        // that threshold to avoid quadratic duplicate checks.
        let mut small_keys = [const { Key::Null }; 8];
        let mut large_keys =
            (entries.len() > small_keys.len()).then(|| HashSet::with_capacity(entries.len()));

        for (index, entry) in entries.iter().enumerate() {
            let key = self.child(entry.key).key()?;
            let unique = if let Some(keys) = &mut large_keys {
                keys.insert(key)
            } else {
                let duplicate = small_keys[..index].contains(&key);
                small_keys[index] = key;
                !duplicate
            };

            if !unique {
                let span = self
                    .context
                    .stream
                    .node(entry.key)
                    .map_or(span, |key| key.span);
                return Err(Error::at("duplicate mapping key", span));
            }
        }

        Ok(())
    }

    fn map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.map_node(self.node()?, visitor)
    }

    fn map_node<V: Visitor<'de>>(self, node: &'s Node<'de>, visitor: V) -> Result<V::Value> {
        self.reject_tag(node)?;
        let NodeKind::Mapping { entries, .. } = &node.kind else {
            return Err(Error::at("expected a mapping", node.span));
        };

        self.check_core_tag(node, YamlType::Mapping)?;

        self.check_mapping_keys(entries, node.span)?;

        let mut access = Mapping {
            de: self,
            entries,
            index: 0,
            pending: None,
        };

        let result = visitor
            .visit_map(&mut access)
            .map_err(|error: Error| error.located(node.span))?;
        if access.index != entries.len() || access.pending.is_some() {
            return Err(Error::at("mapping was not fully consumed", node.span));
        }

        Ok(result)
    }

    fn seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.seq_node(self.node()?, visitor)
    }

    fn seq_node<V: Visitor<'de>>(self, node: &'s Node<'de>, visitor: V) -> Result<V::Value> {
        self.reject_tag(node)?;
        let NodeKind::Sequence { items, .. } = &node.kind else {
            return Err(Error::at("expected a sequence", node.span));
        };

        self.check_core_tag(node, YamlType::Sequence)?;

        let mut access = Sequence {
            de: self,
            items,
            index: 0,
        };

        let result = visitor
            .visit_seq(&mut access)
            .map_err(|error: Error| error.located(node.span))?;
        if access.index != items.len() {
            return Err(Error::at("sequence was not fully consumed", node.span));
        }

        Ok(result)
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Key<'a> {
    Null,
    Bool(bool),
    String(&'a str),
    Integer(bool, u128),
    Float(u64),
    Sequence(Vec<Self>),
    Mapping(Vec<(Self, Self)>),
    Tagged(&'a str, Box<Self>),
}

macro_rules! signed {
    ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
                let (value, radix, span) = self.integer()?;
                let number = <$ty>::from_str_radix(value, radix)
                    .map_err(|_| Error::at("integer out of range", span))?;

                visitor.$visit(number)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for Deserializer<'_, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.raw_node()?;

        if let Some(variant) = self.enum_tag(node) {
            return self.tagged_mapping(node, variant, visitor);
        }

        self.reject_tag(node)?;

        match &node.kind {
            NodeKind::Sequence { .. } => self.seq_node(node, visitor),
            NodeKind::Mapping { .. } => self.map_node(node, visitor),
            NodeKind::Scalar(_) => {
                let (s, kind) = self.scalar_value(node)?;

                match kind {
                    ScalarType::Null => visitor.visit_unit(),
                    ScalarType::Bool => {
                        visitor.visit_bool(matches!(s.value.as_ref(), "true" | "True" | "TRUE"))
                    }
                    ScalarType::String => Self::visit_text(s, visitor),
                    ScalarType::Float => visitor.visit_f64(Self::float_value(s, kind)?),
                    ScalarType::Int => {
                        let (value, radix, span) = Self::integer_value(s, kind)?;
                        if value.starts_with('-') {
                            let n = i128::from_str_radix(value, radix)
                                .map_err(|_| Error::at("integer out of range", span))?;
                            if let Ok(n) = i64::try_from(n) {
                                visitor.visit_i64(n)
                            } else {
                                visitor.visit_i128(n)
                            }
                        } else {
                            let n = u128::from_str_radix(value, radix)
                                .map_err(|_| Error::at("integer out of range", span))?;
                            if let Ok(n) = u64::try_from(n) {
                                visitor.visit_u64(n)
                            } else {
                                visitor.visit_u128(n)
                            }
                        }
                    }
                }
            }
            NodeKind::Alias { target, .. } => Self {
                id: *target,
                depth: self.depth + 1,
                ..self
            }
            .deserialize_any(visitor),
        }
    }

    signed! {
        deserialize_i8 => visit_i8: i8,
        deserialize_i16 => visit_i16: i16,
        deserialize_i32 => visit_i32: i32,
        deserialize_i64 => visit_i64: i64,
        deserialize_i128 => visit_i128: i128,
        deserialize_u8 => visit_u8: u8,
        deserialize_u16 => visit_u16: u16,
        deserialize_u32 => visit_u32: u32,
        deserialize_u64 => visit_u64: u64,
        deserialize_u128 => visit_u128: u128,
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (scalar, kind) = self.scalar(ScalarType::Float)?;

        // Convert decimal text and radix integers directly to f32 to avoid
        // double rounding through f64. Rust cannot parse YAML's .inf/.nan.
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let (value, finite_input) = if let Some(integer) = Self::radix_integer_value(scalar, kind)?
        {
            (integer as f32, true)
        } else {
            let wide = Self::float_value(scalar, kind)?;
            (
                scalar.value.parse::<f32>().unwrap_or(wide as f32),
                wide.is_finite(),
            )
        };

        if finite_input && !value.is_finite() {
            return Err(Error::at("float out of range", scalar.span));
        }

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.float()?)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, kind) = self.scalar(ScalarType::Bool)?;
        if kind != ScalarType::Bool {
            return Err(Error::at("expected a boolean", s.span));
        }

        visitor.visit_bool(matches!(s.value.as_ref(), "true" | "True" | "TRUE"))
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, _) = self.scalar(ScalarType::String)?;
        let mut chars = s.value.chars();

        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(Error::at("expected one character", s.span)),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if matches!(self.node()?.kind, NodeKind::Sequence { .. }) {
            return self.seq(visitor);
        }

        let (s, _) = self.scalar(ScalarType::String)?;

        match &s.value {
            Cow::Borrowed(s) => visitor.visit_borrowed_bytes(s.as_bytes()),
            Cow::Owned(s) => visitor.visit_bytes(s.as_bytes()),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.node()?;
        // Preserve application tags for an inner Tagged<T> or yaml_tag adapter.
        // Core tags retain their normal resolution, including !!null as None.
        let application_tag = self.tag_state == TagState::Unconsumed
            && node
                .tag
                .as_ref()
                .is_some_and(|tag| !is_core_tag(&tag.value));
        if !application_tag
            && matches!(node.kind, NodeKind::Scalar(_))
            && self.scalar_value(node)?.1 == ScalarType::Null
        {
            visitor.visit_none()
        } else {
            visitor.visit_some(Self {
                depth: self.depth + 1,
                ..self
            })
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, kind) = self.scalar(ScalarType::Null)?;
        if kind != ScalarType::Null {
            return Err(Error::at("expected null", s.span));
        }

        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        if name == spanned::NAME {
            return self.spanned(visitor);
        }

        if name == tagged::NAME {
            return self.tagged(visitor);
        }

        self.node()?;
        visitor.visit_newtype_struct(Self {
            depth: self.depth + 1,
            yaml_tag: self.yaml_tag || name == yaml_tag::NAME,
            ..self
        })
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.seq(visitor)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        self.seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.map(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let node = self.node()?;
        if let Some(tag) = self.enum_tag(node) {
            return visitor.visit_enum(Enumeration {
                de: self,
                variant: tag,
                value: VariantValue::Tagged(self.id),
            });
        }

        self.reject_tag(node)?;

        match &node.kind {
            NodeKind::Scalar(_) => {
                let (scalar, _) = self.scalar(ScalarType::String)?;
                visitor.visit_enum(Enumeration {
                    de: self,
                    variant: &scalar.value,
                    value: VariantValue::Unit,
                })
            }
            NodeKind::Mapping { entries, .. } if entries.len() == 1 => {
                self.check_core_tag(node, YamlType::Mapping)?;
                let (key, _) = self.child(entries[0].key).scalar(ScalarType::String)?;
                visitor.visit_enum(Enumeration {
                    de: self,
                    variant: &key.value,
                    value: VariantValue::Node(entries[0].value),
                })
            }
            _ => Err(Error::at(
                "expected an enum name or a single-entry mapping",
                node.span,
            )),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(de::IgnoredAny)?;
        visitor.visit_unit()
    }
}

struct SpannedAccess<'s, 'de> {
    de: Deserializer<'s, 'de>,
    span: Span,
    index: usize,
}

impl<'de> SeqAccess<'de> for &mut SpannedAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let value = match self.index {
            0 => seed.deserialize(self.span.start().into_deserializer()),
            1 => seed.deserialize(self.span.end().into_deserializer()),
            2 => seed.deserialize(self.de),
            3 => self.de.comments(seed),
            _ => return Ok(None),
        }?;

        self.index += 1;
        Ok(Some(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(4 - self.index)
    }
}

struct CommentAccess<'s, 'de> {
    comment: &'s AttachedComment<'de>,
    index: usize,
}

impl<'de> SeqAccess<'de> for CommentAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let value = match self.index {
            0 => seed.deserialize(self.comment.comment.span.start().into_deserializer()),
            1 => seed.deserialize(self.comment.comment.span.end().into_deserializer()),
            2 => seed.deserialize(de::value::BorrowedStrDeserializer::<Error>::new(
                self.comment.comment.text,
            )),
            3 => {
                let placement = match self.comment.placement {
                    CommentPlacement::Leading => 0u8,
                    CommentPlacement::Inline => 1,
                    CommentPlacement::Trailing => 2,
                    CommentPlacement::Detached => 3,
                };

                seed.deserialize(placement.into_deserializer())
            }
            _ => return Ok(None),
        }?;

        self.index += 1;
        Ok(Some(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(4 - self.index)
    }
}

struct Sequence<'s, 'de> {
    de: Deserializer<'s, 'de>,
    items: &'s [NodeId],
    index: usize,
}

impl<'de> SeqAccess<'de> for &mut Sequence<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let Some(&id) = self.items.get(self.index) else {
            return Ok(None);
        };

        self.index += 1;
        seed.deserialize(self.de.child(id)).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.index)
    }
}

struct Mapping<'s, 'de> {
    de: Deserializer<'s, 'de>,
    entries: &'s [Entry],
    index: usize,
    pending: Option<NodeId>,
}

impl<'de> MapAccess<'de> for &mut Mapping<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.pending.is_some() {
            return Err(Error::new("mapping value was not consumed"));
        }

        let Some(entry) = self.entries.get(self.index) else {
            return Ok(None);
        };

        self.pending = Some(entry.value);
        self.index += 1;
        seed.deserialize(self.de.child(entry.key)).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let id = self
            .pending
            .take()
            .ok_or_else(|| Error::new("mapping value requested before key"))?;
        seed.deserialize(self.de.child(id))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len() - self.index)
    }
}

struct TagMapping<'s, 'de> {
    enumeration: Enumeration<'s, 'de>,
    state: TagMappingState,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TagMappingState {
    Key,
    Value,
    End,
}

impl<'de> MapAccess<'de> for TagMapping<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.state {
            TagMappingState::Key => {
                let name = self.enumeration.de.variant_name(self.enumeration.variant)?;
                let key = seed.deserialize(name)?;
                self.state = TagMappingState::Value;
                Ok(Some(key))
            }
            TagMappingState::Value => Err(Error::new("mapping key has no value")),
            TagMappingState::End => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        if self.state != TagMappingState::Value {
            return Err(Error::new("mapping value has no key"));
        }

        self.state = TagMappingState::End;
        seed.deserialize(self.enumeration.payload()?)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(usize::from(self.state != TagMappingState::End))
    }
}

struct Enumeration<'s, 'de> {
    de: Deserializer<'s, 'de>,
    variant: &'s str,
    value: VariantValue,
}

enum VariantValue {
    Unit,
    Node(NodeId),
    Tagged(NodeId),
}

impl<'s, 'de> Enumeration<'s, 'de> {
    fn payload(&self) -> Result<Deserializer<'s, 'de>> {
        let (VariantValue::Node(id) | VariantValue::Tagged(id)) = self.value else {
            return Err(Error::new("expected a variant payload"));
        };

        let node = self.de.node()?;
        let core_tag = node.tag.as_ref().is_some_and(|tag| is_core_tag(&tag.value));
        let tag_state = if matches!(self.value, VariantValue::Tagged(_)) {
            if core_tag {
                TagState::ConsumedCore
            } else {
                TagState::ConsumedApplication
            }
        } else {
            TagState::Unconsumed
        };

        Ok(Deserializer {
            tag_state,
            ..self.de.child(id)
        })
    }
}

impl<'de> EnumAccess<'de> for Enumeration<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let variant = seed.deserialize(self.de.variant_name(self.variant)?)?;
        Ok((variant, self))
    }
}

impl<'de> VariantAccess<'de> for Enumeration<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            VariantValue::Unit => Ok(()),
            VariantValue::Node(_) | VariantValue::Tagged(_) => <()>::deserialize(self.payload()?),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self.payload()?)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        self.payload()?.seq(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.payload()?.map(visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, Context, Deserializer, ScalarType, TagState, VariantValue};
    use serde::{Deserialize, de::SeqAccess};

    #[test]
    fn metadata_protocols_can_be_ignored() {
        use serde::Deserializer as _;

        let stream = astral_yaml::parse("!Text hello").unwrap();
        let context = Context {
            stream: &stream,
            budget: Cell::new(100),
            scalar_budget: Cell::new(100),
            comments: crate::comments::CommentCache::new(0),
        };
        let de = Deserializer {
            context: &context,
            id: stream.documents()[0].root,
            depth: 0,
            tag_state: TagState::Unconsumed,
            yaml_tag: true,
        };

        for name in [
            crate::spanned::NAME,
            crate::tagged::NAME,
            crate::with::yaml_tag::NAME,
            "Transparent",
        ] {
            de.deserialize_newtype_struct(name, serde::de::IgnoredAny)
                .unwrap();
        }
    }

    #[test]
    fn enum_variant_names_charge_the_byte_budget() {
        use serde::de::EnumAccess;
        use std::marker::PhantomData;

        let stream = astral_yaml::parse("!long-tag x").unwrap();

        for bytes in [7, 8] {
            let context = Context {
                stream: &stream,
                budget: Cell::new(10),
                scalar_budget: Cell::new(bytes),
                comments: crate::comments::CommentCache::new(0),
            };
            let de = Deserializer {
                context: &context,
                id: stream.documents()[0].root,
                depth: 0,
                tag_state: TagState::Unconsumed,
                yaml_tag: false,
            };
            let access = super::Enumeration {
                de,
                variant: "long-tag",
                value: VariantValue::Tagged(de.id),
            };

            match access.variant_seed(PhantomData::<String>) {
                Ok((name, _)) => {
                    assert_eq!(name, "long-tag");
                    assert_eq!(context.scalar_budget.get(), 0);
                }
                Err(error) => {
                    assert_eq!(bytes, 7);
                    assert_eq!(error.message, "expanded scalar byte limit exceeded");
                    assert_eq!(
                        error.span.unwrap().slice(stream.source()),
                        Some("!long-tag x")
                    );
                }
            }
        }
    }

    #[test]
    fn nested_comment_lists_charge_each_emitted_record() {
        let stream = astral_yaml::parse("# leading\n42 # inline").unwrap();

        for budget in [2, 11, 12] {
            let visits = Cell::new(budget);
            let bytes = Cell::new(100);
            let de = Deserializer {
                context: &Context {
                    stream: &stream,
                    budget: visits,
                    scalar_budget: bytes,
                    comments: crate::comments::CommentCache::new(0),
                },
                id: stream.documents()[0].root,
                depth: 0,
                tag_state: TagState::Unconsumed,
                yaml_tag: false,
            };

            let result = crate::Spanned::<crate::Spanned<'_, u8>>::deserialize(de);
            if budget == 12 {
                let value = result.unwrap();
                assert_eq!(value.comments.len(), 2);
                assert_eq!(value.comments, value.value.comments);
                assert_eq!(de.context.budget.get(), 0);
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.message, "deserialization resource limit exceeded");
                assert!(error.span.unwrap().slice(stream.source()).is_some());
            }
        }
    }

    #[test]
    fn comment_access_checks_types_and_reports_remaining_elements() {
        struct Seed(bool);

        impl<'de> serde::de::DeserializeSeed<'de> for Seed {
            type Value = serde::de::IgnoredAny;

            fn deserialize<D: serde::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                if self.0 {
                    return Err(serde::de::Error::custom("seed failure"));
                }

                Deserialize::deserialize(deserializer)
            }
        }

        let comment = crate::AttachedComment {
            comment: astral_yaml::Comment {
                span: astral_yaml::Span::new(0, 6).unwrap(),
                text: " text",
            },
            placement: crate::CommentPlacement::Inline,
        };
        let mut access = super::CommentAccess {
            comment: &comment,
            index: 0,
        };
        assert_eq!(access.size_hint(), Some(4));
        assert_eq!(access.next_element::<usize>().unwrap(), Some(0));
        assert_eq!(access.size_hint(), Some(3));
        assert_eq!(access.next_element::<usize>().unwrap(), Some(6));
        assert_eq!(access.size_hint(), Some(2));
        assert!(access.next_element::<usize>().is_err());
        assert_eq!(access.next_element::<&str>().unwrap(), Some(" text"));
        assert_eq!(access.size_hint(), Some(1));
        assert_eq!(access.next_element::<u8>().unwrap(), Some(1));
        assert_eq!(access.size_hint(), Some(0));
        assert!(access.next_element::<usize>().unwrap().is_none());
        assert!(access.next_element::<usize>().unwrap().is_none());

        for placement in [
            crate::CommentPlacement::Leading,
            crate::CommentPlacement::Inline,
            crate::CommentPlacement::Trailing,
            crate::CommentPlacement::Detached,
        ] {
            let comment = crate::AttachedComment {
                placement,
                ..comment.clone()
            };
            let mut access = super::CommentAccess {
                comment: &comment,
                index: 0,
            };
            assert_eq!(
                access.next_element_seed(Seed(true)).unwrap_err().message,
                "seed failure"
            );
            let mut count = 0;

            while access.next_element_seed(Seed(false)).unwrap().is_some() {
                count += 1;
            }

            assert_eq!(count, 4);
        }
    }

    #[test]
    fn scalar_and_visit_budgets_stop_at_the_boundary() {
        let stream = astral_yaml::parse("abc").unwrap();
        let visits = Cell::new(2);
        let bytes = Cell::new(3);
        let de = Deserializer {
            context: &Context {
                stream: &stream,
                budget: visits,
                scalar_budget: bytes,
                comments: crate::comments::CommentCache::new(0),
            },
            id: stream.documents()[0].root,
            depth: 0,
            tag_state: TagState::Unconsumed,
            yaml_tag: false,
        };

        de.node().unwrap();
        assert_eq!(
            (de.context.budget.get(), de.context.scalar_budget.get()),
            (1, 0)
        );
        assert_eq!(
            de.node().unwrap_err().message,
            "expanded scalar byte limit exceeded"
        );
        assert_eq!(
            de.node().unwrap_err().message,
            "deserialization resource limit exceeded"
        );
    }

    #[test]
    fn aliases_charge_the_target_and_preserve_the_tag_policy() {
        let stream = astral_yaml::parse("[&a !custom abc, *a]").unwrap();
        let mut alias = None;

        for node in stream.nodes() {
            if let astral_yaml::NodeKind::Sequence { items, .. } = &node.kind {
                alias = items.last().copied();
            }
        }

        let visits = Cell::new(4);
        let bytes = Cell::new(6);
        let de = Deserializer {
            context: &Context {
                stream: &stream,
                budget: visits,
                scalar_budget: bytes,
                comments: crate::comments::CommentCache::new(0),
            },
            id: alias.unwrap(),
            depth: 0,
            tag_state: TagState::ConsumedApplication,
            yaml_tag: false,
        };

        assert_eq!(de.scalar(ScalarType::String).unwrap().0.value, "abc");
        assert_eq!(
            (de.context.budget.get(), de.context.scalar_budget.get()),
            (2, 3)
        );
        assert_eq!(
            Deserializer {
                tag_state: TagState::Unconsumed,
                yaml_tag: false,
                ..de
            }
            .scalar(ScalarType::String)
            .unwrap_err()
            .message,
            "non-core YAML tags require Tagged<T> or with::yaml_tag"
        );
    }
}
