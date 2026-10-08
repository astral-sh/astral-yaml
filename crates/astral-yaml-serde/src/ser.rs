mod keys;
mod tag;

use crate::{Error, schema::YamlType, tagged, with::yaml_tag};
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq},
};
use std::fmt::Write;

type Result<T> = std::result::Result<T, Error>;

const MAX_DEPTH: usize = 63;

/// Serializes a value as a YAML 1.2 document in flow style.
///
/// Strings are quoted to preserve their type under core-schema resolution.
/// Enums use a name for unit variants and a single-entry mapping otherwise.
///
/// # Errors
/// Returns an error for a tag/content mismatch, duplicate mapping keys, an
/// exceeded resource limit, or a `Serialize` implementation that violates the
/// map protocol.
#[inline]
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let mut serializer = Serializer {
        output: String::new(),
        depth: 0,
        tagged: false,
        core_tag: None,
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
    tagged: bool,
    core_tag: Option<YamlType>,
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
    fn quoted(&mut self, value: &str) {
        self.tagged = false;
        self.yaml_tag = false;
        if value.len() >= 64 {
            self.output.reserve(value.len() + 3);
            self.output.push('"');
            self.quoted_long(value);
            self.output.push('"');
            return;
        }

        // Check eight bytes at a time without alignment requirements. The last
        // word overlaps its predecessor when the length is not a multiple of eight.
        #[allow(clippy::needless_bitwise_bool)]
        let escaped = if value.len() >= 8 {
            let mut chunks = value.as_bytes().chunks_exact(8);
            let mut escaped = chunks.by_ref().fold(false, |escaped, chunk| {
                escaped | Self::word_needs_escape(u64::from_ne_bytes(chunk.try_into().unwrap()))
            });

            if !chunks.remainder().is_empty() {
                let last = &value.as_bytes()[value.len() - 8..];
                escaped |= Self::word_needs_escape(u64::from_ne_bytes(last.try_into().unwrap()));
            }

            escaped
        } else if value.len() >= 4 {
            let first = u32::from_ne_bytes(value.as_bytes()[..4].try_into().unwrap());
            let last = u32::from_ne_bytes(value.as_bytes()[value.len() - 4..].try_into().unwrap());
            Self::word_needs_escape(u64::from(first) | (u64::from(last) << 32))
        } else {
            value.bytes().fold(false, |escaped, byte| {
                escaped | !(b' '..=b'~').contains(&byte) | (byte == b'"') | (byte == b'\\')
            })
        };

        if self.output.is_empty() {
            self.output.reserve(value.len() + 3);
        }

        self.output.push('"');

        if escaped {
            self.escaped(value);
        } else {
            self.output.push_str(value);
        }

        self.output.push('"');
    }

    #[inline]
    fn word_needs_escape(word: u64) -> bool {
        const ONES: u64 = 0x0101_0101_0101_0101;
        const HIGH_BITS: u64 = 0x8080_8080_8080_8080;
        const SPACES: u64 = 0x2020_2020_2020_2020;

        // High bits detect non-ASCII bytes. Adding one detects DEL. Subtracting spaces
        // detects controls. Carries and borrows cannot hide the first offending byte,
        // and printable ASCII causes neither.
        let non_printable = word | word.wrapping_add(ONES) | word.wrapping_sub(SPACES);

        // XOR turns quotes and backslashes into zero bytes. The zero-byte test
        // sets a high bit if either character occurs anywhere in the word.
        let quotes = word ^ u64::from_ne_bytes([b'"'; 8]);
        let slashes = word ^ u64::from_ne_bytes([b'\\'; 8]);
        let escaped_quotes = quotes.wrapping_sub(ONES) & !quotes;
        let escaped_slashes = slashes.wrapping_sub(ONES) & !slashes;

        (non_printable | escaped_quotes | escaped_slashes) & HIGH_BITS != 0
    }

    #[inline(never)]
    fn quoted_long(&mut self, value: &str) {
        // Quotes, backslashes, and line breaks use memchr below. Other controls
        // and non-ASCII text use the full escape scanner.
        #[allow(clippy::needless_bitwise_bool)]
        let unusual = value.bytes().fold(false, |unusual, byte| {
            unusual | (!(b' '..=b'~').contains(&byte) & (byte != b'\n'))
        });

        if unusual {
            self.escaped(value);
        } else {
            let mut start = 0;
            for index in memchr::memchr3_iter(b'"', b'\\', b'\n', value.as_bytes()) {
                self.output.push_str(&value[start..index]);
                self.output.push_str(match value.as_bytes()[index] {
                    b'"' => "\\\"",
                    b'\\' => "\\\\",
                    _ => "\\n",
                });
                start = index + 1;
            }
            self.output.push_str(&value[start..]);
        }
    }

    #[cold]
    #[inline(never)]
    fn escaped(&mut self, mut value: &str) {
        const ESCAPE: [bool; 256] = {
            let mut escape = [false; 256];
            let mut byte = 0;

            while byte < escape.len() {
                escape[byte] =
                    byte < 0x20 || matches!(byte, 0x22 | 0x5c | 0x7f | 0xc2 | 0xe2 | 0xef);
                byte += 1;
            }

            escape
        };

        // Copy runs of text until a possible escape. The non-ASCII characters escaped
        // below start with C2, E2, or EF. Stopping at those leading bytes or at ASCII
        // preserves UTF-8 character boundaries.
        while let Some(index) = value.bytes().position(|byte| ESCAPE[usize::from(byte)]) {
            self.output.push_str(&value[..index]);
            let c = value[index..].chars().next().unwrap();

            match c {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                c if c.is_control()
                    || matches!(
                        c,
                        '\u{85}' | '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}'
                    ) =>
                {
                    // Writing to a String cannot fail.
                    write!(self.output, "\\u{:04X}", u32::from(c)).expect("String write");
                }
                c => self.output.push(c),
            }

            value = &value[index + c.len_utf8()..];
        }

        self.output.push_str(value);
    }

    #[inline]
    fn compound(&mut self, mapping: bool) -> Result<Compound<'_>> {
        // Nested collections without a core tag only need a depth increment.
        if self.depth == 0 || self.depth >= MAX_DEPTH || self.core_tag.is_some() {
            self.enter_compound(mapping)?;
        } else {
            self.depth += 1;
        }

        self.tagged = false;
        self.yaml_tag = false;
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
    fn enter_compound(&mut self, mapping: bool) -> Result<()> {
        self.check_collection_tag(mapping)?;
        self.enter()
    }

    fn variant(&mut self, name: &str, mapping: bool) -> Result<Compound<'_>> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(name)?;
            self.output.push(' ');
            return self.compound(mapping);
        }

        self.enter()?;
        self.check_collection_tag(true)?;
        self.output.push('{');
        self.quoted(name);
        self.output.push_str(": ");

        let mut compound = self.compound(mapping)?;
        compound.variant = true;
        Ok(compound)
    }

    fn number(&mut self, value: impl std::fmt::Display) -> Result<()> {
        self.tagged = false;
        self.yaml_tag = false;
        let start = self.output.len();
        write!(self.output, "{value}").expect("String write");
        self.check_written_scalar_tag(start)
    }

    // Most values have no explicit core tag, so only clear the slot when set.
    #[inline]
    fn check_scalar_tag(&mut self, value: &str) -> Result<()> {
        if let Some(tag) = self.core_tag {
            self.core_tag = None;
            Self::check_tagged_scalar(tag, value)?;
        }

        Ok(())
    }

    #[inline]
    fn check_written_scalar_tag(&mut self, start: usize) -> Result<()> {
        if let Some(tag) = self.core_tag {
            self.core_tag = None;
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

    fn check_collection_tag(&mut self, mapping: bool) -> Result<()> {
        let actual = if mapping {
            YamlType::Mapping
        } else {
            YamlType::Sequence
        };
        if let Some(tag) = self.core_tag {
            self.core_tag = None;
            if tag != actual {
                return Err(Error::new("serialized value does not match its YAML tag"));
            }
        }

        Ok(())
    }

    fn enum_tag(&mut self, variant: &str) -> Result<()> {
        if variant.is_empty() {
            return Err(Error::new("an enum tag requires a nonempty variant name"));
        }

        if self.tagged {
            return Err(Error::new("a YAML node cannot have multiple tags"));
        }

        let tag = ser::Serializer::serialize_str(tag::TagSerializer, &format!("!{variant}"))?;
        self.output.push_str(&tag);
        self.tagged = true;
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

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

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
        self.tagged = false;
        self.yaml_tag = false;
        self.output.push_str(value);
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        if value.is_finite() {
            let start = self.output.len();
            self.tagged = false;
            self.yaml_tag = false;
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
        self.tagged = false;
        self.yaml_tag = false;
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
        self.quoted(value);
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
        self.tagged = false;
        self.yaml_tag = false;
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        if std::mem::take(&mut self.yaml_tag) {
            self.enum_tag(variant)?;
            self.tagged = false;
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

    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a>> {
        self.compound(false)
    }

    fn serialize_tuple(self, _: usize) -> Result<Compound<'a>> {
        self.compound(false)
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Compound<'a>> {
        if name == tagged::NAME {
            if len != 2 {
                return Err(Error::new("a tagged value requires a tag and payload"));
            }

            if self.tagged {
                return Err(Error::new("a YAML node cannot have multiple tags"));
            }

            self.enter()?;
            self.tagged = true;
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
    ) -> Result<Compound<'a>> {
        self.variant(variant, false)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a>> {
        self.compound(true)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>> {
        self.compound(true)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'a>> {
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

struct Compound<'a> {
    serializer: &'a mut Serializer,
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
                self.serializer.core_tag = tag
                    .strip_prefix("!<")
                    .and_then(|tag| tag.strip_suffix('>'))
                    .and_then(YamlType::from_tag);
                self.serializer.output.push_str(&tag);
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
        self.serializer.output.push_str("? ");
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
