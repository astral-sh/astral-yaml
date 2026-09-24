use crate::Error;
use serde::{
    Serialize,
    ser::{self, SerializeMap, SerializeSeq},
};
use std::fmt::Write;

type Result<T> = std::result::Result<T, Error>;

/// Serializes a value as a YAML 1.2 document in flow style.
///
/// Strings are quoted to preserve their type under core-schema resolution.
/// Enums use a name for unit variants and a single-entry mapping otherwise.
///
/// # Errors
/// Returns an error if the value exceeds the nesting limit or its `Serialize`
/// implementation violates the map protocol.
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let mut serializer = Serializer {
        output: String::new(),
        depth: 0,
    };
    value.serialize(&mut serializer)?;
    serializer.output.push('\n');
    Ok(serializer.output)
}

struct Serializer {
    output: String,
    depth: usize,
}
impl Serializer {
    fn enter(&mut self) -> Result<()> {
        if self.depth >= 64 {
            return Err(Error::new("serialization nesting limit exceeded"));
        }
        self.depth += 1;
        Ok(())
    }
    fn quoted(&mut self, value: &str) {
        self.output.push('"');
        for c in value.chars() {
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
        }
        self.output.push('"');
    }
    fn compound(&mut self, mapping: bool) -> Result<Compound<'_>> {
        self.enter()?;
        self.output.push(if mapping { '{' } else { '[' });
        Ok(Compound {
            serializer: self,
            first: true,
            state: if mapping {
                CompoundState::MapKey
            } else {
                CompoundState::Sequence
            },
            variant: false,
        })
    }
    fn variant(&mut self, name: &str, mapping: bool) -> Result<Compound<'_>> {
        self.enter()?;
        self.output.push('{');
        self.quoted(name);
        self.output.push_str(": ");
        let mut compound = self.compound(mapping)?;
        compound.variant = true;
        Ok(compound)
    }
    fn number(&mut self, value: impl std::fmt::Display) {
        write!(self.output, "{value}").expect("String write");
    }
}
macro_rules! integer {
    ($($method:ident : $ty:ty),* $(,)?) => {$ (
        fn $method(self, value: $ty) -> Result<()> { self.number(value); Ok(()) }
    )*};
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
        serialize_i8: i8, serialize_i16: i16, serialize_i32: i32, serialize_i64: i64, serialize_i128: i128,
        serialize_u8: u8, serialize_u16: u16, serialize_u32: u32, serialize_u64: u64, serialize_u128: u128,
    }
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.output.push_str(if value { "true" } else { "false" });
        Ok(())
    }
    fn serialize_f32(self, value: f32) -> Result<()> {
        if value.is_finite() {
            let text = value.to_string();
            self.output.push_str(&text);
            if !text.contains(['.', 'e', 'E']) {
                self.output.push_str(".0");
            }
            Ok(())
        } else {
            self.serialize_f64(f64::from(value))
        }
    }
    fn serialize_f64(self, value: f64) -> Result<()> {
        if value.is_nan() {
            self.output.push_str(".nan");
        } else if value.is_infinite() {
            self.output.push_str(if value.is_sign_negative() {
                "-.inf"
            } else {
                ".inf"
            });
        } else {
            let text = value.to_string();
            self.output.push_str(&text);
            if !text.contains(['.', 'e', 'E']) {
                self.output.push_str(".0");
            }
        }
        Ok(())
    }
    fn serialize_char(self, value: char) -> Result<()> {
        self.quoted(value.encode_utf8(&mut [0; 4]));
        Ok(())
    }
    fn serialize_str(self, value: &str) -> Result<()> {
        self.quoted(value);
        Ok(())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let mut seq = self.compound(false)?;
        for byte in value {
            seq.serialize_element(byte)?;
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
        self.output.push_str("null");
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<()> {
        self.serialize_some(value)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
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
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Compound<'a>> {
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
}

struct Compound<'a> {
    serializer: &'a mut Serializer,
    first: bool,
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
        if self.state == CompoundState::MapValue {
            return Err(Error::new("mapping key has no value"));
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
        key.serialize(&mut *self.serializer)?;
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
