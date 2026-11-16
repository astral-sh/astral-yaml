use super::Value;
use crate::{Error, commented, spanned, tagged};
use serde::de::value::{
    BorrowedStrDeserializer, MapAccessDeserializer, MapDeserializer, SeqDeserializer,
};
use serde::de::{DeserializeOwned, EnumAccess, IntoDeserializer, Visitor};
use serde::{Deserializer, forward_to_deserialize_any};
use std::iter;

/// Deserialize an owned Rust type from a YAML value.
///
/// This reads the types stored in the tree. The original scalar spelling and
/// source metadata are unavailable. Use [`serde::Deserialize::deserialize`]
/// with `&Value` for a result that borrows from the tree.
///
/// Use [`crate::Tagged`] to read an application tag and its payload. Ordinary
/// enums use string names and single-entry mappings. Mapping entries are passed
/// to the destination in order, including duplicates in manually built values.
///
/// # Errors
///
/// Returns an error if the value cannot be deserialized as `T`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the convenience API consumes its input"
)]
pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T, Error> {
    T::deserialize(&value)
}

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

macro_rules! integers {
    ($($method:ident => $visit:ident),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                visitor.$visit(integer(self)?)
            }
        )*
    };
}

impl<'de> Deserializer<'de> for &'de Value {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::Integer(value) => match i64::try_from(*value) {
                Ok(value) => visitor.visit_i64(value),
                Err(_) => visitor.visit_i128(*value),
            },
            Value::Unsigned(value) => match u64::try_from(*value) {
                Ok(value) => visitor.visit_u64(value),
                Err(_) => visitor.visit_u128(*value),
            },
            Value::Float(value) => visitor.visit_f64(*value),
            Value::String(value) => visitor.visit_borrowed_str(value),
            Value::Sequence(items) => SeqDeserializer::new(items.iter()).deserialize_any(visitor),
            Value::Mapping(entries) => {
                MapDeserializer::new(entries.iter().map(|(key, value)| (key, value)))
                    .deserialize_any(visitor)
            }
            Value::Tagged(tagged) => visitor.visit_enum(variant(&tagged.tag, &tagged.value)),
        }
    }

    integers! {
        deserialize_i8 => visit_i8,
        deserialize_i16 => visit_i16,
        deserialize_i32 => visit_i32,
        deserialize_i64 => visit_i64,
        deserialize_i128 => visit_i128,
        deserialize_u8 => visit_u8,
        deserialize_u16 => visit_u16,
        deserialize_u32 => visit_u32,
        deserialize_u64 => visit_u64,
        deserialize_u128 => visit_u128,
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let number = match self {
            Value::Integer(value) => *value as f32,
            Value::Unsigned(value) => *value as f32,
            Value::Float(value) => *value as f32,
            _ => return Err(Error::new("expected a number")),
        };
        if number.is_infinite() && !matches!(self, Value::Float(value) if value.is_infinite()) {
            return Err(Error::new("float out of range"));
        }
        visitor.visit_f32(number)
    }

    #[allow(clippy::cast_precision_loss)]
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let number = match self {
            Value::Integer(value) => *value as f64,
            Value::Unsigned(value) => *value as f64,
            Value::Float(value) => *value,
            _ => return Err(Error::new("expected a number")),
        };
        visitor.visit_f64(number)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::String(value) => visitor.visit_borrowed_bytes(value.as_bytes()),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match name {
            tagged::NAME if matches!(self, Value::Tagged(_)) => self.deserialize_any(visitor),
            tagged::NAME => Err(Error::new("expected an explicit YAML tag")),
            spanned::NAME | commented::NAME => {
                Err(Error::new("a YAML value has no source metadata"))
            }
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        if let Value::String(tag) = self {
            return BorrowedStrDeserializer::new(tag).deserialize_enum(name, variants, visitor);
        }
        if let Some([(Value::String(tag), value)]) = self.as_mapping() {
            return visitor.visit_enum(variant(tag, value));
        }

        Err(Error::new(
            "expected an enum name or a single-entry mapping",
        ))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool char str string unit unit_struct seq tuple tuple_struct map struct identifier
    }
}

fn integer<T: TryFrom<i128> + TryFrom<u128>>(value: &Value) -> Result<T, Error> {
    let converted = match value {
        Value::Integer(value) => T::try_from(*value).ok(),
        Value::Unsigned(value) => T::try_from(*value).ok(),
        _ => return Err(Error::new("expected an integer")),
    };
    converted.ok_or_else(|| Error::new("integer out of range"))
}

fn variant<'de>(name: &'de str, payload: &'de Value) -> impl EnumAccess<'de, Error = Error> {
    let name = BorrowedStrDeserializer::new(name);
    MapAccessDeserializer::new(MapDeserializer::new(iter::once((name, payload))))
}
