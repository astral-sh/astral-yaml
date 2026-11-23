//! Choose headers and separators needed to preserve comments during rendering.

use super::source::Source;
use super::view::Syntax;
use super::{CommentPosition, NodeKind, emit};
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, Span};
use std::num::NonZeroUsize;

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

#[derive(Clone, Copy)]
enum ItemLayout {
    Compact(usize),
    Expanded,
}

impl ItemLayout {
    fn original(source: &Source, item: NodeId) -> Option<Self> {
        let node = source.nodes.get(item.index())?;
        if !emit::block_collection(&node.kind, false) {
            return None;
        }
        // A compact collection starts immediately after its dash and spaces.
        // Skip properties so a header on the dash line stays expanded.
        let start = source.contents[item.index()].start();
        let prefix = source.text[..start].trim_end_matches(' ');
        Some(if prefix.ends_with('-') {
            Self::Compact(start - prefix.len())
        } else {
            Self::Expanded
        })
    }
}

/// The node whose leading comments precede the first field or item.
fn first_block_slot(kind: &NodeKind) -> Option<NodeId> {
    match kind {
        NodeKind::Mapping {
            entries,
            style: CollectionStyle::Block,
        } => entries.first().map(|&(_, value)| value),
        NodeKind::Sequence {
            items,
            style: CollectionStyle::Block,
        } => items.first().copied(),
        _ => None,
    }
}

/// Formatting decisions needed before selecting rewrite regions.
///
/// A block collection may need flow delimiters to keep its comments attached.
/// A final scalar or detached comment may also need separation from an enclosing
/// comment. These decisions must preserve the edited values and requested styles.
pub(super) struct Layout {
    tails: Vec<Tail>,
    presentation: Vec<Presentation>,
    dash_spacing: Vec<Option<NonZeroUsize>>,
}

impl Layout {
    pub(super) fn new<'syntax, Location: 'syntax>(
        document: impl Into<Syntax<'syntax, Location>>,
    ) -> Self {
        let document = document.into();
        let mut order = Vec::new();
        let mut pending = vec![(document.root, false)];
        let mut layout = Self {
            dash_spacing: Vec::new(),
            tails: vec![Tail::Closed; document.nodes.len()],
            presentation: vec![Presentation::Unchanged; document.nodes.len()],
        };
        while let Some((id, prefix_before_slot)) = pending.pop() {
            let node = &document.nodes[id.index()];
            order.push(id);
            if id != document.root
                && emit::block_collection(&node.kind, false)
                && !emit::block_headers_cover_comments(node, prefix_before_slot)
            {
                layout.presentation[id.index()] = Presentation::FlowCollection;
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

    /// Keep each block item's layout. Items without a source layout borrow
    /// the nearest surviving example, preferring the preceding one on ties.
    pub(super) fn preserve_items(&mut self, source: &Source, syntax: Syntax<'_, Option<Span>>) {
        let mut spacing = vec![None; syntax.nodes.len()];
        for node in syntax.nodes {
            let NodeKind::Sequence {
                items,
                style: CollectionStyle::Block,
            } = &node.kind
            else {
                continue;
            };
            let blocks = items.iter().enumerate().filter_map(|(index, &item)| {
                let first = first_block_slot(&syntax.nodes[item.index()].kind)?;
                (!self.flow(item)).then_some((index, item, first))
            });
            let mut examples = blocks
                .clone()
                .filter_map(|(index, item, _)| {
                    ItemLayout::original(source, item).map(|layout| (index, layout))
                })
                .peekable();
            let mut previous = None;
            for (index, item, first) in blocks {
                if let Some(example) = examples.next_if(|&(i, _)| i == index) {
                    previous = Some(example);
                }
                let layout = match (previous, examples.peek().copied()) {
                    (Some((left, layout)), Some((right, _))) if index - left <= right - index => {
                        layout
                    }
                    (_, Some((_, layout))) | (Some((_, layout)), None) => layout,
                    (None, None) => ItemLayout::Compact(1),
                };

                // A leading comment here would become an inline comment on
                // the dash. Properties also need a separate collection header.
                let node = &syntax.nodes[item.index()];
                let prefix = syntax.nodes[first.index()].comments.first();
                if let ItemLayout::Compact(spaces) = layout
                    && node.anchor.is_none()
                    && node.tag.is_none()
                    && !prefix.is_some_and(|comment| comment.placement == CommentPlacement::Leading)
                {
                    spacing[item.index()] = NonZeroUsize::new(spaces);
                }
            }
        }
        self.dash_spacing = spacing;
    }

    pub(super) fn dash_spacing(&self, id: NodeId) -> Option<usize> {
        self.dash_spacing.get(id.index())?.map(NonZeroUsize::get)
    }

    /// Return whether the content style needs multiline indentation.
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

    pub(super) fn scalar_style(&self, id: NodeId, style: ScalarStyle) -> ScalarStyle {
        if matches!(self.presentation[id.index()], Presentation::QuotedScalar) {
            ScalarStyle::DoubleQuoted
        } else {
            style
        }
    }
}
