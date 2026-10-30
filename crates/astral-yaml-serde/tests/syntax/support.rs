use crate::syntax::{DocumentSyntax, Editor};
use astral_yaml::{Error, NodeId};
use astral_yaml_dev::edit_diff;

pub(super) fn normalize(source: &str) -> String {
    let output = DocumentSyntax::parse(source)
        .unwrap()
        .normalized_yaml()
        .unwrap();
    assert_eq!(
        DocumentSyntax::parse(&output)
            .unwrap()
            .normalized_yaml()
            .unwrap(),
        output,
        "normalization must be idempotent: {source:?}",
    );
    edit_diff(source, &output)
}

/// Apply a successful transaction and show its source diff.
pub(super) fn edit(
    source: &str,
    operation: impl FnOnce(&mut Editor<'_>) -> Result<(), Error>,
) -> String {
    edit_diff(source, edited(source, operation).source())
}

pub(super) fn edited(
    source: &str,
    operation: impl FnOnce(&mut Editor<'_>) -> Result<(), Error>,
) -> DocumentSyntax {
    let mut document = DocumentSyntax::parse(source).unwrap();
    document
        .edit_syntax(operation)
        .unwrap_or_else(|error| panic!("{source:?}: {error}"));
    document
}

pub(super) fn edits<const N: usize>(
    sources: [&str; N],
    mut operation: impl FnMut(&mut Editor<'_>) -> Result<(), Error>,
) -> String {
    sources
        .map(|source| edit(source, &mut operation))
        .join("\n")
}

impl DocumentSyntax {
    pub(crate) fn edit<R>(
        &mut self,
        edit: impl FnOnce(crate::edit::Cursor<'_, '_>) -> Result<R, crate::Error>,
    ) -> Result<R, crate::Error> {
        self.try_edit_syntax(|editor| crate::edit::transaction(editor, edit))
    }

    pub(crate) fn edit_syntax<R>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit_syntax(edit)
    }

    pub(crate) fn try_edit_syntax<R, E: From<Error>>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut editor = Editor::new(self);
        let result = edit(&mut editor)?;
        *self = editor.finish()?;
        Ok(result)
    }
}

impl Editor<'_> {
    pub(crate) fn mapping_entries(&self, id: NodeId) -> Result<Vec<(NodeId, NodeId)>, Error> {
        Ok(self.entry_ids(id)?.to_vec())
    }

    pub(crate) fn sequence_items(&self, id: NodeId) -> Result<Vec<NodeId>, Error> {
        Ok(self.item_ids(id)?.to_vec())
    }
}
