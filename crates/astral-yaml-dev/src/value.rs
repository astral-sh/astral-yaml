use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
};

/// A common deserialization target for tests, fuzzing, and benchmarks.
///
/// Supports arbitrary mapping keys and non-finite floats. Integers are limited
/// to 64 bits; tests for wider integers use concrete Rust types. Mapping equality
/// ignores entry order. Floats use ordinary IEEE equality, including NaN != NaN.
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
        self.compare(other, f64::eq)
    }
}

impl Value {
    fn compare(&self, other: &Self, floats_equal: fn(&f64, &f64) -> bool) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Unsigned(left), Self::Unsigned(right)) => left == right,
            (Self::Integer(signed), Self::Unsigned(unsigned))
            | (Self::Unsigned(unsigned), Self::Integer(signed)) => {
                u64::try_from(*signed).ok() == Some(*unsigned)
            }
            (Self::Float(left), Self::Float(right)) => floats_equal(left, right),
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Sequence(left), Self::Sequence(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| left.compare(right, floats_equal))
            }
            (Self::Mapping(left), Self::Mapping(right)) => {
                if left.len() != right.len() {
                    return false;
                }

                let mut remaining: Vec<_> = right.iter().collect();

                for entry in left {
                    let Some(index) = remaining.iter().position(|candidate| {
                        entry.0.compare(&candidate.0, floats_equal)
                            && entry.1.compare(&candidate.1, floats_equal)
                    }) else {
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

/// Asserts that values match, allowing NaN on both sides of a comparison.
///
/// Use for round trips and parser comparisons that may contain NaN, including
/// within sequences, mapping keys, and mapping values.
///
/// # Panics
/// Panics if the values differ by anything other than NaN's non-reflexivity.
#[track_caller]
pub fn assert_eq_with_nan(left: &Value, right: &Value) {
    let matches = left.compare(right, |left, right| {
        left.eq(right) || (left.is_nan() && right.is_nan())
    });

    assert!(matches, "values differ:\nleft: {left:?}\nright: {right:?}");
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
