//! Edit YAML using Rust values inside an [`Editable`](crate::Editable) transaction.

use crate::{Error, to_string};
use astral_yaml::{Span, edit as syntax};
use serde::Serialize;

#[cfg(test)]
mod tests;

pub use astral_yaml::{
    CollectionStyle, ScalarStyle,
    comments::CommentPlacement,
    edit::{CommentPosition, Node, NodeKind},
};

/// An editor for an [`Editable`](crate::Editable) transaction.
///
/// Start at [`Self::root_mut`] to edit Rust values. Use [`Self::syntax_mut`] for
/// aliases or structural edits involving mapping keys that are not strings.
#[derive(Debug)]
pub struct Editor<'a, 'doc> {
    syntax: &'a mut syntax::Editor<'doc>,
    failure: Option<Error>,
}

impl<'a, 'doc> Editor<'a, 'doc> {
    pub(crate) fn new(syntax: &'a mut syntax::Editor<'doc>) -> Self {
        Self {
            syntax,
            failure: None,
        }
    }

    pub(crate) fn finish(self) -> Result<(), Error> {
        self.failure.map_or(Ok(()), Err)
    }

    /// Return a cursor at the document's root.
    pub fn root_mut(&mut self) -> Cursor<'_, 'doc> {
        Cursor {
            syntax: self.syntax.root_mut(),
            failure: &mut self.failure,
            path: "$".to_owned(),
        }
    }

    /// Return the syntax editor for this transaction.
    pub fn syntax_mut(&mut self) -> &mut syntax::Editor<'doc> {
        self.syntax
    }
}

/// A position in the YAML document being edited.
///
/// Navigation consumes the cursor and does not follow aliases. Convert to a
/// [`MappingMut`] or [`SequenceMut`] to perform repeated edits of one collection.
/// A failed mutation prevents the transaction from committing, even if its error
/// is handled. Navigation errors can be handled without preventing a commit.
#[derive(Debug)]
pub struct Cursor<'a, 'doc> {
    syntax: syntax::Cursor<'a, 'doc>,
    failure: &'a mut Option<Error>,
    path: String,
}

impl<'a, 'doc> Cursor<'a, 'doc> {
    /// Select the value at string key `name`, or return `None` if it is absent.
    ///
    /// Keys are compared after decoding. Numeric, boolean, and application-tagged
    /// keys do not match their string spellings. Names are literal, not paths.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping or more than one key matches.
    pub fn get(self, name: &str) -> Result<Option<Self>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        let syntax = self
            .syntax
            .get(name)
            .map_err(|error| context(&path, error))?;
        Ok(syntax.map(|syntax| Self {
            syntax,
            failure: self.failure,
            path,
        }))
    }

    /// Select the required value at the literal string key `name`.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, or the key is missing or ambiguous.
    pub fn field(self, name: &str) -> Result<Self, Error> {
        let path = format!("{}[{name:?}]", self.path);
        self.get(name)?
            .ok_or_else(|| context(&path, Error::new("missing mapping key")))
    }

    /// Select the sequence item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or `index` is out of bounds.
    pub fn item(self, index: usize) -> Result<Self, Error> {
        let syntax = self
            .syntax
            .into_sequence()
            .map_err(|error| context(&self.path, error))?;
        let path = format!("{}[{index}]", self.path);
        let syntax = syntax
            .into_cursor()
            .item(index)
            .map_err(|error| context(&path, error))?;
        Ok(Self {
            syntax,
            failure: self.failure,
            path,
        })
    }

    /// Borrow the selected mapping for repeated edits.
    ///
    /// # Errors
    /// Returns an error if the selected node is not a mapping or an editing limit is exceeded.
    pub fn into_mapping(self) -> Result<MappingMut<'a, 'doc>, Error> {
        let syntax = self
            .syntax
            .into_mapping()
            .map_err(|error| context(&self.path, error))?;
        Ok(MappingMut {
            syntax,
            failure: self.failure,
            path: self.path,
        })
    }

    /// Borrow the selected sequence for repeated edits.
    ///
    /// # Errors
    /// Returns an error if the selected node is not a sequence or an editing limit is exceeded.
    pub fn into_sequence(self) -> Result<SequenceMut<'a, 'doc>, Error> {
        let syntax = self
            .syntax
            .into_sequence()
            .map_err(|error| context(&self.path, error))?;
        Ok(SequenceMut {
            syntax,
            failure: self.failure,
            path: self.path,
        })
    }

    /// Replace the selected node with a serialized Rust value.
    ///
    /// Keep its anchor, outer comments, and compatible style. Strings are serialized
    /// as strings, so `set("false")` and `set(false)` produce different YAML values.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn set(&mut self, value: impl Serialize) -> Result<(), Error> {
        let yaml = mutate(self.failure, &self.path, || to_string(&value))?;
        self.set_yaml(&yaml)
    }

    /// Replace the selected node with YAML, keeping its anchor and compatible style.
    /// Outer comments are retained and tags come from the fragment.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn set_yaml(&mut self, yaml: &str) -> Result<(), Error> {
        mutate(self.failure, &self.path, || self.syntax.set_yaml(yaml))
    }

    /// Replace the selected node with YAML, using the fragment's style and anchor.
    /// Outer comments are retained.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn replace_yaml(&mut self, yaml: &str) -> Result<(), Error> {
        mutate(self.failure, &self.path, || self.syntax.replace_yaml(yaml))
    }

    /// Request a style for the selected scalar.
    ///
    /// # Errors
    /// Returns an error if the node is not a scalar or the style changes its YAML type.
    pub fn set_scalar_style(&mut self, style: ScalarStyle) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.set_scalar_style(style)
        })
    }
}

/// A mapping selected for repeated edits.
///
/// Child selections borrow the view. Their cursors can replace the selected child
/// while the mapping itself remains available for later edits.
#[derive(Debug)]
pub struct MappingMut<'a, 'doc> {
    syntax: syntax::MappingMut<'a, 'doc>,
    failure: &'a mut Option<Error>,
    path: String,
}

impl<'doc> MappingMut<'_, 'doc> {
    /// Select the value at string key `name`, or return `None` if absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn get(&mut self, name: &str) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        let syntax = self
            .syntax
            .get(name)
            .map_err(|error| context(&path, error))?;
        Ok(syntax.map(|syntax| Cursor {
            syntax,
            failure: &mut *self.failure,
            path,
        }))
    }

    /// Select the required value at string key `name`.
    ///
    /// # Errors
    /// Returns an error if the key is missing or ambiguous, or an editing limit is exceeded.
    pub fn field(&mut self, name: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        self.get(name)?
            .ok_or_else(|| context(&path, Error::new("missing mapping key")))
    }

    /// Select an occupied or vacant entry at string key `name`.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn entry(&mut self, name: &str) -> Result<Entry<'_, 'doc>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        let syntax = self
            .syntax
            .entry(name)
            .map_err(|error| context(&path, error))?;
        Ok(Entry {
            syntax,
            failure: self.failure,
            path,
        })
    }

    /// Set the value at string key `name`, appending the entry if absent.
    /// Return a cursor for further edits of that value.
    ///
    /// # Errors
    /// Returns an error if serialization fails, the key is ambiguous, or an editing limit is exceeded.
    pub fn set(&mut self, name: &str, value: impl Serialize) -> Result<Cursor<'_, 'doc>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        let yaml = mutate(self.failure, &path, || to_string(&value))?;
        self.set_yaml(name, &yaml)
    }

    /// Set a YAML value at string key `name`, appending the entry if absent.
    /// Keep the existing value's anchor and compatible style.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid, the key is ambiguous, or an editing limit is exceeded.
    pub fn set_yaml(&mut self, name: &str, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let path = format!("{}[{name:?}]", self.path);
        let syntax = mutate(self.failure, &path, || self.syntax.set_yaml(name, yaml))?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path,
        })
    }

    /// Remove the entry at string key `name`, including its comments.
    /// Return whether an entry was removed.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn remove(&mut self, name: &str) -> Result<bool, Error> {
        let path = format!("{}[{name:?}]", self.path);
        mutate(self.failure, &path, || self.syntax.remove(name))
    }

    /// Select the key at `index`, including keys that are not strings.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn key_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let syntax = self
            .syntax
            .key_at(index)
            .map_err(|error| context(&self.path, error))?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path: format!("{}[entry {index}].key", self.path),
        })
    }

    /// Select the value of the entry at `index`, including entries with non-string keys.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn value_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let syntax = self
            .syntax
            .value_at(index)
            .map_err(|error| context(&self.path, error))?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path: format!("{}[entry {index}].value", self.path),
        })
    }
}

/// A sequence selected for repeated edits.
#[derive(Debug)]
pub struct SequenceMut<'a, 'doc> {
    syntax: syntax::SequenceMut<'a, 'doc>,
    failure: &'a mut Option<Error>,
    path: String,
}

impl<'doc> SequenceMut<'_, 'doc> {
    /// Select the item at `index`, or return `None` if it is absent.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn get(&mut self, index: usize) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        let syntax = self
            .syntax
            .get(index)
            .map_err(|error| context(&self.path, error))?;
        Ok(syntax.map(|syntax| Cursor {
            syntax,
            failure: &mut *self.failure,
            path: format!("{}[{index}]", self.path),
        }))
    }

    /// Select the required item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn item(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let path = format!("{}[{index}]", self.path);
        self.get(index)?
            .ok_or_else(|| context(&path, Error::new("item index out of bounds")))
    }

    /// Append a serialized value and return a cursor to the new item.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn push(&mut self, value: impl Serialize) -> Result<Cursor<'_, 'doc>, Error> {
        let yaml = mutate(self.failure, &self.path, || to_string(&value))?;
        self.push_yaml(&yaml)
    }

    /// Append a YAML fragment and return a cursor to the new item.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn push_yaml(&mut self, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let index = self.len();
        let syntax = mutate(self.failure, &self.path, || self.syntax.push_yaml(yaml))?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path: format!("{}[{index}]", self.path),
        })
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
        let yaml = mutate(self.failure, &self.path, || to_string(&value))?;
        self.insert_yaml(index, &yaml)
    }

    /// Insert a YAML fragment before `index` and return a cursor to it.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid, `index` exceeds the length, or an editing limit is exceeded.
    pub fn insert_yaml(&mut self, index: usize, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let syntax = mutate(self.failure, &self.path, || {
            self.syntax.insert_yaml(index, yaml)
        })?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path: format!("{}[{index}]", self.path),
        })
    }

    /// Remove the item at `index`, including its comments. Later indices shift down.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove(&mut self, index: usize) -> Result<(), Error> {
        mutate(self.failure, &self.path, || self.syntax.remove(index))
    }
}

/// An occupied or vacant string-keyed mapping entry.
#[derive(Debug)]
pub struct Entry<'a, 'doc> {
    syntax: syntax::Entry<'a, 'doc>,
    failure: &'a mut Option<Error>,
    path: String,
}

impl<'a, 'doc> Entry<'a, 'doc> {
    /// Return whether the mapping already contains the entry.
    #[must_use]
    pub fn is_occupied(&self) -> bool {
        self.syntax.is_occupied()
    }

    /// Return the existing value, or insert a serialized default if absent.
    /// An occupied entry does not serialize the unused default.
    ///
    /// # Errors
    /// Returns an error if serialization fails or an editing limit is exceeded.
    pub fn or_insert(self, value: impl Serialize) -> Result<Cursor<'a, 'doc>, Error> {
        if self.is_occupied() {
            return self.or_insert_yaml("");
        }
        let yaml = mutate(self.failure, &self.path, || to_string(&value))?;
        self.or_insert_yaml(&yaml)
    }

    /// Return the existing value, or insert a YAML default if absent.
    /// An occupied entry does not parse the unused fragment.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn or_insert_yaml(self, yaml: &str) -> Result<Cursor<'a, 'doc>, Error> {
        let syntax = mutate(self.failure, &self.path, || {
            self.syntax.or_insert_yaml(yaml)
        })?;
        Ok(Cursor {
            syntax,
            failure: self.failure,
            path: self.path,
        })
    }
}

impl Cursor<'_, '_> {
    /// Return the selected node, including pending edits.
    /// Spans refer to the original source. Inserted nodes have no source span.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.syntax
            .node()
            .map_err(|error| context(&self.path, error))
    }

    /// Request block or flow style for this collection. Empty collections use flow style.
    ///
    /// # Errors
    /// Returns an error if the node is not a collection or an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.set_collection_style(style)
        })
    }

    /// Append a comment. `text` is everything after `#`, including any leading space.
    /// The transaction fails if the placement would change the comment's owner.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.append_comment(text, placement)
        })
    }

    /// Replace the comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.replace_comment(index, text)
        })
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.remove_comment(index)
        })
    }
}

impl<'a, 'doc> MappingMut<'a, 'doc> {
    /// Return the number of entries, accounting for pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.syntax.len()
    }

    /// Return whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.syntax.is_empty()
    }

    /// Return a cursor at this collection, allowing navigation or replacement.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'a, 'doc> {
        Cursor {
            syntax: self.syntax.into_cursor(),
            failure: self.failure,
            path: self.path,
        }
    }

    /// Return the selected node, including pending edits.
    /// Spans refer to the original source. Inserted nodes have no source span.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.syntax
            .node()
            .map_err(|error| context(&self.path, error))
    }

    /// Request block or flow style for this collection. Empty collections use flow style.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.set_collection_style(style)
        })
    }

    /// Append a comment. `text` is everything after `#`, including any leading space.
    /// The transaction fails if the placement would change the comment's owner.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.append_comment(text, placement)
        })
    }

    /// Replace the comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.replace_comment(index, text)
        })
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.remove_comment(index)
        })
    }
}

impl<'a, 'doc> SequenceMut<'a, 'doc> {
    /// Return the number of items, accounting for pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.syntax.len()
    }

    /// Return whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.syntax.is_empty()
    }

    /// Return a cursor at this collection, allowing navigation or replacement.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'a, 'doc> {
        Cursor {
            syntax: self.syntax.into_cursor(),
            failure: self.failure,
            path: self.path,
        }
    }

    /// Return the selected node, including pending edits.
    /// Spans refer to the original source. Inserted nodes have no source span.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.syntax
            .node()
            .map_err(|error| context(&self.path, error))
    }

    /// Request block or flow style for this collection. Empty collections use flow style.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.set_collection_style(style)
        })
    }

    /// Append a comment. `text` is everything after `#`, including any leading space.
    /// The transaction fails if the placement would change the comment's owner.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.append_comment(text, placement)
        })
    }

    /// Replace the comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.replace_comment(index, text)
        })
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        mutate(self.failure, &self.path, || {
            self.syntax.remove_comment(index)
        })
    }
}

fn context(path: &str, error: impl Into<Error>) -> Error {
    let mut error = error.into();
    error.message = format!("{path}: {}", error.message);
    error
}

fn mutate<R, E: Into<Error>>(
    failure: &mut Option<Error>,
    path: &str,
    operation: impl FnOnce() -> Result<R, E>,
) -> Result<R, Error> {
    if let Some(error) = failure.as_ref() {
        return Err(error.clone());
    }
    let result = operation().map_err(|error| context(path, error));
    if let Err(error) = &result {
        *failure = Some(error.clone());
    }
    result
}
