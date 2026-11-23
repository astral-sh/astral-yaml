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

// Start a budget at the public entry point. Nested visitors share it.
macro_rules! delegate {
    ($($method:ident $(($($arg:ident: $ty:ty),+))?),+ $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, $($($arg: $ty,)+)? visitor: V) -> Result<V::Value, Error> {
                let budget = Budget::new(de::DEFAULT_WORK_LIMIT);
                self.deserializer(&budget).$method($($($arg,)+)? visitor)
            }
        )+
    };
}

impl<'de> Deserializer<'de> for &'de Value {
    type Error = Error;

    delegate! {
        deserialize_any,
        deserialize_i8, deserialize_i16, deserialize_i32, deserialize_i64, deserialize_i128,
        deserialize_u8, deserialize_u16, deserialize_u32, deserialize_u64, deserialize_u128,
        deserialize_f32, deserialize_f64,
        deserialize_bytes, deserialize_byte_buf,
        deserialize_option,
        deserialize_newtype_struct(name: &'static str),
        deserialize_map,
        deserialize_struct(name: &'static str, fields: &'static [&'static str]),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
        deserialize_ignored_any,
    }

    forward_to_deserialize_any! {
        bool char str string unit unit_struct seq tuple tuple_struct identifier
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

impl<'de> ValueDeserializer<'_, 'de> {
    fn value(self) -> Result<&'de Value, Error> {
        de::charge_visit(self.depth, self.budget)?;
        Ok(self.value)
    }

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
                visitor.visit_enum(variant(&tagged.tag, self.child(&tagged.value)))
            }
        }
    }
}

macro_rules! integers {
    ($($method:ident => $visit:ident),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                visitor.$visit(integer(self.value()?)?)
            }
        )*
    };
}

impl<'de> Deserializer<'de> for ValueDeserializer<'_, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.value()?;
        self.visit_any(visitor)
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
        let number = match self.value()? {
            Value::Integer(value) => *value as f32,
            Value::Unsigned(value) => *value as f32,
            Value::Float(value) => *value as f32,
            _ => return Err(Error::new("expected a number")),
        };
        if number.is_infinite() && !matches!(self.value, Value::Float(value) if value.is_infinite())
        {
            return Err(Error::new("float out of range"));
        }
        visitor.visit_f32(number)
    }

    #[allow(clippy::cast_precision_loss)]
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let number = match self.value()? {
            Value::Integer(value) => *value as f64,
            Value::Unsigned(value) => *value as f64,
            Value::Float(value) => *value,
            _ => return Err(Error::new("expected a number")),
        };
        visitor.visit_f64(number)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value()? {
            Value::String(value) => visitor.visit_borrowed_bytes(value.as_bytes()),
            _ => self.visit_any(visitor),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value()? {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self.child(self.value)),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let value = self.value()?;
        match name {
            super::NAME => self.visit_any(visitor),
            tagged::NAME if matches!(value, Value::Tagged(_)) => self.visit_any(visitor),
            tagged::NAME => Err(Error::new("expected an explicit YAML tag")),
            spanned::NAME | commented::NAME => {
                Err(Error::new("a YAML value has no source metadata"))
            }
            _ => visitor.visit_newtype_struct(self.child(value)),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if !matches!(self.value()?, Value::Mapping(_)) {
            return Err(Error::new("expected a mapping"));
        }
        self.visit_any(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        let value = self.value()?;
        if let Value::String(tag) = value {
            return BorrowedStrDeserializer::new(tag).deserialize_enum(name, variants, visitor);
        }
        if let Some([(Value::String(tag), value)]) = value.as_mapping() {
            return visitor.visit_enum(variant(tag, self.child(value)));
        }

        Err(Error::new(
            "expected an enum name or a single-entry mapping",
        ))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.value()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool char str string unit unit_struct seq tuple tuple_struct identifier
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
    fn mapping_keys_and_tagged_payloads_share_the_budget() {
        let value = from_str::<Value>("{key: !Tag [one, two]}").unwrap();
        let budget = Budget::new(5);
        let error = Value::deserialize(value.deserializer(&budget)).unwrap_err();
        insta::assert_snapshot!(error, @"deserialization resource limit exceeded");
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
