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

// The private newtype protocol requests (value, comments) from the deserializer.
pub(crate) const NAME: &str = "$astral_yaml::Commented";

/// A deserialized value with its borrowed comments.
///
/// Can wrap a field, mapping key, collection element, or whole document.
///
/// Serialization emits only the value. Mutating the value does not update its
/// comments. Equality and hashing include the value and comments.
///
/// Comments include the node's own comments and those of all syntax descendants,
/// in source order, once per wrapper. Parent and child lists overlap.
/// Alias wrappers contain comments at the alias occurrence. Wrapped children of
/// aliased collections retain comments from the anchor's definition. Comment
/// placement is relative to the direct owner. Comments can lie outside
/// the node's span. Comment text borrows the input even when `T` is owned.
///
/// Deserialization requires this crate's deserializer. Serde's buffering for
/// `#[serde(untagged)]` and `#[serde(flatten)]` discards source
/// metadata: wrap the containing enum or struct instead of fields decoded from that
/// buffer.
///
/// ```
/// use astral_yaml_serde::{Commented, from_str};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config<'a> {
///     #[serde(borrow)]
///     timeout: Commented<'a, u64>,
/// }
///
/// let source = "timeout: 30 # seconds\n";
/// let config: Config<'_> = from_str(source)?;
/// assert_eq!(*config.timeout, 30);
/// assert_eq!(config.timeout.comments[0].comment.text, " seconds");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
/// A wrapper cannot outlive the source of its comments:
///
/// ```compile_fail
/// use astral_yaml_serde::{Commented, from_str};
/// let value = {
///     let source = String::from("42 # answer");
///     from_str::<Commented<'_, u64>>(&source).unwrap()
/// };
/// println!("{:?}", value.comments);
/// ```
///
/// It does not implement `DeserializeOwned`, even for an owned `T`:
///
/// ```compile_fail
/// use astral_yaml_serde::Commented;
/// fn owned<T: serde::de::DeserializeOwned>() {}
/// owned::<Commented<'static, String>>();
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Commented<'a, T> {
    /// The deserialized value.
    pub value: T,
    /// Comments belonging to this node and its syntax descendants, in source order.
    pub comments: Vec<AttachedComment<'a>>,
}

impl<T> Commented<'_, T> {
    /// Returns the value, discarding its metadata.
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

impl<'de: 'a, 'a, T: Deserialize<'de>> Deserialize<'de> for Commented<'a, T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_newtype_struct(NAME, CommentedVisitor(PhantomData))
    }
}

struct CommentedVisitor<'a, T>(PhantomData<(&'a (), T)>);

impl<'de: 'a, 'a, T: Deserialize<'de>> Visitor<'de> for CommentedVisitor<'a, T> {
    type Value = Commented<'a, T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a commented value from astral-yaml-serde")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let value = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let comments: Vec<CommentMetadata<'a>> = seq
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
