use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleStruct,
};
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub(crate) const NAME: &str = "$astral_yaml::Tagged";

/// A deserialized value with its explicit YAML tag.
///
/// The tag is expanded: `!Text` stays `!Text`, while `!!str` becomes
/// `tag:yaml.org,2002:str`. Percent escapes retain their original spelling. Untagged
/// input is rejected. Application tags are consumed by the wrapper. Core tags still
/// determine the payload's type and must match its content.
///
/// The tag is owned, so this wrapper implements `DeserializeOwned` when `T`
/// does. The payload can also borrow from the input. Use
/// [`crate::Spanned`] and [`crate::Commented`] around this wrapper to retain
/// source spans and comments.
/// Serialization with this crate emits the tag and payload as one YAML node.
/// Explicit core tags must match the emitted content. For example, `!!int`
/// permits the string payload `"42"`, but `!!map` requires a mapping payload.
/// Deserialization requires this crate's deserializer.
///
/// `Option<T>` resolves `!!null` as `None`. To capture that tag, put this wrapper
/// outside the option: `Tagged<Option<T>>`.
///
/// This wrapper must reach the YAML deserializer directly. Serde's buffering for
/// derived untagged enums and flattened fields happens before their payload
/// types are visited, so a wrapper inside such a buffer cannot opt into tags.
/// Use [`crate::with::yaml_tag`] to interpret local tags as enum variant names.
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
    /// Returns the payload, discarding its tag.
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
