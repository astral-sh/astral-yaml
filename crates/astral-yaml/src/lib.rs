//! A YAML 1.2 parser that retains source spans and comments.
//!
//! Parsed streams use an arena so aliases do not expand or create recursively
//! owned values. Scalar resolution is separate from syntax parsing.
//! Use `astral-yaml-serde` for source-preserving editing.
//!
//! ```
//! use astral_yaml::NodeKind;
//!
//! let source = "!!str '042' # port\n";
//! let stream = astral_yaml::parse(source)?;
//! let root = stream.node(stream.documents()[0].root).expect("document root");
//! let NodeKind::Scalar(scalar) = &root.kind else {
//!     panic!("expected a scalar");
//! };
//! assert_eq!(scalar.value, "042");
//! assert_eq!(scalar.span.slice(source), Some("'042'"));
//! assert_eq!(root.span.slice(source), Some("!!str '042'"));
//! # Ok::<(), astral_yaml::Error>(())
//! ```

#![warn(missing_docs)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod comments;
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
