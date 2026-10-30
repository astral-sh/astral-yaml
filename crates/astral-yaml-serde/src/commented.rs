use crate::{AttachedComment, CommentPlacement};
use astral_yaml::{Comment, Span};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// The private newtype protocol requests (value, comments) from the deserializer.
pub(crate) const NAME: &str = "$astral_yaml::Commented";

/// A deserialized value with comments borrowed from the YAML source.
///
/// Includes the node's own comments and those of its syntax descendants, in source
/// order. Each comment appears once per wrapper, so nested wrappers can contain
/// some of the same comments. Placement is relative to the node that owns the
/// comment, even when it is collected by a parent wrapper.
///
/// An alias uses the comments at its `*name` expression. Wrapped children of an
/// aliased collection use comments from the anchor's definition. Comments can
/// fall outside their node's span.
///
/// Serialization emits only the value. Changing the value does not update the
/// comments. Equality and hashing include both. Comment text borrows the source
/// even when `T` is owned, so `Commented` cannot be used where
/// [`DeserializeOwned`](serde::de::DeserializeOwned) is required.
///
/// Deserialization requires this crate's deserializer. Serde's `untagged` and
/// `flatten` buffers discard comments. Wrap the containing enum or struct when
/// you need its comments, rather than a field deserialized from a buffer.
///
/// ```
/// use astral_yaml_serde::{Commented, from_str};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config<'source> {
///     #[serde(borrow)]
///     timeout: Commented<'source, u64>,
/// }
///
/// let source = "timeout: 30 # seconds\n";
/// let config: Config<'_> = from_str(source)?;
/// assert_eq!(*config.timeout, 30);
/// assert_eq!(config.timeout.comments[0].comment.text, " seconds");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Commented<'source, T> {
    /// The deserialized value.
    pub value: T,
    /// Comments belonging to this node and its syntax descendants, in source order.
    pub comments: Vec<AttachedComment<'source>>,
}

impl<T> Commented<'_, T> {
    /// Return the value, discarding its comments.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Commented<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Commented<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Serialize> Serialize for Commented<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de: 'source, 'source, T: Deserialize<'de>> Deserialize<'de> for Commented<'source, T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, CommentedVisitor(PhantomData))
    }
}

struct CommentedVisitor<'source, T>(PhantomData<(&'source (), T)>);

impl<'de: 'source, 'source, T: Deserialize<'de>> Visitor<'de> for CommentedVisitor<'source, T> {
    type Value = Commented<'source, T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a commented value from astral-yaml-serde")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let value = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let comments: Vec<CommentMetadata<'source>> = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(3, &self));
        }

        Ok(Commented {
            value,
            comments: comments.into_iter().map(|metadata| metadata.0).collect(),
        })
    }
}

struct CommentMetadata<'source>(AttachedComment<'source>);

impl<'de: 'source, 'source> Deserialize<'de> for CommentMetadata<'source> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (start, end, text, placement) =
            <(usize, usize, &'source str, u8)>::deserialize(deserializer)?;
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
