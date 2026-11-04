//! Edit YAML using Rust values inside an [`Editable`](crate::Editable) transaction.

use crate::Error;
use crate::syntax::{self, Collection, Mapping, Sequence};
use astral_yaml::{NodeId, Span};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

pub use crate::edit_path;
pub use crate::syntax::{Comment, CommentPosition, DocumentSyntax, Node, NodeKind};
pub use astral_yaml::comments::CommentPlacement;
pub use astral_yaml::{CollectionStyle, ScalarStyle};

/// Build an edit path from literal mapping keys and zero-based sequence indices.
///
/// For a path containing only keys or only indices, an array like `["service", "port"]`
/// works without this macro. Keys containing `.` or `/` are always taken literally.
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
#[macro_export]
macro_rules! edit_path {
    () => { [] as [$crate::edit::PathSegment<'static>; 0] };
    ($($segment:expr),+ $(,)?) => {
        [$($crate::edit::PathSegment::from($segment)),+]
    };
}

/// One step in an edit path. Strings are literal mapping keys, not path expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSegment<'key> {
    /// A literal string key in a mapping.
    Key(&'key str),
    /// A zero-based position in a sequence.
    Index(usize),
}

impl<'key> From<&'key str> for PathSegment<'key> {
    fn from(key: &'key str) -> Self {
        Self::Key(key)
    }
}

impl From<usize> for PathSegment<'_> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl<Segment> From<&Segment> for PathSegment<'_>
where
    Segment: Copy + Into<Self>,
{
    fn from(segment: &Segment) -> Self {
        (*segment).into()
    }
}

pub(crate) fn transaction<R, E: From<Error>>(
    editor: &mut syntax::Editor<'_>,
    edit: impl FnOnce(Cursor<'_, '_>) -> Result<R, E>,
) -> Result<R, E> {
    let mut failure = None;
    let context = EditContext {
        failure: &mut failure,
        path: Cow::Borrowed("$"),
    };
    let root = editor.root();
    let result = edit(context.cursor(editor, root))?;
    failure.map_or(Ok(result), |error| Err(error.into()))
}

/// A position in the YAML document being edited.
///
/// Navigation consumes the cursor and does not follow aliases. Convert to a
/// [`MappingMut`] or [`SequenceMut`] to perform repeated edits of one collection.
/// A failed mutation prevents the transaction from committing, even if its error
/// is handled. Navigation and read errors can be handled without preventing a commit.
#[derive(Debug)]
pub struct Cursor<'edit, 'doc, State = ()> {
    editor: &'edit mut syntax::Editor<'doc>,
    id: NodeId,
    state: State,
    context: EditContext<'edit>,
}

impl<'edit, 'doc> Cursor<'edit, 'doc> {
    /// Lend this position to an edit, so navigation can start here again afterwards.
    pub fn reborrow(&mut self) -> Cursor<'_, 'doc> {
        self.context.reborrow().cursor(self.editor, self.id)
    }

    /// Select a descendant by literal mapping keys and sequence indices.
    /// An empty path keeps the current position.
    ///
    /// # Errors
    /// Returns the selection error at the first segment that cannot be followed.
    pub fn at<'p>(
        self,
        path: impl IntoIterator<Item = impl Into<PathSegment<'p>>>,
    ) -> Result<Self, Error> {
        path.into_iter()
            .try_fold(self, |cursor, segment| match segment.into() {
                PathSegment::Key(key) => cursor.field(key),
                PathSegment::Index(index) => cursor.item(index),
            })
    }

    /// Select the value at string key `name`, or return `None` if it is absent.
    ///
    /// Keys are compared after decoding and resolving aliases. Numeric, boolean,
    /// and application-tagged keys do not match their string spellings.
    /// Names are literal, not paths.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping or more than one key matches.
    pub fn get(self, name: &str) -> Result<Option<Self>, Error> {
        let context = self.context.at(format_args!("[{name:?}]"));
        let field = context.with_path(self.editor.string_field(self.id, name))?;
        Ok(field.map(|(_, id)| Self {
            id,
            context,
            ..self
        }))
    }

    /// Select the required value at the literal string key `name`.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, or the key is missing or ambiguous.
    pub fn field(self, name: &str) -> Result<Self, Error> {
        let context = self.context.at(format_args!("[{name:?}]"));
        let field = context.with_path(
            self.editor
                .string_field(self.id, name)
                .and_then(|field| field.ok_or_else(|| self.editor.error("missing mapping key"))),
        )?;
        Ok(Self {
            id: field.1,
            context,
            ..self
        })
    }

    /// Select the sequence item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or `index` is out of bounds.
    pub fn item(self, index: usize) -> Result<Self, Error> {
        let context = self.context.at(format_args!("[{index}]"));
        let id = context.with_path(self.editor.item_ids(self.id).and_then(|items| {
            items
                .get(index)
                .copied()
                .ok_or_else(|| self.editor.error("item index out of bounds"))
        }))?;
        Ok(Self {
            id,
            context,
            ..self
        })
    }

    /// Borrow the selected mapping for repeated edits.
    ///
    /// # Errors
    /// Returns an error if the selected node is not a mapping or an editing limit is exceeded.
    pub fn into_mapping(self) -> Result<MappingMut<'edit, 'doc>, Error> {
        let len = self
            .context
            .with_path(self.editor.entry_ids(self.id))?
            .len();
        Ok(self.into_collection(len))
    }

    /// Borrow the selected sequence for repeated edits.
    ///
    /// # Errors
    /// Returns an error if the selected node is not a sequence or an editing limit is exceeded.
    pub fn into_sequence(self) -> Result<SequenceMut<'edit, 'doc>, Error> {
        let len = self.context.with_path(self.editor.item_ids(self.id))?.len();
        Ok(self.into_collection(len))
    }

    fn into_collection<Kind>(self, len: usize) -> Cursor<'edit, 'doc, Collection<Kind>> {
        Cursor {
            editor: self.editor,
            id: self.id,
            context: self.context,
            state: Collection {
                len,
                kind: PhantomData,
            },
        }
    }

    /// Read the selected value as `T`, transform it, and replace it using [`Self::set`].
    ///
    /// For example, `retries.update(|n: u32| n + 1)?` increments a retry count.
    /// Prefer updating a field over round-tripping a struct that omits other fields.
    /// Updating an alias reads its target but replaces the alias at this position.
    /// As with other mutations, an error prevents the transaction from committing.
    ///
    /// # Errors
    /// Returns an error if reading or replacing the value fails. The transform runs
    /// only after the value is successfully read.
    pub fn update<T: DeserializeOwned + Serialize>(
        &mut self,
        transform: impl FnOnce(T) -> T,
    ) -> Result<(), Error> {
        let value = self
            .context
            .mutate(|| crate::de::from_edit_value(self.editor.value(self.id)))?;
        self.set(transform(value))
    }

    /// Replace the selected node with a serialized Rust value.
    ///
    /// Keep its anchor, outer comments, and compatible style. Strings are serialized
    /// as strings, so `set("false")` and `set(false)` produce different YAML values.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn set(&mut self, value: impl Serialize) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.replace_value(self.id, &value))
    }

    /// Request a style for the selected scalar.
    ///
    /// # Errors
    /// Returns an error if the node is not a scalar or the style changes its YAML type.
    pub fn set_scalar_style(&mut self, style: ScalarStyle) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.set_scalar_style(self.id, style))
    }
}

/// A mapping selected for repeated edits.
///
/// Child selections borrow the view. Their cursors can replace the selected child
/// while the mapping itself remains available for later edits.
pub type MappingMut<'edit, 'doc> = Cursor<'edit, 'doc, Collection<Mapping>>;

impl<'doc> MappingMut<'_, 'doc> {
    /// Select the value at string key `name`, or return `None` if absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn get(&mut self, name: &str) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        self.borrow_cursor().get(name)
    }

    /// Select the required value at string key `name`.
    ///
    /// # Errors
    /// Returns an error if the key is missing or ambiguous, or an editing limit is exceeded.
    pub fn field(&mut self, name: &str) -> Result<Cursor<'_, 'doc>, Error> {
        self.borrow_cursor().field(name)
    }

    /// Select an occupied or vacant entry at string key `name`.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn entry(&mut self, name: &str) -> Result<Entry<'_, 'doc>, Error> {
        let context = self.context.child(format_args!("[{name:?}]"));
        let field = context.with_path(self.editor.string_field(self.id, name))?;
        let state = match field {
            Some((_, id)) => EntryState::Occupied(id),
            None => EntryState::Vacant {
                mapping: self.id,
                name: name.to_owned(),
                len: &mut self.state.len,
            },
        };
        Ok(Entry {
            editor: self.editor,
            state,
            context,
        })
    }

    /// Borrow the mapping at `name`, inserting an empty mapping if the key is absent.
    ///
    /// # Errors
    /// Returns an error if an existing value is not a mapping, the key is ambiguous,
    /// or an editing limit is exceeded. Existing values are never replaced.
    pub fn ensure_mapping(&mut self, name: &str) -> Result<MappingMut<'_, 'doc>, Error> {
        self.entry(name)?.or_mapping()
    }

    /// Borrow the sequence at `name`, inserting an empty sequence if the key is absent.
    ///
    /// # Errors
    /// Returns an error if an existing value is not a sequence, the key is ambiguous,
    /// or an editing limit is exceeded. Existing values are never replaced.
    pub fn ensure_sequence(&mut self, name: &str) -> Result<SequenceMut<'_, 'doc>, Error> {
        self.entry(name)?.or_sequence()
    }

    /// Set the value at string key `name`, appending the entry if absent.
    /// Return a cursor for further edits of that value.
    ///
    /// # Errors
    /// Returns an error if serialization fails, the key is ambiguous, or an editing limit is exceeded.
    pub fn set(&mut self, name: &str, value: impl Serialize) -> Result<Cursor<'_, 'doc>, Error> {
        let mut context = self.context.child(format_args!("[{name:?}]"));
        let id = context.mutate(|| -> Result<_, Error> {
            if let Some((_, id)) = self.editor.string_field(self.id, name)? {
                self.editor.replace_value(id, &value)?;
                Ok(id)
            } else {
                self.editor.append_string_entry(
                    self.id,
                    &mut self.state.len,
                    name,
                    |remaining, span| crate::ser::to_string_with_limit(&value, remaining, span),
                )
            }
        })?;
        Ok(context.cursor(self.editor, id))
    }

    /// Remove the entry at string key `name`, including its comments.
    /// Return whether an entry was removed.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn remove(&mut self, name: &str) -> Result<bool, Error> {
        self.context.child(format_args!("[{name:?}]")).mutate(|| {
            let Some((index, _)) = self.editor.string_field(self.id, name)? else {
                return Ok(false);
            };
            self.editor.remove_entry(self.id, index)?;
            self.state.len -= 1;
            Ok::<_, astral_yaml::Error>(true)
        })
    }

    /// Remove the entry at `index`, including its key, value, and their comments.
    /// Later indices shift down. The key can be any YAML value.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove_at(&mut self, index: usize) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.remove_entry(self.id, index))?;
        self.state.len -= 1;
        Ok(())
    }

    /// Select the key at `index`, including keys that are not strings.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn key_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let id = self.entry_ids_at(index)?.0;
        let context = self.context.child(format_args!("[entry {index}].key"));
        Ok(context.cursor(self.editor, id))
    }

    /// Select the value of the entry at `index`, including entries with non-string keys.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn value_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let id = self.entry_ids_at(index)?.1;
        let context = self.context.child(format_args!("[entry {index}].value"));
        Ok(context.cursor(self.editor, id))
    }

    fn entry_ids_at(&self, index: usize) -> Result<(NodeId, NodeId), Error> {
        self.context
            .with_path(self.editor.entry_ids(self.id).and_then(|entries| {
                entries
                    .get(index)
                    .copied()
                    .ok_or_else(|| self.editor.error("entry index out of bounds"))
            }))
    }
}

/// A sequence selected for repeated edits.
pub type SequenceMut<'edit, 'doc> = Cursor<'edit, 'doc, Collection<Sequence>>;

impl<'doc> SequenceMut<'_, 'doc> {
    /// Select the item at `index`, or return `None` if it is absent.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn get(&mut self, index: usize) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        let items = self.context.with_path(self.editor.item_ids(self.id))?;
        Ok(items.get(index).copied().map(|id| {
            self.context
                .child(format_args!("[{index}]"))
                .cursor(self.editor, id)
        }))
    }

    /// Select the required item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn item(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let items = self.context.with_path(self.editor.item_ids(self.id))?;
        let context = self.context.child(format_args!("[{index}]"));
        let id = context.with_path(
            items
                .get(index)
                .copied()
                .ok_or_else(|| Error::new("item index out of bounds")),
        )?;
        Ok(context.cursor(self.editor, id))
    }

    /// Append a serialized value and return a cursor to the new item.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn push(&mut self, value: impl Serialize) -> Result<Cursor<'_, 'doc>, Error> {
        self.insert(self.len(), value)
    }

    /// Insert a serialized value before `index` and return a cursor to it.
    /// An index equal to the sequence's length appends the item.
    ///
    /// # Errors
    /// Returns an error if serialization fails, `index` exceeds the length, or an editing limit is exceeded.
    pub fn insert(
        &mut self,
        index: usize,
        value: impl Serialize,
    ) -> Result<Cursor<'_, 'doc>, Error> {
        let id = self.context.mutate(|| -> Result<_, Error> {
            let yaml = self.editor.serialize(&value)?;
            Ok(self.editor.insert_item(self.id, index, &yaml)?)
        })?;
        self.state.len += 1;
        let context = self.context.child(format_args!("[{index}]"));
        Ok(context.cursor(self.editor, id))
    }

    /// Remove the item at `index`, including its comments. Later indices shift down.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove(&mut self, index: usize) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.remove_item(self.id, index))?;
        self.state.len -= 1;
        Ok(())
    }
}

/// An occupied or vacant string-keyed mapping entry.
#[derive(Debug)]
pub struct Entry<'edit, 'doc> {
    editor: &'edit mut syntax::Editor<'doc>,
    state: EntryState<'edit>,
    context: EditContext<'edit>,
}

#[derive(Debug)]
enum EntryState<'edit> {
    Occupied(NodeId),
    Vacant {
        mapping: NodeId,
        name: String,
        len: &'edit mut usize,
    },
}

impl<'edit, 'doc> Entry<'edit, 'doc> {
    /// Return whether the mapping already contains the entry.
    #[must_use]
    pub fn is_occupied(&self) -> bool {
        matches!(self.state, EntryState::Occupied(_))
    }

    /// Return the existing value, or insert a serialized default if absent.
    /// An occupied entry does not serialize the unused default.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn or_insert(self, value: impl Serialize) -> Result<Cursor<'edit, 'doc>, Error> {
        self.or_insert_with(|remaining, span| {
            crate::ser::to_string_with_limit(&value, remaining, span)
        })
    }

    fn or_insert_with<S: AsRef<str>>(
        mut self,
        default: impl FnOnce(usize, Span) -> Result<S, Error>,
    ) -> Result<Cursor<'edit, 'doc>, Error> {
        let id = self.context.mutate(|| -> Result<_, Error> {
            match self.state {
                EntryState::Occupied(id) => Ok(id),
                EntryState::Vacant { mapping, name, len } => self
                    .editor
                    .append_string_entry(mapping, len, &name, default),
            }
        })?;
        Ok(self.context.cursor(self.editor, id))
    }

    /// Edit the entry's mapping, inserting an empty mapping only if the entry is absent.
    ///
    /// # Errors
    /// Returns an error if the existing value is not a mapping or the edit exceeds a limit.
    pub fn or_mapping(self) -> Result<MappingMut<'edit, 'doc>, Error> {
        self.or_insert_with(|_, _| Ok("{}"))?.into_mapping()
    }

    /// Edit the entry's sequence, inserting an empty sequence only if the entry is absent.
    ///
    /// # Errors
    /// Returns an error if the existing value is not a sequence or the edit exceeds a limit.
    pub fn or_sequence(self) -> Result<SequenceMut<'edit, 'doc>, Error> {
        self.or_insert_with(|_, _| Ok("[]"))?.into_sequence()
    }
}

impl<State> Cursor<'_, '_, State> {
    /// Deserialize the selected value, including pending edits.
    ///
    /// The result can borrow from this cursor, including decoded scalar text. Reading
    /// an alias follows it to its current target. An error does not prevent commit.
    /// Source spans refer to the original document. Reads that require a source span
    /// for a newly inserted node or comment fail until the edit commits.
    ///
    /// # Errors
    /// Returns an error if the value cannot be deserialized as `T`, an alias target
    /// was removed, or a resource limit is exceeded.
    pub fn read<'de, T: Deserialize<'de>>(&'de self) -> Result<T, Error> {
        self.context
            .with_path(crate::de::from_edit_value(self.editor.value(self.id)))
    }

    /// Return the selected node, including pending edits.
    /// Spans refer to the original source. Inserted nodes have no source span.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.context.with_path(self.editor.node(self.id))
    }

    /// Request block or flow style for this collection. Empty collections use flow style.
    ///
    /// # Errors
    /// Returns an error if the node is not a collection or an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.set_collection_style(self.id, style))
    }

    /// Append a comment. `text` is everything after `#`, including any leading space.
    /// The transaction fails if the placement would change the comment's owner.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.context.mutate(|| {
            let index = self.editor.node(self.id)?.comments().len();
            self.editor.insert_comment(self.id, index, text, placement)
        })
    }

    /// Insert a comment before the comment at `index`, or append it at the end of the list.
    /// As in [`Self::append_comment`], `text` starts immediately after `#`.
    /// A detached comment uses the next gap between children in the comment list,
    /// or follows the node if there is no such gap.
    /// The comment order and placement must be representable in the committed YAML.
    ///
    /// # Errors
    /// Returns an error if `index` exceeds the number of comments, `text` contains a line
    /// break, or an editing limit is exceeded.
    pub fn insert_comment(
        &mut self,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.insert_comment(self.id, index, text, placement))
    }

    /// Replace the comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.replace_comment(self.id, index, text))
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.context
            .mutate(|| self.editor.remove_comment(self.id, index))
    }
}

impl<'edit, 'doc, Kind> Cursor<'edit, 'doc, Collection<Kind>> {
    fn borrow_cursor(&mut self) -> Cursor<'_, 'doc> {
        self.context.reborrow().cursor(self.editor, self.id)
    }

    /// Return the number of entries or items, accounting for pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.state.len
    }

    /// Return whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a cursor at this collection, allowing navigation or replacement.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'edit, 'doc> {
        self.context.cursor(self.editor, self.id)
    }
}

/// The path of this edit and the first mutation error in the transaction.
/// Navigation and reads report their path without recording a failure.
#[derive(Debug)]
struct EditContext<'edit> {
    failure: &'edit mut Option<Error>,
    path: Cow<'edit, str>,
}

impl<'edit> EditContext<'edit> {
    fn cursor<'doc>(
        self,
        editor: &'edit mut syntax::Editor<'doc>,
        id: NodeId,
    ) -> Cursor<'edit, 'doc> {
        Cursor {
            editor,
            id,
            state: (),
            context: self,
        }
    }

    fn reborrow(&mut self) -> EditContext<'_> {
        EditContext {
            path: Cow::Borrowed(&self.path),
            failure: self.failure,
        }
    }

    fn at(mut self, segment: fmt::Arguments<'_>) -> Self {
        use std::fmt::Write;

        self.path.to_mut().write_fmt(segment).expect("String write");
        self
    }

    fn child(&mut self, segment: fmt::Arguments<'_>) -> EditContext<'_> {
        self.reborrow().at(segment)
    }

    fn with_path<T>(&self, result: Result<T, impl Into<Error>>) -> Result<T, Error> {
        result.map_err(|error| error.into().with_path_prefix(&self.path))
    }

    fn mutate<T, E: Into<Error>>(
        &mut self,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, Error> {
        if let Some(error) = self.failure.as_ref() {
            return Err(error.clone());
        }
        self.with_path(operation())
            .inspect_err(|error| *self.failure = Some(error.clone()))
    }
}
