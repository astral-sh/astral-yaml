use crate::{
    Error,
    edit::{self, Cursor, PathSegment},
    from_stream,
};
use astral_yaml::{
    ParseOptions,
    edit::{EditableDocument, Editor as SyntaxEditor},
    parse_with_options,
};
use serde::{Serialize, de::DeserializeOwned};

/// A YAML document you can edit without defining a Rust type for the whole document.
///
/// Read and write selected values using Rust types, or edit YAML fragments directly.
/// Parsing and edits still check the document using Serde's YAML rules, including
/// duplicate-key checks and alias limits. For application tags, use [`Editable<T>`]
/// with an appropriate type, or edit the syntax with [`EditableDocument`].
///
/// ```
/// use astral_yaml_serde::{EditableYaml, edit::edit_path};
///
/// let mut doc = EditableYaml::parse("servers: [{host: 'old'}]\n")?;
/// doc.set_at(edit_path!["servers", 0, "host"], "new")?;
/// assert_eq!(doc.source(), "servers: [{host: 'new'}]\n");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
pub type EditableYaml = Editable<serde::de::IgnoredAny>;

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
/// config.set_at(["timeout"], 60)?;
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
        Self::parse_with_options(source, ParseOptions::default())
    }

    /// Like [`Self::parse`], but use the given parser limits for loading and edits.
    /// The input byte limit applies before line endings are normalized. These
    /// options do not change Serde's separate alias-expansion limits.
    ///
    /// # Errors
    /// Returns an error if the options are invalid, `source` is invalid YAML or
    /// does not contain exactly one document, cannot be deserialized as `T`, or
    /// exceeds a resource limit.
    pub fn parse_with_options(source: &str, options: ParseOptions) -> Result<Self, Error> {
        Self::from_syntax(EditableDocument::parse_with_options(source, options)?)
    }

    fn from_syntax(syntax: EditableDocument) -> Result<Self, Error> {
        let stream = parse_with_options(syntax.source(), syntax.options())?;
        let value = from_stream(&stream)?;
        Ok(Self { value, syntax })
    }

    /// Edit the document starting at its root, and return the callback's result.
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
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.try_edit(edit)
    }

    /// Like [`Self::edit`], but allow the callback to return an application error.
    /// The callback's result is returned only after the changes are committed.
    ///
    /// # Errors
    /// Propagates callback, edit, and deserialization errors, leaving the source
    /// and value unchanged.
    pub fn try_edit<R, E: From<Error>>(
        &mut self,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, E>,
    ) -> Result<R, E> {
        self.try_edit_syntax(|editor| edit::transaction(editor, edit))
    }

    /// Set the value at an existing path, retaining its anchor and compatible style.
    ///
    /// Keys are literal strings, indices are zero-based, and an empty path selects the root.
    /// Use [`edit_path!`](crate::edit::edit_path) to mix keys and indices. To add a missing
    /// mapping entry, use [`MappingMut::set`](crate::edit::MappingMut::set) instead.
    ///
    /// # Errors
    /// Returns a selection, serialization, or edit error without changing the source or value.
    pub fn set_at<'p>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        value: impl Serialize,
    ) -> Result<(), Error> {
        self.edit_at(path, |mut node| node.set(value))
    }

    /// Edit the node at `path`. Keys are literal strings and indices are zero-based.
    /// An empty path selects the root. Missing nodes are not created implicitly.
    /// The complete document must still deserialize as `T` for the edit to commit.
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
    /// Propagates selection, callback, and deserialization errors without committing.
    pub fn try_edit_at<'p, R, E: From<Error>>(
        &mut self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
        edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, E>,
    ) -> Result<R, E> {
        self.try_edit(|root| edit(root.at(path)?))
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
        self.try_edit_syntax(|editor| edit(editor).map_err(Error::from))
    }

    /// Like [`Self::edit_syntax`], but allow the callback to return an application error.
    ///
    /// # Errors
    /// Propagates callback, edit, and deserialization errors without committing.
    pub fn try_edit_syntax<R, E: From<Error>>(
        &mut self,
        edit: impl FnOnce(&mut SyntaxEditor<'_>) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut syntax = self.syntax.clone();
        let result = syntax
            .try_edit_syntax(|editor| edit(editor).map_err(EditError))
            .map_err(|error: EditError<E>| error.0)?;
        *self = Self::from_syntax(syntax)?;
        Ok(result)
    }
}

/// Bridge syntax errors without requiring applications to convert them separately.
struct EditError<E>(E);

impl<E: From<Error>> From<astral_yaml::Error> for EditError<E> {
    fn from(error: astral_yaml::Error) -> Self {
        Self(Error::from(error).into())
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
