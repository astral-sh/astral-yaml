use crate::{Error, from_str};
use astral_yaml::edit::{EditableDocument, Editor};
use serde::de::DeserializeOwned;

/// A read-only typed view paired with authoritative, editable YAML syntax.
///
/// Edits operate on syntax, preserving unknown fields, ordering, comments, and
/// compatible styles. After a successful edit, the typed value is deserialized
/// again. Syntax or deserialization failures leave both members unchanged.
/// Unchanged source is emitted exactly; rewritten collections may normalize
/// whitespace. Use [`Self::to_yaml`] for output, rather than Serde serialization.
///
/// The source and typed value are owned. No `Serialize` or `Clone` implementation
/// is required for `T`.
///
/// ```
/// use astral_yaml_serde::Editable;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct Config { timeout: u64 }
///
/// let mut config = Editable::<Config>::parse("timeout: 30 # seconds\n")?;
/// config.edit(|syntax| {
///     let entries = syntax.mapping_entries(&syntax.root())?;
///     syntax.replace(&entries[0].1, "60")
/// })?;
/// assert_eq!(config.value().timeout, 60);
/// assert_eq!(config.to_yaml(), "timeout: 60 # seconds\n");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
#[derive(Debug)]
pub struct Editable<T> {
    value: T,
    syntax: EditableDocument,
}

impl<T: DeserializeOwned> Editable<T> {
    /// Owns one UTF-8 YAML document and deserializes its typed view.
    ///
    /// # Errors
    /// Returns a syntax, type, document-count, or resource-limit error.
    pub fn parse(source: &str) -> Result<Self, Error> {
        let syntax = EditableDocument::parse(source)?;
        let value = from_str(syntax.source())?;
        Ok(Self { value, syntax })
    }

    /// Applies syntax edits, committing only after deserializing a fresh `T`.
    ///
    /// # Errors
    /// Returns an edit, syntax, type, or resource-limit error. The original typed
    /// value and source remain unchanged, including on duplicate-key errors.
    pub fn edit(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_>) -> Result<(), astral_yaml::Error>,
    ) -> Result<(), Error> {
        let mut syntax = self.syntax.clone();
        syntax.edit(edit)?;
        let value = from_str(syntax.source())?;
        self.syntax = syntax;
        self.value = value;
        Ok(())
    }
}

impl<T> Editable<T> {
    /// The typed projection of the current committed syntax.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// The current committed syntax and source.
    #[must_use]
    pub fn syntax(&self) -> &EditableDocument {
        &self.syntax
    }

    /// Emits the current committed source, retaining untouched formatting.
    #[must_use]
    pub fn to_yaml(&self) -> String {
        self.syntax.to_yaml()
    }

    /// Discards the syntax representation and returns the typed value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}
