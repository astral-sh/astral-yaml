use std::str::FromStr;

use crate::Error;
use crate::edit::{self, Cursor, PathSegment};
use crate::syntax::{DocumentSyntax, Editor};
use astral_yaml::ParseOptions;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A YAML document you can edit without defining a Rust type for the whole document.
///
/// Read and write selected values using Rust types.
/// Parsing and edits still check the document using Serde's YAML rules, including
/// duplicate-key checks and alias limits. For application tags, use [`Editable<T>`]
/// with an appropriate type.
///
/// ```
/// use astral_yaml_serde::edit::edit_path;
/// use astral_yaml_serde::EditableYaml;
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
/// Loading converts CR and CRLF line endings to LF and adds a UTF-8 byte order mark
/// if the source starts with a non-ASCII character. Spans refer to that source.
/// Unchanged regions keep their text, but rewritten regions may change whitespace.
///
/// ```
/// use astral_yaml_serde::Editable;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct Config { timeout: u64 }
///
/// let mut config: Editable<Config> = "timeout: 30 # seconds\n".parse()?;
/// config.set_at(["timeout"], 60)?;
/// assert_eq!(config.value().timeout, 60);
/// assert_eq!(config.to_yaml(), "timeout: 60 # seconds\n");
/// # Ok::<(), astral_yaml_serde::Error>(())
/// ```
#[derive(Debug)]
pub struct Editable<T> {
    value: T,
    syntax: DocumentSyntax,
}

impl<T: DeserializeOwned> FromStr for Editable<T> {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::parse(source)
    }
}

impl<T: DeserializeOwned> Editable<T> {
    /// Parse `source` as a YAML document and deserialize its value as `T`.
    ///
    /// The source uses LF line endings and a byte order mark when required for UTF-8.
    /// Spans and error locations refer to this normalized source.
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
        let syntax = DocumentSyntax::parse_with_options(source, options)?;
        let value = Self::deserialize_syntax(&syntax)?;
        Ok(Self { value, syntax })
    }

    fn deserialize_syntax(syntax: &DocumentSyntax) -> Result<T, Error> {
        let stream = astral_yaml::parse_with_options(syntax.source(), syntax.options())?;
        crate::from_stream(&stream)
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
        let mut editor = Editor::new(&self.syntax);
        let result = edit::transaction(&mut editor, edit)?;
        let syntax = editor.finish().map_err(Error::from)?;
        let value = Self::deserialize_syntax(&syntax)?;
        *self = Self { value, syntax };
        Ok(result)
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
}

impl<T> Editable<T> {
    /// Return the value from the last successful parse or edit.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Return the document's syntax and source.
    #[must_use]
    pub fn syntax(&self) -> &DocumentSyntax {
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
