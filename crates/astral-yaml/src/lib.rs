//! A safe, span-aware YAML 1.2 parser.
//!
//! Parsed streams use an arena so aliases do not expand or create recursively
//! owned values. Scalar resolution is separate from syntax parsing.

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
