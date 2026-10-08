use crate::{Error, edit::Editor, from_str};
use astral_yaml::edit::{EditableDocument, Editor as SyntaxEditor};
use serde::de::DeserializeOwned;

/// A YAML document and its deserialized value, kept in sync by edits.
///
/// Edits preserve comments, unknown fields, and compatible styles. The edited
/// document must still deserialize as `T`, or the changes are discarded.
/// `T` must implement [`DeserializeOwned`], but need not implement `Serialize`
/// or `Clone`.
///
/// Loading converts CR and CRLF line endings to LF. Spans refer to that source.
/// Unchanged regions keep their text, but rewritten regions may change whitespace.
///
/// ```
/// use astral_yaml_serde::Editable;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct Config { timeout: u64 }
///
/// let mut config = Editable::<Config>::parse("timeout: 30 # seconds\n")?;
/// config.edit(|e| e.root_mut().field("timeout")?.set(60))?;
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
    /// Parse `source` as a YAML document and deserialize its value as `T`.
    ///
    /// CR and CRLF line endings are converted to LF. Spans and error locations refer
    /// to this normalized source.
    ///
    /// # Errors
    /// Returns an error if `source` is invalid YAML, does not contain exactly one
    /// document, cannot be deserialized as `T`, or exceeds a resource limit.
    pub fn parse(source: &str) -> Result<Self, Error> {
        let syntax = EditableDocument::parse(source)?;
        let value = from_str(syntax.source())?;
        Ok(Self { value, syntax })
    }

    /// Edit the document and return the callback's result.
    ///
    /// Changes are committed only if the output is valid YAML and deserializes as
    /// `T`. Any failure leaves the source and value unchanged. A failed mutation
    /// prevents commit even if the callback handles its error.
    ///
    /// # Errors
    /// Propagates callback and mutation errors. Also returns an error if the output
    /// cannot preserve the requested edits, cannot deserialize as `T`, contains
    /// duplicate keys, or exceeds a resource limit.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut Editor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut syntax = self.syntax.clone();
        let result = syntax.try_edit(|syntax| {
            let mut editor = Editor::new(syntax);
            let result = edit(&mut editor)?;
            editor.finish()?;
            Ok::<_, Error>(result)
        })?;
        let value = from_str(syntax.source())?;
        self.syntax = syntax;
        self.value = value;
        Ok(result)
    }

    /// Edit the document using YAML fragments and node handles.
    /// Use this for arbitrary mapping keys or aliases. The output is checked and
    /// deserialized as `T`, as with [`Self::edit`].
    ///
    /// # Errors
    /// Returns callback, edit, or deserialization errors without changing the document.
    pub fn edit_syntax<R>(
        &mut self,
        edit: impl FnOnce(&mut SyntaxEditor<'_>) -> Result<R, astral_yaml::Error>,
    ) -> Result<R, Error> {
        self.edit(|editor| Ok(edit(editor.syntax_mut())?))
    }
}

impl<T> Editable<T> {
    /// Return the value from the last successful parse or edit.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Return the document's syntax and source.
    #[must_use]
    pub fn syntax(&self) -> &EditableDocument {
        &self.syntax
    }

    /// Return the YAML source, with LF line endings.
    #[must_use]
    pub fn source(&self) -> &str {
        self.syntax.source()
    }

    /// Return a copy of the YAML source.
    #[must_use]
    pub fn to_yaml(&self) -> String {
        self.syntax.to_yaml()
    }

    /// Consume the document and return its YAML source without copying it.
    #[must_use]
    pub fn into_yaml(self) -> String {
        self.syntax.into_yaml()
    }

    /// Consume the document and return its deserialized value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}
