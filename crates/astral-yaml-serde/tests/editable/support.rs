use astral_yaml_serde::{Editable, Error, edit::Cursor};
use serde::de::DeserializeOwned;

pub(super) fn edit<T: DeserializeOwned>(
    source: &str,
    operation: impl FnOnce(Cursor<'_, '_>) -> Result<(), Error>,
) -> Editable<T> {
    let mut document = Editable::parse(source).unwrap();
    document
        .edit(operation)
        .unwrap_or_else(|error| panic!("{source:?}: {error}"));
    document
}

pub(super) fn edit_syntax<T: DeserializeOwned>(
    source: &str,
    operation: impl FnOnce(&mut astral_yaml::edit::Editor<'_>) -> Result<(), astral_yaml::Error>,
) -> Editable<T> {
    let mut document = Editable::parse(source).unwrap();
    document
        .edit_syntax(operation)
        .unwrap_or_else(|error| panic!("{source:?}: {error}"));
    document
}
