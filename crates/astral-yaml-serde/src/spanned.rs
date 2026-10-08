use astral_yaml::Span;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
};
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// The private newtype protocol requests (start, end, value) from the
// deserializer. Offsets come from the parser, not YAML fields.
pub(crate) const NAME: &str = "$astral_yaml::Spanned";

/// A deserialized value with its source span.
///
/// Can wrap a field, mapping key, collection element, or whole document.
///
/// The span is a half-open byte range in the decoded UTF-8 source. It covers the entire
/// YAML node, including quotes, collection delimiters, tags, and anchors. An alias has
/// the span of its `*name` occurrence. Wrapped children within an aliased collection
/// have spans in the anchor's definition. Implicit nulls can have empty spans. Use
/// [`astral_yaml::Source::original_span`] to map a span back to encoded input bytes. In
/// an editable document, spans refer to its LF-normalized source.
///
/// Serialization emits only the value. Mutating the value does not update its
/// metadata. Equality and hashing include the value and span.
///
/// Deserialization requires this crate's deserializer. Serde's buffering for
/// `#[serde(untagged)]` and `#[serde(flatten)]` discards source
/// metadata: wrap the containing enum or struct instead of fields decoded from that
/// buffer.
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
/// Owned payloads implement `DeserializeOwned`:
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
    /// Returns the value, discarding its metadata.
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
