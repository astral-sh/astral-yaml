use super::{
    Comment, CommentPosition, EditableDocument, Node, NodeKind, draft::Draft, emit, layout::Layout,
    plan::Plan, source::Source,
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;

/// A node reference belonging to one edit transaction. Handles can be cloned,
/// but cannot be used in another transaction or after their node is deleted.
#[derive(Clone, Debug)]
pub struct Handle {
    transaction: Arc<()>,
    id: NodeId,
}

/// Mutable candidate syntax. Obtain handles by navigating from [`Self::root`].
/// Node spans describe the original source; inserted syntax has empty spans.
#[derive(Debug)]
pub struct Editor<'a> {
    original: &'a EditableDocument,
    candidate: Draft,
    transaction: Arc<()>,
    parents: Vec<Option<NodeId>>,
    alive: Vec<bool>,
    rewrites: HashSet<NodeId>,
    root_comments_changed: bool,
    work: Cell<usize>,
    introduced_bytes: usize,
    failure: Option<Error>,
}

impl EditableDocument {
    /// Applies syntax edits atomically. Failed mutation operations poison the transaction,
    /// even if the callback catches their errors.
    ///
    /// # Errors
    /// Returns an error for invalid edits, invalid emitted syntax, altered alias
    /// identity, or exhausted resource limits. The document remains unchanged.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit(edit)
    }

    /// Applies a transaction whose callback uses an application-defined error.
    /// The callback's result is returned only after a successful commit.
    ///
    /// # Errors
    /// Returns callback or edit errors, leaving the document unchanged.
    pub fn try_edit<R, E: From<Error>>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut editor = Editor::new(self);
        let result = edit(&mut editor)?;
        let candidate = editor.finish()?;
        *self = candidate;
        Ok(result)
    }
}

impl<'a> Editor<'a> {
    fn new(original: &'a EditableDocument) -> Self {
        let mut parents = vec![None; original.nodes.len()];
        for (index, node) in original.nodes.iter().enumerate() {
            let mut children = Vec::new();
            emit::children(&node.kind, &mut children);
            for child in children {
                parents[child.0] = Some(NodeId(index));
            }
        }
        Self {
            original,
            candidate: Draft::from(original),
            transaction: Arc::new(()),
            parents,
            alive: vec![true; original.nodes.len()],
            rewrites: HashSet::new(),
            root_comments_changed: false,
            work: Cell::new(original.options.max_nodes),
            introduced_bytes: 0,
            failure: None,
        }
    }

    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.original.nodes[self.original.root.0].span,
            message,
        }
    }

    fn charge(&self, amount: usize) -> Result<(), Error> {
        let remaining = self
            .work
            .get()
            .checked_sub(amount)
            .ok_or_else(|| self.error("editing resource limit exceeded"))?;
        self.work.set(remaining);
        Ok(())
    }

    fn id(&self, handle: &Handle) -> Result<NodeId, Error> {
        self.charge(1)?;
        if !Arc::ptr_eq(&self.transaction, &handle.transaction)
            || !self.alive.get(handle.id.0).copied().unwrap_or(false)
        {
            return Err(self.error("foreign or deleted edit handle"));
        }
        Ok(handle.id)
    }

    fn handle(&self, id: NodeId) -> Handle {
        Handle {
            transaction: Arc::clone(&self.transaction),
            id,
        }
    }

    fn mutate(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = operation(self);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    /// The root of this transaction's candidate document.
    #[must_use]
    pub fn root(&self) -> Handle {
        self.handle(self.candidate.root)
    }

    /// Inspects a live node in this transaction.
    ///
    /// # Errors
    /// Rejects foreign/deleted handles and exhausted work budgets.
    pub fn node(&self, handle: &Handle) -> Result<&Node, Error> {
        Ok(&self.candidate.nodes[self.id(handle)?.0])
    }

    /// Returns sequence-item handles in source order.
    ///
    /// # Errors
    /// Rejects invalid handles, non-sequences, and exhausted work budgets.
    pub fn sequence_items(&self, handle: &Handle) -> Result<Vec<Handle>, Error> {
        let NodeKind::Sequence { items, .. } = &self.node(handle)?.kind else {
            return Err(self.error("expected a sequence"));
        };
        self.charge(items.len())?;
        Ok(items.iter().map(|&id| self.handle(id)).collect())
    }

    /// Returns mapping key/value handles in source order, including complex keys.
    ///
    /// # Errors
    /// Rejects invalid handles, non-mappings, and exhausted work budgets.
    pub fn mapping_entries(&self, handle: &Handle) -> Result<Vec<(Handle, Handle)>, Error> {
        let NodeKind::Mapping { entries, .. } = &self.node(handle)?.kind else {
            return Err(self.error("expected a mapping"));
        };
        self.charge(entries.len())?;
        Ok(entries
            .iter()
            .map(|&(key, value)| (self.handle(key), self.handle(value)))
            .collect())
    }

    fn mark(&mut self, id: NodeId) {
        self.rewrites.insert(id);
    }

    fn mark_parent(&mut self, id: NodeId) {
        self.mark(self.parents[id.0].unwrap_or(id));
    }

    fn remove_tree(&mut self, id: NodeId) -> Result<(), Error> {
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            self.charge(1)?;
            self.alive[id.0] = false;
            self.rewrites.remove(&id);
            self.candidate.requested_styles.remove(&id);
            emit::children(&self.candidate.nodes[id.0].kind, &mut pending);
        }
        Ok(())
    }

    fn import(&mut self, yaml: &str, parent: Option<NodeId>) -> Result<NodeId, Error> {
        self.introduced_bytes = self
            .introduced_bytes
            .checked_add(yaml.len())
            .filter(|&size| size <= self.original.options.max_input_bytes)
            .ok_or_else(|| self.error("inserted YAML exceeds input byte limit"))?;
        let mut depth = 0;
        let mut ancestor = parent;
        while let Some(id) = ancestor {
            depth += 1;
            ancestor = self.parents[id.0];
        }
        if depth >= self.original.options.max_depth {
            return Err(self.error("editing nesting limit exceeded"));
        }
        let available = self
            .original
            .options
            .max_nodes
            .saturating_sub(self.candidate.nodes.len());
        let mut options = self.original.options;
        options.max_nodes = available;
        options.max_depth -= depth;
        let document = EditableDocument::parse_with_options(yaml, options)?;
        self.charge(document.nodes.len())?;
        let offset = self.candidate.nodes.len();
        let root = NodeId(document.root.0 + offset);
        for mut node in document.nodes {
            remap(&mut node, |id| NodeId(id.0 + offset));
            node.span = Span { start: 0, end: 0 };
            for comment in &mut node.comments {
                comment.span = node.span;
            }
            self.candidate.nodes.push(node);
            self.alive.push(true);
            self.parents.push(None);
        }
        self.parents[root.0] = parent;
        for index in offset..self.candidate.nodes.len() {
            let mut children = Vec::new();
            emit::children(&self.candidate.nodes[index].kind, &mut children);
            for child in children {
                self.parents[child.0] = Some(NodeId(index));
            }
        }
        Ok(root)
    }

    /// Replaces a node with a parsed single-node YAML fragment, retaining its
    /// outer comment attachments. Its handle remains valid; handles to its old
    /// descendants expire. Later operations see the replacement's content.
    ///
    /// # Errors
    /// Rejects invalid fragments, handles, or limits.
    pub fn replace(&mut self, handle: &Handle, yaml: &str) -> Result<(), Error> {
        self.replace_impl(handle, yaml, false)
    }

    /// Replaces a node from a YAML fragment, retaining its anchor, outer comments,
    /// and compatible scalar or collection style. The fragment supplies tags.
    /// Unlike an explicit style request, the retained style may fall back when
    /// the new content cannot use it.
    ///
    /// # Errors
    /// Rejects invalid fragments, handles, or limits.
    pub fn replace_preserving_style(&mut self, handle: &Handle, yaml: &str) -> Result<(), Error> {
        self.replace_impl(handle, yaml, true)
    }

    fn replace_impl(&mut self, handle: &Handle, yaml: &str, preserve: bool) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            let offset = editor.candidate.nodes.len();
            let root = editor.import(yaml, editor.parents[id.0])?;
            if preserve {
                let old = &editor.candidate.nodes[id.0];
                let mut replacement = editor.candidate.nodes[root.0].clone();
                replacement.anchor.clone_from(&old.anchor);
                match (&old.kind, &mut replacement.kind) {
                    (
                        NodeKind::Scalar {
                            style: previous, ..
                        },
                        NodeKind::Scalar { style, .. },
                    ) => {
                        if emit::meaning(&editor.candidate.nodes[root.0], *previous)
                            == emit::meaning(&editor.candidate.nodes[root.0], *style)
                        {
                            *style = *previous;
                        }
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
                editor.candidate.nodes[root.0] = replacement;
            }
            let simple = inline_node(&editor.candidate.nodes[id.0])
                && inline_node(&editor.candidate.nodes[root.0]);
            let comments = merged_comments(
                &editor.candidate.nodes[id.0],
                &editor.candidate.nodes[root.0],
            );
            let span = editor.candidate.nodes[id.0].span;
            editor.remove_tree(id)?;
            editor.alive[id.0] = true;
            editor.alive[root.0] = false;
            editor.candidate.nodes[id.0] = editor.candidate.nodes[root.0].clone();
            editor.candidate.nodes[id.0].comments = comments;
            editor.candidate.nodes[id.0].span = span;
            for index in offset..editor.candidate.nodes.len() {
                remap(&mut editor.candidate.nodes[index], |target| {
                    if target == root { id } else { target }
                });
                if editor.parents[index] == Some(root) {
                    editor.parents[index] = Some(id);
                }
            }
            remap(&mut editor.candidate.nodes[id.0], |target| {
                if target == root { id } else { target }
            });
            if simple
                && editor.candidate.nodes[id.0]
                    .comments
                    .iter()
                    .all(|c| !c.span.is_empty())
            {
                editor.mark(id);
            } else {
                editor.mark_parent(id);
            }
            Ok(())
        })
    }

    /// Inserts a parsed item before `index`; `len` appends.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, fragments, node kinds, or limits.
    pub fn insert_item(
        &mut self,
        sequence: &Handle,
        index: usize,
        yaml: &str,
    ) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(sequence)?;
            let child = editor.import(yaml, Some(id))?;
            let NodeKind::Sequence { items, .. } = &mut editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a sequence"));
            };
            if index > items.len() {
                return Err(editor.error("item index out of bounds"));
            }
            items.insert(index, child);
            editor.mark(id);
            Ok(())
        })
    }

    /// Removes an item and comments owned by its subtree.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, node kinds, or limits.
    pub fn remove_item(&mut self, sequence: &Handle, index: usize) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(sequence)?;
            let NodeKind::Sequence { items, .. } = &mut editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a sequence"));
            };
            if index >= items.len() {
                return Err(editor.error("item index out of bounds"));
            }
            let child = items.remove(index);
            let following = items.get(index).copied();
            retarget_comments(&mut editor.candidate.nodes[id.0], child, following);
            editor.remove_tree(child)?;
            editor.mark(id);
            Ok(())
        })
    }

    /// Inserts an ordered mapping entry from two single-node YAML fragments.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, fragments, node kinds, or limits.
    pub fn insert_entry(
        &mut self,
        mapping: &Handle,
        index: usize,
        key: &str,
        value: &str,
    ) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(mapping)?;
            let key = editor.import(key, Some(id))?;
            let value = editor.import(value, Some(id))?;
            let NodeKind::Mapping { entries, .. } = &mut editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a mapping"));
            };
            if index > entries.len() {
                return Err(editor.error("entry index out of bounds"));
            }
            entries.insert(index, (key, value));
            editor.mark(id);
            Ok(())
        })
    }

    /// Removes an entry and comments owned by its key/value subtrees.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, node kinds, or limits.
    pub fn remove_entry(&mut self, mapping: &Handle, index: usize) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(mapping)?;
            let NodeKind::Mapping { entries, .. } = &mut editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a mapping"));
            };
            if index >= entries.len() {
                return Err(editor.error("entry index out of bounds"));
            }
            let (key, value) = entries.remove(index);
            let following = entries.get(index).map(|&(key, _)| key);
            retarget_comments(&mut editor.candidate.nodes[id.0], key, following);
            editor.remove_tree(key)?;
            editor.remove_tree(value)?;
            editor.mark(id);
            Ok(())
        })
    }

    /// Requests a scalar style. Commit rejects styles that change the scalar's
    /// implicit type or cannot represent its text in the surrounding context.
    ///
    /// # Errors
    /// Rejects invalid handles, non-scalars, or exhausted work budgets.
    pub fn set_scalar_style(&mut self, handle: &Handle, style: ScalarStyle) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            let NodeKind::Scalar { style: current, .. } = &mut editor.candidate.nodes[id.0].kind
            else {
                return Err(editor.error("expected a scalar"));
            };
            let before = *current;
            *current = style;
            if super::emit::meaning(&editor.candidate.nodes[id.0], before)
                != super::emit::meaning(&editor.candidate.nodes[id.0], style)
            {
                return Err(editor.error("requested style changes the scalar type"));
            }
            editor.candidate.requested_styles.insert(id);
            if matches!(before, ScalarStyle::Literal | ScalarStyle::Folded)
                || matches!(style, ScalarStyle::Literal | ScalarStyle::Folded)
            {
                editor.mark_parent(id);
            } else {
                editor.mark(id);
            }
            Ok(())
        })
    }

    /// Requests block or flow collection syntax. An empty collection is emitted
    /// in flow style regardless of the requested preference.
    ///
    /// # Errors
    /// Rejects invalid handles, non-collections, or exhausted work budgets.
    pub fn set_collection_style(
        &mut self,
        handle: &Handle,
        style: CollectionStyle,
    ) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            match &mut editor.candidate.nodes[id.0].kind {
                NodeKind::Sequence { style: current, .. }
                | NodeKind::Mapping { style: current, .. } => *current = style,
                _ => return Err(editor.error("expected a collection")),
            }
            editor.candidate.requested_styles.insert(id);
            editor.mark(id);
            Ok(())
        })
    }

    /// Inserts comment text (without `#` or a line break) at an owner-local index.
    /// Inline comments require distinct syntax headers in the emitted layout.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, multiline text, or resource limits.
    /// Commit rejects placements that cannot preserve the node's attachments.
    pub fn insert_comment(
        &mut self,
        handle: &Handle,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            editor.comment_text(text)?;
            let comments = &mut editor.candidate.nodes[id.0].comments;
            if index > comments.len() {
                return Err(editor.error("comment index out of bounds"));
            }
            comments.insert(
                index,
                Comment {
                    text: text.to_owned(),
                    placement,
                    position: if placement == CommentPlacement::Leading {
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
                    },
                    span: Span { start: 0, end: 0 },
                },
            );
            editor.mark_parent(id);
            Ok(())
        })
    }

    /// Replaces an owned comment's text, keeping its placement.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, multiline text, or resource limits.
    pub fn replace_comment(
        &mut self,
        handle: &Handle,
        index: usize,
        text: &str,
    ) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            editor.comment_text(text)?;
            let Some(comment) = editor.candidate.nodes[id.0].comments.get_mut(index) else {
                return Err(editor.error("comment index out of bounds"));
            };
            text.clone_into(&mut comment.text);
            let span = comment.span;
            editor.comment_change(id, span);
            Ok(())
        })
    }

    /// Removes a directly owned comment.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, or resource limits.
    pub fn remove_comment(&mut self, handle: &Handle, index: usize) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            let comments = &mut editor.candidate.nodes[id.0].comments;
            if index >= comments.len() {
                return Err(editor.error("comment index out of bounds"));
            }
            let comment = comments.remove(index);
            editor.comment_change(id, comment.span);
            Ok(())
        })
    }

    fn comment_text(&mut self, text: &str) -> Result<(), Error> {
        if text.contains(['\r', '\n']) {
            return Err(self.error("comment text contains a line break"));
        }
        self.introduced_bytes = self
            .introduced_bytes
            .checked_add(text.len())
            .filter(|&size| size <= self.original.options.max_input_bytes)
            .ok_or_else(|| self.error("inserted YAML exceeds input byte limit"))?;
        self.charge(1)
    }

    fn comment_change(&mut self, id: NodeId, span: Span) {
        let root_span = self.original.nodes[self.original.root.0].span;
        if id == self.original.root
            && !span.is_empty()
            && (span.start() < root_span.start() || span.end() > root_span.end())
        {
            self.root_comments_changed = true;
        } else {
            self.mark_parent(id);
        }
    }

    fn finish(mut self) -> Result<EditableDocument, Error> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        let syntax = Layout::new(&self.candidate);
        let mut source = Source::from(self.original);
        let mut planned = self.candidate.clone();
        if !self.rewrites.is_empty() || self.root_comments_changed {
            if let Some(framed) = super::framing::prepare(&source, &mut planned, &syntax) {
                source = framed;
            }
        }
        let output = self.plan(&source, &planned).render(&syntax)?;
        self.candidate.commit(&output, self.original.options)
    }

    fn plan<'b>(&'b self, source: &'b Source, candidate: &'b Draft) -> Plan<'b> {
        Plan {
            original: source,
            candidate,
            parents: &self.parents,
            rewrites: &self.rewrites,
            root_comments_changed: self.root_comments_changed,
            work: &self.work,
        }
    }
}

fn remap(node: &mut Node, map: impl Fn(NodeId) -> NodeId) {
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
fn retarget_comments(node: &mut Node, removed: NodeId, following: Option<NodeId>) {
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
fn merged_comments(old: &Node, replacement: &Node) -> Vec<Comment> {
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
    let boundary = |comment: &Comment| match comment.position {
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

fn inline_node(node: &Node) -> bool {
    // Comments between properties and content need the parent's header and
    // continuation indentation, even when the content itself fits on one line.
    node.comments
        .iter()
        .all(|comment| comment.position != CommentPosition::Header)
        && matches!(
            node.kind,
            NodeKind::Scalar {
                style: ScalarStyle::Plain | ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted,
                ..
            } | NodeKind::Alias { .. }
        )
}
