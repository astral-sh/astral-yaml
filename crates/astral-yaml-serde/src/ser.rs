mod keys;
mod quoted;
mod tag;

pub(crate) use quoted::write_quoted;

use crate::schema::YamlType;
use crate::with::yaml_tag;
use crate::{Error, tagged};
use serde::Serialize;
use serde::ser::{self, SerializeMap, SerializeSeq};
use std::fmt::Write;

type Result<T> = std::result::Result<T, Error>;

const MAX_DEPTH: usize = 63;
const EXPLICIT_KEY: &str = "? ";

/// Serialize `value` as a YAML 1.2 document in flow style.
///
/// Strings are quoted to preserve their type. Unit enum variants use their name,
/// and variants with data use a mapping with the variant name as its key.
///
/// # Errors
/// Propagates errors from `Serialize`. Also returns an error if a tag conflicts
/// with its value, a mapping repeats a key, keys and values are not paired,
/// or a resource limit is exceeded.
#[inline]
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let mut serializer = Serializer {
        output: String::new(),
        depth: 0,
        pending_tag: None,
        yaml_tag: false,
        keys: keys::Keys::default(),
    };

    value.serialize(&mut serializer)?;
    serializer.output.push('\n');
    Ok(serializer.output)
}

struct Serializer {
    output: String,
    depth: usize,
    pending_tag: Option<tag::Constraint>,
    yaml_tag: bool,
    keys: keys::Keys,
}

impl Serializer {
    fn enter(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            return Err(Error::new("serialization nesting limit exceeded"));
        }

        if self.depth == 0 {
            self.output.reserve(1024);
        }

        self.depth += 1;
        Ok(())
    }

    #[inline]
    fn compound(&mut self, mapping: bool) -> Result<Compound<'_>> {
        // Nested collections without a core tag only need a depth increment.
        let tag = self.take_core_tag();
        if self.depth == 0 || self.depth >= MAX_DEPTH || tag.is_some() {
            self.enter_compound(mapping, tag)?;
        } else {
            self.depth += 1;
        }

        self.output.push(if mapping { '{' } else { '[' });
        let key_scope = self.keys.scope();
        Ok(Compound {
            serializer: self,
            first: true,
            key_scope,
            state: if mapping {
                CompoundState::MapKey
            } else {
                CompoundState::Sequence
            },
            variant: false,
        })
    }

    #[cold]
    fn enter_compound(&mut self, mapping: bool, tag: Option<YamlType>) -> Result<()> {
        Self::check_collection_tag(tag, mapping)?;
        self.enter()
    }

    fn variant(&mut self, name: &str, mapping: bool) -> Result<Compound<'_>> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(name)?;
            self.output.push(' ');
            return self.compound(mapping);
        }

        self.enter()?;
        Self::check_collection_tag(self.take_core_tag(), true)?;
        self.output.push('{');
        write_quoted(&mut self.output, name);
        self.output.push_str(": ");

        let mut compound = self.compound(mapping)?;
        compound.variant = true;
        Ok(compound)
    }

    fn number(&mut self, value: impl std::fmt::Display) -> Result<()> {
        let start = self.output.len();
        write!(self.output, "{value}").expect("String write");
        self.check_written_scalar_tag(start)
    }

    /// Consume the current node's tag state and return any core type constraint.
    #[inline]
    fn take_core_tag(&mut self) -> Option<YamlType> {
        self.yaml_tag = false;
        match self.pending_tag.take() {
            Some(tag::Constraint::Core(kind)) => Some(kind),
            Some(tag::Constraint::Any) | None => None,
        }
    }

    #[inline]
    fn check_scalar_tag(&mut self, value: &str) -> Result<()> {
        if let Some(tag) = self.take_core_tag() {
            Self::check_tagged_scalar(tag, value)?;
        }

        Ok(())
    }

    #[inline]
    fn check_written_scalar_tag(&mut self, start: usize) -> Result<()> {
        if let Some(tag) = self.take_core_tag() {
            Self::check_tagged_scalar(tag, &self.output[start..])?;
        }

        Ok(())
    }

    #[cold]
    fn check_tagged_scalar(tag: YamlType, value: &str) -> Result<()> {
        if !tag.matches_scalar(value) {
            return Err(Error::new("serialized value does not match its YAML tag"));
        }

        Ok(())
    }

    fn check_collection_tag(tag: Option<YamlType>, mapping: bool) -> Result<()> {
        if let Some(tag) = tag {
            let actual = if mapping {
                YamlType::Mapping
            } else {
                YamlType::Sequence
            };
            if tag != actual {
                return Err(Error::new("serialized value does not match its YAML tag"));
            }
        }

        Ok(())
    }

    fn require_untagged(&self) -> Result<()> {
        if self.pending_tag.is_some() {
            return Err(Error::new("a YAML node cannot have multiple tags"));
        }

        Ok(())
    }

    fn write_tag(&mut self, tag: tag::Tag) {
        self.pending_tag = Some(tag.write(&mut self.output));
    }

    fn enum_tag(&mut self, variant: &str) -> Result<()> {
        if variant.is_empty() {
            return Err(Error::new("an enum tag requires a nonempty variant name"));
        }

        self.require_untagged()?;
        self.write_tag(tag::Tag::parse(&format!("!{variant}"))?);
        Ok(())
    }
}

macro_rules! integer {
    ($($method:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method(self, value: $ty) -> Result<()> {
                self.number(value)
            }
        )*
    };
}

impl<'ser> ser::Serializer for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'ser>;
    type SerializeTuple = Compound<'ser>;
    type SerializeTupleStruct = Compound<'ser>;
    type SerializeTupleVariant = Compound<'ser>;
    type SerializeMap = Compound<'ser>;
    type SerializeStruct = Compound<'ser>;
    type SerializeStructVariant = Compound<'ser>;

    integer! {
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
    }

    fn serialize_bool(self, value: bool) -> Result<()> {
        let value = if value { "true" } else { "false" };
        self.check_scalar_tag(value)?;
        self.output.push_str(value);
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        if value.is_finite() {
            let start = self.output.len();
            write!(self.output, "{value}").expect("String write");
            if !self.output[start..].contains(['.', 'e', 'E']) {
                self.output.push_str(".0");
            }

            self.check_written_scalar_tag(start)
        } else {
            self.serialize_f64(f64::from(value))
        }
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        let start = self.output.len();
        if value.is_nan() {
            self.output.push_str(".nan");
        } else if value.is_infinite() {
            self.output.push_str(if value.is_sign_negative() {
                "-.inf"
            } else {
                ".inf"
            });
        } else {
            write!(self.output, "{value}").expect("String write");
            if !self.output[start..].contains(['.', 'e', 'E']) {
                self.output.push_str(".0");
            }
        }

        self.check_written_scalar_tag(start)
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_str(value.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.check_scalar_tag(value)?;
        write_quoted(&mut self.output, value);
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let mut seq = self.compound(false)?;

        for byte in value {
            seq.separator();
            seq.serializer.number(byte)?;
        }

        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<()> {
        self.enter()?;
        value.serialize(&mut *self)?;
        self.depth -= 1;
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        self.check_scalar_tag("null")?;
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        if std::mem::take(&mut self.yaml_tag) {
            // A node immediately after the explicit-key indicator will be followed
            // by a colon. Separate it from the bare tag.
            let is_key = self.output.ends_with(EXPLICIT_KEY);
            self.enum_tag(variant)?;
            self.pending_tag = None;
            if is_key {
                self.output.push(' ');
            }
            return Ok(());
        }

        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<()> {
        let previous = self.yaml_tag;
        self.yaml_tag |= name == yaml_tag::NAME;
        let result = self.serialize_some(value);
        self.yaml_tag = previous;
        result
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(variant)?;
            self.output.push(' ');
            return self.serialize_some(value);
        }

        let mut map = self.compound(true)?;
        map.serialize_entry(variant, value)?;
        SerializeMap::end(map)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'ser>> {
        self.compound(false)
    }

    fn serialize_tuple(self, _: usize) -> Result<Compound<'ser>> {
        self.compound(false)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Compound<'ser>> {
        if name == tagged::NAME {
            if len != 2 {
                return Err(Error::new("a tagged value requires a tag and payload"));
            }

            self.require_untagged()?;
            self.enter()?;
            self.yaml_tag = false;
            let key_scope = self.keys.scope();
            return Ok(Compound {
                serializer: self,
                first: true,
                key_scope,
                state: CompoundState::Tag,
                variant: false,
            });
        }

        self.compound(false)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'ser>> {
        self.variant(variant, false)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'ser>> {
        self.compound(true)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'ser>> {
        self.compound(true)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'ser>> {
        self.variant(variant, true)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CompoundState {
    Sequence,
    MapKey,
    MapValue,
    Tag,
    TaggedValue,
    TaggedEnd,
}

struct Compound<'ser> {
    serializer: &'ser mut Serializer,
    first: bool,
    key_scope: keys::Scope,
    state: CompoundState,
    variant: bool,
}

impl Compound<'_> {
    fn separator(&mut self) {
        if !self.first {
            self.serializer.output.push_str(", ");
        }

        self.first = false;
    }

    fn finish(self) -> Result<()> {
        match self.state {
            CompoundState::Tag | CompoundState::TaggedValue => {
                return Err(Error::new("a tagged value requires a tag and payload"));
            }
            CompoundState::TaggedEnd => {
                self.serializer.depth -= 1;
                return Ok(());
            }
            _ => {}
        }

        if self.state == CompoundState::MapValue {
            return Err(Error::new("mapping key has no value"));
        }

        if self.state == CompoundState::MapKey {
            self.serializer
                .keys
                .finish(&self.serializer.output, self.key_scope)?;
        }

        self.serializer
            .output
            .push(if self.state == CompoundState::Sequence {
                ']'
            } else {
                '}'
            });
        self.serializer.depth -= 1;

        if self.variant {
            self.serializer.output.push('}');
            self.serializer.depth -= 1;
        }

        Ok(())
    }
}

impl SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        match self.state {
            CompoundState::Tag => {
                let tag = value.serialize(tag::TagSerializer)?;
                self.serializer.write_tag(tag);
                self.serializer.output.push(' ');
                self.state = CompoundState::TaggedValue;
                return Ok(());
            }
            CompoundState::TaggedValue => {
                value.serialize(&mut *self.serializer)?;
                self.state = CompoundState::TaggedEnd;
                return Ok(());
            }
            CompoundState::TaggedEnd => {
                return Err(Error::new("a tagged value has more than two fields"));
            }
            _ => {}
        }

        self.separator();
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<()> {
        self.finish()
    }
}

macro_rules! sequence {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = Error;

            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
                SerializeSeq::serialize_element(self, value)
            }

            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}

sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);

impl SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<()> {
        if self.state == CompoundState::MapValue {
            return Err(Error::new("mapping key has no value"));
        }

        self.separator();
        self.serializer.output.push_str(EXPLICIT_KEY);
        let start = self.serializer.output.len();
        key.serialize(&mut *self.serializer)?;
        let signature = self
            .serializer
            .keys
            .insert(&self.serializer.output, start..self.serializer.output.len())?;
        self.key_scope.insert(signature);
        self.serializer.output.push_str(": ");
        self.state = CompoundState::MapValue;
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        if self.state != CompoundState::MapValue {
            return Err(Error::new("mapping value has no key"));
        }

        value.serialize(&mut *self.serializer)?;
        self.state = CompoundState::MapKey;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.finish()
    }
}

macro_rules! structure {
    ($trait:ident) => {
        impl ser::$trait for Compound<'_> {
            type Ok = ();
            type Error = Error;

            fn serialize_field<T: Serialize + ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<()> {
                self.serialize_entry(key, value)
            }

            fn end(self) -> Result<()> {
                self.finish()
            }
        }
    };
}

structure!(SerializeStruct);
structure!(SerializeStructVariant);
