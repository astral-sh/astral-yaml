//! Borrowed syntax used by the deserializer, from a parsed stream or a pending edit.

use crate::{AttachedComment, Error, comments::CommentCache};
use astral_yaml::{NodeId, ScalarStyle, Span, Stream};
use std::{borrow::Cow, cell::Cell, ops::Deref};

pub(super) enum Input<'s, 'de> {
    Parsed {
        stream: &'s Stream<'de>,
        comments: CommentCache<'de>,
    },
    #[cfg(feature = "editable")]
    Editing(astral_yaml::edit::ValueRef<'de>),
}

impl<'s, 'de> Input<'s, 'de> {
    pub(super) fn parsed(stream: &'s Stream<'de>, document: usize) -> Self {
        Self::Parsed {
            stream,
            comments: CommentCache::new(document),
        }
    }

    pub(super) fn node(&self, id: NodeId) -> Result<Node<'s, 'de>, Error> {
        match self {
            Self::Parsed { stream, .. } => {
                use astral_yaml::NodeKind as Parsed;

                let node = stream.node(id).expect("ID belongs to this stream");
                let kind = match &node.kind {
                    Parsed::Scalar(scalar) => Kind::Scalar(Scalar {
                        value: match &scalar.value {
                            Cow::Borrowed(text) => Text::Borrowed(text),
                            Cow::Owned(text) => Text::Decoded(text),
                        },
                        style: scalar.style,
                        span: Some(scalar.span),
                    }),
                    Parsed::Sequence { items, .. } => Kind::Sequence { items },
                    Parsed::Mapping { entries, .. } => Kind::Mapping {
                        entries: Entries::Parsed(entries),
                    },
                    Parsed::Alias { target, .. } => Kind::Alias { target: *target },
                };
                Ok(Node {
                    kind,
                    tag: node.tag.as_ref().map(|tag| tag.value.as_ref()),
                    span: Some(node.span),
                })
            }
            #[cfg(feature = "editable")]
            Self::Editing(value) => {
                use astral_yaml::edit::NodeKind as Edited;

                let node = value.node(id)?;
                let span = node.span();
                let kind = match node.kind() {
                    Edited::Scalar { value, style } => Kind::Scalar(Scalar {
                        value: Text::Borrowed(value),
                        style: *style,
                        span,
                    }),
                    Edited::Sequence { items, .. } => Kind::Sequence { items },
                    Edited::Mapping { entries, .. } => Kind::Mapping {
                        entries: Entries::Editing(entries),
                    },
                    Edited::Alias { target } => Kind::Alias { target: *target },
                };
                Ok(Node {
                    kind,
                    tag: node.tag(),
                    span,
                })
            }
        }
    }

    pub(super) fn comments(
        &self,
        id: NodeId,
        budget: &Cell<usize>,
    ) -> Result<Vec<AttachedComment<'de>>, Error> {
        match self {
            Self::Parsed { stream, comments } => Ok(comments.get(stream, id, budget)?.to_vec()),
            #[cfg(feature = "editable")]
            Self::Editing(value) => {
                use astral_yaml::edit::NodeKind;

                let mut comments = Vec::new();
                let mut pending = vec![id];
                while let Some(id) = pending.pop() {
                    // Reading each node charges the edit's work budget. As with a
                    // parsed stream, comments on an alias belong to its use site.
                    let node = value.node(id)?;
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
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Node<'s, 'de> {
    pub kind: Kind<'s, 'de>,
    pub tag: Option<&'s str>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind<'s, 'de> {
    Scalar(Scalar<'s, 'de>),
    Sequence { items: &'s [NodeId] },
    Mapping { entries: Entries<'s> },
    Alias { target: NodeId },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Scalar<'s, 'de> {
    pub value: Text<'s, 'de>,
    pub style: ScalarStyle,
    pub span: Option<Span>,
}

/// Only text living for the full deserialization lifetime can be borrowed by the
/// result. This includes all decoded strings when reading an edit transaction.
#[derive(Clone, Copy, Debug)]
pub(super) enum Text<'s, 'de> {
    Borrowed(&'de str),
    Decoded(&'s str),
}

impl<'s, 'de: 's> Text<'s, 'de> {
    pub(super) fn as_str(self) -> &'s str {
        let text: Text<'s, 's> = self;
        match text {
            Text::Borrowed(text) | Text::Decoded(text) => text,
        }
    }
}

impl Deref for Text<'_, '_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Entries<'s> {
    Parsed(&'s [astral_yaml::Entry]),
    #[cfg(feature = "editable")]
    Editing(&'s [(NodeId, NodeId)]),
}

impl Entries<'_> {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Parsed(entries) => entries.len(),
            #[cfg(feature = "editable")]
            Self::Editing(entries) => entries.len(),
        }
    }

    pub(super) fn get(self, index: usize) -> Option<(NodeId, NodeId)> {
        match self {
            Self::Parsed(entries) => entries.get(index).map(|entry| (entry.key, entry.value)),
            #[cfg(feature = "editable")]
            Self::Editing(entries) => entries.get(index).copied(),
        }
    }

    pub(super) fn iter(self) -> impl Iterator<Item = (NodeId, NodeId)> {
        (0..self.len()).map(move |index| self.get(index).expect("entry index in bounds"))
    }
}
