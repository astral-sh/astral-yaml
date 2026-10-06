//! Scalar termination requirements shared by framing and subtree emission.

use super::{CommentPosition, NodeKind, emit, view::Syntax};
use crate::{CollectionStyle, NodeId, ScalarStyle, comments::CommentPlacement};

/// A block collection has no closing token: its final scalar can consume the
/// blank separation required by an enclosing comment. Resolve that dependency
/// before choosing rewrite regions, without changing the candidate's meaning or
/// requested styles.
pub(super) struct Layout {
    tails: Vec<Option<NodeId>>,
    quoted: Vec<bool>,
}

impl Layout {
    pub(super) fn new<'a, Location: 'a>(document: impl Into<Syntax<'a, Location>>) -> Self {
        let document = document.into();
        let mut order = Vec::new();
        let mut pending = vec![document.root];
        while let Some(id) = pending.pop() {
            order.push(id);
            emit::children(&document.nodes[id.0].kind, &mut pending);
        }
        let mut layout = Self {
            tails: vec![None; document.nodes.len()],
            quoted: vec![false; document.nodes.len()],
        };
        for id in order.into_iter().rev() {
            let node = &document.nodes[id.0];
            let tail = match &node.kind {
                NodeKind::Scalar {
                    style: ScalarStyle::Literal | ScalarStyle::Folded,
                    ..
                } => Some(id),
                NodeKind::Sequence {
                    items,
                    style: CollectionStyle::Block,
                } => items.last().and_then(|&child| layout.tail(child)),
                NodeKind::Mapping {
                    entries,
                    style: CollectionStyle::Block,
                } => entries.last().and_then(|&(_, child)| layout.tail(child)),
                _ => None,
            };
            layout.tails[id.0] = tail;
            // Root suffixes can use document markers. Nested suffixes need a
            // scalar with an explicit closing token before their blank line.
            if id != document.root {
                if let Some(comment) = node.comments.iter().find(|comment| {
                    comment.position == CommentPosition::After
                        && comment.placement != CommentPlacement::Inline
                }) {
                    if let Some(tail) = tail {
                        if comment.placement == CommentPlacement::Detached {
                            layout.quoted[tail.0] = true;
                        }
                        layout.tails[id.0] = None;
                    }
                }
            }
        }
        layout
    }

    pub(super) fn tail(&self, id: NodeId) -> Option<NodeId> {
        self.tails[id.0].filter(|tail| !self.quoted[tail.0])
    }

    pub(super) fn scalar_style(&self, id: NodeId, style: ScalarStyle) -> ScalarStyle {
        if self.quoted[id.0] {
            ScalarStyle::DoubleQuoted
        } else {
            style
        }
    }
}
