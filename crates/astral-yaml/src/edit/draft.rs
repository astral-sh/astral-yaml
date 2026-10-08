//! A mutable tree owns node identity, membership, and style requests. Freezing it
//! ends mutation and supplies the intended syntax for planning and verification.

use super::{Comment, CommentPosition, EditableDocument, Node, NodeKind, emit, view::Syntax};
use crate::{
    CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span, comments::CommentPlacement,
};
use std::{cell::Cell, collections::HashMap};

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

/// The final intended syntax. Planning can borrow it, but cannot mutate it.
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

impl From<&EditableDocument> for Draft {
    fn from(document: &EditableDocument) -> Self {
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
            diagnostic_span: document.nodes[document.root.0].span,
            work: Cell::new(document.options.max_nodes),
            introduced_bytes: 0,
        };
        for index in 0..draft.slots.len() {
            draft.link_children(NodeId(index));
        }
        draft
    }
}

impl Draft {
    pub(super) fn root(&self) -> NodeId {
        self.root
    }

    pub(super) fn node(&self, id: NodeId) -> &Node<Option<Span>> {
        &self.slots[id.0].node
    }

    pub(super) fn contains(&self, id: NodeId) -> bool {
        self.slots
            .get(id.0)
            .is_some_and(|slot| slot.membership.is_some())
    }

    pub(super) fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.membership(id).parent
    }

    fn membership(&self, id: NodeId) -> &Membership {
        self.slots[id.0].membership.as_ref().expect("live node")
    }

    fn membership_mut(&mut self, id: NodeId) -> &mut Membership {
        self.slots[id.0].membership.as_mut().expect("live node")
    }

    pub(super) fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.diagnostic_span,
            message,
        }
    }

    pub(super) fn charge(&self, amount: usize) -> Result<(), Error> {
        let remaining = self
            .work
            .get()
            .checked_sub(amount)
            .ok_or_else(|| self.error("editing resource limit exceeded"))?;
        self.work.set(remaining);
        Ok(())
    }

    fn introduce(&mut self, bytes: usize) -> Result<(), Error> {
        self.introduced_bytes = self
            .introduced_bytes
            .checked_add(bytes)
            .filter(|&size| size <= self.options.max_input_bytes)
            .ok_or_else(|| self.error("inserted YAML exceeds input byte limit"))?;
        Ok(())
    }

    fn fragment(
        &mut self,
        yaml: &str,
        parent: Option<NodeId>,
        available: usize,
    ) -> Result<EditableDocument, Error> {
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
        let document = EditableDocument::parse_with_options(yaml, options)?;
        self.charge(document.source.nodes.len())?;
        Ok(document)
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

    /// Assign final identities before attaching a parsed fragment. A replacement
    /// root reuses its destination directly, including self-referential aliases.
    fn attach(
        &mut self,
        document: EditableDocument,
        parent: Option<NodeId>,
        replacement: Option<(NodeId, bool)>,
    ) -> NodeId {
        let mut document = document.source;
        let fragment_root = document.root;
        let root = replacement.map_or(NodeId(self.slots.len()), |(id, _)| id);
        let offset = self.slots.len() + usize::from(replacement.is_none());
        let map = |id: NodeId| {
            if id == fragment_root {
                root
            } else {
                NodeId(offset + id.0 - usize::from(id.0 > fragment_root.0))
            }
        };
        let mut node = map_locations(document.nodes.remove(fragment_root.0), |_| None);
        remap(&mut node, map);
        if let Some((id, preserve)) = replacement {
            let old = self.node(id);
            if preserve {
                preserve_properties(old, &mut node);
            }
            node.comments = merged_comments(old, &node);
            node.span = old.span;
            self.slots[id.0] = Slot::live(node, parent);
        } else {
            self.slots.push(Slot::live(node, parent));
        }
        for node in document.nodes {
            let mut node = map_locations(node, |_| None);
            remap(&mut node, map);
            self.slots.push(Slot::live(node, None));
        }
        self.link_children(root);
        for index in offset..self.slots.len() {
            self.link_children(NodeId(index));
        }
        root
    }

    /// Collect and charge the whole removal before changing membership. A failed
    /// budget check must not leave half a subtree retired.
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
            self.slots[id.0].membership = None;
        }
    }

    pub(super) fn replace(&mut self, id: NodeId, yaml: &str, preserve: bool) -> Result<(), Error> {
        let parent = self.parent(id);
        let document = self.fragment(yaml, parent, self.available() + 1)?;
        let removed = self.removal([id])?;
        // Attaching resets the root's style request and membership. Retire its
        // previous descendants afterwards; they cannot refer to imported slots.
        self.attach(document, parent, Some((id, preserve)));
        self.retire(
            removed
                .into_iter()
                .filter(|&removed| removed != id)
                .collect(),
        );
        Ok(())
    }

    fn sequence_items_mut(&mut self, id: NodeId) -> Result<&mut Vec<NodeId>, Error> {
        match &mut self.slots[id.0].node.kind {
            NodeKind::Sequence { items, .. } => Ok(items),
            _ => Err(Error {
                span: self.diagnostic_span,
                message: "expected a sequence",
            }),
        }
    }

    fn mapping_entries_mut(&mut self, id: NodeId) -> Result<&mut Vec<(NodeId, NodeId)>, Error> {
        match &mut self.slots[id.0].node.kind {
            NodeKind::Mapping { entries, .. } => Ok(entries),
            _ => Err(Error {
                span: self.diagnostic_span,
                message: "expected a mapping",
            }),
        }
    }

    fn scalar_style_mut(&mut self, id: NodeId) -> Result<&mut ScalarStyle, Error> {
        match &mut self.slots[id.0].node.kind {
            NodeKind::Scalar { style, .. } => Ok(style),
            _ => Err(Error {
                span: self.diagnostic_span,
                message: "expected a scalar",
            }),
        }
    }

    pub(super) fn insert_item(
        &mut self,
        id: NodeId,
        index: usize,
        yaml: &str,
    ) -> Result<(), Error> {
        let items = self.sequence_items_mut(id)?;
        if index > items.len() {
            return Err(self.error("item index out of bounds"));
        }
        let document = self.fragment(yaml, Some(id), self.available())?;
        let child = self.attach(document, Some(id), None);
        self.sequence_items_mut(id)?.insert(index, child);
        Ok(())
    }

    pub(super) fn remove_item(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let items = self.sequence_items_mut(id)?;
        let Some(&child) = items.get(index) else {
            return Err(self.error("item index out of bounds"));
        };
        let following = items.get(index + 1).copied();
        let removed = self.removal([child])?;
        self.sequence_items_mut(id)?.remove(index);
        retarget_comments(&mut self.slots[id.0].node, child, following);
        self.retire(removed);
        Ok(())
    }

    pub(super) fn insert_entry(
        &mut self,
        id: NodeId,
        index: usize,
        key: &str,
        value: &str,
    ) -> Result<(), Error> {
        let entries = self.mapping_entries_mut(id)?;
        if index > entries.len() {
            return Err(self.error("entry index out of bounds"));
        }
        // Parse both fragments before attaching either one. Failed parsing cannot
        // leave a live key without an entry owning it.
        let key = self.fragment(key, Some(id), self.available())?;
        let value = self.fragment(
            value,
            Some(id),
            self.available().saturating_sub(key.source.nodes.len()),
        )?;
        let key = self.attach(key, Some(id), None);
        let value = self.attach(value, Some(id), None);
        self.mapping_entries_mut(id)?.insert(index, (key, value));
        Ok(())
    }

    pub(super) fn remove_entry(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let entries = self.mapping_entries_mut(id)?;
        let Some(&(key, value)) = entries.get(index) else {
            return Err(self.error("entry index out of bounds"));
        };
        let following = entries.get(index + 1).map(|&(key, _)| key);
        let removed = self.removal([key, value])?;
        self.mapping_entries_mut(id)?.remove(index);
        retarget_comments(&mut self.slots[id.0].node, key, following);
        self.retire(removed);
        Ok(())
    }

    pub(super) fn set_scalar_style(&mut self, id: NodeId, style: ScalarStyle) -> Result<(), Error> {
        let before = *self.scalar_style_mut(id)?;
        let node = self.node(id);
        if emit::meaning(node, before) != emit::meaning(node, style) {
            return Err(self.error("requested style changes the scalar type"));
        }
        *self.scalar_style_mut(id)? = style;
        self.membership_mut(id).requested_style = true;
        Ok(())
    }

    pub(super) fn set_collection_style(
        &mut self,
        id: NodeId,
        style: CollectionStyle,
    ) -> Result<(), Error> {
        match &mut self.slots[id.0].node.kind {
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
        position: Option<CommentPosition>,
    ) -> Result<(), Error> {
        self.comment_text(text)?;
        if let Some(CommentPosition::BeforeChild(child)) = position {
            if placement != CommentPlacement::Detached {
                return Err(self.error("collection gaps require detached comments"));
            }
            let direct = match &self.node(id).kind {
                NodeKind::Sequence { items, .. } => items.contains(&child),
                NodeKind::Mapping { entries, .. } => entries.iter().any(|&(key, _)| key == child),
                _ => false,
            };
            if !direct {
                return Err(self.error("comment boundary is not a collection child"));
            }
        }
        let comments = &mut self.slots[id.0].node.comments;
        if index > comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        let position = position.unwrap_or_else(|| {
            if placement == CommentPlacement::Leading {
                CommentPosition::Before
            } else if placement == CommentPlacement::Detached {
                comments[index..]
                    .iter()
                    .find_map(|comment| match comment.position {
                        CommentPosition::BeforeChild(_) => Some(comment.position),
                        _ => None,
                    })
                    .unwrap_or(CommentPosition::After)
            } else {
                CommentPosition::After
            }
        });
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
        let Some(comment) = self.slots[id.0].node.comments.get_mut(index) else {
            return Err(self.error("comment index out of bounds"));
        };
        text.clone_into(&mut comment.text);
        Ok(())
    }

    pub(super) fn remove_comment(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        let comments = &mut self.slots[id.0].node.comments;
        if index >= comments.len() {
            return Err(self.error("comment index out of bounds"));
        }
        comments.remove(index);
        Ok(())
    }

    /// Planning only receives a graph whose live aliases have live, anchored
    /// targets. A transaction may temporarily remove a definition while also
    /// removing its aliases, so this check belongs at the end of mutation.
    pub(super) fn freeze(self) -> Result<Candidate, Error> {
        for slot in self.slots.iter().filter(|slot| slot.membership.is_some()) {
            if let NodeKind::Alias { target } = slot.node.kind {
                if !self.contains(target) {
                    return Err(self.error("alias target was removed"));
                }
                if self.node(target).anchor.is_none() {
                    return Err(self.error("alias target has no anchor"));
                }
            }
        }

        let mut nodes = Vec::with_capacity(self.slots.len());
        let mut parents = Vec::with_capacity(self.slots.len());
        let mut requested_styles = Vec::new();
        for (index, slot) in self.slots.into_iter().enumerate() {
            nodes.push(slot.node);
            let parent = if let Some(membership) = slot.membership {
                if membership.requested_style {
                    requested_styles.push(NodeId(index));
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
    pub(super) fn parents(&self) -> &[Option<NodeId>] {
        &self.parents
    }

    pub(super) fn work(&self) -> &Cell<usize> {
        &self.work
    }

    /// Only reparsed output that agrees with the frozen candidate can commit.
    pub(super) fn commit(&self, output: &str) -> Result<EditableDocument, Error> {
        let error = |message| Error {
            span: self.diagnostic_span,
            message,
        };
        let parsed = EditableDocument::parse_with_options(output, self.options)?;
        let correspondence = emit::correspondence(self, &parsed)
            .ok_or_else(|| error("emission changed YAML content or alias identity"))?;
        for &id in &self.requested_styles {
            let target = correspondence[id.0].expect("styled node is retained");
            if !same_style(&self.nodes[id.0].kind, &parsed.source.nodes[target.0].kind) {
                return Err(error(
                    "requested style is not representable in this context",
                ));
            }
        }
        if !emit::comments_preserved(self, &parsed, &correspondence) {
            return Err(error("emission changed comment ownership or placement"));
        }
        Ok(parsed)
    }
}

fn preserve_properties(old: &Node<Option<Span>>, replacement: &mut Node<Option<Span>>) {
    if let Some(anchor) = &old.anchor {
        replacement.anchor = Some(anchor.clone());
    }
    let compatible = match (&old.kind, &replacement.kind) {
        (
            NodeKind::Scalar {
                style: previous, ..
            },
            NodeKind::Scalar { style, .. },
        ) => emit::meaning(replacement, *previous) == emit::meaning(replacement, *style),
        _ => true,
    };
    match (&old.kind, &mut replacement.kind) {
        (
            NodeKind::Scalar {
                style: previous, ..
            },
            NodeKind::Scalar { style, .. },
        ) if compatible => {
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
    for comment in &mut node.comments {
        if let CommentPosition::BeforeChild(child) = &mut comment.position {
            *child = map(*child);
        }
    }
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
        NodeKind::Alias { target, .. } => *target = map(*target),
        NodeKind::Scalar { .. } => {}
    }
}

/// A removed child leaves its preceding gap before the next surviving child.
fn retarget_comments(node: &mut Node<Option<Span>>, removed: NodeId, following: Option<NodeId>) {
    for comment in &mut node.comments {
        if comment.position == CommentPosition::BeforeChild(removed) {
            comment.position =
                following.map_or(CommentPosition::After, CommentPosition::BeforeChild);
        }
    }
}

/// Merge the retained and imported comments at their structural positions.
/// Each list keeps its own order; retained comments precede imports at a shared
/// boundary. Collection gaps retain their ordinal position in the replacement.
fn merged_comments(
    old: &Node<Option<Span>>,
    replacement: &Node<Option<Span>>,
) -> Vec<Comment<Option<Span>>> {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum Boundary {
        InlineBefore,
        DetachedBefore,
        LeadingBefore,
        Header,
        Opening,
        BeforeChild(usize),
        InlineAfter,
        TrailingAfter,
        DetachedAfter,
    }
    fn boundaries(kind: &NodeKind) -> Vec<NodeId> {
        match kind {
            NodeKind::Sequence { items, .. } => items.clone(),
            NodeKind::Mapping { entries, .. } => entries.iter().map(|&(key, _)| key).collect(),
            _ => Vec::new(),
        }
    }
    let old_children = boundaries(&old.kind);
    let new_children = boundaries(&replacement.kind);
    let following: HashMap<_, _> = old_children
        .into_iter()
        .enumerate()
        .map(|(index, child)| (child, new_children.get(index).copied()))
        .collect();
    let mut comments = old.comments.clone();
    for comment in &mut comments {
        if let CommentPosition::BeforeChild(child) = comment.position {
            comment.position =
                following[&child].map_or(CommentPosition::After, CommentPosition::BeforeChild);
        }
    }
    let positions: HashMap<_, _> = new_children
        .into_iter()
        .enumerate()
        .map(|(index, child)| (child, index))
        .collect();
    let boundary = |comment: &Comment<Option<Span>>| match comment.position {
        CommentPosition::Before => match comment.placement {
            CommentPlacement::Inline => Boundary::InlineBefore,
            CommentPlacement::Detached => Boundary::DetachedBefore,
            _ => Boundary::LeadingBefore,
        },
        CommentPosition::Header => Boundary::Header,
        CommentPosition::Opening => Boundary::Opening,
        CommentPosition::BeforeChild(child) => Boundary::BeforeChild(positions[&child]),
        CommentPosition::After if comment.placement == CommentPlacement::Inline => {
            Boundary::InlineAfter
        }
        CommentPosition::After if comment.placement == CommentPlacement::Detached => {
            Boundary::DetachedAfter
        }
        CommentPosition::After => Boundary::TrailingAfter,
    };
    let mut retained = comments.into_iter().peekable();
    let mut imported = replacement.comments.iter().cloned().peekable();
    let mut merged = Vec::with_capacity(retained.len() + imported.len());
    while let (Some(left), Some(right)) = (retained.peek(), imported.peek()) {
        let next = if boundary(left) <= boundary(right) {
            retained.next()
        } else {
            imported.next()
        };
        merged.push(next.expect("remaining comment"));
    }
    merged.extend(retained);
    merged.extend(imported);
    merged
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

/// Change provenance without changing any syntax or node identity.
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
