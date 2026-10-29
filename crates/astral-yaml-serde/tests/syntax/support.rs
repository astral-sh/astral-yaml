use crate::edit::PathSegment;
use crate::syntax::{Cursor, DocumentSyntax, Editor};
use astral_yaml::Error;
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
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.edit_syntax(|editor| edit(editor.root_mut()))
    }

    pub(crate) fn set_yaml_at<'p>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        yaml: &str,
    ) -> Result<(), Error> {
        self.edit_at(path, |mut node| node.set_yaml(yaml))
    }

    pub(crate) fn edit_at<'p, R>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.edit(|root| edit(root.at(path)?))
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
