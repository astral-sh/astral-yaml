//! Choose headers and separators needed to preserve comments during rendering.

use super::lines::LineIndex;
use super::source::Source;
use super::view::Syntax;
use super::{CommentPosition, NodeKind, emit};
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, Span};

#[derive(Clone, Copy)]
enum Tail {
    Closed,
    Scalar(NodeId),
    Detached(NodeId),
    Trailing,
}

#[derive(Clone, Copy)]
enum Presentation {
    Unchanged,
    QuotedScalar,
    FlowCollection,
}

#[derive(Clone, Copy)]
pub(super) enum ItemLayout {
    /// Spaces between the dash and the first entry.
    Compact(usize),
    /// Indentation of the entries relative to the dash.
    Expanded(usize),
}

impl ItemLayout {
    fn original(source: &Source, item: NodeId, dash_column: usize) -> Self {
        let start = source.contents[item.index()].start();
        let prefix = source.text[..start].trim_end_matches([' ', '\t']);
        let spaces = start - prefix.len();
        if prefix.ends_with('-') {
            Self::Compact(spaces)
        } else {
            Self::Expanded(spaces.saturating_sub(dash_column).max(1))
        }
    }
}

impl NodeKind {
    fn block_items(&self) -> Option<&[NodeId]> {
        match self {
            Self::Sequence {
                items,
                style: CollectionStyle::Block,
            } => Some(items),
            _ => None,
        }
    }

    /// The node whose leading comments precede the first field or item.
    fn first_block_slot(&self) -> Option<NodeId> {
        match self {
            Self::Mapping {
                entries,
                style: CollectionStyle::Block,
            } => entries.first().map(|&(_, value)| value),
            _ => self.block_items()?.first().copied(),
        }
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
    items: Vec<Option<ItemLayout>>,
}

impl Layout {
    pub(super) fn new<'syntax, Location: 'syntax>(
        document: impl Into<Syntax<'syntax, Location>>,
    ) -> Self {
        let document = document.into();
        let mut order = Vec::new();
        let mut pending = vec![(document.root, false)];
        let mut layout = Self {
            items: Vec::new(),
            tails: vec![Tail::Closed; document.nodes.len()],
            presentation: vec![Presentation::Unchanged; document.nodes.len()],
        };
        while let Some((id, prefix_before_slot)) = pending.pop() {
            let node = &document.nodes[id.index()];
            order.push(id);
            let header = node.has_properties();
            // A property header takes the first child's prefix before the
            // slot. Flow delimiters let the child use both sides of its slot.
            let needs_prefix = header
                && node
                    .kind
                    .first_block_slot()
                    .is_some_and(|child| emit::needs_slot_prefix(&document.nodes[child.index()]));
            if emit::block_collection(&node.kind, false)
                && (needs_prefix
                    || (id != document.root
                        && !emit::block_headers_cover_comments(node, prefix_before_slot)))
            {
                layout.presentation[id.index()] = Presentation::FlowCollection;
            }
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
        layout.resolve_tails(&document, order);
        layout
    }

    /// Resolve each suffix after the presentation of its children is known.
    fn resolve_tails<Location>(&mut self, document: &Syntax<'_, Location>, order: Vec<NodeId>) {
        for id in order.into_iter().rev() {
            let node = &document.nodes[id.index()];
            let mut tail = match &node.kind {
                _ if self.flow(id) => Tail::Closed,
                NodeKind::Scalar {
                    style: ScalarStyle::Literal | ScalarStyle::Folded,
                    ..
                } => Tail::Scalar(id),
                NodeKind::Sequence {
                    items,
                    style: CollectionStyle::Block,
                } => items
                    .last()
                    .map_or(Tail::Closed, |child| self.tails[child.index()]),
                NodeKind::Mapping {
                    entries,
                    style: CollectionStyle::Block,
                } => entries
                    .last()
                    .map_or(Tail::Closed, |(_, child)| self.tails[child.index()]),
                _ => Tail::Closed,
            };
            let mut detached_root = false;
            for comment in node.comments.iter().filter(|comment| {
                comment.position == CommentPosition::After
                    && comment.placement != CommentPlacement::Inline
            }) {
                // A nested detached suffix must end inside its collection if
                // an enclosing trailing comment needs adjacency to the body.
                if comment.placement == CommentPlacement::Trailing
                    && let Tail::Detached(owner) = tail
                {
                    self.presentation[owner.index()] = Presentation::FlowCollection;
                    tail = Tail::Closed;
                }
                // Root suffixes have a separate document-marker plan.
                if id == document.root {
                    if comment.placement == CommentPlacement::Detached {
                        detached_root = true;
                    } else if detached_root && emit::block_collection(&node.kind, self.flow(id)) {
                        self.presentation[id.index()] = Presentation::FlowCollection;
                        tail = Tail::Closed;
                    }
                    continue;
                }
                if comment.placement == CommentPlacement::Detached {
                    if let Tail::Scalar(scalar) = tail {
                        self.presentation[scalar.index()] = Presentation::QuotedScalar;
                    }
                    tail = if emit::block_collection(&node.kind, self.flow(id)) {
                        Tail::Detached(id)
                    } else {
                        Tail::Closed
                    };
                } else if emit::block_collection(&node.kind, self.flow(id)) {
                    tail = Tail::Trailing;
                } else {
                    tail = Tail::Closed;
                }
            }
            self.tails[id.index()] = tail;
        }
    }

    /// Keep each block item's layout. Items without a source layout borrow
    /// the nearest surviving example, preferring the preceding one on ties.
    pub(super) fn preserve_items(&mut self, source: &Source, syntax: Syntax<'_, Option<Span>>) {
        let mut layouts = vec![None; syntax.nodes.len()];
        let mut lines = LineIndex::new(&source.text);
        for (id, node) in syntax.nodes.iter().enumerate() {
            let Some(items) = node.kind.block_items() else {
                continue;
            };
            // A sequence converted from flow style has no source dashes.
            let dash_column = source.nodes.get(id).and_then(|old| {
                old.kind.block_items()?;
                let start = source.contents[id].start();
                Some(lines.at(start).column(start))
            });
            let blocks = items.iter().enumerate().filter_map(|(index, &item)| {
                let first = syntax.nodes[item.index()].kind.first_block_slot()?;
                (!self.flow(item)).then_some((index, item, first))
            });
            let original = |item: NodeId, style| {
                let column = dash_column?;
                let old = source.nodes.get(item.index())?;
                (emit::collection_style(&old.kind, false)? == style)
                    .then(|| ItemLayout::original(source, item, column))
            };
            let mut examples = blocks
                .clone()
                .filter_map(|(index, item, _)| {
                    Some((index, original(item, CollectionStyle::Block)?))
                })
                .peekable();
            let mut previous = None;
            for (index, item, first) in blocks {
                let layout = if let Some((_, layout)) = examples.next_if(|&(i, _)| i == index) {
                    previous = Some((index, layout));
                    layout
                } else if let Some(layout) = original(item, CollectionStyle::Flow) {
                    layout
                } else {
                    [previous, examples.peek().copied()]
                        .into_iter()
                        .flatten()
                        .min_by_key(|(i, _)| index.abs_diff(*i))
                        .map_or(ItemLayout::Compact(1), |(_, layout)| layout)
                };

                // A leading comment here would become an inline comment on
                // the dash. Properties also need a separate collection header.
                let node = &syntax.nodes[item.index()];
                let prefix = syntax.nodes[first.index()].comments.first();
                if !node.has_properties()
                    && !prefix.is_some_and(|comment| comment.placement == CommentPlacement::Leading)
                {
                    layouts[item.index()] = Some(layout);
                }
            }
        }
        self.items = layouts;
    }

    pub(super) fn item(&self, id: NodeId) -> Option<ItemLayout> {
        self.items.get(id.index()).copied().flatten()
    }

    pub(super) fn compact(&self, id: NodeId) -> bool {
        matches!(self.item(id), Some(ItemLayout::Compact(_)))
    }

    /// Return whether the content style needs multiline indentation.
    pub(super) fn multiline(&self, id: NodeId, kind: &NodeKind, flow: bool) -> bool {
        match kind {
            NodeKind::Scalar { value, style } => match self.scalar_style(id, *style) {
                ScalarStyle::Literal | ScalarStyle::Folded => true,
                ScalarStyle::Plain | ScalarStyle::SingleQuoted => value.contains('\n'),
                ScalarStyle::DoubleQuoted => false,
            },
            _ => emit::block_collection(kind, flow || self.flow(id)),
        }
    }

    pub(super) fn has_suffix(&self, id: NodeId) -> bool {
        matches!(self.tails[id.index()], Tail::Detached(_) | Tail::Trailing)
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
