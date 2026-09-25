//! Serde support for YAML 1.2, with source-located errors and bounded alias expansion.
mod comments;
mod de;
mod error;
mod ser;
mod spanned;

pub use comments::{AttachedComment, CommentPlacement};
pub use de::{from_document, from_str, from_stream};
pub use error::Error;
pub use ser::to_string;
pub use spanned::Spanned;
