use super::{
    Comment, CommentPosition, EditableDocument, Node, NodeKind, emit,
    framing::{RootComments, RootLayout},
    layout::Layout,
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
    candidate: EditableDocument,
    transaction: Arc<()>,
    parents: Vec<Option<NodeId>>,
    alive: Vec<bool>,
    rewrites: HashSet<NodeId>,
    requested_styles: HashSet<NodeId>,
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
            candidate: original.clone(),
            transaction: Arc::new(()),
            parents,
            alive: vec![true; original.nodes.len()],
            rewrites: HashSet::new(),
            requested_styles: HashSet::new(),
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

    fn ancestor(&self, ancestor: NodeId, mut node: NodeId) -> bool {
        loop {
            if ancestor == node {
                return true;
            }
            let Some(parent) = self.parents[node.0] else {
                return false;
            };
            node = parent;
        }
    }

    fn mark(&mut self, id: NodeId) {
        self.rewrites.insert(id);
    }

    /// Select the syntax context from the final candidate, after all mutations.
    fn rewrite_root(&self, mut id: NodeId, syntax: &Layout) -> NodeId {
        // Changes below a key can make an enclosing implicit key multiline or
        // too long. Include its mapping so emission can use explicit notation.
        let edited = id;
        let mut ancestor = id;
        while let Some(parent) = self.parents[ancestor.0] {
            if ancestor != edited
                && matches!(&self.candidate.nodes[parent.0].kind,
                    NodeKind::Mapping { entries, .. }
                        if entries.iter().any(|&(key, _)| key == ancestor))
            {
                id = parent;
            }
            ancestor = parent;
        }

        while let Some(parent) = self.parents[id.0] {
            if id.0 < self.original.nodes.len() {
                let multiline = syntax.multiline(id, &self.candidate.nodes[id.0].kind);
                let start = self.original.nodes[id.0].span.start();
                let prefix = self.original.source[..start]
                    .rsplit('\n')
                    .next()
                    .unwrap_or("");
                if (!multiline || prefix.bytes().all(|byte| byte == b' '))
                    && !self.needs_mapping_rewrite(id, parent)
                {
                    break;
                }
            }
            id = parent;
        }
        id
    }

    fn needs_mapping_rewrite(&self, id: NodeId, parent: NodeId) -> bool {
        let NodeKind::Mapping { entries, style } = &self.original.nodes[parent.0].kind else {
            return false;
        };
        let source = &self.original.source;
        let span = self.original.nodes[id.0].span;
        for &(key, value) in entries {
            if key == id {
                // Key notation and separation belong to the mapping, outside
                // the key's span. Reuse the emitter's implicit-key check.
                let node = &self.candidate.nodes[id.0];
                let separator = crate::comments::skip_trivia(source, span.end());
                let plain = matches!(
                    node.kind,
                    NodeKind::Scalar {
                        style: ScalarStyle::Plain,
                        ..
                    }
                );
                return !emit::simple_key(node, *style == CollectionStyle::Flow)
                    || (plain
                        && source[separator..].strip_prefix(':').is_some_and(|suffix| {
                            suffix.starts_with(|c: char| {
                                !c.is_ascii_whitespace() && !matches!(c, ',' | ']' | '}')
                            })
                        }));
            }
            if value != id {
                continue;
            }
            // A block sequence may be indentless. Render its mapping when it
            // becomes flow syntax so the value can follow ':' on the same line.
            if matches!(
                self.original.nodes[id.0].kind,
                NodeKind::Sequence {
                    style: CollectionStyle::Block,
                    ..
                }
            ) && matches!(&self.candidate.nodes[id.0].kind,
                    NodeKind::Sequence { items, style }
                        if items.is_empty() || *style == CollectionStyle::Flow)
            {
                return true;
            }
            if span.is_empty() {
                // A value omitted without ':' has no source range in which
                // a scalar replacement can supply its value indicator.
                let separator =
                    crate::comments::skip_trivia(source, self.original.nodes[key.0].span.end());
                return source.as_bytes().get(separator) != Some(&b':');
            }
        }
        false
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
            self.requested_styles.remove(&id);
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
            editor.requested_styles.insert(id);
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
            editor.requested_styles.insert(id);
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

    fn rewrite_regions(
        &self,
        layout: &RootLayout,
        syntax: &Layout,
    ) -> Result<Vec<(NodeId, Span)>, Error> {
        let mut ids: Vec<_> = self.rewrites.iter().copied().collect();
        if (!ids.is_empty() && layout.needs_suffix_context())
            || (layout.needs_scalar_boundary(&self.candidate)
                && syntax
                    .tail(self.candidate.root)
                    .is_some_and(|tail| ids.iter().any(|&id| self.ancestor(id, tail))))
        {
            ids.push(self.candidate.root);
        }
        loop {
            if !ids.is_empty() {
                self.charge(self.original.nodes.len())?;
            }
            let roots: HashSet<_> = ids
                .into_iter()
                .map(|id| self.rewrite_root(id, syntax))
                .collect();
            ids = roots
                .iter()
                .copied()
                .filter(|&id| {
                    let mut current = self.parents[id.0];
                    while let Some(parent) = current {
                        if roots.contains(&parent) {
                            return false;
                        }
                        current = self.parents[parent.0];
                    }
                    true
                })
                .collect();
            let mut regions: Vec<_> = ids
                .iter()
                .map(|&id| {
                    let mut region = emit::region(self.original, id, false);
                    if id == self.candidate.root
                        && emit::has_inline_header(&self.candidate.nodes[id.0], false)
                    {
                        // A root's inline comment must move with its content
                        // when the emitted layout puts it on a header. Framing
                        // comments on other lines remain outside the rewrite.
                        for comment in &self.original.nodes[id.0].comments {
                            if comment.placement == CommentPlacement::Inline
                                && comment.span.start() >= region.end()
                                && self.original.source[region.end()..comment.span.start()]
                                    .bytes()
                                    .all(|byte| matches!(byte, b' ' | b'\t'))
                            {
                                region.end = comment.span.end();
                            }
                        }
                    }
                    if id == self.candidate.root {
                        region = super::framing::root_region(self.original, layout, region);
                    }
                    (id, region)
                })
                .collect();

            let mut expanded = false;
            for &(id, region) in &regions {
                let Some(parent) = self.parents[id.0] else {
                    continue;
                };
                let original = &self.original.nodes[id.0];
                let collection = matches!(
                    self.candidate.nodes[id.0].kind,
                    NodeKind::Sequence { .. } | NodeKind::Mapping { .. }
                );
                let outer_comments = collection
                    && original.comments.iter().any(|comment| {
                        comment.span.start() < original.span.start()
                            || comment.span.end() > original.span.end()
                    });
                // A descendant prefix can move the region before the node's
                // indentation or field header. A collection's outer comments
                // can also change attachment when its delimiters move. Render
                // the parent to supply both the syntax and comment context.
                // The document root keeps its separate framing-comment plan.
                if region.start() < original.span.start() || outer_comments {
                    ids.push(parent);
                    expanded = true;
                }
            }
            if expanded {
                continue;
            }
            regions.sort_by_key(|(_, span)| span.start());
            let Some(pair) = regions
                .windows(2)
                .find(|pair| pair[0].1.end() > pair[1].1.start())
            else {
                return Ok(regions);
            };
            let (left, right) = (pair[0].0, pair[1].0);
            let mut parent = left;
            while !self.ancestor(parent, right) {
                parent = self.parents[parent.0].expect("common document root");
            }
            ids.push(parent);
        }
    }

    fn root_comments(
        &self,
        region: Option<Span>,
        layout: &RootLayout,
    ) -> Result<RootComments, Error> {
        self.charge(
            self.original.nodes[self.original.root.0].comments.len()
                + self.candidate.nodes[self.candidate.root.0].comments.len(),
        )?;
        RootComments::plan(self.original, &self.candidate, region, layout)
    }

    fn finish(mut self) -> Result<EditableDocument, Error> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        let syntax = Layout::new(&self.candidate);
        let framed = if self.rewrites.is_empty() && !self.root_comments_changed {
            None
        } else {
            super::framing::prepare(self.original, &mut self.candidate, &syntax)
        };
        let editor = Editor {
            original: framed.as_ref().unwrap_or(self.original),
            ..self
        };
        editor.render(&syntax)
    }

    fn render(self, syntax: &Layout) -> Result<EditableDocument, Error> {
        let layout = RootLayout::new(self.original, &self.candidate, syntax);
        let regions = self.rewrite_regions(&layout, syntax)?;
        let root_region = regions.iter().find(|&&(id, _)| id == self.candidate.root);
        let mut root_comments = if root_region.is_some() || self.root_comments_changed {
            let region = root_region.map(|&(_, region)| region);
            Some(self.root_comments(region, &layout)?)
        } else {
            None
        };
        let mut patches = root_comments
            .as_mut()
            .map_or_else(Vec::new, |comments| std::mem::take(&mut comments.patches));
        for &(id, region) in &regions {
            let flow = self.flow_context(id);
            let indent = emit::indent(&self.original.source, region.start());
            let context = if self.parents[id.0].is_some_and(|parent| {
                matches!(
                    &self.candidate.nodes[parent.0].kind,
                    NodeKind::Mapping { entries, .. } if entries.iter().any(|&(key, _)| key == id)
                )
            }) {
                emit::Context::key(indent, flow)
            } else {
                emit::Context::value(
                    indent,
                    flow,
                    if id == self.candidate.root { 0 } else { indent },
                )
            };
            let rendered = if id == self.candidate.root {
                let comments = root_comments.as_ref().expect("root comment plan");
                emit::render_with_comments(
                    &self.candidate,
                    id,
                    region,
                    context,
                    &comments.rendered,
                    syntax,
                )?
            } else {
                emit::render_with_comments(
                    &self.candidate,
                    id,
                    region,
                    context,
                    &self.candidate.nodes[id.0].comments,
                    syntax,
                )?
            };
            patches.push((rendered.span, rendered.text));
        }
        if let Some(comments) = root_comments.as_mut() {
            patches.append(&mut comments.after_body);
            comments.finish_suffix(self.original, &mut patches);
        }
        // Prefix insertions at a replacement's start precede that replacement.
        // Suffix insertions at an empty root's position follow its syntax.
        // Stable sorting retains candidate order for insertions at the same point.
        patches.sort_by_key(|(span, _)| (span.start(), span.end()));
        let source = apply_patches(
            &self.original.source,
            patches,
            self.original.options.max_input_bytes,
        )
        .map_err(|message| self.error(message))?;
        let parsed = EditableDocument::parse_with_options(&source, self.original.options)?;
        let correspondence = emit::correspondence(&self.candidate, &parsed)
            .ok_or_else(|| self.error("emission changed YAML content or alias identity"))?;
        for &id in &self.requested_styles {
            // Removing a subtree also removes its style requests.
            let target = correspondence[id.0].expect("styled node is retained");
            if !same_style(
                &self.candidate.nodes[id.0].kind,
                &parsed.nodes[target.0].kind,
            ) {
                return Err(self.error("requested style is not representable in this context"));
            }
        }
        if !emit::comments_preserved(&self.candidate, &parsed, &correspondence) {
            return Err(self.error("emission changed comment ownership or placement"));
        }
        Ok(parsed)
    }

    fn flow_context(&self, mut id: NodeId) -> bool {
        while let Some(parent) = self.parents[id.0] {
            match &self.candidate.nodes[parent.0].kind {
                NodeKind::Sequence {
                    style: CollectionStyle::Flow,
                    ..
                }
                | NodeKind::Mapping {
                    style: CollectionStyle::Flow,
                    ..
                } => return true,
                _ => {}
            }
            id = parent;
        }
        false
    }
}

/// Apply ordered, disjoint byte patches. Validate each span's UTF-8 boundaries
/// before slicing, and include the retained suffix in the output limit.
fn apply_patches(
    source: &str,
    patches: Vec<(Span, String)>,
    limit: usize,
) -> Result<String, &'static str> {
    let mut output = String::new();
    let mut position = 0;
    for (span, replacement) in patches {
        let prefix = source
            .get(position..span.start())
            .ok_or("overlapping or invalid source patches")?;
        source
            .get(span.start()..span.end())
            .ok_or("overlapping or invalid source patches")?;
        output
            .len()
            .checked_add(prefix.len())
            .and_then(|size| size.checked_add(replacement.len()))
            .filter(|&size| size <= limit)
            .ok_or("emitted YAML exceeds input byte limit")?;
        output.push_str(prefix);
        output.push_str(&replacement);
        position = span.end();
    }
    let suffix = &source[position..];
    output
        .len()
        .checked_add(suffix.len())
        .filter(|&size| size <= limit)
        .ok_or("emitted YAML exceeds input byte limit")?;
    output.push_str(suffix);
    Ok(output)
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

fn same_style(left: &NodeKind, right: &NodeKind) -> bool {
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
