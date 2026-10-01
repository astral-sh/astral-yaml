use super::{Comment, EditableDocument, Node, NodeKind, emit};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::{cell::Cell, collections::HashSet, sync::Arc};

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
    replacements: Vec<NodeId>,
    requested_styles: HashSet<NodeId>,
    patches: Vec<(Span, String)>,
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
    pub fn edit(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut editor = Editor::new(self);
        edit(&mut editor)?;
        let candidate = editor.finish()?;
        *self = candidate;
        Ok(())
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
            replacements: Vec::new(),
            requested_styles: HashSet::new(),
            patches: Vec::new(),
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

    fn mark(&mut self, mut id: NodeId) {
        while let Some(parent) = self.parents[id.0] {
            if id.0 < self.original.nodes.len() {
                let block = matches!(&self.candidate.nodes[id.0].kind,
                    NodeKind::Sequence { items, style: CollectionStyle::Block } if !items.is_empty())
                    || matches!(&self.candidate.nodes[id.0].kind,
                    NodeKind::Mapping { entries, style: CollectionStyle::Block } if !entries.is_empty());
                let start = self.original.nodes[id.0].span.start();
                let prefix = self.original.source[..start]
                    .rsplit(['\n', '\r'])
                    .next()
                    .unwrap_or("");
                if !block || prefix.bytes().all(|byte| byte == b' ') {
                    break;
                }
            }
            id = parent;
        }
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
            remap(&mut node.kind, |id| NodeId(id.0 + offset));
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
    /// outer comment attachments. Overlapping replacements are rejected.
    ///
    /// # Errors
    /// Rejects invalid fragments, handles, overlapping replacements, or limits.
    pub fn replace(&mut self, handle: &Handle, yaml: &str) -> Result<(), Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            if editor
                .replacements
                .iter()
                .any(|&other| editor.ancestor(id, other) || editor.ancestor(other, id))
            {
                return Err(editor.error("overlapping node replacements"));
            }
            let offset = editor.candidate.nodes.len();
            let root = editor.import(yaml, editor.parents[id.0])?;
            let simple = inline_node(&editor.candidate.nodes[id.0])
                && inline_node(&editor.candidate.nodes[root.0]);
            let comments = editor.candidate.nodes[id.0].comments.clone();
            let span = editor.candidate.nodes[id.0].span;
            editor.remove_tree(id)?;
            editor.alive[id.0] = true;
            editor.alive[root.0] = false;
            editor.candidate.nodes[id.0] = editor.candidate.nodes[root.0].clone();
            editor.candidate.nodes[id.0].comments.splice(0..0, comments);
            editor.candidate.nodes[id.0].span = span;
            for index in offset..editor.candidate.nodes.len() {
                remap(&mut editor.candidate.nodes[index].kind, |target| {
                    if target == root { id } else { target }
                });
                if editor.parents[index] == Some(root) {
                    editor.parents[index] = Some(id);
                }
            }
            remap(&mut editor.candidate.nodes[id.0].kind, |target| {
                if target == root { id } else { target }
            });
            editor.replacements.push(id);
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
            let NodeKind::Sequence { items, .. } = &editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a sequence"));
            };
            if index > items.len() {
                return Err(editor.error("item index out of bounds"));
            }
            let child = editor.import(yaml, Some(id))?;
            let NodeKind::Sequence { items, .. } = &mut editor.candidate.nodes[id.0].kind else {
                unreachable!()
            };
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
            let NodeKind::Mapping { entries, .. } = &editor.candidate.nodes[id.0].kind else {
                return Err(editor.error("expected a mapping"));
            };
            if index > entries.len() {
                return Err(editor.error("entry index out of bounds"));
            }
            let key = editor.import(key, Some(id))?;
            let value = editor.import(value, Some(id))?;
            let NodeKind::Mapping { entries, .. } = &mut editor.candidate.nodes[id.0].kind else {
                unreachable!()
            };
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
    /// Only one inline comment may be added to a node.
    ///
    /// # Errors
    /// Rejects invalid handles, indices, multiline text, conflicting inline
    /// comments, or resource limits.
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
            if placement == CommentPlacement::Inline
                && comments.iter().any(|c| c.placement == placement)
            {
                return Err(editor.error("node already has an inline comment"));
            }
            comments.insert(
                index,
                Comment {
                    text: text.to_owned(),
                    placement,
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
            editor.comment_change(id, span, format!("#{text}"));
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
            editor.comment_change(id, comment.span, String::new());
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

    fn comment_change(&mut self, id: NodeId, span: Span, replacement: String) {
        let root_span = self.original.nodes[self.original.root.0].span;
        if id == self.original.root
            && !span.is_empty()
            && (span.start() < root_span.start() || span.end() > root_span.end())
        {
            self.patches.retain(|(previous, _)| *previous != span);
            self.patches.push((span, replacement));
        } else {
            self.mark_parent(id);
        }
    }

    fn rewrite_regions(&self) -> Result<Vec<(NodeId, Span)>, Error> {
        let mut ids: Vec<_> = self
            .rewrites
            .iter()
            .copied()
            .filter(|&id| {
                let mut current = self.parents[id.0];
                while let Some(parent) = current {
                    if self.rewrites.contains(&parent) {
                        return false;
                    }
                    current = self.parents[parent.0];
                }
                true
            })
            .collect();
        loop {
            if !ids.is_empty() {
                self.charge(self.original.nodes.len())?;
            }
            let mut regions: Vec<_> = ids
                .iter()
                .map(|&id| (id, emit::region(self.original, id, false)))
                .collect();
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
            ids.retain(|&id| !self.ancestor(parent, id));
            ids.push(parent);
        }
    }

    fn finish(self) -> Result<EditableDocument, Error> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let regions = self.rewrite_regions()?;
        let mut patches = Vec::new();
        for &(id, region) in &regions {
            let flow = self.flow_context(id);
            let indent = emit::indent(&self.original.source, region.start());
            let mut body = emit::render(&self.candidate, id, region, indent, flow)?;
            if self.original.source[..region.end()].ends_with(['\r', '\n'])
                && !body.ends_with(['\r', '\n'])
            {
                body.push_str(emit::newline(&self.original.source));
            }
            patches.push((region, body));
        }
        patches.extend(
            self.patches
                .iter()
                .filter(|(span, _)| {
                    !regions.iter().any(|(_, region)| {
                        span.start() >= region.start() && span.end() <= region.end()
                    })
                })
                .cloned(),
        );
        patches.sort_by_key(|(span, _)| span.start());
        let mut source = String::new();
        let mut position = 0;
        for (span, replacement) in patches {
            source.push_str(&self.original.source[position..span.start()]);
            source.push_str(&replacement);
            position = span.end();
            if source.len() > self.original.options.max_input_bytes {
                return Err(self.error("emitted YAML exceeds input byte limit"));
            }
        }
        source.push_str(&self.original.source[position..]);
        let parsed = EditableDocument::parse_with_options(&source, self.original.options)?;
        let correspondence = emit::correspondence(&self.candidate, &parsed)
            .ok_or_else(|| self.error("emission changed YAML content or alias identity"))?;
        for &id in &self.requested_styles {
            if let Some(target) = correspondence[id.0] {
                if !same_style(
                    &self.candidate.nodes[id.0].kind,
                    &parsed.nodes[target.0].kind,
                ) {
                    return Err(self.error("requested style is not representable in this context"));
                }
            }
        }
        let mut expected_comments = Vec::new();
        for (id, target) in correspondence.iter().enumerate() {
            if let Some(target) = target {
                for comment in &self.candidate.nodes[id].comments {
                    expected_comments.push((
                        target.0,
                        comment.text.as_str(),
                        comment.placement as u8,
                    ));
                }
            }
        }
        let mut actual_comments: Vec<_> = parsed
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(id, node)| {
                node.comments
                    .iter()
                    .map(move |c| (id, c.text.as_str(), c.placement as u8))
            })
            .collect();
        expected_comments.sort_unstable();
        actual_comments.sort_unstable();
        if expected_comments != actual_comments {
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
                } => return true,
                NodeKind::Mapping { entries, style }
                    if *style == CollectionStyle::Flow
                        || entries.iter().any(|&(key, _)| key == id) =>
                {
                    return true;
                }
                _ => {}
            }
            id = parent;
        }
        false
    }
}

fn remap(kind: &mut NodeKind, map: impl Fn(NodeId) -> NodeId) {
    match kind {
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

fn inline_node(node: &Node) -> bool {
    matches!(
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
