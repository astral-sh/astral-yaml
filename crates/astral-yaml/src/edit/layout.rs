//! Header and termination requirements shared by framing and subtree emission.

use super::{CommentPosition, NodeKind, emit, view::Syntax};
use crate::{CollectionStyle, NodeId, ScalarStyle, comments::CommentPlacement};

#[derive(Clone, Copy)]
enum Tail {
    Closed,
    Scalar(NodeId),
    Detached(NodeId),
}

/// Block collections need headers for inline comments and have no closing
/// token. Their final scalar or detached comment can affect an enclosing
/// comment's separation. Resolve these
/// dependencies before choosing rewrite regions, without changing the
/// candidate's meaning or requested styles.
pub(super) struct Layout {
    tails: Vec<Tail>,
    quoted: Vec<bool>,
    flow: Vec<bool>,
}

impl Layout {
    pub(super) fn new<'a, Location: 'a>(document: impl Into<Syntax<'a, Location>>) -> Self {
        let document = document.into();
        let mut order = Vec::new();
        let mut pending = vec![(document.root, false)];
        let mut layout = Self {
            tails: vec![Tail::Closed; document.nodes.len()],
            quoted: vec![false; document.nodes.len()],
            flow: vec![false; document.nodes.len()],
        };
        while let Some((id, prefix_before_slot)) = pending.pop() {
            let node = &document.nodes[id.0];
            order.push(id);
            if id != document.root && emit::block_collection(&node.kind, false) {
                // Block collections have no content delimiter. Inline comments
                // need a property or the enclosing slot's indicator. A prefix
                // kept inside that slot makes its indicator unavailable.
                let properties =
                    usize::from(node.tag.is_some()) + usize::from(node.anchor.is_some());
                let mut comments = node.comments.iter().peekable();
                if prefix_before_slot {
                    while comments
                        .peek()
                        .is_some_and(|comment| comment.placement == CommentPlacement::Leading)
                    {
                        comments.next();
                    }
                }
                let slot_header = comments
                    .peek()
                    .is_some_and(|comment| comment.placement == CommentPlacement::Inline);
                let inline = comments
                    .filter(|comment| comment.placement == CommentPlacement::Inline)
                    .count();
                layout.flow[id.0] = inline > properties + usize::from(slot_header);
            }
            let header = node.anchor.is_some() || node.tag.is_some();
            match &node.kind {
                NodeKind::Sequence { items, .. } => {
                    pending.extend(
                        items
                            .iter()
                            .enumerate()
                            .map(|(index, &child)| (child, index != 0 || !header)),
                    );
                }
                NodeKind::Mapping { entries, .. } => {
                    for (index, &(key, value)) in entries.iter().enumerate() {
                        pending.extend([(key, false), (value, index != 0 || !header)]);
                    }
                }
                _ => {}
            }
        }
        for id in order.into_iter().rev() {
            let node = &document.nodes[id.0];
            let mut tail = match &node.kind {
                _ if layout.flow(id) => Tail::Closed,
                NodeKind::Scalar {
                    style: ScalarStyle::Literal | ScalarStyle::Folded,
                    ..
                } => Tail::Scalar(id),
                NodeKind::Sequence {
                    items,
                    style: CollectionStyle::Block,
                } => items
                    .last()
                    .map_or(Tail::Closed, |child| layout.tails[child.0]),
                NodeKind::Mapping {
                    entries,
                    style: CollectionStyle::Block,
                } => entries
                    .last()
                    .map_or(Tail::Closed, |(_, child)| layout.tails[child.0]),
                _ => Tail::Closed,
            };
            for comment in node.comments.iter().filter(|comment| {
                comment.position == CommentPosition::After
                    && comment.placement != CommentPlacement::Inline
            }) {
                // A nested detached suffix must end inside its collection if
                // an enclosing trailing comment needs adjacency to the body.
                if comment.placement == CommentPlacement::Trailing {
                    if let Tail::Detached(owner) = tail {
                        layout.flow[owner.0] = true;
                        tail = Tail::Closed;
                    }
                }
                // Root suffixes have a separate document-marker plan.
                if id == document.root {
                    continue;
                }
                if comment.placement == CommentPlacement::Detached {
                    if let Tail::Scalar(scalar) = tail {
                        layout.quoted[scalar.0] = true;
                    }
                    tail = if emit::block_collection(&node.kind, layout.flow(id)) {
                        Tail::Detached(id)
                    } else {
                        Tail::Closed
                    };
                } else {
                    tail = Tail::Closed;
                }
            }
            layout.tails[id.0] = tail;
        }
        layout
    }

    /// Multiline syntax needs its enclosing block indentation when the old
    /// token shares a line with a field or compact collection indicator.
    pub(super) fn multiline(&self, id: NodeId, kind: &NodeKind) -> bool {
        match kind {
            NodeKind::Scalar { value, style } => match self.scalar_style(id, *style) {
                ScalarStyle::Literal | ScalarStyle::Folded => true,
                ScalarStyle::Plain | ScalarStyle::SingleQuoted => value.contains('\n'),
                ScalarStyle::DoubleQuoted => false,
            },
            _ => !self.flow(id) && emit::block_collection(kind, false),
        }
    }

    pub(super) fn tail(&self, id: NodeId) -> Option<NodeId> {
        match self.tails[id.0] {
            Tail::Scalar(scalar) if !self.quoted[scalar.0] => Some(scalar),
            _ => None,
        }
    }

    pub(super) fn flow(&self, id: NodeId) -> bool {
        self.flow[id.0]
    }

    pub(super) fn scalar_style(&self, id: NodeId, style: ScalarStyle) -> ScalarStyle {
        if self.quoted[id.0] {
            ScalarStyle::DoubleQuoted
        } else {
            style
        }
    }
}
