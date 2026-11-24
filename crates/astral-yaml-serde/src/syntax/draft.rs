//! Mutable syntax for an edit transaction.
//! [`Draft::freeze`] ends mutation and produces the syntax used to check the output.

use super::view::Syntax;
use super::{Comment, CommentPosition, DocumentSyntax, Node, NodeKind, emit};
use crate::ser::INSERTED_BYTE_LIMIT;
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{Budget, CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
struct Membership {
    parent: Option<NodeId>,
    is_key: bool,
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
                is_key: false,
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
    work: Budget,
    inserted_bytes: Budget,
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
    work: Budget,
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
                .map(|mut node| {
                    // Empty delimiters tell us nothing about the style to use
                    // if this collection grows during the transaction.
                    if let Some(style) = empty_collection_style(&mut node.kind) {
                        *style = CollectionStyle::Block;
                    }
                    Slot::live(node.map_locations(Some), None)
                })
                .collect(),
            root: document.root,
            options: document.options,
            diagnostic_span: document.nodes[document.root.index()].span,
            work: Budget::new(document.options.max_nodes),
            inserted_bytes: Budget::new(document.options.max_input_bytes),
        };
        for index in 0..draft.slots.len() {
            draft.link_children(NodeId::from_index(index), false);
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

#[derive(Clone, Copy)]
enum Destination {
    Insert(NodeId),
    Replace(NodeId, ReplacementStyle),
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

    pub(super) fn byte_budget(&self) -> (usize, Span) {
        (self.inserted_bytes.remaining(), self.diagnostic_span)
    }

    fn introduce(&self, bytes: usize) -> Result<(), Error> {
        self.inserted_bytes
            .try_charge(bytes)
            .ok_or_else(|| self.error(INSERTED_BYTE_LIMIT))
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

    fn link_children(&mut self, id: NodeId, generated: bool) {
        match &self.node(id).kind {
            NodeKind::Mapping { entries, .. } => {
                for (key, value) in entries.clone() {
                    self.link_key(key, id, generated);
                    self.membership_mut(value).parent = Some(id);
                }
            }
            NodeKind::Sequence { items, .. } => {
                for child in items.clone() {
                    self.membership_mut(child).parent = Some(id);
                }
            }
            NodeKind::Scalar { .. } | NodeKind::Alias { .. } => {}
        }
    }

    fn link_key(&mut self, key: NodeId, parent: NodeId, generated: bool) {
        let membership = self.membership_mut(key);
        membership.parent = Some(parent);
        membership.is_key = true;
        if generated {
            prefer_plain_key(&mut self.slots[key.index()].node.kind);
        }
    }

    /// Insert the serialized tree, reusing the selected node’s ID on replacement.
    fn attach(
        &mut self,
        value: SerializedValue<'_>,
        destination: Destination,
    ) -> Result<NodeId, Error> {
        let SerializedValue {
            root: value_root,
            mut nodes,
        } = value;
        let next = self.slots.len();
        let (root, offset, generated) = match destination {
            Destination::Insert(_) => (NodeId::from_index(next), next + 1, true),
            Destination::Replace(id, style) => (id, next, style == ReplacementStyle::Preserve),
        };
        // Child IDs skip the root, which has its own slot.
        let map = |id: NodeId| {
            NodeId::from_index(offset + id.index() - usize::from(id.index() > value_root.index()))
        };
        let relocate = |parsed| {
            let mut node = Node::from_parsed(parsed).map_locations(|_| None);
            node.kind.remap_children(map);
            // Serde serializes every collection in flow style. Prefer block for each
            // new collection and let the final enclosing flow context override it.
            if generated
                && let NodeKind::Sequence { style, .. } | NodeKind::Mapping { style, .. } =
                    &mut node.kind
            {
                *style = CollectionStyle::Block;
            }
            node
        };
        let mut node = relocate(nodes.remove(value_root.index()));
        if let Destination::Insert(parent) = destination {
            self.slots.push(Slot::live(node, Some(parent)));
        } else {
            let requested = if generated {
                self.preserve_style(root, &mut node)?
            } else {
                false
            };
            self.membership_mut(root).requested_style = requested;
            let old = &mut self.slots[root.index()].node;
            node.anchor = old.anchor.take();
            node.comments = std::mem::take(&mut old.comments);
            remap_comment_positions(&mut node.comments, &old.kind, &node.kind);
            node.span = old.span;
            *old = node;
        }
        for node in nodes {
            self.slots.push(Slot::live(relocate(node), None));
        }
        self.link_children(root, generated);
        for index in offset..self.slots.len() {
            self.link_children(NodeId::from_index(index), generated);
        }
        Ok(root)
    }

    /// Retain compatible styles and report whether the request still applies.
    fn preserve_style(
        &self,
        id: NodeId,
        replacement: &mut Node<Option<Span>>,
    ) -> Result<bool, Error> {
        let membership = self.membership(id);
        if membership.is_key {
            prefer_plain_key(&mut replacement.kind);
        }
        let old = self.node(id);
        match (&old.kind, &mut replacement.kind) {
            (
                NodeKind::Scalar {
                    value: old_value,
                    style: previous,
                },
                NodeKind::Scalar { value, style },
            ) => {
                // Serde can buffer a local enum variant as an untagged value.
                // For a new payload, preserve its implicit type by default.
                let resolution_tag = replacement.tag.as_deref().filter(|&tag| {
                    membership.requested_style
                        || crate::with::yaml_tag::local_name(tag).is_none()
                        || (old.tag() == Some(tag) && old_value == value)
                });
                if !emit::same_scalar_type(value, resolution_tag, *previous, *style) {
                    if membership.requested_style {
                        return Err(self.error("requested style changes the scalar type"));
                    }
                    return Ok(false);
                }
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
            _ => return Ok(false),
        }
        Ok(membership.requested_style)
    }

    /// Collect all nodes to remove before retiring any of them.
    /// Charge the traversal here so a budget error cannot leave a partially removed subtree.
    fn removal(&self, roots: impl IntoIterator<Item = NodeId>) -> Result<Vec<NodeId>, Error> {
        let mut pending: Vec<_> = roots.into_iter().collect();
        let mut removed = Vec::new();
        while let Some(id) = pending.pop() {
            self.charge(1)?;
            self.node(id).kind.append_children_to(&mut pending);
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
        self.charge(1)?;
        let mut children = Vec::new();
        self.node(id).kind.append_children_to(&mut children);
        let removed = self.removal(children)?;
        self.attach(document, Destination::Replace(id, style))?;
        self.retire(removed);
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
        let shifted = items.len() - index;
        self.charge(shifted)?;

        let document = self.parse_value(yaml, Some(id), self.available())?;
        let child = self.attach(document, Destination::Insert(id))?;
        self.sequence_items_mut(id)?.insert(index, child);
        Ok(child)
    }

    pub(super) fn remove_item(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let items = self.sequence_items_mut(id)?;
        let Some(&child) = items.get(index) else {
            return Err(self.error("item index out of bounds"));
        };
        let following = items.get(index + 1).copied();
        // Retargeting comments visits the whole list, even for the last item.
        let shifted = items.len() - index - 1;
        self.charge(shifted + self.node(id).comments.len())?;
        let removed = self.removal([child])?;
        self.sequence_items_mut(id)?.remove(index);
        self.slots[id.index()]
            .node
            .retarget_child_comments(child, following);
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
        let shifted = entries.len() - index;
        self.charge(shifted)?;

        // Parse the key and value before attaching either one. Failed parsing cannot
        // leave a live key without an entry owning it.
        let key = self.parse_value(key, Some(id), self.available())?;
        let value = self.parse_value(
            value,
            Some(id),
            self.available().saturating_sub(key.nodes.len()),
        )?;
        let key = self.attach(key, Destination::Insert(id))?;
        self.link_key(key, id, true);
        let value = self.attach(value, Destination::Insert(id))?;
        self.mapping_entries_mut(id)?.insert(index, (key, value));
        Ok((key, value))
    }

    pub(super) fn remove_entry(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let entries = self.mapping_entries_mut(id)?;
        let Some(&(key, value)) = entries.get(index) else {
            return Err(self.error("entry index out of bounds"));
        };
        let following = entries.get(index + 1).map(|&(key, _)| key);
        let shifted = entries.len() - index - 1;
        self.charge(shifted + self.node(id).comments.len())?;
        let removed = self.removal([key, value])?;
        self.mapping_entries_mut(id)?.remove(index);
        self.slots[id.index()]
            .node
            .retarget_child_comments(key, following);
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
        let comments = &self.node(id).comments;
        if index > comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        self.charge(comments.len() - index)?;
        let position = match placement {
            CommentPlacement::Leading => CommentPosition::Before,
            CommentPlacement::Detached => comments[index..]
                .iter()
                .map(|comment| comment.position)
                .find(|position| matches!(position, CommentPosition::BeforeChild(_)))
                .unwrap_or(CommentPosition::After),
            CommentPlacement::Inline | CommentPlacement::Trailing => CommentPosition::After,
        };
        self.slots[id.index()].node.comments.insert(
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
        let comments = &self.node(id).comments;
        if index >= comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        self.charge(comments.len() - index - 1)?;
        self.slots[id.index()].node.comments.remove(index);
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
        for (index, mut slot) in self.slots.into_iter().enumerate() {
            if let Some(style) = empty_collection_style(&mut slot.node.kind) {
                *style = CollectionStyle::Flow;
            }
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

    pub(super) fn work(&self) -> &Budget {
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

/// Prefer plain spelling for generated string keys. The emitter may still
/// need to quote them in the final flow context.
fn prefer_plain_key(kind: &mut NodeKind) {
    if let NodeKind::Scalar { value, style } = kind
        && *style == ScalarStyle::DoubleQuoted
        && astral_yaml::emit::plain_block_string_key(value)
    {
        *style = ScalarStyle::Plain;
    }
}

fn empty_collection_style(kind: &mut NodeKind) -> Option<&mut CollectionStyle> {
    match kind {
        NodeKind::Sequence { items, style } if items.is_empty() => Some(style),
        NodeKind::Mapping { entries, style } if entries.is_empty() => Some(style),
        _ => None,
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
