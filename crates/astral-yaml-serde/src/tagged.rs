use serde::de::{EnumAccess, VariantAccess, Visitor};
use serde::ser::SerializeTupleStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub(crate) const NAME: &str = "$astral_yaml::Tagged";

/// A deserialized value with its explicit YAML tag.
///
/// Tag handles are expanded, so `!!str` becomes `tag:yaml.org,2002:str`, while
/// `!Text` stays `!Text`. Percent escapes keep their original spelling.
/// Untagged input is an error. Capturing a core tag does not disable its type
/// check on the value.
///
/// This crate's serializer emits the tag and value together. Core tags must match the
/// emitted YAML. For example, `!!int` allows a string value of `"42"`, but `!!map`
/// requires a mapping. Use `Tagged<Option<T>>` to capture a `!!null` tag before
/// the option deserializes as `None`.
///
/// The tag is owned, so `Tagged<T>` implements `DeserializeOwned` when `T` does.
/// The value can still borrow from the input. Wrap this type in [`crate::Spanned`]
/// or [`crate::Commented`] to retain its source span or comments.
///
/// Deserialization requires this crate's deserializer. A wrapper inside Serde's
/// `untagged` or `flatten` buffer cannot opt into tags. Use
/// [`crate::with::yaml_tag`] to interpret local tags as enum variant names.
///
/// ```
/// use astral_yaml_serde::{Tagged, from_str, to_string};
///
/// let mut text: Tagged<String> = from_str("!Text hello")?;
/// assert_eq!(text.tag, "!Text");
/// assert_eq!(*text, "hello");
/// text.push_str(" world");
/// assert_eq!(from_str::<Tagged<String>>(&to_string(&text)?)?, text);
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tagged<T> {
    /// The expanded, explicit YAML tag, including `!` for a local tag.
    pub tag: String,
    /// The deserialized payload.
    pub value: T,
}

impl<T> Tagged<T> {
    /// Return the value, discarding its tag.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Tagged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Tagged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Serialize> Serialize for Tagged<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // A tuple-struct protocol carries a runtime tag through Serde, whose
        // enum serialization methods require static variant names.
        let mut tagged = serializer.serialize_tuple_struct(NAME, 2)?;
        tagged.serialize_field(&self.tag)?;
        tagged.serialize_field(&self.value)?;
        tagged.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Tagged<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, TaggedVisitor(PhantomData))
    }
}

struct TaggedVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for TaggedVisitor<T> {
    type Value = Tagged<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an explicitly tagged YAML value")
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let (tag, payload) = data.variant::<String>()?;
        Ok(Tagged {
            tag,
            value: payload.newtype_variant()?,
        })
    }
}
