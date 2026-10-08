use crate::{Error, edit::Editor, from_str};
use astral_yaml::edit::{EditableDocument, Editor as SyntaxEditor};
use serde::de::DeserializeOwned;

/// An owned YAML document and read-only typed view, available with `editable`.
///
/// Transactions retain unknown fields, comments, and compatible styles, then
/// deserialize a fresh `T`. Any failure leaves the source and value unchanged.
/// `T` needs neither `Serialize` nor `Clone`.
///
/// Physical CR and CRLF line endings normalize to LF on load. Spans and errors
/// use this source. Unchanged regions retain their text; rewrites may normalize
/// whitespace. Use [`Self::to_yaml`] to retain syntax in the output.
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
    /// Normalizes physical line endings to LF, then owns one UTF-8 YAML document
    /// and deserializes its typed view. Spans and errors use the normalized source.
    ///
    /// # Errors
    /// Returns a syntax, type, document-count, or resource-limit error.
    pub fn parse(source: &str) -> Result<Self, Error> {
        let syntax = EditableDocument::parse(source)?;
        let value = from_str(syntax.source())?;
        Ok(Self { value, syntax })
    }

    /// Edits Rust values and presentation, committing after deserializing a fresh `T`.
    /// Returns the callback's result only after commit. Failed mutations poison
    /// the transaction even when the callback catches their errors.
    ///
    /// # Errors
    /// Returns an edit, syntax, type, or resource-limit error. The original typed
    /// value and source remain unchanged, including on duplicate-key errors.
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

    /// Edits syntax handles and raw YAML fragments with the same typed commit
    /// checks as [`Self::edit`]. Use this for arbitrary mapping keys and aliases.
    ///
    /// # Errors
    /// Returns callback, syntax, or deserialization errors without committing.
    pub fn edit_syntax<R>(
        &mut self,
        edit: impl FnOnce(&mut SyntaxEditor<'_>) -> Result<R, astral_yaml::Error>,
    ) -> Result<R, Error> {
        self.edit(|editor| Ok(edit(editor.syntax_mut())?))
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

    /// Borrows the current committed source with LF line endings.
    #[must_use]
    pub fn source(&self) -> &str {
        self.syntax.source()
    }

    /// Emits the current committed source with LF line endings.
    #[must_use]
    pub fn to_yaml(&self) -> String {
        self.syntax.to_yaml()
    }

    /// Consumes both views and returns the committed source without cloning it.
    #[must_use]
    pub fn into_yaml(self) -> String {
        self.syntax.into_yaml()
    }

    /// Discards the syntax representation and returns the typed value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}
