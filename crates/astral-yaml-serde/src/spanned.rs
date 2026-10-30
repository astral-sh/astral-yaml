use astral_yaml::Span;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// The private newtype protocol requests (start, end, value) from the
// deserializer. Offsets come from the parser, not YAML fields.
pub(crate) const NAME: &str = "$astral_yaml::Spanned";

/// A deserialized value and its source [`Span`].
///
/// The span uses byte offsets in the decoded UTF-8 source. It includes the node's
/// quotes, delimiters, tags, and anchors. An omitted value can have an empty span.
/// Use [`astral_yaml::Source::original_span`] to map it to encoded input bytes.
/// For editable documents, offsets refer to the LF-normalized source.
///
/// An alias uses the span of its `*name` expression. Wrapped children of an
/// aliased collection use spans from the anchor's definition.
///
/// Serialization emits only the value. Changing the value does not update the
/// span. Equality and hashing include both.
///
/// Deserialization requires this crate's deserializer. Serde's `untagged` and
/// `flatten` buffers discard spans. Wrap the containing enum or struct when
/// you need its span, rather than a field deserialized from a buffer.
///
/// ```
/// use astral_yaml_serde::{Spanned, from_str};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     timeout: Spanned<u64>,
/// }
///
/// let source = "timeout: 30 # seconds\n";
/// let config: Config = from_str(source)?;
/// assert_eq!(*config.timeout, 30);
/// assert_eq!(config.timeout.span.slice(source), Some("30"));
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
/// `Spanned<T>` implements `DeserializeOwned` when `T` does:
///
/// ```
/// use astral_yaml_serde::Spanned;
/// fn owned<T: serde::de::DeserializeOwned>() {}
/// owned::<Spanned<String>>();
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Spanned<T> {
    /// The deserialized value.
    pub value: T,
    /// The source range of the YAML node from which the value was deserialized.
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Return the value, discarding its span.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Serialize> Serialize for Spanned<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Spanned<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, SpannedVisitor(PhantomData))
    }
}

struct SpannedVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for SpannedVisitor<T> {
    type Value = Spanned<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a source-spanned value from astral-yaml-serde")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let (start, end, value) =
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(&mut seq))?;
        let span =
            Span::new(start, end).ok_or_else(|| de::Error::custom("span starts after its end"))?;
        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &self));
        }

        Ok(Spanned { value, span })
    }
}
