use super::{Editor, Handle};
use crate::{
    CollectionStyle, Error, ScalarStyle, Span,
    comments::CommentPlacement,
    edit::{Node, NodeKind, emit::quoted},
    schema::{ScalarType, classify},
};

#[cfg(test)]
mod tests;

/// A selected node in an edit transaction.
///
/// Navigation consumes the cursor and never follows aliases. Convert a collection
/// into [`MappingMut`] or [`SequenceMut`] to borrow it for repeated child edits.
/// Failed mutations prevent commit, even if their errors are handled.
#[derive(Debug)]
pub struct Cursor<'a, 'doc> {
    editor: &'a mut Editor<'doc>,
    handle: Handle,
}

impl<'a, 'doc> Cursor<'a, 'doc> {
    pub(super) fn new(editor: &'a mut Editor<'doc>, handle: Handle) -> Self {
        Self { editor, handle }
    }

    fn reborrow(&mut self) -> Cursor<'_, 'doc> {
        Cursor::new(self.editor, self.handle.clone())
    }

    fn mutate<R>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(error) = &self.editor.failure {
            return Err(error.clone());
        }
        let result = operation(self);
        if let Err(error) = &result {
            self.editor.failure = Some(error.clone());
        }
        result
    }

    /// Return the selected node, including pending edits.
    ///
    /// # Errors
    /// Returns an error if the transaction's work limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.editor.node(&self.handle)
    }

    fn field_index(&self, name: &str) -> Result<Option<(usize, Handle)>, Error> {
        let NodeKind::Mapping { entries, .. } = self.node()?.kind() else {
            return Err(self.editor.error("expected a mapping"));
        };
        self.editor.draft.charge(entries.len())?;
        let mut found = None;
        for (index, &(key, value)) in entries.iter().enumerate() {
            if string_key(self.editor.draft.node(key)) == Some(name) {
                if found.is_some() {
                    return Err(self.editor.error("ambiguous mapping key"));
                }
                found = Some((index, self.editor.handle(value)));
            }
        }
        Ok(found)
    }

    /// Select the value at string key `name`, or return `None` if absent.
    ///
    /// Names are literal decoded strings, including any `.` or `/`. Numeric,
    /// boolean, and custom-tagged keys do not match their string spellings.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, the key is ambiguous, or
    /// the work limit is exceeded.
    pub fn get(mut self, name: &str) -> Result<Option<Self>, Error> {
        if let Some((_, handle)) = self.field_index(name)? {
            self.handle = handle;
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }

    /// Select the value at string key `name`, requiring the key to exist.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, the key is missing or
    /// ambiguous, or the work limit is exceeded.
    pub fn field(mut self, name: &str) -> Result<Self, Error> {
        let (_, handle) = self
            .field_index(name)?
            .ok_or_else(|| self.editor.error("missing mapping key"))?;
        self.handle = handle;
        Ok(self)
    }

    /// Select the sequence item at `index`, accounting for earlier edits.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence, the index is out of bounds,
    /// or the work limit is exceeded.
    pub fn item(mut self, index: usize) -> Result<Self, Error> {
        self.handle = self
            .editor
            .sequence_item(&self.handle, index)?
            .ok_or_else(|| self.editor.error("item index out of bounds"))?;
        Ok(self)
    }

    /// Convert this selection into a reusable mapping view.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping or the work limit is exceeded.
    pub fn into_mapping(self) -> Result<MappingMut<'a, 'doc>, Error> {
        let NodeKind::Mapping { entries, .. } = self.node()?.kind() else {
            return Err(self.editor.error("expected a mapping"));
        };
        let len = entries.len();
        Ok(MappingMut { cursor: self, len })
    }

    /// Convert this selection into a reusable sequence view.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or the work limit is exceeded.
    pub fn into_sequence(self) -> Result<SequenceMut<'a, 'doc>, Error> {
        let NodeKind::Sequence { items, .. } = self.node()?.kind() else {
            return Err(self.editor.error("expected a sequence"));
        };
        let len = items.len();
        Ok(SequenceMut { cursor: self, len })
    }

    /// Replace the selected node with a YAML fragment, retaining its anchor,
    /// outer comments, and compatible style. The fragment supplies tags.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn set_yaml(&mut self, yaml: &str) -> Result<(), Error> {
        self.editor.replace_preserving_style(&self.handle, yaml)
    }

    /// Replace the selected node using the fragment's style and anchor, retaining
    /// the node's outer comments.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    /// Commit fails if an alias loses its anchor or changes its target.
    pub fn replace_yaml(&mut self, yaml: &str) -> Result<(), Error> {
        self.editor.replace(&self.handle, yaml)
    }

    /// Request a scalar style for the selected node.
    ///
    /// # Errors
    /// Returns an error if the node is not a scalar or the style changes its type.
    pub fn set_scalar_style(&mut self, style: ScalarStyle) -> Result<(), Error> {
        self.editor.set_scalar_style(&self.handle, style)
    }

    /// Request block or flow style. Empty collections always use flow style.
    ///
    /// # Errors
    /// Returns an error if the node is not a collection or an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.editor.set_collection_style(&self.handle, style)
    }

    /// Append a comment. `text` is everything after `#`, including leading space.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    /// Commit fails if the placement cannot preserve comment ownership.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.mutate(|cursor| {
            let index = cursor.node()?.comments().len();
            cursor
                .editor
                .insert_comment(&cursor.handle, index, text, placement)
        })
    }

    /// Replace a comment's text, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.editor.replace_comment(&self.handle, index, text)
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.editor.remove_comment(&self.handle, index)
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

/// A mapping borrowed for repeated edits.
///
/// Child cursors borrow this view, allowing sibling edits without repeating the
/// path from the root. Its kind and length remain valid because child cursors
/// cannot replace the mapping or edit its ancestors.
#[derive(Debug)]
pub struct MappingMut<'a, 'doc> {
    cursor: Cursor<'a, 'doc>,
    len: usize,
}

impl<'a, 'doc> MappingMut<'a, 'doc> {
    /// Return the number of entries, including pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return whether the mapping has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Return a cursor to this mapping, allowing replacement or further navigation.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'a, 'doc> {
        self.cursor
    }

    /// Borrow the value at string key `name`, or return `None` if absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or the work limit is exceeded.
    pub fn get(&mut self, name: &str) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        self.cursor.reborrow().get(name)
    }

    /// Borrow the value at string key `name`, requiring the key to exist.
    ///
    /// # Errors
    /// Returns an error if the key is absent or ambiguous, or the work limit is exceeded.
    pub fn field(&mut self, name: &str) -> Result<Cursor<'_, 'doc>, Error> {
        self.cursor.reborrow().field(name)
    }

    /// Borrow the key at `index`. Keys can be any kind of YAML node.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn key_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let (key, _) = self
            .cursor
            .editor
            .mapping_entry(&self.cursor.handle, index)?
            .ok_or_else(|| self.cursor.editor.error("entry index out of bounds"))?;
        Ok(Cursor::new(self.cursor.editor, key))
    }

    /// Borrow the value at `index`, including entries with non-string keys.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn value_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        let (_, value) = self
            .cursor
            .editor
            .mapping_entry(&self.cursor.handle, index)?
            .ok_or_else(|| self.cursor.editor.error("entry index out of bounds"))?;
        Ok(Cursor::new(self.cursor.editor, value))
    }

    /// Select a string entry for inserting a default if it is absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or the work limit is exceeded.
    pub fn entry(&mut self, name: &str) -> Result<Entry<'_, 'doc>, Error> {
        let state = if let Some((_, handle)) = self.cursor.field_index(name)? {
            EntryState::Occupied(Cursor::new(self.cursor.editor, handle))
        } else {
            EntryState::Vacant {
                cursor: self.cursor.reborrow(),
                name: name.to_owned(),
                len: &mut self.len,
            }
        };
        Ok(Entry { state })
    }

    /// Set a string entry's YAML value and return a cursor to it.
    /// Existing values keep their anchor and compatible style. Absent entries
    /// append a safely quoted string key.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous, the fragment is invalid, or an
    /// editing limit is exceeded.
    pub fn set_yaml(&mut self, name: &str, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let (handle, inserted) = self.cursor.mutate(|cursor| {
            if let Some((_, handle)) = cursor.field_index(name)? {
                cursor.editor.replace_preserving_style(&handle, yaml)?;
                Ok((handle, false))
            } else {
                let (_, handle) =
                    cursor
                        .editor
                        .insert_entry(&cursor.handle, self.len, &quoted(name), yaml)?;
                Ok((handle, true))
            }
        })?;
        self.len += usize::from(inserted);
        Ok(Cursor::new(self.cursor.editor, handle))
    }

    /// Remove a string entry and its comments, returning whether it existed.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn remove(&mut self, name: &str) -> Result<bool, Error> {
        let removed = self.cursor.mutate(|cursor| {
            if let Some((index, _)) = cursor.field_index(name)? {
                cursor.editor.remove_entry(&cursor.handle, index)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        self.len -= usize::from(removed);
        Ok(removed)
    }
}

/// A sequence borrowed for repeated edits.
///
/// Each selected or inserted item borrows this view for its edit. Later indices
/// account for earlier insertions and removals.
#[derive(Debug)]
pub struct SequenceMut<'a, 'doc> {
    cursor: Cursor<'a, 'doc>,
    len: usize,
}

impl<'a, 'doc> SequenceMut<'a, 'doc> {
    /// Return the number of items, including pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return whether the sequence has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Return a cursor to this sequence, allowing replacement or further navigation.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'a, 'doc> {
        self.cursor
    }

    /// Borrow the item at `index`, or return `None` if out of bounds.
    ///
    /// # Errors
    /// Returns an error if the work limit is exceeded.
    pub fn get(&mut self, index: usize) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        let handle = self
            .cursor
            .editor
            .sequence_item(&self.cursor.handle, index)?;
        Ok(handle.map(|handle| Cursor::new(self.cursor.editor, handle)))
    }

    /// Borrow the item at `index`, requiring it to exist.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn item(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        self.cursor.reborrow().item(index)
    }

    /// Append a YAML fragment and return a cursor to the inserted item.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn push_yaml(&mut self, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        self.insert_yaml(self.len, yaml)
    }

    /// Insert a YAML fragment before `index` and return a cursor to the new item.
    /// An index equal to the sequence's length appends the item.
    ///
    /// # Errors
    /// Returns an error if the index exceeds the length, the fragment is invalid,
    /// or an editing limit is exceeded.
    pub fn insert_yaml(&mut self, index: usize, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let handle = self
            .cursor
            .editor
            .insert_item(&self.cursor.handle, index, yaml)?;
        self.len += 1;
        Ok(Cursor::new(self.cursor.editor, handle))
    }

    /// Remove an item and its comments. Later indices shift down by one.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or an editing limit is exceeded.
    pub fn remove(&mut self, index: usize) -> Result<(), Error> {
        self.cursor.editor.remove_item(&self.cursor.handle, index)?;
        self.len -= 1;
        Ok(())
    }
}

/// A selected string entry, which may already exist in its mapping.
#[derive(Debug)]
pub struct Entry<'a, 'doc> {
    state: EntryState<'a, 'doc>,
}

#[derive(Debug)]
enum EntryState<'a, 'doc> {
    Occupied(Cursor<'a, 'doc>),
    Vacant {
        cursor: Cursor<'a, 'doc>,
        name: String,
        len: &'a mut usize,
    },
}

impl<'a, 'doc> Entry<'a, 'doc> {
    /// Return whether this entry already has a value.
    #[must_use]
    pub fn is_occupied(&self) -> bool {
        matches!(self.state, EntryState::Occupied(_))
    }

    /// Return the existing value or append `yaml` as its default.
    /// An occupied entry leaves its value unchanged and does not parse `yaml`.
    ///
    /// # Errors
    /// Returns an error if a needed fragment is invalid or an editing limit is exceeded.
    pub fn or_insert_yaml(self, yaml: &str) -> Result<Cursor<'a, 'doc>, Error> {
        match self.state {
            EntryState::Occupied(cursor) => Ok(cursor),
            EntryState::Vacant { cursor, name, len } => {
                let (_, handle) =
                    cursor
                        .editor
                        .insert_entry(&cursor.handle, *len, &quoted(&name), yaml)?;
                *len += 1;
                Ok(Cursor::new(cursor.editor, handle))
            }
        }
    }
}

impl MappingMut<'_, '_> {
    /// Return the collection node, including pending edits.
    ///
    /// # Errors
    /// Returns an error if the work limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.cursor.node()
    }

    /// Request block or flow style. Empty collections always use flow style.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.cursor.set_collection_style(style)
    }

    /// Append a comment. `text` is everything after `#`, including leading space.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    /// Commit fails if the placement cannot preserve comment ownership.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.cursor.append_comment(text, placement)
    }

    /// Replace a comment's text, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.cursor.replace_comment(index, text)
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.cursor.remove_comment(index)
    }
}

impl SequenceMut<'_, '_> {
    /// Return the collection node, including pending edits.
    ///
    /// # Errors
    /// Returns an error if the work limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.cursor.node()
    }

    /// Request block or flow style. Empty collections always use flow style.
    ///
    /// # Errors
    /// Returns an error if an editing limit is exceeded.
    pub fn set_collection_style(&mut self, style: CollectionStyle) -> Result<(), Error> {
        self.cursor.set_collection_style(style)
    }

    /// Append a comment. `text` is everything after `#`, including leading space.
    ///
    /// # Errors
    /// Returns an error if `text` contains a line break or an editing limit is exceeded.
    /// Commit fails if the placement cannot preserve comment ownership.
    pub fn append_comment(&mut self, text: &str, placement: CommentPlacement) -> Result<(), Error> {
        self.cursor.append_comment(text, placement)
    }

    /// Replace a comment's text, keeping its placement.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or `text` contains a line break.
    pub fn replace_comment(&mut self, index: usize, text: &str) -> Result<(), Error> {
        self.cursor.replace_comment(index, text)
    }

    /// Remove the comment at `index`.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or an editing limit is exceeded.
    pub fn remove_comment(&mut self, index: usize) -> Result<(), Error> {
        self.cursor.remove_comment(index)
    }
}
