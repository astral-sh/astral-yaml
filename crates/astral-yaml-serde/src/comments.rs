use crate::Error;
use astral_yaml::{CollectionStyle, Comment, NodeId, NodeKind, Span, Stream};
use std::{
    cell::{Cell, OnceCell},
    collections::HashMap,
    ops::Range,
};

/// A comment's position relative to its direct syntax owner.
/// Ancestor wrappers retain this placement when collecting descendant comments.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CommentPlacement {
    /// Immediately precedes a field or sequence item, without a blank line.
    Leading,
    /// Follows syntax on the same line, including a node's header.
    Inline,
    /// Follows the last child of a collection, or the document's root value.
    Trailing,
    /// Separated by a blank line, or otherwise belongs to the enclosing container.
    Detached,
}

/// A source comment and its placement, with text borrowed from the input.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AttachedComment<'a> {
    /// Text excludes `#`; the span includes it and excludes the line break.
    pub comment: Comment<'a>,
    /// Placement relative to the direct owner, even in an ancestor's comment list.
    pub placement: CommentPlacement,
}

pub(crate) struct CommentCache<'de> {
    document: usize,
    index: OnceCell<Result<CommentIndex<'de>, Error>>,
}

impl<'de> CommentCache<'de> {
    pub(crate) fn new(document: usize) -> Self {
        Self {
            document,
            index: OnceCell::new(),
        }
    }

    pub(crate) fn get(
        &self,
        stream: &Stream<'de>,
        id: NodeId,
        budget: &Cell<usize>,
    ) -> Result<&[AttachedComment<'de>], Error> {
        let index = self
            .index
            .get_or_init(|| CommentIndex::new(stream, self.document, budget))
            .as_ref()
            .map_err(Clone::clone)?;

        Ok(index
            .ranges
            .get(&id)
            .map_or(&[][..], |range| &index.comments[range.clone()]))
    }
}

#[derive(Default)]
struct CommentIndex<'de> {
    ranges: HashMap<NodeId, Range<usize>>,
    comments: Vec<AttachedComment<'de>>,
}

impl<'de> CommentIndex<'de> {
    fn new(stream: &Stream<'de>, document: usize, budget: &Cell<usize>) -> Result<Self, Error> {
        let doc = &stream.documents()[document];
        let start = if document == 0 { 0 } else { doc.span.start() };
        let end = stream
            .documents()
            .get(document + 1)
            .map_or(stream.source().len(), |doc| doc.span.start());
        let comments = stream.comments();
        let comments = &comments[comments.partition_point(|c| c.span.start() < start)
            ..comments.partition_point(|c| c.span.start() < end)];
        if comments.is_empty() {
            return Ok(Self::default());
        }

        let mut tree = Tree {
            source: stream.source(),
            lines: Lines::new(stream.source(), start..end, budget)?,
            budget,
            nodes: Vec::with_capacity(32),
            slots: Vec::with_capacity(16),
        };
        tree.push(stream, doc.root)?;

        let mut marker_lines: Vec<_> = doc
            .directives
            .iter()
            .map(|d| tree.lines.line(d.span.start()))
            .collect();
        marker_lines.extend(doc.start_marker.map(|s| tree.lines.line(s.start())));
        marker_lines.extend(
            stream
                .document_end_markers()
                .iter()
                .filter(|span| start <= span.start() && span.start() < end)
                .map(|span| tree.lines.line(span.start())),
        );
        marker_lines.sort_unstable();

        let mut owned: Vec<_> = comments
            .iter()
            .map(|comment| {
                charge(budget, comment.span)?;
                let (owner, placement) = if marker_lines
                    .binary_search(&tree.lines.line(comment.span.start()))
                    .is_ok()
                {
                    (0, CommentPlacement::Inline)
                } else {
                    tree.owner(0, comment)
                };

                Ok((
                    owner,
                    AttachedComment {
                        comment: comment.clone(),
                        placement,
                    },
                ))
            })
            .collect::<Result<_, Error>>()?;
        owned.sort_by_key(|(owner, comment)| (*owner, comment.comment.span.start()));

        // Preorder makes each subtree a contiguous interval. Store comments by
        // owner here; a wrapper sorts its subtree's comments into source order.
        let mut offsets = Vec::with_capacity(tree.nodes.len() + 1);
        let mut comment_index = 0;

        for node_index in 0..tree.nodes.len() {
            offsets.push(comment_index);

            while owned
                .get(comment_index)
                .is_some_and(|(owner, _)| *owner == node_index)
            {
                comment_index += 1;
            }
        }

        offsets.push(comment_index);
        let ranges = tree
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id, offsets[index]..offsets[node.end]))
            .filter(|(_, range)| !range.is_empty())
            .collect();

        Ok(Self {
            ranges,
            comments: owned.into_iter().map(|(_, comment)| comment).collect(),
        })
    }
}

struct Node {
    id: NodeId,
    span: Span,
    end: usize,
    collection: Option<Collection>,
}

struct Collection {
    start: usize,
    flow: bool,
    slots: Range<usize>,
}

#[derive(Default)]
struct Slot {
    start: usize,
    end: usize,
    key: Option<usize>,
    colon: Option<usize>,
    value: usize,
}

struct Tree<'a, 'b> {
    source: &'a str,
    lines: Lines,
    budget: &'b Cell<usize>,
    nodes: Vec<Node>,
    slots: Vec<Slot>,
}

impl Tree<'_, '_> {
    fn push(&mut self, stream: &Stream<'_>, id: NodeId) -> Result<usize, Error> {
        let node = stream.node(id).expect("syntax node belongs to stream");
        charge(self.budget, node.span)?;
        let index = self.nodes.len();
        self.nodes.push(Node {
            id,
            span: node.span,
            end: 0,
            collection: None,
        });

        let source = self.source;
        let collection_start = || {
            let properties_end = node
                .anchor
                .iter()
                .chain(node.tag.iter())
                .map(|property| property.span.end())
                .max()
                .unwrap_or(node.span.start());
            skip_trivia(source, properties_end)
        };

        let collection = match &node.kind {
            NodeKind::Mapping { entries, style } => {
                // Reserve the parent's slots before appending any child slots.
                let slots = self.slots.len()..self.slots.len() + entries.len();
                self.slots.resize_with(slots.end, Slot::default);

                for (offset, entry) in entries.iter().enumerate() {
                    let key = self.push(stream, entry.key)?;
                    let value = self.push(stream, entry.value)?;
                    let separator = skip_trivia(self.source, self.nodes[key].span.end());
                    let colon =
                        (self.source.as_bytes().get(separator) == Some(&b':')).then_some(separator);
                    self.slots[slots.start + offset] = Slot {
                        start: entry.span.start(),
                        end: entry.span.end(),
                        key: Some(key),
                        colon,
                        value,
                    };
                }

                Some(Collection {
                    start: collection_start(),
                    flow: *style == CollectionStyle::Flow,
                    slots,
                })
            }
            NodeKind::Sequence { items, style } => {
                let start = collection_start();
                let slots = self.slots.len()..self.slots.len() + items.len();
                self.slots.resize_with(slots.end, Slot::default);
                let mut previous_end = start;

                for (offset, &item) in items.iter().enumerate() {
                    let value = self.push(stream, item)?;
                    let span = self.nodes[value].span;
                    let header = if *style == CollectionStyle::Block {
                        skip_trivia(self.source, previous_end)
                    } else {
                        span.start()
                    };
                    self.slots[slots.start + offset] = Slot {
                        start: header,
                        end: span.end(),
                        key: None,
                        colon: None,
                        value,
                    };
                    previous_end = span.end();
                }

                Some(Collection {
                    start,
                    flow: *style == CollectionStyle::Flow,
                    slots,
                })
            }
            NodeKind::Scalar(_) | NodeKind::Alias { .. } => None,
        };

        self.nodes[index].collection = collection;
        self.nodes[index].end = self.nodes.len();
        Ok(index)
    }

    fn owner(&self, index: usize, comment: &Comment<'_>) -> (usize, CommentPlacement) {
        let node = &self.nodes[index];
        let at = comment.span.start();
        let inline = self.lines.inline(at);
        let Some(collection) = &node.collection else {
            return (index, self.placement(node.span, comment));
        };

        // Properties and opening/closing delimiters belong to the collection,
        // rather than to the first or last child sharing that line.
        if (node.span.start() <= at && at < collection.start)
            || (collection.flow && at >= node.span.end())
        {
            return (index, self.placement(node.span, comment));
        }

        let slots = &self.slots[collection.slots.clone()];
        let next = slots.partition_point(|slot| slot.start <= at);
        let previous = next.checked_sub(1).map(|i| &slots[i]);
        if let Some(slot) = previous {
            if at < slot.end || (inline && self.lines.line(slot.end) == self.lines.line(at)) {
                return self.slot_owner(slot, comment);
            }
        }

        let following = slots.get(next);
        if let Some(slot) = following {
            if !inline
                && (collection.flow || self.lines.column(at) == self.lines.column(collection.start))
                && !self.lines.blank_between(comment.span.end(), slot.start)
            {
                return (slot.value, CommentPlacement::Leading);
            }
        }

        if let Some(slot) = previous {
            if let Some(child) = &self.nodes[slot.value].collection {
                if !child.flow
                    && self.lines.column(child.start) > self.lines.column(collection.start)
                    && self.lines.column(at) >= self.lines.column(child.start)
                {
                    return self.owner(slot.value, comment);
                }
            }
        }

        let placement = if inline {
            CommentPlacement::Inline
        } else if following.is_none()
            && previous.is_some_and(|slot| !self.lines.blank_between(slot.end, at))
        {
            CommentPlacement::Trailing
        } else {
            CommentPlacement::Detached
        };

        (index, placement)
    }

    fn slot_owner(&self, slot: &Slot, comment: &Comment<'_>) -> (usize, CommentPlacement) {
        let at = comment.span.start();
        if let Some(key) = slot.key {
            if at < self.nodes[key].span.end() || slot.colon.is_some_and(|colon| at < colon) {
                return self.owner(key, comment);
            }
        }

        let value = &self.nodes[slot.value];
        if at < value.span.start() && (self.lines.inline(at) || value.collection.is_none()) {
            return (slot.value, self.placement(value.span, comment));
        }

        self.owner(slot.value, comment)
    }

    fn placement(&self, span: Span, comment: &Comment<'_>) -> CommentPlacement {
        let at = comment.span.start();
        if self.lines.inline(at) {
            CommentPlacement::Inline
        } else if at < span.start() {
            if self.lines.blank_between(comment.span.end(), span.start()) {
                CommentPlacement::Detached
            } else {
                CommentPlacement::Leading
            }
        } else if self.lines.blank_between(span.end(), at) {
            CommentPlacement::Detached
        } else {
            CommentPlacement::Trailing
        }
    }
}

fn skip_trivia(source: &str, mut at: usize) -> usize {
    let bytes = source.as_bytes();

    while let Some(&byte) = bytes.get(at) {
        if byte.is_ascii_whitespace() {
            at += 1;
        } else if byte == b'#' {
            at += memchr::memchr2(b'\r', b'\n', &bytes[at..]).unwrap_or(bytes.len() - at);
        } else {
            break;
        }
    }

    at
}

struct Line {
    start: usize,
    column: usize,
    content: usize,
    // Number of blank lines before this line.
    blanks: usize,
}

struct Lines {
    lines: Vec<Line>,
}

impl Lines {
    fn new(source: &str, range: Range<usize>, budget: &Cell<usize>) -> Result<Self, Error> {
        let bytes = &source.as_bytes()[..range.end];
        let mut result = Self {
            lines: Vec::with_capacity(32),
        };
        let mut start = range.start;
        let mut blanks = 0;

        loop {
            charge(budget, Span::new(start, start).unwrap())?;
            let end = start
                + memchr::memchr2(b'\r', b'\n', &bytes[start..]).unwrap_or(bytes.len() - start);
            let column_start = start
                + if bytes[start..end].starts_with(b"\xef\xbb\xbf") {
                    3
                } else {
                    0
                };
            let content = column_start
                + bytes[column_start..end]
                    .iter()
                    .take_while(|b| matches!(b, b' ' | b'\t'))
                    .count();
            result.lines.push(Line {
                start,
                column: column_start,
                content,
                blanks,
            });
            blanks += usize::from(content == end);

            if end == bytes.len() {
                break;
            }

            start = end + 1;
            if bytes[end] == b'\r' && bytes.get(start) == Some(&b'\n') {
                start += 1;
            }
        }

        Ok(result)
    }

    fn line(&self, at: usize) -> usize {
        self.lines.partition_point(|line| line.start <= at) - 1
    }

    fn column(&self, at: usize) -> usize {
        at.saturating_sub(self.lines[self.line(at)].column)
    }

    fn inline(&self, at: usize) -> bool {
        self.lines[self.line(at)].content < at
    }

    fn blank_between(&self, before: usize, after: usize) -> bool {
        let before = self.line(before);
        let after = self.line(after);
        after > before && self.lines[after].blanks > self.lines[before + 1].blanks
    }
}

fn charge(budget: &Cell<usize>, span: Span) -> Result<(), Error> {
    let remaining = budget
        .get()
        .checked_sub(1)
        .ok_or_else(|| Error::at("deserialization resource limit exceeded", span))?;
    budget.set(remaining);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Cell, CommentCache};

    #[test]
    fn indexing_is_bounded_and_cached_including_failures() {
        let stream = astral_yaml::parse("42 # note\n").unwrap();
        let root = stream.documents()[0].root;

        // Two lines, one node, and one comment ownership record.
        for limit in 0..=4 {
            let budget = Cell::new(limit);
            let cache = CommentCache::new(0);
            let result = cache.get(&stream, root, &budget);
            if limit == 4 {
                assert_eq!(result.unwrap().len(), 1);
                assert_eq!(budget.get(), 0);
                assert_eq!(cache.get(&stream, root, &budget).unwrap().len(), 1);
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.message, "deserialization resource limit exceeded");
                assert!(error.span.unwrap().slice(stream.source()).is_some());
                budget.set(100);
                assert!(cache.get(&stream, root, &budget).is_err());
                assert_eq!(budget.get(), 100);
            }
        }
    }

    #[test]
    fn documents_without_comments_do_not_build_a_tree() {
        let stream = astral_yaml::parse("---\n1 # first\n---\n2\n").unwrap();
        let budget = Cell::new(0);
        let cache = CommentCache::new(1);
        assert!(
            cache
                .get(&stream, stream.documents()[1].root, &budget)
                .unwrap()
                .is_empty()
        );
    }
}
