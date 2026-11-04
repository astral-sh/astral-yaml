use super::draft::{Draft, ReplacementStyle};
use super::layout::Layout;
use super::plan::{Changes, Plan};
use super::{DocumentSyntax, Node, NodeKind};
use crate::ser;
use astral_yaml::comments::CommentPlacement;
use astral_yaml::schema::{self, ScalarType};
use astral_yaml::{CollectionStyle, Error, NodeId, ScalarStyle, Span};
use serde::Serialize;

#[cfg(test)]
mod tests;

/// A read-only view of a selected value, including pending edits.
///
/// Start at [`Self::root`] and follow node IDs in the syntax, including alias
/// targets outside the selected subtree. Source spans refer to the original
/// document and are absent on newly inserted nodes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ValueRef<'draft> {
    draft: &'draft Draft,
    root: NodeId,
}

impl<'draft> ValueRef<'draft> {
    /// Return the ID of the selected node.
    #[must_use]
    pub(crate) fn root(self) -> NodeId {
        self.root
    }

    /// Read a node by its ID in this transaction.
    ///
    /// # Errors
    /// Returns an error if the node was removed or the work limit is exceeded.
    pub(crate) fn node(self, id: NodeId) -> Result<&'draft Node<Option<Span>>, Error> {
        self.draft.charge(1)?;
        if !self.draft.contains(id) {
            return Err(self.draft.error("node is no longer in the document"));
        }
        Ok(self.draft.node(id))
    }
}

#[derive(Debug)]
pub(crate) struct Editor<'doc> {
    original: &'doc DocumentSyntax,
    draft: Draft,
}

impl<'doc> Editor<'doc> {
    pub(crate) fn new(original: &'doc DocumentSyntax) -> Self {
        Self {
            original,
            draft: Draft::from(original),
        }
    }

    pub(crate) fn root(&self) -> NodeId {
        self.draft.root()
    }

    pub(crate) fn value(&self, root: NodeId) -> ValueRef<'_> {
        ValueRef {
            draft: &self.draft,
            root,
        }
    }

    pub(crate) fn node(&self, id: NodeId) -> Result<&Node<Option<Span>>, Error> {
        self.value(id).node(id)
    }

    pub(crate) fn error(&self, message: &'static str) -> Error {
        self.draft.error(message)
    }

    fn mutate<R>(
        &mut self,
        id: NodeId,
        operation: impl FnOnce(&mut Draft, NodeId) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.node(id)?;
        operation(&mut self.draft, id)
    }

    pub(crate) fn item_ids(&self, id: NodeId) -> Result<&[NodeId], Error> {
        match self.node(id)?.kind() {
            NodeKind::Sequence { items, .. } => Ok(items),
            _ => Err(self.error("expected a sequence")),
        }
    }

    pub(crate) fn entry_ids(&self, id: NodeId) -> Result<&[(NodeId, NodeId)], Error> {
        match self.node(id)?.kind() {
            NodeKind::Mapping { entries, .. } => Ok(entries),
            _ => Err(self.error("expected a mapping")),
        }
    }

    pub(crate) fn string_field(
        &self,
        mapping: NodeId,
        name: &str,
    ) -> Result<Option<(usize, NodeId)>, Error> {
        let entries = self.entry_ids(mapping)?;
        self.draft.charge(entries.len())?;
        let mut found = None;
        for (index, &(key, value)) in entries.iter().enumerate() {
            if string_key(self.draft.node(key)) == Some(name) {
                if found.is_some() {
                    return Err(self.error("ambiguous mapping key"));
                }
                found = Some((index, value));
            }
        }
        Ok(found)
    }

    pub(crate) fn serialize(&self, value: impl Serialize) -> Result<String, crate::Error> {
        let (remaining, span) = self.draft.byte_budget();
        ser::to_string_with_limit(&value, remaining, span)
    }

    pub(crate) fn append_string_entry<S: AsRef<str>>(
        &mut self,
        id: NodeId,
        len: &mut usize,
        name: &str,
        serialize: impl FnOnce(usize, Span) -> Result<S, crate::Error>,
    ) -> Result<NodeId, crate::Error> {
        let (remaining, span) = self.draft.byte_budget();
        let key = ser::quote_with_limit(name, remaining, span)?;
        let yaml = serialize(remaining - key.len(), span)?;
        let (_, value) = self.insert_entry(id, *len, &key, yaml.as_ref())?;
        *len += 1;
        Ok(value)
    }

    #[cfg(test)]
    pub(crate) fn replace(&mut self, id: NodeId, yaml: &str) -> Result<(), Error> {
        self.mutate(id, |draft, id| {
            draft.replace(id, yaml, ReplacementStyle::FromValue)
        })
    }

    pub(crate) fn replace_preserving_style(&mut self, id: NodeId, yaml: &str) -> Result<(), Error> {
        self.mutate(id, |draft, id| {
            draft.replace(id, yaml, ReplacementStyle::Preserve)
        })
    }

    pub(crate) fn insert_item(
        &mut self,
        id: NodeId,
        index: usize,
        yaml: &str,
    ) -> Result<NodeId, Error> {
        self.mutate(id, |draft, id| draft.insert_item(id, index, yaml))
    }

    pub(crate) fn remove_item(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.remove_item(id, index))
    }

    pub(crate) fn insert_entry(
        &mut self,
        id: NodeId,
        index: usize,
        key: &str,
        value: &str,
    ) -> Result<(NodeId, NodeId), Error> {
        self.mutate(id, |draft, id| draft.insert_entry(id, index, key, value))
    }

    pub(crate) fn remove_entry(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.remove_entry(id, index))
    }

    pub(crate) fn set_scalar_style(&mut self, id: NodeId, style: ScalarStyle) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.set_scalar_style(id, style))
    }

    pub(crate) fn set_collection_style(
        &mut self,
        id: NodeId,
        style: CollectionStyle,
    ) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.set_collection_style(id, style))
    }

    pub(crate) fn insert_comment(
        &mut self,
        id: NodeId,
        index: usize,
        text: &str,
        placement: CommentPlacement,
    ) -> Result<(), Error> {
        self.mutate(id, |draft, id| {
            draft.insert_comment(id, index, text, placement)
        })
    }

    pub(crate) fn replace_comment(
        &mut self,
        id: NodeId,
        index: usize,
        text: &str,
    ) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.replace_comment(id, index, text))
    }

    pub(crate) fn remove_comment(&mut self, id: NodeId, index: usize) -> Result<(), Error> {
        self.mutate(id, |draft, id| draft.remove_comment(id, index))
    }

    pub(crate) fn finish(self) -> Result<DocumentSyntax, Error> {
        self.finish_rewriting([])
    }

    pub(super) fn finish_rewriting(
        self,
        rewrites: impl IntoIterator<Item = NodeId>,
    ) -> Result<DocumentSyntax, Error> {
        let mut candidate = self.draft.freeze()?;
        let syntax = Layout::new(&candidate);
        let original = &self.original.source;
        let mut changes = Changes::between(original, &candidate, &syntax);
        changes.roots.extend(rewrites);
        if changes.is_empty() {
            return candidate.commit(&original.text);
        }

        candidate.restore_comment_spans(&original.nodes);
        let framed = super::framing::prepare(original, (&candidate).into(), &syntax);
        let (source, planned) = framed.as_ref().map_or_else(
            || (original, (&candidate).into()),
            |framed| (&framed.source, framed.syntax()),
        );
        let output = Plan {
            original: source,
            candidate: planned,
            parents: candidate.parents(),
            changes,
            work: candidate.work(),
        }
        .render(&syntax)?;
        candidate.commit(&output)
    }
}

fn string_key<Location>(node: &Node<Location>) -> Option<&str> {
    let NodeKind::Scalar { value, style } = node.kind() else {
        return None;
    };
    let kind = match node.tag() {
        Some("!" | "tag:yaml.org,2002:str") => ScalarType::String,
        Some(_) => return None,
        None if *style == ScalarStyle::Plain => schema::classify(value),
        None => ScalarType::String,
    };
    (kind == ScalarType::String).then_some(value)
}
