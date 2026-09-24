use crate::Error;
use astral_yaml::{
    Entry, Node, NodeId, NodeKind, Scalar, ScalarStyle, Span, Stream,
    schema::{ScalarType, classify},
};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::{borrow::Cow, cell::Cell};

type Result<T> = std::result::Result<T, Error>;

/// Deserializes exactly one YAML document using YAML 1.2 core-schema resolution.
///
/// Plain and quoted scalars without escapes can be borrowed from the input.
/// Aliases are visited with depth and work budgets, including cyclic aliases.
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
    let budget = Cell::new(1_000_000usize);
    T::deserialize(Deserializer {
        stream,
        id: doc.root,
        depth: 0,
        budget: &budget,
        ignore_tag: false,
    })
    .map_err(|error| error.located(stream.node(doc.root).map_or(doc.span, |node| node.span)))
}

#[derive(Clone, Copy)]
struct Deserializer<'s, 'de> {
    stream: &'s Stream<'de>,
    id: NodeId,
    depth: usize,
    budget: &'s Cell<usize>,
    ignore_tag: bool,
}
impl<'s, 'de> Deserializer<'s, 'de> {
    fn node(self) -> Result<&'s Node<'de>> {
        let mut id = self.id;
        for _ in 0..128 {
            let node = self
                .stream
                .node(id)
                .ok_or_else(|| Error::new("invalid node ID"))?;
            if self.depth >= 64 || self.budget.get() == 0 {
                return Err(Error::at(
                    "deserialization resource limit exceeded",
                    node.span,
                ));
            }
            self.budget.set(self.budget.get() - 1);
            if let NodeKind::Alias { target, .. } = node.kind {
                id = target;
            } else {
                return Ok(node);
            }
        }
        Err(Error::new("alias chain limit exceeded"))
    }
    fn child(self, id: NodeId) -> Self {
        Self {
            id,
            depth: self.depth + 1,
            ignore_tag: false,
            ..self
        }
    }
    fn scalar(self) -> Result<(&'s Scalar<'de>, ScalarType)> {
        let node = self.node()?;
        let NodeKind::Scalar(s) = &node.kind else {
            return Err(Error::at("expected a scalar", node.span));
        };
        let tag = if self.ignore_tag {
            None
        } else {
            node.tag.as_ref().map(|t| t.value.as_ref())
        };
        let kind = match tag {
            Some("tag:yaml.org,2002:str" | "!") => ScalarType::String,
            Some("tag:yaml.org,2002:null") => ScalarType::Null,
            Some("tag:yaml.org,2002:bool") => ScalarType::Bool,
            Some("tag:yaml.org,2002:int") => ScalarType::Int,
            Some("tag:yaml.org,2002:float") => ScalarType::Float,
            Some(_) => return Err(Error::at("unsupported scalar tag", node.span)),
            None if s.style == ScalarStyle::Plain => classify(&s.value),
            None => ScalarType::String,
        };
        if kind != ScalarType::String
            && classify(&s.value) != kind
            && !(kind == ScalarType::Float && classify(&s.value) == ScalarType::Int)
        {
            return Err(Error::at("scalar does not match its tag", s.span));
        }
        Ok((s, kind))
    }
    fn text<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, _) = self.scalar()?;
        match &s.value {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_str(s),
        }
    }
    fn integer(self) -> Result<(&'s str, u32, Span)> {
        let (s, kind) = self.scalar()?;
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
        let (s, kind) = self.scalar()?;
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
        if kind == ScalarType::Int && (s.value.starts_with("0x") || s.value.starts_with("0o")) {
            let (value, radix, span) = self.integer()?;
            let value = u128::from_str_radix(value, radix)
                .map_err(|_| Error::at("integer out of range", span))?;
            // Parsing the decimal representation gives the same rounded conversion as other numbers.
            return value
                .to_string()
                .parse()
                .map_err(|_| Error::at("invalid float", span));
        }
        s.value
            .parse()
            .map_err(|_| Error::at("invalid float", s.span))
    }
    fn map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let node = self.node()?;
        let NodeKind::Mapping { entries, .. } = &node.kind else {
            return Err(Error::at("expected a mapping", node.span));
        };
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
        let node = self.node()?;
        let NodeKind::Sequence { items, .. } = &node.kind else {
            return Err(Error::at("expected a sequence", node.span));
        };
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

macro_rules! signed {
    ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {$ (
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (value, radix, span) = self.integer()?;
            let number = <$ty>::from_str_radix(value, radix).map_err(|_| Error::at("integer out of range", span))?;
            visitor.$visit(number)
        }
    )*};
}

impl<'de> de::Deserializer<'de> for Deserializer<'_, 'de> {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.node()?.kind {
            NodeKind::Sequence { .. } => self.seq(visitor),
            NodeKind::Mapping { .. } => self.map(visitor),
            NodeKind::Scalar(_) => {
                let (s, kind) = self.scalar()?;
                match kind {
                    ScalarType::Null => visitor.visit_unit(),
                    ScalarType::Bool => {
                        visitor.visit_bool(matches!(s.value.as_ref(), "true" | "True" | "TRUE"))
                    }
                    ScalarType::String => self.text(visitor),
                    ScalarType::Float => visitor.visit_f64(self.float()?),
                    ScalarType::Int => {
                        let (value, radix, span) = self.integer()?;
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
            NodeKind::Alias { .. } => Err(Error::new("unresolved alias")),
        }
    }
    signed! {
        deserialize_i8 => visit_i8: i8, deserialize_i16 => visit_i16: i16,
        deserialize_i32 => visit_i32: i32, deserialize_i64 => visit_i64: i64,
        deserialize_i128 => visit_i128: i128,
        deserialize_u8 => visit_u8: u8, deserialize_u16 => visit_u16: u16,
        deserialize_u32 => visit_u32: u32, deserialize_u64 => visit_u64: u64,
        deserialize_u128 => visit_u128: u128,
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_f64(visitor)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.float()?)
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, kind) = self.scalar()?;
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
        let (s, _) = self.scalar()?;
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
        let (s, _) = self.scalar()?;
        match &s.value {
            Cow::Borrowed(s) => visitor.visit_borrowed_bytes(s.as_bytes()),
            Cow::Owned(s) => visitor.visit_bytes(s.as_bytes()),
        }
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if matches!(self.node()?.kind, NodeKind::Scalar(_)) && self.scalar()?.1 == ScalarType::Null
        {
            visitor.visit_none()
        } else {
            visitor.visit_some(self.child(self.id))
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let (s, kind) = self.scalar()?;
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
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self.child(self.id))
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
        if let Some(tag) = node
            .tag
            .as_ref()
            .and_then(|tag| tag.value.strip_prefix('!'))
        {
            if !tag.is_empty() {
                return visitor.visit_enum(Enumeration {
                    de: self,
                    variant: tag,
                    value: Some(self.id),
                    tagged: true,
                });
            }
        }
        match &node.kind {
            NodeKind::Scalar(s) => visitor.visit_enum(Enumeration {
                de: self,
                variant: &s.value,
                value: None,
                tagged: false,
            }),
            NodeKind::Mapping { entries, .. } if entries.len() == 1 => {
                let (key, _) = self.child(entries[0].key).scalar()?;
                visitor.visit_enum(Enumeration {
                    de: self,
                    variant: &key.value,
                    value: Some(entries[0].value),
                    tagged: false,
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
        visitor.visit_unit()
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
struct Enumeration<'s, 'de> {
    de: Deserializer<'s, 'de>,
    variant: &'s str,
    value: Option<NodeId>,
    tagged: bool,
}
impl<'de> EnumAccess<'de> for Enumeration<'_, 'de> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let variant = seed.deserialize(de::value::StrDeserializer::<Error>::new(self.variant))?;
        Ok((variant, self))
    }
}
impl<'de> VariantAccess<'de> for Enumeration<'_, 'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        match self.value {
            None => Ok(()),
            Some(id) if self.tagged => {
                let node = self
                    .de
                    .stream
                    .node(id)
                    .ok_or_else(|| Error::new("invalid enum ID"))?;
                if matches!(&node.kind, NodeKind::Scalar(s) if s.value.is_empty()) {
                    Ok(())
                } else {
                    Err(Error::at("expected an empty unit variant", node.span))
                }
            }
            Some(id) => <()>::deserialize(self.de.child(id)),
        }
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let id = self
            .value
            .ok_or_else(|| Error::new("expected a variant payload"))?;
        seed.deserialize(Deserializer {
            ignore_tag: self.tagged,
            ..self.de.child(id)
        })
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        let id = self
            .value
            .ok_or_else(|| Error::new("expected a variant payload"))?;
        self.de.child(id).seq(visitor)
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let id = self
            .value
            .ok_or_else(|| Error::new("expected a variant payload"))?;
        self.de.child(id).map(visitor)
    }
}
