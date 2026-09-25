use crate::{AttachedComment, CommentPlacement};
use astral_yaml::{Comment, Span};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
};
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// A private newtype protocol transports (start, end, value, comments) from our deserializer.
// The offsets come from the parser, never from fields in the YAML document.
pub(crate) const NAME: &str = "$astral_yaml::Spanned";

/// A deserialized value together with its source span and borrowed comments.
///
/// Wrap any deserializable type, including individual fields, mapping keys, or
/// collection elements. Borrowed values retain the input's lifetime.
///
/// The span is a half-open byte range in the decoded UTF-8 source. It covers the
/// entire YAML node, including quotes, collection delimiters, tags, and anchors.
/// An alias has the span of its `*name` occurrence; wrapped children within an
/// aliased collection have spans in the anchor's definition. Implicit nulls can
/// have empty spans. Use [`astral_yaml::Source::original_span`] to map a span back
/// to encoded input bytes.
///
/// Serialization emits only the value. Mutating the value does not update its
/// metadata. Equality and hashing include the value, span, and comments.
///
/// Comments include the node's own comments and those of all syntax descendants,
/// in source order, once per wrapper. Parent and child lists intentionally overlap.
/// Aliases are leaves for comment aggregation: their wrappers contain use-site
/// comments, while wrapped children of aliased collections retain definition-site
/// comments. Placement always describes the direct owner. Comments can lie outside
/// the node's span. Comment text borrows the input even when `T` is owned.
///
/// Deserialization requires this crate's deserializer. Serde's intermediate
/// buffering, used by `#[serde(untagged)]` and `#[serde(flatten)]`, discards source
/// metadata: wrap the containing enum or struct instead of fields decoded from that
/// buffer.
///
/// ```
/// use astral_yaml_serde::{Spanned, from_str};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config<'a> {
///     #[serde(borrow)]
///     timeout: Spanned<'a, u64>,
/// }
///
/// let source = "timeout: 30 # seconds\n";
/// let config: Config<'_> = from_str(source)?;
/// assert_eq!(*config.timeout, 30);
/// assert_eq!(config.timeout.span.slice(source), Some("30"));
/// assert_eq!(config.timeout.comments[0].comment.text, " seconds");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
/// A wrapper cannot outlive the source of its comments:
///
/// ```compile_fail
/// use astral_yaml_serde::{Spanned, from_str};
/// let value = {
///     let source = String::from("42 # answer");
///     from_str::<Spanned<'_, u64>>(&source).unwrap()
/// };
/// println!("{:?}", value.comments);
/// ```
///
/// It does not implement `DeserializeOwned`, even for an owned `T`:
///
/// ```compile_fail
/// use astral_yaml_serde::Spanned;
/// fn owned<T: serde::de::DeserializeOwned>() {}
/// owned::<Spanned<'static, String>>();
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Spanned<'a, T> {
    /// The deserialized value.
    pub value: T,
    /// The source range of the YAML node from which the value was deserialized.
    pub span: Span,
    /// Comments belonging to this node and its syntax descendants, in source order.
    pub comments: Vec<AttachedComment<'a>>,
}

impl<T> Spanned<'_, T> {
    /// Returns the value, discarding its metadata.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Spanned<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Serialize> Serialize for Spanned<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de: 'a, 'a, T: Deserialize<'de>> Deserialize<'de> for Spanned<'a, T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, SpannedVisitor(PhantomData))
    }
}

struct SpannedVisitor<'a, T>(PhantomData<(&'a (), T)>);

impl<'de: 'a, 'a, T: Deserialize<'de>> Visitor<'de> for SpannedVisitor<'a, T> {
    type Value = Spanned<'a, T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a source-spanned value from astral-yaml-serde")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let start = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let end = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let Some(span) = Span::new(start, end) else {
            return Err(de::Error::custom("span starts after its end"));
        };

        let value = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let comments: Vec<CommentMetadata<'a>> = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(5, &self));
        }

        Ok(Spanned {
            value,
            span,
            comments: comments.into_iter().map(|metadata| metadata.0).collect(),
        })
    }
}

struct CommentMetadata<'a>(AttachedComment<'a>);

impl<'de: 'a, 'a> Deserialize<'de> for CommentMetadata<'a> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (start, end, text, placement) =
            <(usize, usize, &'a str, u8)>::deserialize(deserializer)?;
        let span = Span::new(start, end)
            .ok_or_else(|| de::Error::custom("comment span starts after its end"))?;
        let placement = match placement {
            0 => CommentPlacement::Leading,
            1 => CommentPlacement::Inline,
            2 => CommentPlacement::Trailing,
            3 => CommentPlacement::Detached,
            _ => return Err(de::Error::custom("invalid comment placement")),
        };

        Ok(Self(AttachedComment {
            comment: Comment { span, text },
            placement,
        }))
    }
}
