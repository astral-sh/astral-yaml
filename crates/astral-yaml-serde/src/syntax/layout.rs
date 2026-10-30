//! Choose headers and separators needed to preserve comments during rendering.

use super::{CommentPosition, NodeKind, emit, view::Syntax};
use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, comments::CommentPlacement};

#[derive(Clone, Copy)]
enum Tail {
    Closed,
    Scalar(NodeId),
    Detached(NodeId),
}

#[derive(Clone, Copy)]
enum Presentation {
    Unchanged,
    QuotedScalar,
    FlowCollection,
}

/// Formatting decisions needed before selecting rewrite regions.
///
/// A block collection may need flow delimiters to keep its comments attached.
/// A final scalar or detached comment may also need separation from an enclosing
/// comment. These decisions must preserve the edited values and requested styles.
pub(super) struct Layout {
    tails: Vec<Tail>,
    presentation: Vec<Presentation>,
    descendant_inline: bool,
}

impl Layout {
    pub(super) fn new<'a, Location: 'a>(document: impl Into<Syntax<'a, Location>>) -> Self {
        let document = document.into();
        let mut order = Vec::new();
        let mut pending = vec![(document.root, false)];
        let mut layout = Self {
            tails: vec![Tail::Closed; document.nodes.len()],
            presentation: vec![Presentation::Unchanged; document.nodes.len()],
            descendant_inline: false,
        };
        while let Some((id, prefix_before_slot)) = pending.pop() {
            let node = &document.nodes[id.index()];
            order.push(id);
            if id != document.root {
                layout.descendant_inline |= node
                    .comments
                    .iter()
                    .any(|comment| comment.placement == CommentPlacement::Inline);
                if emit::block_collection(&node.kind, false)
                    && !emit::block_headers_cover_comments(node, prefix_before_slot)
                {
                    layout.presentation[id.index()] = Presentation::FlowCollection;
                }
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
            let node = &document.nodes[id.index()];
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
                    .map_or(Tail::Closed, |child| layout.tails[child.index()]),
                NodeKind::Mapping {
                    entries,
                    style: CollectionStyle::Block,
                } => entries
                    .last()
                    .map_or(Tail::Closed, |(_, child)| layout.tails[child.index()]),
                _ => Tail::Closed,
            };
            for comment in node.comments.iter().filter(|comment| {
                comment.position == CommentPosition::After
                    && comment.placement != CommentPlacement::Inline
            }) {
                // A nested detached suffix must end inside its collection if
                // an enclosing trailing comment needs adjacency to the body.
                if comment.placement == CommentPlacement::Trailing
                    && let Tail::Detached(owner) = tail
                {
                    layout.presentation[owner.index()] = Presentation::FlowCollection;
                    tail = Tail::Closed;
                }
                // Root suffixes have a separate document-marker plan.
                if id == document.root {
                    continue;
                }
                if comment.placement == CommentPlacement::Detached {
                    if let Tail::Scalar(scalar) = tail {
                        layout.presentation[scalar.index()] = Presentation::QuotedScalar;
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
            layout.tails[id.index()] = tail;
        }
        layout
    }

    /// Return whether the content style needs multiline indentation.
    /// A node that shared a line with a field or item indicator then needs an
    /// enclosing rewrite.
    pub(super) fn multiline(&self, id: NodeId, kind: &NodeKind) -> bool {
        match kind {
            NodeKind::Scalar { value, style } => match self.scalar_style(id, *style) {
                ScalarStyle::Literal | ScalarStyle::Folded => true,
                ScalarStyle::Plain | ScalarStyle::SingleQuoted => value.contains('\n'),
                ScalarStyle::DoubleQuoted => false,
            },
            _ => emit::block_collection(kind, self.flow(id)),
        }
    }

    pub(super) fn tail(&self, id: NodeId) -> Option<NodeId> {
        match self.tails[id.index()] {
            Tail::Scalar(scalar)
                if !matches!(
                    self.presentation[scalar.index()],
                    Presentation::QuotedScalar
                ) =>
            {
                Some(scalar)
            }
            _ => None,
        }
    }

    pub(super) fn flow(&self, id: NodeId) -> bool {
        matches!(self.presentation[id.index()], Presentation::FlowCollection)
    }

    /// Return whether the root must start on a separate line from the document marker.
    /// Otherwise its descendants' inline comments could attach to that marker.
    pub(super) fn separate_root(&self) -> bool {
        self.descendant_inline
    }

    pub(super) fn scalar_style(&self, id: NodeId, style: ScalarStyle) -> ScalarStyle {
        if matches!(self.presentation[id.index()], Presentation::QuotedScalar) {
            ScalarStyle::DoubleQuoted
        } else {
            style
        }
    }
}
