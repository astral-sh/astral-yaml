use astral_yaml::{
    Error,
    edit::{EditableDocument, Editor},
};
use astral_yaml_dev::edit_diff;

pub(super) fn normalize(source: &str) -> String {
    let output = EditableDocument::parse(source)
        .unwrap()
        .normalized_yaml()
        .unwrap();
    assert_eq!(
        EditableDocument::parse(&output)
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
) -> EditableDocument {
    let mut document = EditableDocument::parse(source).unwrap();
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
