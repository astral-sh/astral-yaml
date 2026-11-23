use super::Value;
use crate::{Error, commented, de, spanned, tagged};
use astral_yaml::Budget;
use serde::de::value::{
    BorrowedStrDeserializer, MapAccessDeserializer, MapDeserializer, SeqDeserializer,
};
use serde::de::{EnumAccess, IntoDeserializer, Visitor};
use serde::{Deserializer, forward_to_deserialize_any};
use std::iter;

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ValueDeserializer<'budget, 'de> {
    value: &'de Value,
    budget: &'budget Budget,
    depth: usize,
}

impl Value {
    pub(crate) fn deserializer<'budget, 'de>(
        &'de self,
        budget: &'budget Budget,
    ) -> ValueDeserializer<'budget, 'de> {
        ValueDeserializer {
            value: self,
            budget,
            depth: 0,
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ValueDeserializer<'_, 'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

// Start a budget for a bare value and charge once per Serde call, including
// when a wrapper asks to deserialize the same value again.
macro_rules! deserialize {
    (
        $de:ident, $visitor:ident;
        any: $($any:ident)*;
        $($method:ident($($arg:ident: $ty:ty),*) => $body:expr),+ $(,)?
    ) => {
        impl<'de> Deserializer<'de> for &'de Value {
            type Error = Error;

            $(
                fn $method<V: Visitor<'de>>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Error> {
                    let budget = Budget::new(de::DEFAULT_WORK_LIMIT);
                    self.deserializer(&budget).$method($($arg,)* visitor)
                }
            )+

            forward_to_deserialize_any! { $($any)* }
        }

        impl<'de> Deserializer<'de> for ValueDeserializer<'_, 'de> {
            type Error = Error;

            $(
                fn $method<V: Visitor<'de>>($de, $($arg: $ty,)* $visitor: V) -> Result<V::Value, Error> {
                    de::charge_visit($de.depth, $de.budget)?;
                    $body
                }
            )+

            forward_to_deserialize_any! { $($any)* }
        }
    };
}

deserialize! {
    self, visitor;
    any: bool char str string unit unit_struct seq tuple tuple_struct identifier;

    deserialize_any() => self.visit_any(visitor),
    deserialize_i8() => visitor.visit_i8(integer(self.value)?),
    deserialize_i16() => visitor.visit_i16(integer(self.value)?),
    deserialize_i32() => visitor.visit_i32(integer(self.value)?),
    deserialize_i64() => visitor.visit_i64(integer(self.value)?),
    deserialize_i128() => visitor.visit_i128(integer(self.value)?),
    deserialize_u8() => visitor.visit_u8(integer(self.value)?),
    deserialize_u16() => visitor.visit_u16(integer(self.value)?),
    deserialize_u32() => visitor.visit_u32(integer(self.value)?),
    deserialize_u64() => visitor.visit_u64(integer(self.value)?),
    deserialize_u128() => visitor.visit_u128(integer(self.value)?),
    deserialize_f32() => self.visit_f32(visitor),
    deserialize_f64() => self.visit_f64(visitor),
    deserialize_bytes() => self.visit_bytes(visitor),
    deserialize_byte_buf() => self.visit_bytes(visitor),
    deserialize_option() => self.visit_option(visitor),
    deserialize_newtype_struct(name: &'static str) => self.visit_newtype_struct(name, visitor),
    deserialize_map() => self.visit_map(visitor),
    deserialize_struct(_name: &'static str, _fields: &'static [&'static str]) =>
        self.visit_map(visitor),
    deserialize_enum(name: &'static str, variants: &'static [&'static str]) =>
        self.visit_enum(name, variants, visitor),
    deserialize_ignored_any() => visitor.visit_unit(),
}

impl<'de> ValueDeserializer<'_, 'de> {
    fn child(self, value: &'de Value) -> Self {
        Self {
            value,
            depth: self.depth + 1,
            ..self
        }
    }

    fn visit_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
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
            Value::Sequence(items) => {
                SeqDeserializer::new(items.iter().map(|item| self.child(item)))
                    .deserialize_any(visitor)
            }
            Value::Mapping(entries) => {
                let entries = entries
                    .iter()
                    .map(|(key, value)| (self.child(key), self.child(value)));
                MapDeserializer::new(entries).deserialize_any(visitor)
            }
            Value::Tagged(tagged) => {
                // A tag and its payload belong to the same YAML node. In a
                // manually built tree, each additional tag adds a level.
                let value = &tagged.value;
                let payload = if matches!(value, Value::Tagged(_)) {
                    self.child(value)
                } else {
                    Self { value, ..self }
                };
                visitor.visit_enum(variant(&tagged.tag, payload))
            }
        }
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn visit_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let number = match self.value {
            Value::Integer(value) => *value as f32,
            Value::Unsigned(value) => *value as f32,
            Value::Float(value) => *value as f32,
            _ => return Err(Error::new("expected a number")),
        };
        if number.is_infinite() && !self.value.as_f64().is_some_and(f64::is_infinite) {
            return Err(Error::new("float out of range"));
        }
        visitor.visit_f32(number)
    }

    #[allow(clippy::cast_precision_loss)]
    fn visit_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let number = match self.value {
            Value::Integer(value) => *value as f64,
            Value::Unsigned(value) => *value as f64,
            Value::Float(value) => *value,
            _ => return Err(Error::new("expected a number")),
        };
        visitor.visit_f64(number)
    }

    fn visit_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            Value::String(value) => visitor.visit_borrowed_bytes(value.as_bytes()),
            _ => self.visit_any(visitor),
        }
    }

    fn visit_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self.child(self.value)),
        }
    }

    fn visit_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match name {
            super::NAME => self.visit_any(visitor),
            tagged::NAME if matches!(self.value, Value::Tagged(_)) => self.visit_any(visitor),
            tagged::NAME => Err(Error::new("expected an explicit YAML tag")),
            spanned::NAME | commented::NAME => {
                Err(Error::new("a YAML value has no source metadata"))
            }
            _ => visitor.visit_newtype_struct(self.child(self.value)),
        }
    }

    fn visit_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if !matches!(self.value, Value::Mapping(_)) {
            return Err(Error::new("expected a mapping"));
        }
        self.visit_any(visitor)
    }

    fn visit_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        if let Value::String(tag) = self.value {
            return BorrowedStrDeserializer::new(tag).deserialize_enum(name, variants, visitor);
        }
        if let Some([(Value::String(tag), value)]) = self.value.as_mapping() {
            return visitor.visit_enum(variant(tag, self.child(value)));
        }

        Err(Error::new(
            "expected an enum name or a single-entry mapping",
        ))
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

fn variant<'de>(
    name: &'de str,
    payload: ValueDeserializer<'_, 'de>,
) -> impl EnumAccess<'de, Error = Error> {
    let name = BorrowedStrDeserializer::new(name);
    MapAccessDeserializer::new(MapDeserializer::new(iter::once((name, payload))))
}

#[cfg(test)]
mod tests {
    use crate::{Value, from_str};
    use astral_yaml::Budget;
    use serde::Deserialize;

    #[test]
    fn each_value_in_a_tagged_mapping_costs_one_visit() {
        let value = from_str::<Value>("{key: !Tag [one, two]}").unwrap();
        let budget = Budget::new(6);
        Value::deserialize(value.deserializer(&budget)).unwrap();
        assert_eq!(budget.remaining(), 0);
    }

    #[test]
    fn enum_payloads_share_the_budget() {
        #[derive(Debug, Deserialize)]
        enum Choice {
            Unit(()),
        }

        let value = from_str::<Value>("{Unit: null}").unwrap();
        let budget = Budget::new(1);
        let error = Choice::deserialize(value.deserializer(&budget)).unwrap_err();
        insta::assert_snapshot!(error, @"deserialization resource limit exceeded");
    }
}
