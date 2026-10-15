use super::{
    CommentPosition, EditableDocument, Node, NodeKind,
    draft::Draft,
    layout::Layout,
    plan::{Changes, Plan},
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::sync::Arc;

mod cursor;
pub use cursor::{
    Collection, Cursor, Entry, Mapping, MappingMut, PathSegment, Sequence, SequenceMut, ValueRef,
};

#[cfg(test)]
mod tests;

/// A reference to a node within an edit transaction.
///
/// Handles can be cloned. A handle becomes invalid when its node is deleted or
/// the transaction ends, and cannot be used in another transaction.
#[derive(Clone, Debug)]
pub struct Handle {
    transaction: Arc<()>,
    id: NodeId,
}

/// An editor for a pending transaction.
///
/// Obtain handles by navigating from [`Self::root`]. Reads include earlier edits
/// in the same transaction, but spans still refer to the original source.
/// Inserted nodes and comments have no source span.
///
/// Methods returning `Result` enforce the document's resource limits and reject handles
/// from other transactions or deleted nodes.
#[derive(Debug)]
pub struct Editor<'a> {
    original: &'a EditableDocument,
    draft: Draft,
    transaction: Arc<()>,
    failure: Option<Error>,
}

impl EditableDocument {
    /// Edit the document starting at its root, and return the callback's result.
    ///
    /// The output is parsed and checked against the requested changes before it
    /// replaces the document. Any failure leaves the document unchanged. A failed
    /// mutation prevents commit even if the callback handles its error.
    ///
    /// # Errors
    /// Returns an error if the callback fails, an edit is invalid or cannot be
    /// represented in YAML, or a resource limit is exceeded.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit(edit)
    }

    /// Like [`Self::edit`], but allow the callback to return an application error.
    /// The callback's result is returned only after the changes are committed.
    ///
    /// # Errors
    /// Propagates callback and edit errors, leaving the document unchanged.
    pub fn try_edit<R, E: From<Error>>(
        &mut self,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, E>,
    ) -> Result<R, E> {
        self.try_edit_syntax(|editor| edit(editor.root_mut()))
    }

    /// Edit the node at `path`. Keys are literal strings and indices are zero-based.
    /// An empty path selects the root. Missing nodes are not created implicitly.
    ///
    /// # Errors
    /// Returns a selection error without calling `edit`, or an error from the edit.
    pub fn edit_at<'p, R>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit_at(path, edit)
    }

    /// Like [`Self::edit_at`], but allow the callback to return an application error.
    ///
    /// # Errors
    /// Propagates selection, callback, and edit errors, leaving the document unchanged.
    pub fn try_edit_at<'p, R, E: From<Error>>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, E>,
    ) -> Result<R, E> {
        self.try_edit(|root| edit(root.at(path)?))
    }

    /// Edit with node handles, for example to revisit a node after inserting a sibling.
    ///
    /// # Errors
    /// Propagates callback and edit errors, leaving the document unchanged.
    pub fn edit_syntax<R>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit_syntax(edit)
    }

    /// Like [`Self::edit_syntax`], but allow the callback to return an application error.
    ///
    /// # Errors
    /// Propagates callback and edit errors, leaving the document unchanged.
    pub fn try_edit_syntax<R, E: From<Error>>(
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
        Self {
            original,
            draft: Draft::from(original),
            transaction: Arc::new(()),
            failure: None,
        }
    }

    fn error(&self, message: &'static str) -> Error {
        self.draft.error(message)
    }

    fn id(&self, handle: &Handle) -> Result<NodeId, Error> {
        self.draft.charge(1)?;
        if !Arc::ptr_eq(&self.transaction, &handle.transaction) || !self.draft.contains(handle.id) {
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

    fn mutate_node<R>(
        &mut self,
        handle: &Handle,
        operation: impl FnOnce(&mut Draft, NodeId) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.mutate(|editor| {
            let id = editor.id(handle)?;
            operation(&mut editor.draft, id)
        })
    }

    fn mutate<R>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        operation(self).inspect_err(|error| self.failure = Some(error.clone()))
    }

    /// Return a handle to the document's root.
    #[must_use]
    pub fn root(&self) -> Handle {
        self.handle(self.draft.root())
    }

    /// Return a cursor to the document's root.
    pub fn root_mut(&mut self) -> Cursor<'_, 'a> {
        let handle = self.root();
        Cursor::new(self, handle)
    }

    /// Return a cursor to a node selected by a handle.
    ///
    /// # Errors
    /// Returns an error if the handle is invalid or the work limit is exceeded.
    pub fn cursor_mut(&mut self, handle: &Handle) -> Result<Cursor<'_, 'a>, Error> {
        self.id(handle)?;
        Ok(Cursor::new(self, handle.clone()))
    }

    /// Return the node for `handle`, including pending edits.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or the work limit is exceeded.
    pub fn node(&self, handle: &Handle) -> Result<&Node<Option<Span>>, Error> {
        Ok(self.draft.node(self.id(handle)?))
    }

    /// Return handles to the sequence items at `handle`, in their current order.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or does not refer to a sequence.
    pub fn sequence_items(&self, handle: &Handle) -> Result<Vec<Handle>, Error> {
        let items = self.item_ids(handle)?;
        self.draft.charge(items.len())?;
        Ok(items.iter().map(|&id| self.handle(id)).collect())
    }

    /// Return the item at `index`, or `None` if the index is out of bounds.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or does not refer to a sequence.
    pub fn sequence_item(&self, handle: &Handle, index: usize) -> Result<Option<Handle>, Error> {
        let items = self.item_ids(handle)?;
        Ok(items.get(index).map(|&id| self.handle(id)))
    }

    fn item_ids(&self, sequence: &Handle) -> Result<&[NodeId], Error> {
        match self.node(sequence)?.kind() {
            NodeKind::Sequence { items, .. } => Ok(items),
            _ => Err(self.error("expected a sequence")),
        }
    }

    /// Return handles to the mapping keys and values at `handle`, in their current order.
    /// Keys can be any kind of YAML node.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or does not refer to a mapping.
    pub fn mapping_entries(&self, handle: &Handle) -> Result<Vec<(Handle, Handle)>, Error> {
        let entries = self.entry_ids(handle)?;
        self.draft.charge(entries.len())?;
        Ok(entries
            .iter()
            .map(|&(key, value)| (self.handle(key), self.handle(value)))
            .collect())
    }

    /// Return the key and value at `index`, or `None` if the index is out of bounds.
    /// Keys can be any kind of YAML node.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or does not refer to a mapping.
    pub fn mapping_entry(
        &self,
        handle: &Handle,
        index: usize,
    ) -> Result<Option<(Handle, Handle)>, Error> {
        let entries = self.entry_ids(handle)?;
        Ok(entries
            .get(index)
            .map(|&(key, value)| (self.handle(key), self.handle(value))))
    }

    fn entry_ids(&self, mapping: &Handle) -> Result<&[(NodeId, NodeId)], Error> {
        match self.node(mapping)?.kind() {
            NodeKind::Mapping { entries, .. } => Ok(entries),
            _ => Err(self.error("expected a mapping")),
        }
    }

    /// Replace `handle`'s node with the parsed `yaml`, keeping its outer comments.
    ///
    /// The handle continues to refer to the replacement, but handles to its old
    /// descendants become invalid. Existing aliases use the replacement's anchor.
    /// Commit fails if an alias loses its anchor or would refer to a different node.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or `yaml` cannot be parsed as a
    /// single document within the resource limits.
    pub fn replace(&mut self, handle: &Handle, yaml: &str) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.replace(id, yaml, false))
    }

    /// Like [`Self::replace`], but keep the node's anchor and compatible style.
    ///
    /// The fragment supplies tags and, if the old node has no anchor, its anchor.
    /// Aliases within the fragment are updated to refer to the retained anchor.
    /// A style that cannot represent the new value is replaced with a suitable one.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or `yaml` cannot be parsed as a
    /// single document within the resource limits.
    pub fn replace_preserving_style(&mut self, handle: &Handle, yaml: &str) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.replace(id, yaml, true))
    }

    /// Parse `yaml` and insert it into `sequence` before `index`.
    /// An index equal to the sequence's length appends the item.
    ///
    /// # Errors
    /// Returns an error if `sequence` is invalid or not a sequence, `index` exceeds
    /// its length, or `yaml` cannot be parsed as a single document.
    pub fn insert_item(
        &mut self,
        sequence: &Handle,
        index: usize,
        yaml: &str,
    ) -> Result<Handle, Error> {
        let child = self.mutate_node(sequence, |draft, id| draft.insert_item(id, index, yaml))?;
        Ok(self.handle(child))
    }

    /// Remove the item at `index` from `sequence`, including its descendants' comments.
    ///
    /// # Errors
    /// Returns an error if `sequence` is invalid or not a sequence, or `index` is
    /// out of bounds.
    pub fn remove_item(&mut self, sequence: &Handle, index: usize) -> Result<(), Error> {
        self.mutate_node(sequence, |draft, id| draft.remove_item(id, index))
    }

    /// Parse `key` and `value` and insert an entry into `mapping` before `index`.
    /// An index equal to the mapping's length appends the entry.
    ///
    /// # Errors
    /// Returns an error if `mapping` is invalid or not a mapping, `index` exceeds
    /// its length, or either fragment cannot be parsed as a single document.
    pub fn insert_entry(
        &mut self,
        mapping: &Handle,
        index: usize,
        key: &str,
        value: &str,
    ) -> Result<(Handle, Handle), Error> {
        let (key, value) = self.mutate_node(mapping, |draft, id| {
            draft.insert_entry(id, index, key, value)
        })?;
        Ok((self.handle(key), self.handle(value)))
    }

    /// Remove the entry at `index` from `mapping`, including its key, value, and
    /// their descendants' comments.
    ///
    /// # Errors
    /// Returns an error if `mapping` is invalid or not a mapping, or `index` is
    /// out of bounds.
    pub fn remove_entry(&mut self, mapping: &Handle, index: usize) -> Result<(), Error> {
        self.mutate_node(mapping, |draft, id| draft.remove_entry(id, index))
    }

    /// Request a style for the scalar at `handle`.
    /// The transaction fails if that style cannot represent the scalar's text in its
    /// surrounding YAML.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid, the node is not a scalar, or the style
    /// changes its YAML type.
    pub fn set_scalar_style(&mut self, handle: &Handle, style: ScalarStyle) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.set_scalar_style(id, style))
    }

    /// Request block or flow style for the collection at `handle`.
    /// Empty collections always use flow style.
    ///
    /// # Errors
    /// Returns an error if `handle` is invalid or the node is not a collection.
    pub fn set_collection_style(
        &mut self,
        handle: &Handle,
        style: CollectionStyle,
    ) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.set_collection_style(id, style))
    }

    /// Insert a comment at `index` in the node's own comment list.
    /// `text` is everything after `#`, including any leading space.
    ///
    /// Leading comments go before the node. A detached comment uses the next
    /// collection gap already recorded in the comment list, or goes after the node
    /// if there is none. Other placements go after the node.
    /// Use [`Self::insert_comment_at`] to choose a specific position.
    ///
    /// # Errors
    /// Returns an error if `handle` or `index` is invalid, or `text` contains a line
    /// break. Commit fails if the placement cannot preserve comment ownership.
    pub fn insert_comment(
        &mut self,
        handle: &Handle,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| {
            draft.insert_comment(id, index, text, placement, None)
        })
    }

    /// Return the position of the node's comment at `index`.
    /// Capture this before removing a comment to reinsert it at the same header or
    /// collection gap with [`Self::insert_comment_at`].
    ///
    /// # Errors
    /// Returns an error if `handle` or `index` is invalid.
    pub fn comment_position(
        &self,
        handle: &Handle,
        index: usize,
    ) -> Result<CommentPosition<Handle>, Error> {
        let comment = self
            .node(handle)?
            .comments
            .get(index)
            .ok_or_else(|| self.error("comment index out of bounds"))?;
        comment.position().try_map(|child| Ok(self.handle(child)))
    }

    /// Insert a comment at `index` in the node's list and at `position` in its syntax.
    ///
    /// [`CommentPosition::BeforeChild`] requires detached placement and a sequence
    /// item or mapping key belonging to the node at `handle`.
    ///
    /// # Errors
    /// Returns an error if a handle, index, or child position is invalid, or `text`
    /// contains a line break. Commit fails if the position or placement cannot
    /// preserve comment ownership.
    pub fn insert_comment_at(
        &mut self,
        handle: &Handle,
        index: usize,
        text: &str,
        placement: CommentPlacement,
        position: CommentPosition<Handle>,
    ) -> Result<(), Error> {
        let position = position.try_map(|child| self.id(&child));
        self.mutate_node(handle, |draft, id| {
            draft.insert_comment(id, index, text, placement, Some(position?))
        })
    }

    /// Replace the text of the node's comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `handle` or `index` is invalid, or `text` contains a line break.
    pub fn replace_comment(
        &mut self,
        handle: &Handle,
        index: usize,
        text: &str,
    ) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.replace_comment(id, index, text))
    }

    /// Remove the node's comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `handle` or `index` is invalid.
    pub fn remove_comment(&mut self, handle: &Handle, index: usize) -> Result<(), Error> {
        self.mutate_node(handle, |draft, id| draft.remove_comment(id, index))
    }

    fn finish(self) -> Result<EditableDocument, Error> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let candidate = self.draft.freeze()?;
        let syntax = Layout::new(&candidate);
        let original = &self.original.source;
        let changes = Changes::between(original, &candidate, &syntax);
        if changes.is_empty() {
            return candidate.commit(&original.text);
        }

        let framed = super::framing::prepare(original, (&candidate).into(), &syntax);
        let (source, planned) = framed.as_ref().map_or_else(
            || (original, (&candidate).into()),
            |framed| (&framed.source, framed.syntax()),
        );
        let output = Plan {
            original: source,
            candidate: planned,
            parents: candidate.parents(),
            changes,
            work: candidate.work(),
        }
        .render(&syntax)?;
        candidate.commit(&output)
    }
}
