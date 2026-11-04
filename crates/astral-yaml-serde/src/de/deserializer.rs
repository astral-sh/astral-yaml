use super::input::{Entry, Input, Kind as NodeKind, Node, Parsed, Scalar, ScalarOf};
use crate::schema::YamlType;
use crate::with::yaml_tag;
use crate::{AttachedComment, CommentPlacement, Error, commented, spanned, tagged};
use astral_yaml::schema::{self, ScalarType};
use astral_yaml::{NodeId, ScalarStyle, Span, Stream};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use std::cell::Cell;
use std::collections::HashSet;

type Result<T> = std::result::Result<T, Error>;

fn is_core_tag(tag: &str) -> bool {
    tag == "!" || YamlType::from_tag(tag).is_some()
}

/// Deserialize `source` as a YAML 1.2 document using the core schema.
///
/// Plain and quoted scalars can borrow from the source when they need no escape
/// decoding or line folding. Alias expansion is bounded by depth and work limits,
/// including for cyclic aliases.
///
/// Application tags require [`crate::Tagged`] or [`crate::with::yaml_tag`].
/// Core tags must match the destination type, although integer tags also allow
/// floating-point destinations. Strings, characters, scalar bytes, and enum names
/// accept untagged scalar text, but require a string tag when explicitly tagged.
///
/// # Errors
/// Returns an error if `source` is invalid YAML, does not contain exactly one
/// document, cannot be deserialized as `T`, or exceeds a resource limit.
pub fn from_str<'de, T: Deserialize<'de>>(source: &'de str) -> Result<T> {
    let stream = astral_yaml::parse(source)?;
    from_stream(&stream)
}

/// Deserialize a parsed stream containing exactly one document.
///
/// # Errors
/// Returns an error if the stream does not contain exactly one document,
/// the value cannot be deserialized as `T`, or a resource limit is exceeded.
pub fn from_stream<'de, T: Deserialize<'de>>(stream: &Stream<'de>) -> Result<T> {
    if stream.documents().len() != 1 {
        return Err(Error::at("expected exactly one document", stream.span()));
    }

    from_document(stream, 0)
}

/// Deserialize the document at zero-based `index` in `stream`.
///
/// # Errors
/// Returns an error if `index` is out of bounds, the value cannot be deserialized
/// as `T`, or a resource limit is exceeded.
pub fn from_document<'de, T: Deserialize<'de>>(stream: &Stream<'de>, index: usize) -> Result<T> {
    let doc = stream
        .documents()
        .get(index)
        .ok_or_else(|| Error::at("document index out of bounds", stream.span()))?;

    from_input(Parsed::new(stream, index), doc.root)
}

#[cfg(feature = "editable")]
pub(crate) fn from_edit_value<'de, T: Deserialize<'de>>(
    value: crate::syntax::ValueRef<'de>,
) -> Result<T> {
    from_input(value, value.root())
}

fn from_input<'de, T: Deserialize<'de>>(input: impl Input<'de>, id: NodeId) -> Result<T> {
    let context = Context::new(input);
    T::deserialize(context.deserializer(id)).map_err(|mut error| {
        if error.span.is_none() {
            error.span = context.input.node(id).ok().and_then(Node::span);
        }
        error.path.get_or_insert_with(|| "$".to_owned());
        error
    })
}

struct Context<I> {
    input: I,
    budget: Cell<usize>,
    // Shared by expanded scalar text, enum names, and tags used as mapping keys.
    scalar_budget: Cell<usize>,
}

impl<I> Context<I> {
    fn new(input: I) -> Self {
        Self {
            input,
            budget: Cell::new(1_000_000),
            scalar_budget: Cell::new(64 * 1024 * 1024),
        }
    }

    fn deserializer(&self, id: NodeId) -> Deserializer<&Self> {
        Deserializer {
            context: self,
            id,
            depth: 0,
            tag_state: TagState::Unconsumed,
            yaml_tag: false,
        }
    }
}

/// An application visitor can skip a mapping or recover from a duplicate-key error.
/// Check each mapping in the editable document independently of that visitor.
#[cfg(feature = "editable")]
pub(crate) fn check_document_keys(stream: &Stream<'_>) -> Result<()> {
    let context = Context::new(Parsed::new(stream, 0));
    for (index, node) in stream.nodes().iter().enumerate() {
        if let astral_yaml::NodeKind::Mapping { entries, .. } = &node.kind
            && entries.len() > 1
        {
            context
                .deserializer(NodeId::from_index(index))
                .check_mapping_keys(entries, Some(node.span))
                .map_err(|error| Error::at(error.message, error.span))?;
        }
    }
    Ok(())
}

/// Canonical identity for a serialized mapping key. String identities use the
/// serializer's quoting so they compare with its allocation-free string path.
pub(crate) fn canonical_key(source: &str) -> Result<String> {
    let stream = astral_yaml::parse(source)?;
    // The serializer emits exactly one node for each key.
    let document = &stream.documents()[0];
    let context = Context::new(Parsed::new(&stream, 0));
    let key = context.deserializer(document.root).key(&mut Vec::new())?;

    let mut identity = String::new();
    key.write_identity(&mut identity);
    Ok(identity)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TagState {
    Unconsumed,
    // Core tags still determine the payload's type after capture.
    ConsumedCore,
    // Application tags are metadata. Resolve the payload without its tag.
    ConsumedApplication,
}

#[derive(Clone, Copy)]
struct Deserializer<C> {
    context: C,
    id: NodeId,
    depth: usize,
    tag_state: TagState,
    // Enable local tags as enum names for this value only.
    yaml_tag: bool,
}

impl<'input, 'de: 'input, I: Input<'de>> Deserializer<&'input Context<I>> {
    #[inline]
    fn raw_node(self) -> Result<I::Node<'input>> {
        let node = self.context.input.node(self.id)?;
        if self.depth >= 64 || self.context.budget.get() == 0 {
            return Err(Error::at(
                "deserialization resource limit exceeded",
                node.span(),
            ));
        }

        self.context.budget.set(self.context.budget.get() - 1);
        if let NodeKind::Scalar(scalar) = node.kind() {
            let remaining = self
                .context
                .scalar_budget
                .get()
                .checked_sub(scalar.value().len())
                .ok_or_else(|| Error::at("expanded scalar byte limit exceeded", node.span()))?;
            self.context.scalar_budget.set(remaining);
        }

        Ok(node)
    }

    #[inline]
    fn node(mut self) -> Result<I::Node<'input>> {
        loop {
            let node = self.raw_node()?;
            let NodeKind::Alias { target } = node.kind() else {
                return Ok(node);
            };
            self.id = target;
            self.depth += 1;
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

    fn metadata<V: Visitor<'de>>(self, visitor: V, metadata: Metadata) -> Result<V::Value> {
        let span = metadata.span();
        let mut access = MetadataAccess {
            de: Self {
                depth: self.depth + 1,
                ..self
            },
            metadata,
            index: 0,
        };

        let value = visitor
            .visit_seq(&mut access)
            .map_err(|error| error.located(span))?;
        if access.index != metadata.len() {
            return Err(Error::at("metadata value was not fully consumed", span));
        }

        Ok(value)
    }

    fn tagged<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.node()?;
        let tag = node
            .tag()
            .filter(|_| self.tag_state == TagState::Unconsumed)
            .ok_or_else(|| Error::at("expected an explicit YAML tag", node.span()))?;

        visitor.visit_enum(Enumeration {
            de: self,
            variant: tag,
            value: VariantValue::Tagged,
        })
    }

    fn tagged_mapping<V: Visitor<'de>>(
        self,
        node: I::Node<'input>,
        variant: &'input str,
        visitor: V,
    ) -> Result<V::Value> {
        // Serde's untagged-enum buffer accepts maps but rejects enum input.
        // Keep this conversion confined to the explicitly opted-in node.
        let mut map = TagMapping {
            enumeration: Enumeration {
                de: self,
                variant,
                value: VariantValue::Tagged,
            },
            state: TagMappingState::Key,
        };
        let value = visitor
            .visit_map(&mut map)
            .map_err(|error: Error| error.located(node.span()))?;
        if map.state != TagMappingState::End {
            return Err(Error::at(
                "tagged mapping was not fully consumed",
                node.span(),
            ));
        }

        Ok(value)
    }

    fn comments<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let mut comments = self.context.input.comments(self.id, &self.context.budget)?;
        let remaining = self
            .context
            .budget
            .get()
            .checked_sub(comments.len())
            .ok_or_else(|| {
                Error::at(
                    "deserialization resource limit exceeded",
                    self.context.input.node(self.id).ok().and_then(Node::span),
                )
            })?;
        self.context.budget.set(remaining);

        comments.sort_unstable_by_key(|comment| comment.comment.span.start());
        let records = comments.iter().map(|comment| {
            de::value::SeqAccessDeserializer::new(CommentAccess { comment, index: 0 })
        });

        seed.deserialize(de::value::SeqDeserializer::new(records))
    }

    fn scalar(self, expected: ScalarType) -> Result<(ScalarOf<'input, 'de, I>, ScalarType)> {
        let node = self.node()?;

        // Text destinations accept every untagged scalar spelling, so they do
        // not need implicit boolean or numeric classification.
        if expected == ScalarType::String && node.tag().is_none() {
            let NodeKind::Scalar(scalar) = node.kind() else {
                return Err(Error::at("expected a scalar", node.span()));
            };

            return Ok((scalar, ScalarType::String));
        }

        self.reject_tag(node)?;
        let value = self.scalar_value(node)?;
        self.check_core_tag(node, YamlType::Scalar(expected))?;

        Ok(value)
    }

    fn reject_tag(self, node: I::Node<'input>) -> Result<()> {
        if self.tag_state == TagState::Unconsumed && node.tag().is_some_and(|tag| !is_core_tag(tag))
        {
            return Err(Error::at(
                "non-core YAML tags require Tagged<T> or with::yaml_tag",
                node.span(),
            ));
        }

        Ok(())
    }

    fn enum_tag(self, node: I::Node<'input>) -> Option<&'input str> {
        node.tag()
            .filter(|_| self.yaml_tag && self.tag_state == TagState::Unconsumed)
            .and_then(|tag| tag.strip_prefix('!'))
            .filter(|name| !name.is_empty())
    }

    fn variant_name(self, name: &'input str) -> Result<de::value::StrDeserializer<'input, Error>> {
        let remaining = self
            .context
            .scalar_budget
            .get()
            .checked_sub(name.len())
            .ok_or_else(|| {
                let span = self.context.input.node(self.id).ok().and_then(Node::span);
                Error::at("expanded scalar byte limit exceeded", span)
            })?;
        self.context.scalar_budget.set(remaining);

        Ok(de::value::StrDeserializer::new(name))
    }

    #[inline]
    fn scalar_value(self, node: I::Node<'input>) -> Result<(ScalarOf<'input, 'de, I>, ScalarType)> {
        let NodeKind::Scalar(s) = node.kind() else {
            return Err(Error::at("expected a scalar", node.span()));
        };

        let tag = if self.tag_state == TagState::ConsumedApplication {
            None
        } else {
            node.tag()
        };

        let kind = match tag {
            Some("!") => ScalarType::String,
            Some(tag) => match YamlType::from_tag(tag) {
                Some(YamlType::Scalar(kind)) => kind,
                _ => return Err(Error::at("unsupported scalar tag", node.span())),
            },
            None if s.style() == ScalarStyle::Plain => schema::classify(s.value()),
            None => ScalarType::String,
        };

        if tag.is_some() && kind != ScalarType::String {
            let actual = schema::classify(s.value());
            if actual != kind && !(kind == ScalarType::Float && actual == ScalarType::Int) {
                return Err(Error::at("scalar does not match its tag", s.span()));
            }
        }

        Ok((s, kind))
    }

    fn text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (scalar, _) = self.scalar(ScalarType::String)?;
        Self::visit_text(scalar, visitor)
    }

    fn visit_text<V: Visitor<'de>>(
        scalar: ScalarOf<'input, 'de, I>,
        visitor: V,
    ) -> Result<V::Value> {
        match scalar.borrowed() {
            Some(text) => visitor.visit_borrowed_str(text),
            None => visitor.visit_str(scalar.value()),
        }
    }

    fn integer(self) -> Result<(&'input str, u32, Option<Span>)> {
        let (scalar, kind) = self.scalar(ScalarType::Int)?;
        Self::integer_value(scalar, kind)
    }

    fn integer_value(
        s: ScalarOf<'input, 'de, I>,
        kind: ScalarType,
    ) -> Result<(&'input str, u32, Option<Span>)> {
        if kind != ScalarType::Int {
            return Err(Error::at("expected an integer", s.span()));
        }

        let value = s.value();
        let (value, radix) = if let Some(s) = value.strip_prefix("0x") {
            (s, 16)
        } else if let Some(s) = value.strip_prefix("0o") {
            (s, 8)
        } else {
            (value, 10)
        };

        Ok((value, radix, s.span()))
    }

    fn float(self) -> Result<f64> {
        let (scalar, kind) = self.scalar(ScalarType::Float)?;
        Self::float_value(scalar, kind)
    }

    fn radix_integer_value(s: ScalarOf<'input, 'de, I>, kind: ScalarType) -> Result<Option<u128>> {
        if kind != ScalarType::Int || !(s.value().starts_with("0x") || s.value().starts_with("0o"))
        {
            return Ok(None);
        }

        let (value, radix, span) = Self::integer_value(s, kind)?;
        u128::from_str_radix(value, radix)
            .map(Some)
            .map_err(|_| Error::at("integer out of range", span))
    }

    fn wide_radix_float(s: ScalarOf<'input, 'de, I>, kind: ScalarType) -> Result<f64> {
        let (digits, radix, span) = Self::integer_value(s, kind)?;
        let digit_bits = if radix == 16 { 4 } else { 3 };
        let mut significand = 0_u64;
        let mut significant_bits = 0;
        let mut discarded_bits = 0;
        let mut guard = false;
        let mut sticky = false;

        for digit in digits.bytes() {
            let digit = char::from(digit)
                .to_digit(radix)
                .expect("classified integer has valid digits");

            for position in (0..digit_bits).rev() {
                let bit = (digit >> position) & 1;
                if significant_bits == 0 && bit == 0 {
                    continue;
                }

                if significant_bits < f64::MANTISSA_DIGITS {
                    significand = (significand << 1) | u64::from(bit);
                    significant_bits += 1;
                } else {
                    if discarded_bits == 0 {
                        guard = bit != 0;
                    } else {
                        sticky |= bit != 0;
                    }

                    discarded_bits += 1;
                    let mantissa_bits = i32::try_from(f64::MANTISSA_DIGITS)
                        .expect("f64 mantissa width fits in i32");
                    if discarded_bits > f64::MAX_EXP - mantissa_bits {
                        return Err(Error::at("float out of range", span));
                    }
                }
            }
        }

        // Round to nearest, ties to even, before scaling by a power of two.
        if guard && (sticky || significand & 1 != 0) {
            significand += 1;
        }

        #[allow(clippy::cast_precision_loss)]
        let value = (significand as f64) * 2_f64.powi(discarded_bits);
        if !value.is_finite() {
            return Err(Error::at("float out of range", span));
        }

        Ok(value)
    }

    fn float_value(s: ScalarOf<'input, 'de, I>, kind: ScalarType) -> Result<f64> {
        if !matches!(kind, ScalarType::Float | ScalarType::Int) {
            return Err(Error::at("expected a number", s.span()));
        }

        let unsigned = s.value().trim_start_matches(['+', '-']);
        if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
            return Ok(if s.value().starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }

        if matches!(unsigned, ".nan" | ".NaN" | ".NAN") {
            return Ok(f64::NAN);
        }

        match Self::radix_integer_value(s, kind) {
            Ok(Some(value)) => {
                // YAML integer-to-float conversion rounds when the mantissa cannot hold every bit.
                #[allow(clippy::cast_precision_loss)]
                return Ok(value as f64);
            }
            Err(_) => return Self::wide_radix_float(s, kind),
            Ok(None) => {}
        }

        let value: f64 = s
            .value()
            .parse()
            .map_err(|_| Error::at("invalid float", s.span()))?;
        if !value.is_finite() {
            return Err(Error::at("float out of range", s.span()));
        }

        Ok(value)
    }

    fn check_core_tag(self, node: I::Node<'input>, expected: YamlType) -> Result<()> {
        if self.tag_state == TagState::ConsumedApplication {
            return Ok(());
        }

        // Non-specific `!` follows the node's shape (and forces scalar text).
        // Application tags are checked separately by reject_tag.
        if node
            .tag()
            .and_then(YamlType::from_tag)
            .is_some_and(|declared| !expected.accepts(declared))
        {
            return Err(Error::at("tag does not match requested type", node.span()));
        }

        Ok(())
    }

    fn key(self, ancestors: &mut Vec<NodeId>) -> Result<Key<'input>> {
        let node = self.raw_node()?;

        if let Some(tag) = node
            .tag()
            .filter(|tag| self.tag_state != TagState::ConsumedApplication && !is_core_tag(tag))
        {
            // Aliases can bring the same tag into many keys. Charge for each
            // occurrence before hashing or comparing the tag text.
            let remaining = self
                .context
                .scalar_budget
                .get()
                .checked_sub(tag.len())
                .ok_or_else(|| Error::at("expanded key tag byte limit exceeded", node.span()))?;
            self.context.scalar_budget.set(remaining);

            // Application tags define their own scalar types. Without an
            // application-specific canonical form, compare the scalar text.
            let value = if let NodeKind::Scalar(scalar) = node.kind() {
                Key::String(scalar.value())
            } else {
                Self {
                    tag_state: TagState::ConsumedApplication,
                    ..self
                }
                .key(ancestors)?
            };

            return Ok(Key::Tagged(tag, Box::new(value)));
        }

        match node.kind() {
            NodeKind::Scalar(_) => {
                let (scalar, kind) = self.scalar_value(node)?;
                Ok(match kind {
                    ScalarType::Null => Key::Null,
                    ScalarType::Bool => {
                        Key::Bool(matches!(scalar.value(), "true" | "True" | "TRUE"))
                    }
                    ScalarType::String => Key::String(scalar.value()),
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
                ancestors.push(self.id);
                let key = items
                    .iter()
                    .map(|&id| self.child(id).key(ancestors))
                    .collect::<Result<Vec<_>>>()
                    .map(Key::Sequence);
                ancestors.pop();
                key
            }
            NodeKind::Mapping { entries, .. } => {
                self.check_core_tag(node, YamlType::Mapping)?;
                ancestors.push(self.id);
                let mut pairs = entries
                    .iter()
                    .map(Entry::ids)
                    .map(|(key, value)| {
                        Ok((
                            self.child(key).key(ancestors)?,
                            self.child(value).key(ancestors)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                ancestors.pop();
                pairs.sort();
                if pairs.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(Error::at("duplicate mapping key", node.span()));
                }

                Ok(Key::Mapping(pairs))
            }
            NodeKind::Alias { target, .. } => {
                // YAML leaves equality of recursive collections implementation-defined.
                // Compare a back edge by the identity of the node it refers to.
                if ancestors.contains(&target) {
                    Ok(Key::Reference(target.index()))
                } else {
                    self.child(target).key(ancestors)
                }
            }
        }
    }

    fn check_mapping_keys(self, entries: &[I::Entry], span: Option<Span>) -> Result<()> {
        // Avoid constructing canonical keys for small string-key mappings.
        // Other key types fall back to check_general_keys.
        if entries.len() > 8 {
            return self.check_general_keys(entries, span);
        }

        let mut strings = [""; 8];

        for (index, (key, _)) in entries.iter().map(Entry::ids).enumerate() {
            let de = self.child(key);
            let node = de.raw_node()?;
            if !matches!(node.kind(), NodeKind::Scalar(_)) {
                return self.check_general_keys(entries, span);
            }

            if node.tag().is_some_and(|tag| !is_core_tag(tag)) {
                return self.check_general_keys(entries, span);
            }

            let (scalar, kind) = de.scalar_value(node)?;
            if kind != ScalarType::String {
                return self.check_general_keys(entries, span);
            }

            let key = scalar.value();
            if strings[..index].contains(&key) {
                let error = Error::at("duplicate mapping key", node.span());
                return Err(error.in_path(format_args!(".key_at({index})")));
            }

            strings[index] = key;
        }

        Ok(())
    }

    fn check_general_keys(self, entries: &[I::Entry], span: Option<Span>) -> Result<()> {
        // Keep up to eight canonical keys on the stack. Use a hash set beyond
        // that threshold to avoid quadratic duplicate checks.
        let mut small_keys = [const { Key::Null }; 8];
        let mut large_keys =
            (entries.len() > small_keys.len()).then(|| HashSet::with_capacity(entries.len()));

        let mut ancestors = Vec::new();
        for (index, (id, _)) in entries.iter().map(Entry::ids).enumerate() {
            let key = self
                .child(id)
                .key(&mut ancestors)
                .map_err(|error| error.in_path(format_args!(".key_at({index})")))?;
            let unique = if let Some(keys) = &mut large_keys {
                keys.insert(key)
            } else {
                let duplicate = small_keys[..index].contains(&key);
                small_keys[index] = key;
                !duplicate
            };

            if !unique {
                let span = self.context.input.node(id).map_or(span, Node::span);
                let error = Error::at("duplicate mapping key", span);
                return Err(error.in_path(format_args!(".key_at({index})")));
            }
        }

        Ok(())
    }

    fn map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.map_node(self.node()?, visitor)
    }

    fn map_node<V: Visitor<'de>>(self, node: I::Node<'input>, visitor: V) -> Result<V::Value> {
        self.reject_tag(node)?;
        let NodeKind::Mapping { entries, .. } = node.kind() else {
            return Err(Error::at("expected a mapping", node.span()));
        };

        self.check_core_tag(node, YamlType::Mapping)?;

        self.check_mapping_keys(entries, node.span())?;

        let mut access = Mapping {
            de: self,
            entries,
            index: 0,
            pending: None,
        };

        let result = visitor
            .visit_map(&mut access)
            .map_err(|error: Error| error.located(node.span()))?;
        if access.index != entries.len() || access.pending.is_some() {
            return Err(Error::at("mapping was not fully consumed", node.span()));
        }

        Ok(result)
    }

    fn seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.seq_node(self.node()?, visitor)
    }

    fn seq_node<V: Visitor<'de>>(self, node: I::Node<'input>, visitor: V) -> Result<V::Value> {
        self.reject_tag(node)?;
        let NodeKind::Sequence { items, .. } = node.kind() else {
            return Err(Error::at("expected a sequence", node.span()));
        };

        self.check_core_tag(node, YamlType::Sequence)?;

        let mut access = Sequence {
            de: self,
            items,
            index: 0,
        };

        let result = visitor
            .visit_seq(&mut access)
            .map_err(|error: Error| error.located(node.span()))?;
        if access.index != items.len() {
            return Err(Error::at("sequence was not fully consumed", node.span()));
        }

        Ok(result)
    }

    fn visit_number<V: Visitor<'de>>(
        s: ScalarOf<'input, 'de, I>,
        kind: ScalarType,
        visitor: V,
    ) -> Result<V::Value> {
        if kind == ScalarType::Float {
            return visitor.visit_f64(Self::float_value(s, kind)?);
        }

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

    fn visit_any<V: Visitor<'de>>(
        self,
        visitor: V,
        visit_number: impl FnOnce(ScalarOf<'input, 'de, I>, ScalarType, V) -> Result<V::Value>,
    ) -> Result<V::Value> {
        let node = self.raw_node()?;

        if let Some(variant) = self.enum_tag(node) {
            return self.tagged_mapping(node, variant, visitor);
        }

        self.reject_tag(node)?;

        match node.kind() {
            NodeKind::Sequence { .. } => self.seq_node(node, visitor),
            NodeKind::Mapping { .. } => self.map_node(node, visitor),
            NodeKind::Scalar(_) => {
                let (s, kind) = self.scalar_value(node)?;

                match kind {
                    ScalarType::Null => visitor.visit_unit(),
                    ScalarType::Bool => {
                        visitor.visit_bool(matches!(s.value(), "true" | "True" | "TRUE"))
                    }
                    ScalarType::String => Self::visit_text(s, visitor),
                    ScalarType::Int | ScalarType::Float => visit_number(s, kind, visitor),
                }
            }
            NodeKind::Alias { target, .. } => Self {
                id: target,
                depth: self.depth + 1,
                ..self
            }
            .visit_any(visitor, visit_number),
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Key<'input> {
    Null,
    Bool(bool),
    String(&'input str),
    Integer(bool, u128),
    Float(u64),
    Reference(usize),
    Sequence(Vec<Self>),
    Mapping(Vec<(Self, Self)>),
    Tagged(&'input str, Box<Self>),
}

impl Key<'_> {
    fn write_identity(&self, output: &mut String) {
        use std::fmt::Write;

        match self {
            Self::Null => output.push_str("n;"),
            Self::Bool(value) => output.push_str(if *value { "b1;" } else { "b0;" }),
            Self::String(value) => crate::ser::write_quoted(output, value),
            Self::Integer(negative, magnitude) => {
                write!(output, "i{}{magnitude};", if *negative { '-' } else { '+' })
                    .expect("String write");
            }
            Self::Float(bits) => write!(output, "f{bits:x};").expect("String write"),
            Self::Reference(id) => write!(output, "r{id};").expect("String write"),
            Self::Sequence(items) => {
                output.push('[');
                for item in items {
                    item.write_identity(output);
                }
                output.push(']');
            }
            Self::Mapping(entries) => {
                output.push('{');
                for (key, value) in entries {
                    key.write_identity(output);
                    value.write_identity(output);
                }
                output.push('}');
            }
            Self::Tagged(tag, value) => {
                output.push('t');
                Self::String(tag).write_identity(output);
                value.write_identity(output);
            }
        }
    }
}

macro_rules! integer_methods {
    ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
                let (value, radix, span) = self.integer()?;
                let value = value
                    .strip_prefix('-')
                    .filter(|digits| digits.bytes().all(|digit| digit == b'0'))
                    .unwrap_or(value);
                let number = <$ty>::from_str_radix(value, radix)
                    .map_err(|_| Error::at("integer out of range", span))?;

                visitor.$visit(number)
            }
        )*
    };
}

impl<'input, 'de: 'input, I: Input<'de>> de::Deserializer<'de>
    for Deserializer<&'input Context<I>>
{
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_any(visitor, Self::visit_number)
    }

    integer_methods! {
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
                scalar.value().parse::<f32>().unwrap_or(wide as f32),
                wide.is_finite(),
            )
        };

        if finite_input && !value.is_finite() {
            return Err(Error::at("float out of range", scalar.span()));
        }

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.float()?)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, kind) = self.scalar(ScalarType::Bool)?;
        if kind != ScalarType::Bool {
            return Err(Error::at("expected a boolean", s.span()));
        }

        visitor.visit_bool(matches!(s.value(), "true" | "True" | "TRUE"))
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, _) = self.scalar(ScalarType::String)?;
        let mut chars = s.value().chars();

        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(Error::at("expected one character", s.span())),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if matches!(self.node()?.kind(), NodeKind::Sequence { .. }) {
            return self.seq(visitor);
        }

        let (s, _) = self.scalar(ScalarType::String)?;

        match s.borrowed() {
            Some(text) => visitor.visit_borrowed_bytes(text.as_bytes()),
            None => visitor.visit_bytes(s.value().as_bytes()),
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
            && node.tag().is_some_and(|tag| !is_core_tag(tag));
        if !application_tag
            && matches!(node.kind(), NodeKind::Scalar(_))
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
            return Err(Error::at("expected null", s.span()));
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
            let span = self
                .raw_node()?
                .span()
                .ok_or_else(|| Error::new("new node has no source span until the edit commits"))?;
            return self.metadata(visitor, Metadata::Span(span));
        }

        if name == commented::NAME {
            let span = self.raw_node()?.span();
            return self.metadata(visitor, Metadata::Comments(span));
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
                value: VariantValue::Tagged,
            });
        }

        self.reject_tag(node)?;

        match node.kind() {
            NodeKind::Scalar(_) => {
                let (scalar, _) = self.scalar(ScalarType::String)?;
                visitor.visit_enum(Enumeration {
                    de: self,
                    variant: scalar.value(),
                    value: VariantValue::Unit,
                })
            }
            NodeKind::Mapping { entries, .. } if entries.len() == 1 => {
                self.check_core_tag(node, YamlType::Mapping)?;
                let (key, value) = entries[0].ids();
                let (key, _) = self.child(key).scalar(ScalarType::String)?;
                visitor
                    .visit_enum(Enumeration {
                        de: self,
                        variant: key.value(),
                        value: VariantValue::Node(value),
                    })
                    .map_err(|error| error.in_path(format_args!("[{:?}]", key.value())))
            }
            _ => Err(Error::at(
                "expected an enum name or a single-entry mapping",
                node.span(),
            )),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.text(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.visit_any(de::IgnoredAny, |s, kind, ignored| {
            // Explicit float tags require a decimal spelling for integers.
            if kind == ScalarType::Float && matches!(s.value().as_bytes(), [b'0', b'o' | b'x', ..])
            {
                return Err(Error::at("invalid float", s.span()));
            }

            Ok(ignored)
        })?;
        visitor.visit_unit()
    }
}

struct MetadataAccess<'input, I> {
    de: Deserializer<&'input Context<I>>,
    metadata: Metadata,
    index: usize,
}

#[derive(Clone, Copy)]
enum Metadata {
    Span(Span),
    Comments(Option<Span>),
}

impl Metadata {
    fn span(self) -> Option<Span> {
        match self {
            Self::Span(span) => Some(span),
            Self::Comments(span) => span,
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Span(_) => 3,
            Self::Comments(_) => 2,
        }
    }
}

impl<'input, 'de: 'input, I: Input<'de>> SeqAccess<'de> for &mut MetadataAccess<'input, I> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let value = match (self.metadata, self.index) {
            (Metadata::Span(span), 0) => seed.deserialize(span.start().into_deserializer()),
            (Metadata::Span(span), 1) => seed.deserialize(span.end().into_deserializer()),
            (Metadata::Span(_), 2) | (Metadata::Comments(_), 0) => seed.deserialize(self.de),
            (Metadata::Comments(_), 1) => self.de.comments(seed),
            _ => return Ok(None),
        }?;

        self.index += 1;
        Ok(Some(value))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.metadata.len() - self.index)
    }
}

struct CommentAccess<'input, 'de> {
    comment: &'input AttachedComment<'de>,
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

struct Sequence<'input, I> {
    de: Deserializer<&'input Context<I>>,
    items: &'input [NodeId],
    index: usize,
}

impl<'input, 'de: 'input, I: Input<'de>> SeqAccess<'de> for &mut Sequence<'input, I> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let index = self.index;
        let Some(&id) = self.items.get(index) else {
            return Ok(None);
        };

        self.index += 1;
        seed.deserialize(self.de.child(id))
            .map(Some)
            .map_err(|error| error.in_path(format_args!("[{index}]")))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.index)
    }
}

struct Mapping<'input, I, E> {
    de: Deserializer<&'input Context<I>>,
    entries: &'input [E],
    index: usize,
    pending: Option<&'input E>,
}

impl<'input, 'de: 'input, I: Input<'de, Entry = E>, E: Entry + 'input> Mapping<'input, I, E> {
    fn value_error(&self, key: NodeId, error: Error) -> Error {
        let key = self
            .de
            .context
            .input
            .node(key)
            .and_then(|node| self.de.scalar_value(node));
        if let Ok((scalar, ScalarType::String)) = key {
            error.in_path(format_args!("[{:?}]", scalar.value()))
        } else {
            let index = self.index - 1;
            error.in_path(format_args!(".value_at({index})"))
        }
    }
}

impl<'input, 'de: 'input, I: Input<'de, Entry = E>, E: Entry + 'input> MapAccess<'de>
    for &mut Mapping<'input, I, E>
{
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.pending.is_some() {
            return Err(Error::new("mapping value was not consumed"));
        }

        let index = self.index;
        let Some(entry) = self.entries.get(index) else {
            return Ok(None);
        };

        self.pending = Some(entry);
        self.index += 1;
        seed.deserialize(self.de.child(entry.ids().0))
            .map(Some)
            .map_err(|error| error.in_path(format_args!(".key_at({index})")))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let (key, value) = self
            .pending
            .take()
            .ok_or_else(|| Error::new("mapping value requested before key"))?
            .ids();
        seed.deserialize(self.de.child(value))
            .map_err(|error| self.value_error(key, error))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len() - self.index)
    }
}

struct TagMapping<'input, I> {
    enumeration: Enumeration<'input, I>,
    state: TagMappingState,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TagMappingState {
    Key,
    Value,
    End,
}

impl<'input, 'de: 'input, I: Input<'de>> MapAccess<'de> for TagMapping<'input, I> {
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

struct Enumeration<'input, I> {
    de: Deserializer<&'input Context<I>>,
    variant: &'input str,
    value: VariantValue,
}

enum VariantValue {
    Unit,
    Node(NodeId),
    Tagged,
}

impl<'input, 'de: 'input, I: Input<'de>> Enumeration<'input, I> {
    fn payload(&self) -> Result<Deserializer<&'input Context<I>>> {
        match self.value {
            VariantValue::Unit => Err(Error::new("expected a variant payload")),
            VariantValue::Node(id) => {
                self.de.node()?;
                Ok(self.de.child(id))
            }
            VariantValue::Tagged => {
                let node = self.de.node()?;
                let tag_state = if node.tag().is_some_and(is_core_tag) {
                    TagState::ConsumedCore
                } else {
                    TagState::ConsumedApplication
                };

                // The tag and its payload belong to the same YAML node.
                Ok(Deserializer {
                    tag_state,
                    yaml_tag: false,
                    ..self.de
                })
            }
        }
    }
}

impl<'input, 'de: 'input, I: Input<'de>> EnumAccess<'de> for Enumeration<'input, I> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let variant = seed.deserialize(self.de.variant_name(self.variant)?)?;
        Ok((variant, self))
    }
}

impl<'input, 'de: 'input, I: Input<'de>> VariantAccess<'de> for Enumeration<'input, I> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            VariantValue::Unit => Ok(()),
            VariantValue::Node(_) | VariantValue::Tagged => <()>::deserialize(self.payload()?),
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
    use serde::Deserialize;
    use serde::de::SeqAccess;

    #[test]
    fn recursive_key_identity_ignores_anchor_names_and_scalar_spelling() {
        let source = "&a [null, true, false, 1, -2, 1.5, !tag {child: *a}]";
        let equivalent = "&b [~, TRUE, FALSE, 01, -02, 1.50, !tag {'child': *b}]";
        let identity = super::canonical_key(source).unwrap();
        assert_eq!(identity, super::canonical_key(equivalent).unwrap());
        assert_ne!(
            identity,
            super::canonical_key(&source.replace("*a", "'*a'")).unwrap()
        );
    }

    #[test_case::test_case(18, true; "exact budget")]
    #[test_case::test_case(16, false; "second alias exceeds budget")]
    fn tagged_aliases_charge_the_key_byte_budget(bytes: usize, accepted: bool) {
        let source = "[&a !shared x, {? [*a, 1]: null, ? [*a, 2]: null}]";
        let stream = astral_yaml::parse(source).unwrap();
        let (id, entries, span) = stream
            .nodes()
            .iter()
            .enumerate()
            .find_map(|(index, node)| match &node.kind {
                astral_yaml::NodeKind::Mapping { entries, .. } => {
                    Some((astral_yaml::NodeId::from_index(index), entries, node.span))
                }
                _ => None,
            })
            .unwrap();
        let context = Context::new(super::Parsed::new(&stream, 0));
        context.scalar_budget.set(bytes);

        let result = context
            .deserializer(id)
            .check_mapping_keys(entries, Some(span));
        if accepted {
            result.unwrap();
            assert_eq!(context.scalar_budget.get(), 0);
        } else {
            let error = result.unwrap_err();
            let message = format!("{}: {}", error.path.unwrap(), error.message);
            let diagnostic =
                astral_yaml_dev::span_snapshot(source, [(message, error.span.unwrap())]);
            insta::assert_snapshot!(diagnostic, @"
              |
            1 | [&a !shared x, {? [*a, 1]: null, ? [*a, 2]: null}]
              |  ^^^^^^^^^^^^ $.key_at(1): expanded key tag byte limit exceeded
            ");
        }
    }

    #[test]
    fn metadata_protocols_can_be_ignored() {
        use serde::Deserializer as _;

        let stream = astral_yaml::parse("!Text hello").unwrap();
        let context = Context {
            input: super::Parsed::new(&stream, 0),
            budget: Cell::new(100),
            scalar_budget: Cell::new(100),
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
            crate::commented::NAME,
            crate::tagged::NAME,
            crate::with::yaml_tag::NAME,
            "Transparent",
        ] {
            context.scalar_budget.set(100);
            de.deserialize_newtype_struct(name, serde::de::IgnoredAny)
                .unwrap();
        }

        // The wrapper consumes one visit before the payload is requested.
        context.budget.set(1);
        assert!(
            de.deserialize_newtype_struct(crate::spanned::NAME, serde::de::IgnoredAny)
                .is_err()
        );
    }

    #[test]
    fn enum_variant_names_charge_the_byte_budget() {
        use serde::de::EnumAccess;
        use std::marker::PhantomData;

        let stream = astral_yaml::parse("!long-tag x").unwrap();

        for bytes in [7, 8] {
            let context = Context {
                input: super::Parsed::new(&stream, 0),
                budget: Cell::new(10),
                scalar_budget: Cell::new(bytes),
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
                value: VariantValue::Tagged,
            };

            match access.variant_seed(PhantomData::<String>) {
                Ok((name, _)) => {
                    assert_eq!(name, "long-tag");
                    assert_eq!(context.scalar_budget.get(), 0);
                }
                Err(error) => {
                    assert_eq!(bytes, 7);
                    let snippet = astral_yaml_dev::span_snapshot(
                        stream.source(),
                        [(error.message, error.span.unwrap())],
                    );
                    insta::assert_snapshot!(snippet, @"
                      |
                    1 | !long-tag x
                      | ^^^^^^^^^^^ expanded scalar byte limit exceeded
                    ");
                }
            }
        }
    }

    #[test]
    fn spans_do_not_index_comments() {
        let stream = astral_yaml::parse("# before\n42 # after").unwrap();
        let context = Context {
            input: super::Parsed::new(&stream, 0),
            budget: Cell::new(2),
            scalar_budget: Cell::new(100),
        };
        let de = Deserializer {
            context: &context,
            id: stream.documents()[0].root,
            depth: 0,
            tag_state: TagState::Unconsumed,
            yaml_tag: false,
        };
        assert_eq!(crate::Spanned::<u8>::deserialize(de).unwrap().value, 42);
    }

    #[test]
    fn nested_comment_lists_charge_each_emitted_record() {
        let stream = astral_yaml::parse("# leading\n42 # inline").unwrap();
        let mut spans = Vec::new();

        for budget in [2, 11, 12] {
            let visits = Cell::new(budget);
            let bytes = Cell::new(100);
            let de = Deserializer {
                context: &Context {
                    input: super::Parsed::new(&stream, 0),
                    budget: visits,
                    scalar_budget: bytes,
                },
                id: stream.documents()[0].root,
                depth: 0,
                tag_state: TagState::Unconsumed,
                yaml_tag: false,
            };

            let result = crate::Commented::<crate::Commented<'_, u8>>::deserialize(de);
            if budget == 12 {
                let value = result.unwrap();
                assert_eq!(value.comments.len(), 2);
                assert_eq!(value.comments, value.value.comments);
                assert_eq!(de.context.budget.get(), 0);
            } else {
                let error = result.unwrap_err();
                spans.push((
                    format!("budget {budget}: {}", error.message),
                    error.span.unwrap(),
                ));
            }
        }
        insta::assert_snapshot!(astral_yaml_dev::span_snapshot(stream.source(), spans), @"
          |
        1 | # leading
        2 | 42 # inline
          | ^^
          | |
          | budget 2: deserialization resource limit exceeded
          | budget 11: deserialization resource limit exceeded
        ");
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
                ..comment
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
                input: super::Parsed::new(&stream, 0),
                budget: visits,
                scalar_budget: bytes,
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
                input: super::Parsed::new(&stream, 0),
                budget: visits,
                scalar_budget: bytes,
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
