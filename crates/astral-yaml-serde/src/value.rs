use crate::{Tagged, tagged::is_core_tag};
use serde::de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::ser;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

mod de;

pub use de::from_value;

pub(crate) const NAME: &str = "$astral_yaml::Value";

/// An owned YAML value that can be read and written through Serde.
///
/// Use [`from_value`] to decode an owned Rust type or
/// [`Deserialize::deserialize`] with `&Value` to borrow from the tree.
///
/// Explicit core tags resolve through the YAML 1.2 core schema. Application
/// tags retain their expanded name and decoded scalar text, so `!Port 080`
/// contains `"080"`. Use [`Tagged<T>`] for a specific Rust payload type.
///
/// Serde's `flatten` and untagged-enum buffers cannot capture application tags.
/// Alias expansion is bounded by the deserializer's depth and resource limits.
///
/// Mapping equality ignores entry order. Signed and unsigned integers compare
/// by value, and all NaNs compare equal.
///
/// ```
/// use astral_yaml_serde::{Value, from_str, to_string};
///
/// let mut ports: Value = from_str("[80, 443]")?;
/// if let Some(items) = ports.as_sequence_mut() {
///     items.push(Value::Unsigned(8080));
/// }
/// assert_eq!(to_string(&ports)?, "[80, 443, 8080]\n");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
#[derive(Clone, Debug, Default, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// A YAML null.
    #[default]
    Null,
    /// A YAML boolean.
    Bool(bool),
    /// A signed YAML integer.
    Integer(i128),
    /// An unsigned YAML integer.
    Unsigned(u128),
    /// A YAML floating-point number, including infinities and NaN.
    Float(f64),
    /// A decoded YAML string.
    String(String),
    /// An ordered sequence of values.
    Sequence(Vec<Self>),
    /// Mapping entries in input order. Both keys and values can be any YAML value.
    ///
    /// YAML serialization rejects duplicate keys.
    Mapping(#[serde(serialize_with = "serialize_mapping")] Vec<(Self, Self)>),
    /// A value with an explicit application tag.
    ///
    /// The payload must be a string, sequence, or mapping. YAML serialization
    /// rejects an invalid tag or a payload of any other type.
    Tagged(#[serde(serialize_with = "serialize_tagged")] Box<Tagged<Self>>),
}

impl Value {
    /// Return the boolean value, if this is a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Return the string contents, if this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Return the integer if it fits in an `i128`.
    #[must_use]
    pub fn as_i128(&self) -> Option<i128> {
        match self {
            Self::Integer(value) => Some(*value),
            Self::Unsigned(value) => i128::try_from(*value).ok(),
            _ => None,
        }
    }

    /// Return the integer if it fits in a `u128`.
    #[must_use]
    pub fn as_u128(&self) -> Option<u128> {
        match self {
            Self::Integer(value) => u128::try_from(*value).ok(),
            Self::Unsigned(value) => Some(*value),
            _ => None,
        }
    }

    /// Return the floating-point value, if this is a float.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// Return the sequence items as a slice.
    #[must_use]
    pub fn as_sequence(&self) -> Option<&[Self]> {
        match self {
            Self::Sequence(items) => Some(items),
            _ => None,
        }
    }

    /// Return the sequence's item vector for modification.
    #[must_use]
    pub fn as_sequence_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Sequence(items) => Some(items),
            _ => None,
        }
    }

    /// Return the mapping's entries in order.
    #[must_use]
    pub fn as_mapping(&self) -> Option<&[(Self, Self)]> {
        match self {
            Self::Mapping(entries) => Some(entries),
            _ => None,
        }
    }

    /// Return the mapping's entry vector for modification.
    #[must_use]
    pub fn as_mapping_mut(&mut self) -> Option<&mut Vec<(Self, Self)>> {
        match self {
            Self::Mapping(entries) => Some(entries),
            _ => None,
        }
    }
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
                u128::try_from(*signed).ok() == Some(*unsigned)
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

                // Consume one matching entry per iteration. Searching in reverse
                // lets swap_remove pop entries when the mappings share their order.
                for entry in left.iter().rev() {
                    let Some(index) = remaining.iter().rposition(|candidate| *candidate == entry)
                    else {
                        return false;
                    };
                    remaining.swap_remove(index);
                }

                true
            }
            (Self::Tagged(left), Self::Tagged(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

fn serialize_mapping<S: Serializer>(
    entries: &[(Value, Value)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_map(entries.iter().map(|(key, value)| (key, value)))
}

fn serialize_tagged<S: Serializer>(
    tagged: &Tagged<Value>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if is_core_tag(&tagged.tag) {
        return Err(ser::Error::custom("expected an application YAML tag"));
    }
    if !matches!(
        tagged.value,
        Value::String(_) | Value::Sequence(_) | Value::Mapping(_)
    ) {
        return Err(ser::Error::custom(
            "a tagged YAML value requires a string, sequence, or mapping payload",
        ));
    }
    tagged.serialize(serializer)
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a YAML value")
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Integer(value.into()))
    }

    fn visit_i128<E>(self, value: i128) -> Result<Value, E> {
        Ok(Value::Integer(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Unsigned(value.into()))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Value, E> {
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

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Value, A::Error> {
        let (tag, payload) = data.variant::<String>()?;
        let value = payload.newtype_variant()?;
        Ok(Value::Tagged(Box::new(Tagged { tag, value })))
    }
}
