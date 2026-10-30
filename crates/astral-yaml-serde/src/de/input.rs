//! Borrowed syntax used by the deserializer, from a parsed stream or a pending edit.

use crate::comments::CommentCache;
use crate::{AttachedComment, Error};
use astral_yaml::{NodeId, ScalarStyle, Span, Stream};
use std::borrow::Cow;
use std::cell::Cell;

/// The input chooses a borrowed node representation at compile time. A parsed
/// document can then be deserialized directly from its syntax tree, even when
/// support for deserializing pending edits is enabled.
pub(super) trait Input<'de> {
    type Node<'s>: Node<'s, 'de, Entry = Self::Entry>
    where
        Self: 's,
        'de: 's;
    type Entry: Entry;

    fn node(&self, id: NodeId) -> Result<Self::Node<'_>, Error>;
    fn comments(
        &self,
        id: NodeId,
        budget: &Cell<usize>,
    ) -> Result<Vec<AttachedComment<'de>>, Error>;
}

pub(super) trait Node<'s, 'de>: Copy {
    type Scalar: Scalar<'s, 'de>;
    type Entry: Entry + 's;

    fn kind(self) -> Kind<'s, Self::Scalar, Self::Entry>;
    fn tag(self) -> Option<&'s str>;
    fn span(self) -> Option<Span>;
}

pub(super) struct Parsed<'s, 'de> {
    stream: &'s Stream<'de>,
    comments: CommentCache<'de>,
}

impl<'s, 'de> Parsed<'s, 'de> {
    pub(super) fn new(stream: &'s Stream<'de>, document: usize) -> Self {
        Self {
            stream,
            comments: CommentCache::new(document),
        }
    }
}

impl<'de> Input<'de> for Parsed<'_, 'de> {
    type Node<'s>
        = &'s astral_yaml::Node<'de>
    where
        Self: 's,
        'de: 's;
    type Entry = astral_yaml::Entry;

    #[inline]
    fn node(&self, id: NodeId) -> Result<Self::Node<'_>, Error> {
        Ok(self.stream.node(id).expect("ID belongs to this stream"))
    }

    fn comments(
        &self,
        id: NodeId,
        budget: &Cell<usize>,
    ) -> Result<Vec<AttachedComment<'de>>, Error> {
        Ok(self.comments.get(self.stream, id, budget)?.to_vec())
    }
}

impl<'s, 'de: 's> Node<'s, 'de> for &'s astral_yaml::Node<'de> {
    type Scalar = &'s astral_yaml::Scalar<'de>;
    type Entry = astral_yaml::Entry;

    #[inline]
    fn kind(self) -> Kind<'s, Self::Scalar, Self::Entry> {
        use astral_yaml::NodeKind as Parsed;

        match &self.kind {
            Parsed::Scalar(scalar) => Kind::Scalar(scalar),
            Parsed::Sequence { items, .. } => Kind::Sequence { items },
            Parsed::Mapping { entries, .. } => Kind::Mapping { entries },
            Parsed::Alias { target, .. } => Kind::Alias { target: *target },
        }
    }

    fn tag(self) -> Option<&'s str> {
        self.tag.as_ref().map(|tag| tag.value.as_ref())
    }

    fn span(self) -> Option<Span> {
        Some(self.span)
    }
}

#[cfg(feature = "editable")]
impl<'de> Input<'de> for crate::syntax::ValueRef<'de> {
    type Node<'s>
        = &'de crate::syntax::Node<Option<Span>>
    where
        'de: 's;
    type Entry = (NodeId, NodeId);

    fn node(&self, id: NodeId) -> Result<Self::Node<'_>, Error> {
        Ok((*self).node(id)?)
    }

    fn comments(
        &self,
        id: NodeId,
        _budget: &Cell<usize>,
    ) -> Result<Vec<AttachedComment<'de>>, Error> {
        use crate::syntax::NodeKind;

        let mut comments = Vec::new();
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            // Reading each node charges the edit's work budget. As with a
            // parsed stream, comments on an alias belong to its use site.
            let node = (*self).node(id)?;
            for comment in node.comments() {
                let span = comment.span().ok_or_else(|| {
                    Error::new("new comment has no source span until the edit commits")
                })?;
                comments.push(AttachedComment {
                    comment: astral_yaml::Comment {
                        text: comment.text(),
                        span,
                    },
                    placement: comment.placement(),
                });
            }

            match node.kind() {
                NodeKind::Sequence { items, .. } => pending.extend(items),
                NodeKind::Mapping { entries, .. } => {
                    pending.extend(entries.iter().flat_map(|&(key, value)| [key, value]));
                }
                NodeKind::Scalar { .. } | NodeKind::Alias { .. } => {}
            }
        }
        Ok(comments)
    }
}

#[cfg(feature = "editable")]
impl<'s, 'de: 's> Node<'s, 'de> for &'de crate::syntax::Node<Option<Span>> {
    type Scalar = EditedScalar<'de>;
    type Entry = (NodeId, NodeId);

    #[inline]
    fn kind(self) -> Kind<'s, Self::Scalar, Self::Entry> {
        use crate::syntax::NodeKind as Edited;

        match (*self).kind() {
            Edited::Scalar { value, style } => Kind::Scalar(EditedScalar {
                value,
                style: *style,
                span: (*self).span(),
            }),
            Edited::Sequence { items, .. } => Kind::Sequence { items },
            Edited::Mapping { entries, .. } => Kind::Mapping { entries },
            Edited::Alias { target } => Kind::Alias { target: *target },
        }
    }

    fn tag(self) -> Option<&'s str> {
        (*self).tag()
    }

    fn span(self) -> Option<Span> {
        (*self).span()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind<'s, S, E> {
    Scalar(S),
    Sequence { items: &'s [NodeId] },
    Mapping { entries: &'s [E] },
    Alias { target: NodeId },
}

pub(super) trait Entry {
    fn ids(&self) -> (NodeId, NodeId);
}

impl Entry for astral_yaml::Entry {
    fn ids(&self) -> (NodeId, NodeId) {
        (self.key, self.value)
    }
}

#[cfg(feature = "editable")]
impl Entry for (NodeId, NodeId) {
    fn ids(&self) -> (NodeId, NodeId) {
        *self
    }
}

pub(super) type ScalarOf<'s, 'de, I> = <<I as Input<'de>>::Node<'s> as Node<'s, 'de>>::Scalar;

pub(super) trait Scalar<'s, 'de>: Copy {
    fn value(self) -> &'s str;

    /// Text the result can borrow for the full deserialization lifetime.
    /// This includes decoded strings when reading an edit transaction.
    fn borrowed(self) -> Option<&'de str>;

    fn style(self) -> ScalarStyle;
    fn span(self) -> Option<Span>;
}

impl<'s, 'de> Scalar<'s, 'de> for &'s astral_yaml::Scalar<'de> {
    fn value(self) -> &'s str {
        &self.value
    }

    fn borrowed(self) -> Option<&'de str> {
        match &self.value {
            Cow::Borrowed(text) => Some(text),
            Cow::Owned(_) => None,
        }
    }

    fn style(self) -> ScalarStyle {
        self.style
    }

    fn span(self) -> Option<Span> {
        Some(self.span)
    }
}

#[cfg(feature = "editable")]
#[derive(Clone, Copy)]
pub(super) struct EditedScalar<'de> {
    value: &'de str,
    style: ScalarStyle,
    span: Option<Span>,
}

#[cfg(feature = "editable")]
impl<'s, 'de: 's> Scalar<'s, 'de> for EditedScalar<'de> {
    fn value(self) -> &'s str {
        self.value
    }

    fn borrowed(self) -> Option<&'de str> {
        Some(self.value)
    }

    fn style(self) -> ScalarStyle {
        self.style
    }

    fn span(self) -> Option<Span> {
        self.span
    }
}
