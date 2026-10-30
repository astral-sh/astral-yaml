//! Borrowed syntax used to compare trees and render replacements.

use super::source::Source;
use super::{DocumentSyntax, Node};
use astral_yaml::{NodeId, Span};

/// A borrowed syntax tree, independent of its source text.
#[derive(Clone, Copy)]
pub(super) struct Syntax<'syntax, Location> {
    pub(super) nodes: &'syntax [Node<Location>],
    pub(super) root: NodeId,
}

impl<'syntax> From<&'syntax DocumentSyntax> for Syntax<'syntax, Span> {
    fn from(document: &'syntax DocumentSyntax) -> Self {
        (&document.source).into()
    }
}

/// Syntax and source text used to render a replacement.
/// Node and comment spans refer to this source. Generated syntax has no span.
#[derive(Clone, Copy)]
pub(super) struct Emission<'syntax, Location> {
    pub(super) syntax: Syntax<'syntax, Location>,
    pub(super) source: &'syntax Source,
}

impl<'syntax> From<&'syntax DocumentSyntax> for Emission<'syntax, Span> {
    fn from(document: &'syntax DocumentSyntax) -> Self {
        document.source.emission(document.into())
    }
}
