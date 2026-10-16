//! A YAML 1.2 parser that retains source spans and comments.
//!
//! Parsed streams use an arena so aliases do not expand or create recursively
//! owned values. Scalar resolution is separate from syntax parsing.
//! Enable the `editable` feature for owned, source-preserving editing.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod comments;
#[cfg(feature = "editable")]
pub mod edit;
mod error;
mod model;
mod parser;
pub mod schema;
mod source;

pub use error::Error;
pub use model::{
    CollectionStyle, Comment, Directive, Document, Entry, Node, NodeId, NodeKind, Property, Scalar,
    ScalarStyle, Span, Stream,
};
pub use parser::{ParseOptions, parse, parse_with_options};

pub use source::{Encoding, Source};
