//! Borrowed syntax used to compare trees and render replacements.

use super::{EditableDocument, Node};
use crate::{NodeId, ParseOptions, Span};

/// A borrowed syntax tree, independent of its source text.
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

/// Syntax and source text used to render a replacement.
/// Node and comment spans refer to this source. Generated syntax has no span.
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
