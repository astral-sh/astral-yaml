//! Edit YAML using Rust values inside an [`Editable`](crate::Editable) transaction.

use crate::{Error, to_string};
use astral_yaml::{
    Span,
    edit::{Editor as SyntaxEditor, Handle},
    schema::{ScalarType, classify},
};
use serde::Serialize;
use std::fmt::Write;

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
/// YAML fragments, aliases, or mapping keys that are not strings.
#[derive(Debug)]
pub struct Editor<'a, 'doc> {
    syntax: &'a mut SyntaxEditor<'doc>,
    failure: Option<Error>,
}

impl<'a, 'doc> Editor<'a, 'doc> {
    pub(crate) fn new(syntax: &'a mut SyntaxEditor<'doc>) -> Self {
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
        let handle = self.syntax.root();
        Cursor {
            syntax: self.syntax,
            failure: &mut self.failure,
            handle,
            path: "$".to_owned(),
        }
    }

    /// Return the syntax editor for this transaction.
    pub fn syntax_mut(&mut self) -> &mut SyntaxEditor<'doc> {
        self.syntax
    }
}

/// A position in the YAML document being edited.
///
/// Navigation consumes the cursor. Use [`Self::reborrow`] when you need to return
/// to the same position later. Navigation does not follow aliases.
///
/// Methods returning `Result` enforce the document's resource limits. A failed mutation
/// prevents the transaction from committing, even if its error is handled.
#[derive(Debug)]
pub struct Cursor<'a, 'doc> {
    syntax: &'a mut SyntaxEditor<'doc>,
    failure: &'a mut Option<Error>,
    handle: Handle,
    path: String,
}

impl<'doc> Cursor<'_, 'doc> {
    /// Borrow this position for an edit without consuming the original cursor.
    pub fn reborrow(&mut self) -> Cursor<'_, 'doc> {
        Cursor {
            syntax: self.syntax,
            failure: self.failure,
            handle: self.handle.clone(),
            path: self.path.clone(),
        }
    }

    /// Return the selected node, including pending edits.
    /// Its span refers to the original source, even after replacement. Inserted nodes
    /// have no source span.
    ///
    /// # Errors
    /// Returns an error if the transaction's work limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.syntax
            .node(&self.handle)
            .map_err(|error| self.context(error.into()))
    }

    fn context(&self, mut error: Error) -> Error {
        error.message = format!("{}: {}", self.path, error.message);
        error
    }

    fn mutate<R>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(error) = self.failure.as_ref() {
            return Err(error.clone());
        }
        let result = operation(self).map_err(|error| self.context(error));
        if let Err(error) = &result {
            *self.failure = Some(error.clone());
        }
        result
    }

    fn field_index(&self, name: &str) -> Result<Option<(usize, Handle)>, Error> {
        let mut found = None;
        for (index, (key, value)) in self
            .syntax
            .mapping_entries(&self.handle)?
            .into_iter()
            .enumerate()
        {
            if string_key(self.syntax.node(&key)?) == Some(name) {
                if found.is_some() {
                    return Err(Error::new(format!("ambiguous mapping key {name:?}")));
                }
                found = Some((index, value));
            }
        }
        Ok(found)
    }

    /// Select the value at string key `name`, or return `None` if it is absent.
    ///
    /// Keys are compared after decoding. For example, `'name'` matches `name`, but
    /// `!custom name` does not. Numeric and boolean keys do not match their string
    /// spellings.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping or more than one key matches.
    pub fn get(mut self, name: &str) -> Result<Option<Self>, Error> {
        let found = self
            .field_index(name)
            .map_err(|error| self.context(error))?;
        if let Some((_, handle)) = found {
            self.handle = handle;
            let _ = write!(self.path, "[{name:?}]");
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }

    /// Like [`Self::get`], but require the key to exist.
    /// Characters such as `.` and `/` in `name` are literal, not path separators.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, or the key is missing or ambiguous.
    pub fn field(self, name: &str) -> Result<Self, Error> {
        let path = format!("{}[{name:?}]", self.path);
        self.get(name)?
            .ok_or_else(|| Error::new(format!("{path}: missing mapping key")))
    }

    /// Select the sequence item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or `index` is out of bounds.
    pub fn item(mut self, index: usize) -> Result<Self, Error> {
        let items = self
            .syntax
            .sequence_items(&self.handle)
            .map_err(|error| self.context(error.into()))?;
        let _ = write!(self.path, "[{index}]");
        self.handle = items
            .into_iter()
            .nth(index)
            .ok_or_else(|| self.context(Error::new("item index out of bounds")))?;
        Ok(self)
    }

    /// Replace the selected node with a serialized Rust value.
    ///
    /// The replacement keeps the node's anchor, outer comments, and compatible style.
    /// Tags come from `value`. Strings are serialized as strings, so `set("false")`
    /// and `set(false)` produce different YAML values.
    ///
    /// # Errors
    /// Returns an error if `value` cannot be serialized or an editing limit is exceeded.
    pub fn set(&mut self, value: impl Serialize) -> Result<(), Error> {
        self.mutate(|cursor| {
            let yaml = to_string(&value)?;
            cursor
                .syntax
                .replace_preserving_style(&cursor.handle, &yaml)?;
            Ok(())
        })
    }

    /// Set the value at string key `name` and return a cursor to it.
    /// If the key is absent, append a new entry to the mapping.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, the key is ambiguous, or
    /// `value` cannot be serialized.
    pub fn set_entry(mut self, name: &str, value: impl Serialize) -> Result<Self, Error> {
        let handle = self.mutate(|cursor| {
            let yaml = to_string(&value)?;
            if let Some((_, handle)) = cursor.field_index(name)? {
                cursor.syntax.replace_preserving_style(&handle, &yaml)?;
                Ok(handle)
            } else {
                let index = cursor.syntax.mapping_entries(&cursor.handle)?.len();
                let key = to_string(name)?;
                cursor
                    .syntax
                    .insert_entry(&cursor.handle, index, &key, &yaml)?;
                Ok(cursor
                    .syntax
                    .mapping_entries(&cursor.handle)?
                    .remove(index)
                    .1)
            }
        })?;
        self.handle = handle;
        let _ = write!(self.path, "[{name:?}]");
        Ok(self)
    }

    /// Remove the entry at string key `name`, including its comments.
    /// Return whether an entry was removed.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping or the key is ambiguous.
    pub fn remove_field(&mut self, name: &str) -> Result<bool, Error> {
        self.mutate(|cursor| {
            if let Some((index, _)) = cursor.field_index(name)? {
                cursor.syntax.remove_entry(&cursor.handle, index)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }

    /// Append `value` to the sequence and return a cursor to the new item.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or `value` cannot be serialized.
    pub fn push(mut self, value: impl Serialize) -> Result<Self, Error> {
        let index =
            self.mutate(|cursor| Ok(cursor.syntax.sequence_items(&cursor.handle)?.len()))?;
        self.insert(index, value)
    }

    /// Insert `value` before `index` and return a cursor to the new item.
    /// An index equal to the sequence's length appends the item.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence, `index` exceeds its length,
    /// or `value` cannot be serialized.
    pub fn insert(mut self, index: usize, value: impl Serialize) -> Result<Self, Error> {
        let handle = self.mutate(|cursor| {
            let yaml = to_string(&value)?;
            cursor.syntax.insert_item(&cursor.handle, index, &yaml)?;
            Ok(cursor.syntax.sequence_items(&cursor.handle)?.remove(index))
        })?;
        self.handle = handle;
        let _ = write!(self.path, "[{index}]");
        Ok(self)
    }

    /// Remove the item at `index`, including its comments.
    /// Later items shift down by one index.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or `index` is out of bounds.
    pub fn remove_item(&mut self, index: usize) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.remove_item(&cursor.handle, index)?))
    }

    /// Request a style for the selected scalar.
    /// The transaction fails if that style cannot represent the scalar's text in its
    /// surrounding YAML.
    ///
    /// # Errors
    /// Returns an error if the node is not a scalar or the style changes its YAML type.
    pub fn set_scalar_style(&mut self, style: ScalarStyle) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.set_scalar_style(&cursor.handle, style)?))
    }

    /// Request block or flow style for the selected collection.
    /// Empty collections always use flow style.
    ///
    /// # Errors
    /// Returns an error if the node is not a collection.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.set_collection_style(&cursor.handle, style)?))
    }

    /// Append a comment to the selected node.
    /// `text` is everything after `#`, including any leading space.
    ///
    /// The transaction fails if the placement cannot be represented without changing
    /// which node owns a comment.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.mutate(|cursor| {
            let index = cursor.syntax.node(&cursor.handle)?.comments().len();
            cursor
                .syntax
                .insert_comment(&cursor.handle, index, text, placement)?;
            Ok(())
        })
    }

    /// Replace the text of the node's comment at `index`, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.replace_comment(&cursor.handle, index, text)?))
    }

    /// Remove the node's comment at `index`.
    ///
    /// # Errors
    /// Returns an error if `index` is out of bounds.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.remove_comment(&cursor.handle, index)?))
    }
}

fn string_key<Location>(node: &Node<Location>) -> Option<&str> {
    let NodeKind::Scalar { value, style } = node.kind() else {
        return None;
    };
    let kind = match node.tag() {
        Some("!" | "tag:yaml.org,2002:str") => ScalarType::String,
        Some(_) => return None,
        None if *style == ScalarStyle::Plain => classify(value),
        None => ScalarType::String,
    };
    (kind == ScalarType::String).then_some(value)
}
