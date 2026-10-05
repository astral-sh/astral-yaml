//! Value-based editing inside an [`Editable`](crate::Editable) transaction.

use crate::{Error, to_string};
use astral_yaml::{
    edit::{Editor as SyntaxEditor, Handle},
    schema::{ScalarType, classify},
};
use serde::Serialize;
use std::fmt::Write;

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;

pub use astral_yaml::{
    CollectionStyle, ScalarStyle,
    comments::CommentPlacement,
    edit::{Node, NodeKind},
};

/// A transaction's value-based editor. Obtain a cursor with [`Self::root_mut`].
/// Raw YAML fragments are accepted only by the lower-level [`Self::syntax_mut`] API.
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

    /// Selects the candidate document's root.
    pub fn root_mut(&mut self) -> Cursor<'_, 'doc> {
        let handle = self.syntax.root();
        Cursor {
            syntax: self.syntax,
            failure: &mut self.failure,
            handle,
            path: "$".to_owned(),
        }
    }

    /// Accesses syntax handles, arbitrary YAML keys, and raw-fragment operations.
    pub fn syntax_mut(&mut self) -> &mut SyntaxEditor<'doc> {
        self.syntax
    }
}

/// A mutable position in the candidate syntax, scoped to one transaction.
/// Navigation consumes the cursor; [`Self::reborrow`] retains a parent position.
/// Aliases are occurrences and are not followed implicitly.
#[derive(Debug)]
pub struct Cursor<'a, 'doc> {
    syntax: &'a mut SyntaxEditor<'doc>,
    failure: &'a mut Option<Error>,
    handle: Handle,
    path: String,
}

impl<'doc> Cursor<'_, 'doc> {
    /// Borrows this position temporarily, for navigating or inserting while
    /// retaining the parent cursor.
    pub fn reborrow(&mut self) -> Cursor<'_, 'doc> {
        Cursor {
            syntax: self.syntax,
            failure: self.failure,
            handle: self.handle.clone(),
            path: self.path.clone(),
        }
    }

    /// Inspects the candidate node. Spans refer to the original source.
    ///
    /// # Errors
    /// Returns an error if the transaction's work budget is exhausted.
    pub fn node(&self) -> Result<&Node, Error> {
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

    /// Selects a decoded string key, returning `None` when it is absent.
    /// Application-tagged keys do not match their scalar text.
    ///
    /// # Errors
    /// Rejects non-mappings, ambiguous keys, and exhausted work budgets.
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

    /// Selects a required decoded string key. Names are literal YAML keys;
    /// periods, slashes, and brackets have no special meaning.
    ///
    /// # Errors
    /// Rejects missing or ambiguous keys, non-mappings, and exhausted budgets.
    pub fn field(self, name: &str) -> Result<Self, Error> {
        let path = format!("{}[{name:?}]", self.path);
        self.get(name)?
            .ok_or_else(|| Error::new(format!("{path}: missing mapping key")))
    }

    /// Selects a sequence item at its current index.
    ///
    /// # Errors
    /// Rejects non-sequences, out-of-range indices, and exhausted budgets.
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

    /// Replaces this node with a serialized Rust value. Strings remain strings.
    /// Retains its anchor, outer comments, and compatible presentation style;
    /// tags come from the new value. Old descendants are removed.
    ///
    /// # Errors
    /// Serialization and mutation failures poison the transaction, even if caught.
    pub fn set(&mut self, value: impl Serialize) -> Result<(), Error> {
        self.mutate(|cursor| {
            let yaml = to_string(&value)?;
            cursor
                .syntax
                .replace_preserving_style(&cursor.handle, &yaml)?;
            Ok(())
        })
    }

    /// Replaces an existing string-keyed value in place, or appends a missing
    /// entry. Selects the value so its style and comments can be edited.
    ///
    /// # Errors
    /// Rejects ambiguous keys, non-mappings, serialization errors, and limits.
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

    /// Removes a string-keyed entry and its owned comments. Returns whether the
    /// entry existed; a missing key leaves the mapping unchanged.
    ///
    /// # Errors
    /// Rejects non-mappings, ambiguous keys, and exhausted budgets.
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

    /// Appends a serialized value and selects the inserted item.
    ///
    /// # Errors
    /// Rejects non-sequences, serialization errors, and resource limits.
    pub fn push(mut self, value: impl Serialize) -> Result<Self, Error> {
        let index =
            self.mutate(|cursor| Ok(cursor.syntax.sequence_items(&cursor.handle)?.len()))?;
        self.insert(index, value)
    }

    /// Inserts a serialized value before `index` and selects it. `len` appends.
    ///
    /// # Errors
    /// Rejects non-sequences, invalid indices, serialization errors, and limits.
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

    /// Removes an item and its owned comments. Subsequent indices reflect removal.
    ///
    /// # Errors
    /// Rejects non-sequences, invalid indices, and resource limits.
    pub fn remove_item(&mut self, index: usize) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.remove_item(&cursor.handle, index)?))
    }

    /// Requests an exact scalar style; commit rejects an incompatible request.
    ///
    /// # Errors
    /// Rejects non-scalars, type-changing styles, and resource limits.
    pub fn set_scalar_style(&mut self, style: ScalarStyle) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.set_scalar_style(&cursor.handle, style)?))
    }

    /// Requests block or flow syntax. Empty collections remain in flow style.
    ///
    /// # Errors
    /// Rejects non-collections and exhausted resource limits.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.set_collection_style(&cursor.handle, style)?))
    }

    /// Appends an owned comment without `#` or a line break. Include a leading
    /// space in `text` when the emitted comment should start with `# `.
    ///
    /// # Errors
    /// Rejects invalid text, conflicting inline comments, and resource limits.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.mutate(|cursor| {
            let index = cursor.syntax.node(&cursor.handle)?.comments().len();
            cursor
                .syntax
                .insert_comment(&cursor.handle, index, text, placement)?;
            Ok(())
        })
    }

    /// Replaces a directly owned comment's text, retaining its placement.
    ///
    /// # Errors
    /// Rejects invalid indices, multiline text, and resource limits.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.replace_comment(&cursor.handle, index, text)?))
    }

    /// Removes a directly owned comment.
    ///
    /// # Errors
    /// Rejects invalid indices and exhausted resource limits.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.mutate(|cursor| Ok(cursor.syntax.remove_comment(&cursor.handle, index)?))
    }
}

fn string_key(node: &Node) -> Option<&str> {
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
