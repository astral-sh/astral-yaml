use super::{Draft, Editor, Handle};
use crate::{
    CollectionStyle, Error, NodeId, ScalarStyle, Span,
    comments::CommentPlacement,
    edit::{Node, NodeKind, emit::quoted},
    schema::{ScalarType, classify},
};
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

/// Build an edit path from literal mapping keys and zero-based sequence indices.
///
/// For a path containing only keys or only indices, an array like `["service", "port"]`
/// works without this macro. Keys containing `.` or `/` are always taken literally.
///
/// ```
/// use astral_yaml::edit::{EditableDocument, edit_path};
///
/// let mut doc = EditableDocument::parse("servers: [{host: 'old'}]\n")?;
/// doc.set_yaml_at(edit_path!["servers", 0, "host"], "new")?;
/// assert_eq!(doc.source(), "servers: [{host: 'new'}]\n");
/// # Ok::<(), astral_yaml::Error>(())
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
pub enum PathSegment<'a> {
    /// A literal string key in a mapping.
    Key(&'a str),
    /// A zero-based position in a sequence.
    Index(usize),
}

impl<'a> From<&'a str> for PathSegment<'a> {
    fn from(key: &'a str) -> Self {
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

/// A selected node in an edit transaction.
///
/// Navigation consumes the cursor and never follows aliases. Convert a collection
/// into [`MappingMut`] or [`SequenceMut`] to borrow it for repeated child edits.
/// Failed mutations prevent commit, even if their errors are handled.
#[derive(Debug)]
pub struct Cursor<'a, 'doc, State = ()> {
    editor: &'a mut Editor<'doc>,
    handle: Handle,
    state: State,
}

/// A read-only view of a selected value, including pending edits.
///
/// Start at [`Self::root`] and follow node IDs in the syntax, including alias
/// targets outside the selected subtree. Source spans refer to the original
/// document and are absent on newly inserted nodes.
#[derive(Clone, Copy, Debug)]
pub struct ValueRef<'a> {
    draft: &'a Draft,
    root: NodeId,
}

impl<'a> ValueRef<'a> {
    /// Return the ID of the selected node.
    #[must_use]
    pub fn root(self) -> NodeId {
        self.root
    }

    /// Read a node by its ID in this transaction.
    ///
    /// # Errors
    /// Returns an error if the node was removed or the work limit is exceeded.
    pub fn node(self, id: NodeId) -> Result<&'a Node<Option<Span>>, Error> {
        self.draft.charge(1)?;
        if !self.draft.contains(id) {
            return Err(self.draft.error("node is no longer in the document"));
        }
        Ok(self.draft.node(id))
    }
}

impl<'a, 'doc> Cursor<'a, 'doc> {
    pub(super) fn new(editor: &'a mut Editor<'doc>, handle: Handle) -> Self {
        Self {
            editor,
            handle,
            state: (),
        }
    }

    /// Lend this position to an edit, so navigation can start here again afterwards.
    pub fn reborrow(&mut self) -> Cursor<'_, 'doc> {
        self.borrow_cursor()
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

    /// Select the value at string key `name`, or return `None` if absent.
    ///
    /// Names are literal decoded strings, including any `.` or `/`. Numeric,
    /// boolean, and custom-tagged keys do not match their string spellings.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, the key is ambiguous, or
    /// the work limit is exceeded.
    pub fn get(self, name: &str) -> Result<Option<Self>, Error> {
        let field = self.editor.string_field(&self.handle, name)?;
        Ok(field.map(|(_, handle)| Self { handle, ..self }))
    }

    /// Select the value at string key `name`, requiring the key to exist.
    ///
    /// # Errors
    /// Returns an error if the node is not a mapping, the key is missing or
    /// ambiguous, or the work limit is exceeded.
    pub fn field(self, name: &str) -> Result<Self, Error> {
        let (_, handle) = self
            .editor
            .string_field(&self.handle, name)?
            .ok_or_else(|| self.editor.error("missing mapping key"))?;
        Ok(Self { handle, ..self })
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
        let len = self.editor.entry_ids(&self.handle)?.len();
        Ok(self.into_collection(len))
    }

    /// Convert this selection into a reusable sequence view.
    ///
    /// # Errors
    /// Returns an error if the node is not a sequence or the work limit is exceeded.
    pub fn into_sequence(self) -> Result<SequenceMut<'a, 'doc>, Error> {
        let len = self.editor.item_ids(&self.handle)?.len();
        Ok(self.into_collection(len))
    }

    fn into_collection<Kind>(self, len: usize) -> Cursor<'a, 'doc, Collection<Kind>> {
        Cursor {
            editor: self.editor,
            handle: self.handle,
            state: Collection {
                len,
                kind: PhantomData,
            },
        }
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
}

impl<'doc, State> Cursor<'_, 'doc, State> {
    fn borrow_cursor(&mut self) -> Cursor<'_, 'doc> {
        Cursor::new(self.editor, self.handle.clone())
    }

    /// Return the selected node, including pending edits.
    ///
    /// # Errors
    /// Returns an error if the transaction's work limit is exceeded.
    pub fn node(&self) -> Result<&Node<Option<Span>>, Error> {
        self.editor.node(&self.handle)
    }

    /// Borrow the selected value and the nodes it references, including pending edits.
    #[must_use]
    pub fn value(&self) -> ValueRef<'_> {
        ValueRef {
            draft: &self.editor.draft,
            root: self.handle.id,
        }
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
        self.editor.mutate(|editor| {
            let index = editor.node(&self.handle)?.comments().len();
            editor.insert_comment(&self.handle, index, text, placement)
        })
    }

    /// Insert a comment at `index`, or append when `index` is the number of comments.
    ///
    /// # Errors
    /// Returns an error for an invalid index, text with a line break, or an editing limit.
    pub fn insert_comment(
        &mut self,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.editor
            .insert_comment(&self.handle, index, text, placement)
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

impl Editor<'_> {
    fn string_field(&self, mapping: &Handle, name: &str) -> Result<Option<(usize, Handle)>, Error> {
        let entries = self.entry_ids(mapping)?;
        self.draft.charge(entries.len())?;
        let mut found = None;
        for (index, &(key, value)) in entries.iter().enumerate() {
            if string_key(self.draft.node(key)) == Some(name) {
                if found.is_some() {
                    return Err(self.error("ambiguous mapping key"));
                }
                found = Some((index, self.handle(value)));
            }
        }
        Ok(found)
    }

    fn append_string_entry(
        &mut self,
        mapping: &Handle,
        len: &mut usize,
        name: &str,
        yaml: &str,
    ) -> Result<Handle, Error> {
        let (_, value) = self.insert_entry(mapping, *len, &quoted(name), yaml)?;
        *len += 1;
        Ok(value)
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
pub type MappingMut<'a, 'doc> = Cursor<'a, 'doc, Collection<Mapping>>;

/// A sequence borrowed for repeated edits.
///
/// Each selected or inserted item borrows this view for its edit. Later indices
/// account for earlier insertions and removals.
pub type SequenceMut<'a, 'doc> = Cursor<'a, 'doc, Collection<Sequence>>;

/// Cursor state for a collection borrowed for repeated edits.
#[derive(Debug)]
pub struct Collection<Kind> {
    len: usize,
    kind: PhantomData<Kind>,
}

/// The collection kind of a [`MappingMut`] cursor.
#[derive(Debug)]
pub enum Mapping {}

/// The collection kind of a [`SequenceMut`] cursor.
#[derive(Debug)]
pub enum Sequence {}

impl<'a, 'doc, Kind> Cursor<'a, 'doc, Collection<Kind>> {
    /// Return the number of entries or items, including pending edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.state.len
    }

    /// Return whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// End the collection borrow, allowing replacement or further navigation.
    #[must_use]
    pub fn into_cursor(self) -> Cursor<'a, 'doc> {
        Cursor::new(self.editor, self.handle)
    }
}

impl<'doc> MappingMut<'_, 'doc> {
    /// Borrow the value at string key `name`, or return `None` if absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or the work limit is exceeded.
    pub fn get(&mut self, name: &str) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        self.borrow_cursor().get(name)
    }

    /// Borrow the value at string key `name`, requiring the key to exist.
    ///
    /// # Errors
    /// Returns an error if the key is absent or ambiguous, or the work limit is exceeded.
    pub fn field(&mut self, name: &str) -> Result<Cursor<'_, 'doc>, Error> {
        self.borrow_cursor().field(name)
    }

    /// Borrow the key at `index`. Keys can be any kind of YAML node.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn key_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        self.entry_at(index, |(key, _)| key)
    }

    /// Borrow the value at `index`, including entries with non-string keys.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn value_at(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        self.entry_at(index, |(_, value)| value)
    }

    fn entry_at(
        &mut self,
        index: usize,
        select: impl FnOnce((NodeId, NodeId)) -> NodeId,
    ) -> Result<Cursor<'_, 'doc>, Error> {
        let entries = self.editor.entry_ids(&self.handle)?;
        let &entry = entries
            .get(index)
            .ok_or_else(|| self.editor.error("entry index out of bounds"))?;
        let handle = self.editor.handle(select(entry));
        Ok(Cursor::new(self.editor, handle))
    }

    /// Select a string entry for inserting a default if it is absent.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or the work limit is exceeded.
    pub fn entry(&mut self, name: &str) -> Result<Entry<'_, 'doc>, Error> {
        let state = if let Some((_, handle)) = self.editor.string_field(&self.handle, name)? {
            EntryState::Occupied(Cursor::new(self.editor, handle))
        } else {
            EntryState::Vacant {
                cursor: Cursor::new(self.editor, self.handle.clone()),
                name: name.to_owned(),
                len: &mut self.state.len,
            }
        };
        Ok(Entry { state })
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

    /// Set a string entry's YAML value and return a cursor to it.
    /// Existing values keep their anchor and compatible style. Absent entries
    /// append a safely quoted string key.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous, the fragment is invalid, or an
    /// editing limit is exceeded.
    pub fn set_yaml(&mut self, name: &str, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let handle = self.editor.mutate(|editor| {
            if let Some((_, handle)) = editor.string_field(&self.handle, name)? {
                editor.replace_preserving_style(&handle, yaml)?;
                Ok(handle)
            } else {
                editor.append_string_entry(&self.handle, &mut self.state.len, name, yaml)
            }
        })?;
        Ok(Cursor::new(self.editor, handle))
    }

    /// Remove a string entry and its comments, returning whether it existed.
    ///
    /// # Errors
    /// Returns an error if the key is ambiguous or an editing limit is exceeded.
    pub fn remove(&mut self, name: &str) -> Result<bool, Error> {
        self.editor.mutate(|editor| {
            let Some((index, _)) = editor.string_field(&self.handle, name)? else {
                return Ok(false);
            };
            editor.remove_entry(&self.handle, index)?;
            self.state.len -= 1;
            Ok(true)
        })
    }
}

impl<'doc> SequenceMut<'_, 'doc> {
    /// Borrow the item at `index`, or return `None` if out of bounds.
    ///
    /// # Errors
    /// Returns an error if the work limit is exceeded.
    pub fn get(&mut self, index: usize) -> Result<Option<Cursor<'_, 'doc>>, Error> {
        let handle = self.editor.sequence_item(&self.handle, index)?;
        Ok(handle.map(|handle| Cursor::new(self.editor, handle)))
    }

    /// Borrow the item at `index`, requiring it to exist.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or the work limit is exceeded.
    pub fn item(&mut self, index: usize) -> Result<Cursor<'_, 'doc>, Error> {
        self.borrow_cursor().item(index)
    }

    /// Append a YAML fragment and return a cursor to the inserted item.
    ///
    /// # Errors
    /// Returns an error if the fragment is invalid or an editing limit is exceeded.
    pub fn push_yaml(&mut self, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        self.insert_yaml(self.len(), yaml)
    }

    /// Insert a YAML fragment before `index` and return a cursor to the new item.
    /// An index equal to the sequence's length appends the item.
    ///
    /// # Errors
    /// Returns an error if the index exceeds the length, the fragment is invalid,
    /// or an editing limit is exceeded.
    pub fn insert_yaml(&mut self, index: usize, yaml: &str) -> Result<Cursor<'_, 'doc>, Error> {
        let handle = self.editor.insert_item(&self.handle, index, yaml)?;
        self.state.len += 1;
        Ok(Cursor::new(self.editor, handle))
    }

    /// Remove an item and its comments. Later indices shift down by one.
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds or an editing limit is exceeded.
    pub fn remove(&mut self, index: usize) -> Result<(), Error> {
        self.editor.remove_item(&self.handle, index)?;
        self.state.len -= 1;
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
                let handle = cursor
                    .editor
                    .append_string_entry(&cursor.handle, len, &name, yaml)?;
                Ok(Cursor::new(cursor.editor, handle))
            }
        }
    }

    /// Edit the entry's mapping, inserting an empty mapping only if the entry is absent.
    ///
    /// # Errors
    /// Returns an error if the existing value is not a mapping or the edit exceeds a limit.
    pub fn or_mapping(self) -> Result<MappingMut<'a, 'doc>, Error> {
        self.or_insert_yaml("{}")?.into_mapping()
    }

    /// Edit the entry's sequence, inserting an empty sequence only if the entry is absent.
    ///
    /// # Errors
    /// Returns an error if the existing value is not a sequence or the edit exceeds a limit.
    pub fn or_sequence(self) -> Result<SequenceMut<'a, 'doc>, Error> {
        self.or_insert_yaml("[]")?.into_sequence()
    }
}
