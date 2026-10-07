use super::{
    EditableDocument, Node, NodeKind,
    draft::Draft,
    layout::Layout,
    plan::{Changes, Plan},
    source::Source,
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::sync::Arc;

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
/// Node and comment locations refer to their original occurrences, even after
/// replacement. Inserted syntax has no source location (`None`).
#[derive(Debug)]
pub struct Editor<'a> {
    original: &'a EditableDocument,
    draft: Draft,
    transaction: Arc<()>,
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
        Self {
            original,
            draft: Draft::from(original),
            transaction: Arc::new(()),
            failure: None,
        }
    }

    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.original.nodes[self.original.root.0].span,
            message,
        }
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
        self.handle(self.draft.root())
    }

    /// Inspects a live node in this transaction.
    ///
    /// # Errors
    /// Rejects foreign/deleted handles and exhausted work budgets.
    pub fn node(&self, handle: &Handle) -> Result<&Node<Option<Span>>, Error> {
        Ok(self.draft.node(self.id(handle)?))
    }

    /// Returns sequence-item handles in source order.
    ///
    /// # Errors
    /// Rejects invalid handles, non-sequences, and exhausted work budgets.
    pub fn sequence_items(&self, handle: &Handle) -> Result<Vec<Handle>, Error> {
        let NodeKind::Sequence { items, .. } = &self.node(handle)?.kind else {
            return Err(self.error("expected a sequence"));
        };
        self.draft.charge(items.len())?;
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
        self.draft.charge(entries.len())?;
        Ok(entries
            .iter()
            .map(|&(key, value)| (self.handle(key), self.handle(value)))
            .collect())
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
            editor.draft.replace(id, yaml, preserve)?;
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
            editor.draft.insert_item(id, index, yaml)?;
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
            editor.draft.remove_item(id, index)?;
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
            editor.draft.insert_entry(id, index, key, value)?;
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
            editor.draft.remove_entry(id, index)?;
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
            editor.draft.set_scalar_style(id, style)?;
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
            editor.draft.set_collection_style(id, style)?;
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
            editor.draft.insert_comment(id, index, text, placement)?;
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
            editor.draft.replace_comment(id, index, text)?;
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
            editor.draft.remove_comment(id, index)?;
            Ok(())
        })
    }

    fn finish(mut self) -> Result<EditableDocument, Error> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        let candidate = self.draft.freeze();
        let syntax = Layout::new(&candidate);
        let original = Source::from(self.original);
        let changes = Changes::between(&original, &candidate, &syntax);
        let framed = if changes.is_empty() {
            None
        } else {
            super::framing::prepare(&original, (&candidate).into(), &syntax)
        };
        let (source, planned) = framed.as_ref().map_or_else(
            || (&original, (&candidate).into()),
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
