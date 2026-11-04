use std::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A common deserialization target for tests, fuzzing, and benchmarks.
///
/// Supports arbitrary mapping keys and non-finite floats. Integers are limited to 64
/// bits. Tests for wider integers use concrete Rust types. Mapping equality ignores
/// entry order. All NaNs compare equal, regardless of their bit patterns.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    String(String),
    Sequence(Vec<Self>),
    Mapping(Vec<(Self, Self)>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Unsigned(left), Self::Unsigned(right)) => left == right,
            (Self::Integer(signed), Self::Unsigned(unsigned))
            | (Self::Unsigned(unsigned), Self::Integer(signed)) => {
                u64::try_from(*signed).ok() == Some(*unsigned)
            }
            (Self::Float(left), Self::Float(right)) => {
                left == right || (left.is_nan() && right.is_nan())
            }
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Sequence(left), Self::Sequence(right)) => left == right,
            (Self::Mapping(left), Self::Mapping(right)) => {
                if left.len() != right.len() {
                    return false;
                }

                let mut remaining: Vec<_> = right.iter().collect();

                // The serializer preserves entry order. Searching from the end
                // lets swap_remove pop each matching entry on a round trip.
                for entry in left.iter().rev() {
                    let Some(index) = remaining.iter().rposition(|candidate| *candidate == entry)
                    else {
                        return false;
                    };

                    remaining.swap_remove(index);
                }

                true
            }
            _ => false,
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Unsigned(value) => serializer.serialize_u64(*value),
            Self::Float(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Sequence(items) => items.serialize(serializer),
            Self::Mapping(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;

                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a scalar, sequence, or mapping")
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Integer(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Unsigned(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        Ok(Value::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();

        while let Some(item) = sequence.next_element()? {
            items.push(item);
        }

        Ok(Value::Sequence(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut entries = Vec::new();

        while let Some(entry) = map.next_entry()? {
            entries.push(entry);
        }

        Ok(Value::Mapping(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn test_values_preserve_scalar_and_collection_types() {
        for (source, expected) in [
            ("null", Value::Null),
            ("true", Value::Bool(true)),
            ("-42", Value::Integer(-42)),
            ("18446744073709551615", Value::Unsigned(u64::MAX)),
            ("1.5", Value::Float(1.5)),
            (".nan", Value::Float(f64::NAN)),
            (".inf", Value::Float(f64::INFINITY)),
            ("hello", Value::String("hello".to_owned())),
            ("[null]", Value::Sequence(vec![Value::Null])),
            (
                "{[true]: false}",
                Value::Mapping(vec![(
                    Value::Sequence(vec![Value::Bool(true)]),
                    Value::Bool(false),
                )]),
            ),
        ] {
            let value = astral_yaml_serde::from_str::<Value>(source).unwrap();
            assert_eq!(value, expected, "{source}");

            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<Value>(&yaml).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_value_equality_ignores_mapping_order_but_preserves_types_and_entries() {
        for (left, right, equal) in [
            ("{a: 1, b: 2}", "{b: 2, a: 1}", true),
            ("{a: 1}", "{b: 1}", false),
            ("{a: 1}", "{a: 2}", false),
            ("{a: 1}", "{}", false),
            ("{[true]: 1}", "{[1]: 1}", false),
            ("[1, 2]", "[2, 1]", false),
            ("1", "1.0", false),
            (".nan", ".nan", true),
            ("[.nan]", "[.nan]", true),
            ("{key: .nan}", "{key: .nan}", true),
            ("{.nan: value}", "{.nan: value}", true),
            (".inf", "-.inf", false),
        ] {
            let left = astral_yaml_serde::from_str::<Value>(left).unwrap();
            let right = astral_yaml_serde::from_str::<Value>(right).unwrap();
            assert_eq!(left == right, equal, "{left:?} == {right:?}");
        }

        assert_eq!(Value::Integer(42), Value::Unsigned(42));
        assert_eq!(Value::Unsigned(42), Value::Integer(42));
        assert_ne!(Value::Integer(-1), Value::Unsigned(u64::MAX));

        let entry = (Value::Null, Value::Bool(true));
        assert_ne!(
            Value::Mapping(vec![entry.clone(), entry.clone()]),
            Value::Mapping(vec![entry, (Value::Null, Value::Bool(false))]),
        );
    }

    #[test]
    fn nan_values_compare_equal_and_roundtrip() {
        for bits in [
            f64::NAN.to_bits(),
            (-f64::NAN).to_bits(),
            0x7ff0_0000_0000_0001,
            0x7ff8_0000_0000_0042,
        ] {
            assert_eq!(Value::Float(f64::from_bits(bits)), Value::Float(f64::NAN));
        }

        for source in [
            ".nan",
            "[.nan]",
            "{key: .nan}",
            "{.nan: value}",
            "{[.nan]: {nested: [.nan]}}",
        ] {
            let value = astral_yaml_serde::from_str::<Value>(source).unwrap();
            assert_eq!(value, value.clone());

            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            let decoded = astral_yaml_serde::from_str::<Value>(&yaml).unwrap();
            assert_eq!(value, decoded);
        }

        for (left, right) in [
            (".nan", "1.0"),
            ("[.nan]", "[.inf]"),
            ("{.nan: 1}", "{.nan: 2}"),
            ("{key: .nan}", "{other: .nan}"),
        ] {
            let left = astral_yaml_serde::from_str::<Value>(left).unwrap();
            let right = astral_yaml_serde::from_str::<Value>(right).unwrap();
            assert_ne!(left, right);
        }
    }
}
