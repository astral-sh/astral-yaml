//! Owned YAML syntax for source-preserving editing.

mod collection;
mod document_syntax;
mod draft;
mod emit;
mod framing;
mod layout;
mod lines;
mod node;
mod plan;
mod scope;
mod source;
mod splice;
#[cfg(test)]
#[path = "../tests/syntax/mod.rs"]
mod tests;
mod transaction;
mod view;

pub(crate) use collection::{Collection, Mapping, Sequence};
pub use document_syntax::DocumentSyntax;
pub use node::{Comment, CommentPosition, Node, NodeKind};
pub(crate) use transaction::{Editor, ValueRef};

use astral_yaml::{Budget, Error, Span};

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end).expect("ordered source range")
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}

/// Charge the same work budget while editing the draft and planning its source rewrites.
fn charge_work(work: &Budget, amount: usize, span: Span) -> Result<(), Error> {
    work.try_charge(amount).ok_or(Error {
        span,
        message: "editing resource limit exceeded",
    })
}

fn single_document<'stream, 'source>(
    stream: &'stream astral_yaml::Stream<'source>,
) -> Result<&'stream astral_yaml::Document<'source>, Error> {
    match stream.documents() {
        [document] => Ok(document),
        _ => Err(Error {
            span: stream.span(),
            message: "expected exactly one document",
        }),
    }
}
