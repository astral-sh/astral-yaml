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
    budget: Option<&'budget Budget>,
    depth: usize,
    application_scalar: bool,
}

impl Value {
    pub(crate) fn deserializer<'budget, 'de>(
        &'de self,
        budget: Option<&'budget Budget>,
    ) -> ValueDeserializer<'budget, 'de> {
        ValueDeserializer {
            value: self,
            budget,
            depth: 0,
            application_scalar: false,
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ValueDeserializer<'_, 'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

// Check depth and charge any supplied budget once per Serde call.
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
                    self.deserializer(None).$method($($arg,)* visitor)
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
    any: char str string seq tuple tuple_struct identifier;

    deserialize_any() => self.visit_any(visitor),
    deserialize_bool() => match self.application_scalar() {
        Some(scalar) => visitor.visit_bool(scalar.boolean()?),
        None => self.visit_any(visitor),
    },
    deserialize_unit() => self.visit_unit(visitor),
    deserialize_unit_struct(_name: &'static str) => self.visit_unit(visitor),
    deserialize_i8() => visitor.visit_i8(self.integer(i8::from_str_radix)?),
    deserialize_i16() => visitor.visit_i16(self.integer(i16::from_str_radix)?),
    deserialize_i32() => visitor.visit_i32(self.integer(i32::from_str_radix)?),
    deserialize_i64() => visitor.visit_i64(self.integer(i64::from_str_radix)?),
    deserialize_i128() => visitor.visit_i128(self.integer(i128::from_str_radix)?),
    deserialize_u8() => visitor.visit_u8(self.integer(u8::from_str_radix)?),
    deserialize_u16() => visitor.visit_u16(self.integer(u16::from_str_radix)?),
    deserialize_u32() => visitor.visit_u32(self.integer(u32::from_str_radix)?),
    deserialize_u64() => visitor.visit_u64(self.integer(u64::from_str_radix)?),
    deserialize_u128() => visitor.visit_u128(self.integer(u128::from_str_radix)?),
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

    fn application_scalar(self) -> Option<de::ResolvedScalar<'de>> {
        match self.value {
            Value::String(text) if self.application_scalar => {
                Some(de::ResolvedScalar::application(text))
            }
            _ => None,
        }
    }

    fn integer<T: TryFrom<i128> + TryFrom<u128>>(
        self,
        parse: impl FnOnce(&str, u32) -> Result<T, std::num::ParseIntError>,
    ) -> Result<T, Error> {
        if let Some(scalar) = self.application_scalar() {
            return scalar.integer(parse);
        }

        let converted = match self.value {
            Value::Integer(value) => T::try_from(*value).ok(),
            Value::Unsigned(value) => T::try_from(*value).ok(),
            _ => return Err(Error::new("expected an integer")),
        };
        converted.ok_or_else(|| Error::new("integer out of range"))
    }

    fn visit_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if let Some(scalar) = self.application_scalar() {
            scalar.unit()?;
            return visitor.visit_unit();
        }

        self.visit_any(visitor)
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
                    Self {
                        value,
                        application_scalar: matches!(value, Value::String(_))
                            && !tagged::is_core_tag(&tagged.tag),
                        ..self
                    }
                };
                visitor.visit_enum(variant(&tagged.tag, payload))
            }
        }
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn visit_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if let Some(scalar) = self.application_scalar() {
            return visitor.visit_f32(scalar.float32()?);
        }

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
        if let Some(scalar) = self.application_scalar() {
            return visitor.visit_f64(scalar.float64()?);
        }

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
        if matches!(self.value, Value::Null)
            || self
                .application_scalar()
                .is_some_and(de::ResolvedScalar::is_null)
        {
            visitor.visit_none()
        } else {
            visitor.visit_some(self.child(self.value))
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
        Value::deserialize(value.deserializer(Some(&budget))).unwrap();
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
        let error = Choice::deserialize(value.deserializer(Some(&budget))).unwrap_err();
        insta::assert_snapshot!(error, @"deserialization resource limit exceeded");
    }
}
