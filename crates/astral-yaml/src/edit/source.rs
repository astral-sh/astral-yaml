//! Original source and its index during render planning. Generated document
//! boundaries may relocate the index without reparsing it. This is deliberately
//! a different type from a committed, parsed `EditableDocument`.

use super::{
    EditableDocument, Node,
    draft::Draft,
    view::{Emission, Syntax},
};
use crate::{NodeId, ParseOptions, Span};

#[derive(Clone, Debug)]
pub(super) struct Source {
    pub(super) text: String,
    pub(super) nodes: Vec<Node>,
    pub(super) root: NodeId,
    pub(super) options: ParseOptions,
    pub(super) start_marker: Option<Span>,
    pub(super) end_marker: Option<Span>,
    pub(super) extra_end_markers: Vec<Span>,
}

impl From<&EditableDocument> for Source {
    fn from(document: &EditableDocument) -> Self {
        let EditableDocument {
            source,
            nodes,
            root,
            options,
            start_marker,
            end_marker,
            extra_end_markers,
        } = document.clone();
        Self {
            text: source,
            nodes,
            root,
            options,
            start_marker,
            end_marker,
            extra_end_markers,
        }
    }
}

impl<'a> From<&'a Source> for Syntax<'a, Span> {
    fn from(source: &'a Source) -> Self {
        Self {
            nodes: &source.nodes,
            root: source.root,
        }
    }
}

impl Source {
    pub(super) fn emission<'a>(&'a self, draft: &'a Draft) -> Emission<'a, Span> {
        Emission {
            nodes: &draft.nodes,
            root: draft.root,
            source: &self.text,
            options: self.options,
            end_marker: self.end_marker,
        }
    }
}
