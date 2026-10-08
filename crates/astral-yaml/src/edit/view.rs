//! Borrowed inputs for syntax comparison and emission. Neither view claims that
//! the syntax has been parsed from the source used as the rendering context.

use super::{EditableDocument, Node};
use crate::{NodeId, ParseOptions, Span};

/// The tree's meaning and presentation, independent of source text.
#[derive(Clone, Copy)]
pub(super) struct Syntax<'a, Location> {
    pub(super) nodes: &'a [Node<Location>],
    pub(super) root: NodeId,
}

impl<'a> From<&'a EditableDocument> for Syntax<'a, Span> {
    fn from(document: &'a EditableDocument) -> Self {
        let document = &document.source;
        Self {
            nodes: &document.nodes,
            root: document.root,
        }
    }
}

/// Syntax to emit and the source into which it will be spliced. Locations refer
/// to this source; generated syntax has no source location.
#[derive(Clone, Copy)]
pub(super) struct Emission<'a, Location> {
    pub(super) nodes: &'a [Node<Location>],
    pub(super) root: NodeId,
    pub(super) source: &'a str,
    pub(super) options: ParseOptions,
    pub(super) end_marker: Option<Span>,
}

impl<'a> From<&'a EditableDocument> for Emission<'a, Span> {
    fn from(document: &'a EditableDocument) -> Self {
        let document = &document.source;
        Self {
            nodes: &document.nodes,
            root: document.root,
            source: &document.text,
            options: document.options,
            end_marker: document.end_marker,
        }
    }
}
