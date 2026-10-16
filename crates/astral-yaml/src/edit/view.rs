//! Borrowed syntax used to compare trees and render replacements.

use super::{EditableDocument, Node, source::Source};
use crate::{NodeId, Span};

/// A borrowed syntax tree, independent of its source text.
#[derive(Clone, Copy)]
pub(super) struct Syntax<'a, Location> {
    pub(super) nodes: &'a [Node<Location>],
    pub(super) root: NodeId,
}

impl<'a> From<&'a EditableDocument> for Syntax<'a, Span> {
    fn from(document: &'a EditableDocument) -> Self {
        (&document.source).into()
    }
}

/// Syntax and source text used to render a replacement.
/// Node and comment spans refer to this source. Generated syntax has no span.
#[derive(Clone, Copy)]
pub(super) struct Emission<'a, Location> {
    pub(super) syntax: Syntax<'a, Location>,
    pub(super) source: &'a Source,
}

impl<'a> From<&'a EditableDocument> for Emission<'a, Span> {
    fn from(document: &'a EditableDocument) -> Self {
        document.source.emission(document.into())
    }
}
