//! Mutable syntax for an edit transaction.
//! [`Draft::freeze`] ends mutation and produces the syntax used to check the output.

use super::view::Syntax;
use super::{Comment, CommentPosition, DocumentSyntax, Node, NodeKind, emit};
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span};
use std::cell::Cell;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
struct Membership {
    parent: Option<NodeId>,
    requested_style: bool,
}

#[derive(Clone, Debug)]
struct Slot {
    node: Node<Option<Span>>,
    // Retired slots keep their identity and syntax for source correspondence,
    // but cannot own children or carry a live style request.
    membership: Option<Membership>,
}

impl Slot {
    fn live(node: Node<Option<Span>>, parent: Option<NodeId>) -> Self {
        Self {
            node,
            membership: Some(Membership {
                parent,
                requested_style: false,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Draft {
    slots: Vec<Slot>,
    root: NodeId,
    options: ParseOptions,
    diagnostic_span: Span,
    work: Cell<usize>,
    introduced_bytes: usize,
}

/// The intended result of a transaction, used to check the rendered YAML.
#[derive(Debug)]
pub(super) struct Candidate {
    nodes: Vec<Node<Option<Span>>>,
    root: NodeId,
    parents: Vec<Option<NodeId>>,
    requested_styles: Vec<NodeId>,
    options: ParseOptions,
    diagnostic_span: Span,
    work: Cell<usize>,
}

/// A parsed Serde value. Its nodes form a tree, so only child IDs need relocation.
struct SerializedValue<'source> {
    root: NodeId,
    nodes: Vec<astral_yaml::Node<'source>>,
}

impl From<&DocumentSyntax> for Draft {
    fn from(document: &DocumentSyntax) -> Self {
        let document = &document.source;
        let mut draft = Self {
            slots: document
                .nodes
                .iter()
                .cloned()
                .map(|node| Slot::live(map_locations(node, Some), None))
                .collect(),
            root: document.root,
            options: document.options,
            diagnostic_span: document.nodes[document.root.index()].span,
            work: Cell::new(document.options.max_nodes),
            introduced_bytes: 0,
        };
        for index in 0..draft.slots.len() {
            draft.link_children(NodeId::from_index(index));
        }
        draft
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ReplacementStyle {
    Preserve,
    #[cfg(test)]
    FromValue,
}

impl Draft {
    pub(super) fn root(&self) -> NodeId {
        self.root
    }

    pub(super) fn node(&self, id: NodeId) -> &Node<Option<Span>> {
        &self.slots[id.index()].node
    }

    pub(super) fn contains(&self, id: NodeId) -> bool {
        self.slots
            .get(id.index())
            .is_some_and(|slot| slot.membership.is_some())
    }

    pub(super) fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.membership(id).parent
    }

    fn membership(&self, id: NodeId) -> &Membership {
        self.slots[id.index()]
            .membership
            .as_ref()
            .expect("live node")
    }

    fn membership_mut(&mut self, id: NodeId) -> &mut Membership {
        self.slots[id.index()]
            .membership
            .as_mut()
            .expect("live node")
    }

    pub(super) fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.diagnostic_span,
            message,
        }
    }

    pub(super) fn charge(&self, amount: usize) -> Result<(), Error> {
        super::charge_work(&self.work, amount, self.diagnostic_span)
    }

    fn introduce(&mut self, bytes: usize) -> Result<(), Error> {
        self.introduced_bytes = self
            .introduced_bytes
            .checked_add(bytes)
            .filter(|&size| size <= self.options.max_input_bytes)
            .ok_or_else(|| self.error("inserted YAML exceeds input byte limit"))?;
        Ok(())
    }

    fn parse_value<'source>(
        &mut self,
        yaml: &'source str,
        parent: Option<NodeId>,
        available: usize,
    ) -> Result<SerializedValue<'source>, Error> {
        self.introduce(yaml.len())?;
        let mut depth = 0;
        let mut ancestor = parent;
        while let Some(id) = ancestor {
            depth += 1;
            ancestor = self.parent(id);
        }
        if depth >= self.options.max_depth {
            return Err(self.error("editing nesting limit exceeded"));
        }
        let mut options = self.options;
        options.max_nodes = available;
        options.max_depth -= depth;
        let stream = astral_yaml::parse_with_options(yaml, options)?;
        self.charge(stream.nodes().len())?;
        let root = super::single_document(&stream)?.root;
        Ok(SerializedValue {
            root,
            nodes: stream.into_nodes(),
        })
    }

    fn available(&self) -> usize {
        self.options.max_nodes.saturating_sub(self.slots.len())
    }

    fn link_children(&mut self, id: NodeId) {
        let mut children = Vec::new();
        emit::children(&self.node(id).kind, &mut children);
        for child in children {
            self.membership_mut(child).parent = Some(id);
        }
    }

    /// Insert the serialized tree, reusing the selected node’s ID on replacement.
    fn attach(
        &mut self,
        value: SerializedValue<'_>,
        parent: Option<NodeId>,
        replacement: Option<(NodeId, ReplacementStyle)>,
    ) -> NodeId {
        let SerializedValue {
            root: value_root,
            mut nodes,
        } = value;
        let root = replacement.map_or(NodeId::from_index(self.slots.len()), |(id, _)| id);
        let offset = self.slots.len() + usize::from(replacement.is_none());
        // Child IDs skip the root, which has its own slot.
        let map = |id: NodeId| {
            NodeId::from_index(offset + id.index() - usize::from(id.index() > value_root.index()))
        };
        let mut node = map_locations(Node::from_parsed(nodes.remove(value_root.index())), |_| {
            None
        });
        remap(&mut node, map);
        if let Some((id, style)) = replacement {
            let slot = &mut self.slots[id.index()];
            let old = &mut slot.node;
            node.anchor = old.anchor.take();
            if style == ReplacementStyle::Preserve {
                preserve_style(old, &mut node);
            }
            node.comments = std::mem::take(&mut old.comments);
            remap_comment_positions(&mut node.comments, &old.kind, &node.kind);
            node.span = old.span;
            *slot = Slot::live(node, parent);
        } else {
            self.slots.push(Slot::live(node, parent));
        }
        for node in nodes {
            let mut node = map_locations(Node::from_parsed(node), |_| None);
            remap(&mut node, map);
            self.slots.push(Slot::live(node, None));
        }
        self.link_children(root);
        for index in offset..self.slots.len() {
            self.link_children(NodeId::from_index(index));
        }
        root
    }

    /// Collect all nodes to remove before retiring any of them.
    /// Charge the traversal here so a budget error cannot leave a partially removed subtree.
    fn removal(&self, roots: impl IntoIterator<Item = NodeId>) -> Result<Vec<NodeId>, Error> {
        let mut pending: Vec<_> = roots.into_iter().collect();
        let mut removed = Vec::new();
        while let Some(id) = pending.pop() {
            self.charge(1)?;
            emit::children(&self.node(id).kind, &mut pending);
            removed.push(id);
        }
        Ok(removed)
    }

    fn retire(&mut self, removed: Vec<NodeId>) {
        for id in removed {
            self.slots[id.index()].membership = None;
        }
    }

    pub(super) fn replace(
        &mut self,
        id: NodeId,
        yaml: &str,
        style: ReplacementStyle,
    ) -> Result<(), Error> {
        let parent = self.parent(id);
        let document = self.parse_value(yaml, parent, self.available() + 1)?;
        let removed = self.removal([id])?;
        // Retired slots keep their syntax. Attaching reuses the root's ID and
        // restores its membership, but leaves its former descendants retired.
        self.retire(removed);
        self.attach(document, parent, Some((id, style)));
        Ok(())
    }

    fn sequence_items_mut(&mut self, id: NodeId) -> Result<&mut Vec<NodeId>, Error> {
        match &mut self.slots[id.index()].node.kind {
            NodeKind::Sequence { items, .. } => Ok(items),
            _ => Err(Error {
                span: self.diagnostic_span,
                message: "expected a sequence",
            }),
        }
    }

    fn mapping_entries_mut(&mut self, id: NodeId) -> Result<&mut Vec<(NodeId, NodeId)>, Error> {
        match &mut self.slots[id.index()].node.kind {
            NodeKind::Mapping { entries, .. } => Ok(entries),
            _ => Err(Error {
                span: self.diagnostic_span,
                message: "expected a mapping",
            }),
        }
    }

    pub(super) fn insert_item(
        &mut self,
        id: NodeId,
        index: usize,
        yaml: &str,
    ) -> Result<NodeId, Error> {
        let items = self.sequence_items_mut(id)?;
        if index > items.len() {
            return Err(self.error("item index out of bounds"));
        }
        let document = self.parse_value(yaml, Some(id), self.available())?;
        let child = self.attach(document, Some(id), None);
        self.sequence_items_mut(id)?.insert(index, child);
        Ok(child)
    }

    pub(super) fn remove_item(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let items = self.sequence_items_mut(id)?;
        let Some(&child) = items.get(index) else {
            return Err(self.error("item index out of bounds"));
        };
        let following = items.get(index + 1).copied();
        let removed = self.removal([child])?;
        self.sequence_items_mut(id)?.remove(index);
        retarget_comments(&mut self.slots[id.index()].node, child, following);
        self.retire(removed);
        Ok(())
    }

    pub(super) fn insert_entry(
        &mut self,
        id: NodeId,
        index: usize,
        key: &str,
        value: &str,
    ) -> Result<(NodeId, NodeId), Error> {
        let entries = self.mapping_entries_mut(id)?;
        if index > entries.len() {
            return Err(self.error("entry index out of bounds"));
        }
        // Parse the key and value before attaching either one. Failed parsing cannot
        // leave a live key without an entry owning it.
        let key = self.parse_value(key, Some(id), self.available())?;
        let value = self.parse_value(
            value,
            Some(id),
            self.available().saturating_sub(key.nodes.len()),
        )?;
        let key = self.attach(key, Some(id), None);
        let value = self.attach(value, Some(id), None);
        self.mapping_entries_mut(id)?.insert(index, (key, value));
        Ok((key, value))
    }

    pub(super) fn remove_entry(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let entries = self.mapping_entries_mut(id)?;
        let Some(&(key, value)) = entries.get(index) else {
            return Err(self.error("entry index out of bounds"));
        };
        let following = entries.get(index + 1).map(|&(key, _)| key);
        let removed = self.removal([key, value])?;
        self.mapping_entries_mut(id)?.remove(index);
        retarget_comments(&mut self.slots[id.index()].node, key, following);
        self.retire(removed);
        Ok(())
    }

    pub(super) fn set_scalar_style(&mut self, id: NodeId, style: ScalarStyle) -> Result<(), Error> {
        let node = &mut self.slots[id.index()].node;
        let NodeKind::Scalar {
            value,
            style: current,
        } = &mut node.kind
        else {
            return Err(self.error("expected a scalar"));
        };
        if !emit::same_scalar_type(value, node.tag.as_deref(), *current, style) {
            return Err(self.error("requested style changes the scalar type"));
        }
        *current = style;
        self.membership_mut(id).requested_style = true;
        Ok(())
    }

    pub(super) fn set_collection_style(
        &mut self,
        id: NodeId,
        style: CollectionStyle,
    ) -> Result<(), Error> {
        match &mut self.slots[id.index()].node.kind {
            NodeKind::Sequence { style: current, .. }
            | NodeKind::Mapping { style: current, .. } => *current = style,
            _ => return Err(self.error("expected a collection")),
        }
        self.membership_mut(id).requested_style = true;
        Ok(())
    }

    fn comment_text(&mut self, text: &str) -> Result<(), Error> {
        if text.contains(['\r', '\n']) {
            return Err(self.error("comment text contains a line break"));
        }
        self.introduce(text.len())?;
        self.charge(1)
    }

    pub(super) fn insert_comment(
        &mut self,
        id: NodeId,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.comment_text(text)?;
        let comments = &mut self.slots[id.index()].node.comments;
        if index > comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        let position = match placement {
            CommentPlacement::Leading => CommentPosition::Before,
            CommentPlacement::Detached => comments[index..]
                .iter()
                .map(|comment| comment.position)
                .find(|position| matches!(position, CommentPosition::BeforeChild(_)))
                .unwrap_or(CommentPosition::After),
            CommentPlacement::Inline | CommentPlacement::Trailing => CommentPosition::After,
        };
        comments.insert(
            index,
            Comment {
                text: text.to_owned(),
                placement,
                position,
                span: None,
            },
        );
        Ok(())
    }

    pub(super) fn replace_comment(
        &mut self,
        id: NodeId,
        index: usize,
        text: &str,
    ) -> Result<(), Error> {
        self.comment_text(text)?;
        let Some(comment) = self.slots[id.index()].node.comments.get_mut(index) else {
            return Err(self.error("comment index out of bounds"));
        };
        text.clone_into(&mut comment.text);
        Ok(())
    }

    pub(super) fn remove_comment(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let comments = &mut self.slots[id.index()].node.comments;
        if index >= comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        comments.remove(index);
        Ok(())
    }

    /// Finish mutation and reject aliases whose targets were removed.
    /// This check runs after all edits so an anchor can be removed before its aliases.
    pub(super) fn freeze(self) -> Result<Candidate, Error> {
        for slot in self.slots.iter().filter(|slot| slot.membership.is_some()) {
            if let NodeKind::Alias { target } = slot.node.kind
                && !self.contains(target)
            {
                return Err(self.error("alias target was removed"));
            }
        }

        let mut nodes = Vec::with_capacity(self.slots.len());
        let mut parents = Vec::with_capacity(self.slots.len());
        let mut requested_styles = Vec::new();
        for (index, slot) in self.slots.into_iter().enumerate() {
            nodes.push(slot.node);
            let parent = if let Some(membership) = slot.membership {
                if membership.requested_style {
                    requested_styles.push(NodeId::from_index(index));
                }
                membership.parent
            } else {
                None
            };
            parents.push(parent);
        }
        Ok(Candidate {
            nodes,
            root: self.root,
            parents,
            requested_styles,
            options: self.options,
            diagnostic_span: self.diagnostic_span,
            work: self.work,
        })
    }
}

impl<'a> From<&'a Candidate> for Syntax<'a, Option<Span>> {
    fn from(candidate: &'a Candidate) -> Self {
        Self {
            nodes: &candidate.nodes,
            root: candidate.root,
        }
    }
}

impl Candidate {
    /// Reuse the original layout when comment edits cancel within a larger transaction.
    pub(super) fn restore_comment_spans(&mut self, original: &[Node]) {
        for (node, old) in self.nodes.iter_mut().zip(original) {
            let new = &mut node.comments;
            if new.iter().all(|comment| comment.span.is_some()) {
                continue;
            }

            let mut old = old.comments.as_slice();
            for group in new.split_inclusive_mut(|comment| comment.span.is_some()) {
                // Each retained comment bounds a gap in both lists. Its original
                // span stays with it even if a new neighbor has identical text.
                let (previous, missing) = if let Some(span) = group.last().and_then(|c| c.span) {
                    let at = old
                        .iter()
                        .position(|comment| comment.span == span)
                        .expect("retained comments preserve their source order");
                    let (previous, anchored) = old.split_at(at);
                    old = &anchored[1..];
                    let count = group.len() - 1;
                    (previous, &mut group[..count])
                } else {
                    (old, group)
                };
                let prefix = restore_matching_comments(previous.iter().zip(missing.iter_mut()));
                restore_matching_comments(
                    previous[prefix..]
                        .iter()
                        .rev()
                        .zip(missing[prefix..].iter_mut().rev()),
                );
            }
        }
    }

    pub(super) fn parents(&self) -> &[Option<NodeId>] {
        &self.parents
    }

    pub(super) fn work(&self) -> &Cell<usize> {
        &self.work
    }

    /// Reparse the rendered YAML and check it against the intended edit.
    pub(super) fn commit(&self, output: &str) -> Result<DocumentSyntax, Error> {
        emit::parse_rendered(
            self.into(),
            output,
            self.options,
            self.diagnostic_span,
            &self.requested_styles,
        )
    }
}

fn restore_matching_comments<'a>(
    pairs: impl Iterator<Item = (&'a Comment, &'a mut Comment<Option<Span>>)>,
) -> usize {
    let mut matched = 0;
    for (old, comment) in pairs {
        if comment.attachment() != old.attachment() {
            break;
        }
        comment.span = Some(old.span);
        matched += 1;
    }
    matched
}

fn preserve_style(old: &Node<Option<Span>>, replacement: &mut Node<Option<Span>>) {
    match (&old.kind, &mut replacement.kind) {
        (
            NodeKind::Scalar {
                style: previous, ..
            },
            NodeKind::Scalar { value, style },
        ) if emit::same_scalar_type(value, replacement.tag.as_deref(), *previous, *style) => {
            *style = *previous;
        }
        (
            NodeKind::Sequence {
                style: previous, ..
            },
            NodeKind::Sequence { style, .. },
        )
        | (
            NodeKind::Mapping {
                style: previous, ..
            },
            NodeKind::Mapping { style, .. },
        ) => {
            *style = *previous;
        }
        _ => {}
    }
}

fn remap(node: &mut Node<Option<Span>>, map: impl Fn(NodeId) -> NodeId) {
    match &mut node.kind {
        NodeKind::Sequence { items, .. } => {
            for item in items {
                *item = map(*item);
            }
        }
        NodeKind::Mapping { entries, .. } => {
            for (key, value) in entries {
                *key = map(*key);
                *value = map(*value);
            }
        }
        NodeKind::Scalar { .. } | NodeKind::Alias { .. } => {}
    }
}

/// Move `BeforeChild(removed)` comments to `following`, or to the collection's
/// end if there is no following child.
fn retarget_comments(node: &mut Node<Option<Span>>, removed: NodeId, following: Option<NodeId>) {
    for comment in &mut node.comments {
        if comment.position == CommentPosition::BeforeChild(removed) {
            comment.position =
                following.map_or(CommentPosition::After, CommentPosition::BeforeChild);
        }
    }
}

/// A gap comment precedes a sequence item or a mapping key.
fn comment_boundaries(kind: &NodeKind) -> impl Iterator<Item = NodeId> {
    let (items, entries): (&[NodeId], &[(NodeId, NodeId)]) = match kind {
        NodeKind::Sequence { items, .. } => (items, &[]),
        NodeKind::Mapping { entries, .. } => (&[], entries),
        NodeKind::Scalar { .. } | NodeKind::Alias { .. } => (&[], &[]),
    };
    items
        .iter()
        .copied()
        .chain(entries.iter().map(|&(key, _)| key))
}

/// Keep collection-owned gap comments before the corresponding child in the new value.
fn remap_comment_positions(comments: &mut [Comment<Option<Span>>], old: &NodeKind, new: &NodeKind) {
    if comments
        .iter()
        .any(|comment| matches!(comment.position, CommentPosition::BeforeChild(_)))
    {
        let corresponding: HashMap<_, _> = comment_boundaries(old)
            .zip(comment_boundaries(new))
            .collect();
        for comment in comments {
            if let CommentPosition::BeforeChild(child) = comment.position {
                comment.position = corresponding
                    .get(&child)
                    .copied()
                    .map_or(CommentPosition::After, CommentPosition::BeforeChild);
            }
        }
    }
}

pub(super) fn same_style(left: &NodeKind, right: &NodeKind) -> bool {
    match (left, right) {
        (NodeKind::Scalar { style: a, .. }, NodeKind::Scalar { style: b, .. }) => a == b,
        (NodeKind::Sequence { items, style: a }, NodeKind::Sequence { style: b, .. }) => {
            items.is_empty() || a == b
        }
        (NodeKind::Mapping { entries, style: a }, NodeKind::Mapping { style: b, .. }) => {
            entries.is_empty() || a == b
        }
        _ => false,
    }
}

/// Map node and comment locations without changing the syntax or node IDs.
pub(super) fn map_locations<Location, Target>(
    node: Node<Location>,
    map: impl Fn(Location) -> Target,
) -> Node<Target> {
    Node {
        span: map(node.span),
        kind: node.kind,
        anchor: node.anchor,
        tag: node.tag,
        comments: node
            .comments
            .into_iter()
            .map(|comment| Comment {
                span: map(comment.span),
                text: comment.text,
                placement: comment.placement,
                position: comment.position,
            })
            .collect(),
    }
}
