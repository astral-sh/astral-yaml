//! Serialize and deserialize YAML 1.2 with Serde.
//! Enable `editable` for YAML editing.

#![warn(missing_docs)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

mod commented;
mod comments;
mod de;
#[cfg(feature = "editable")]
mod editable;
mod error;
mod schema;
mod ser;
mod spanned;
mod tagged;

#[cfg(feature = "editable")]
pub mod edit;
pub mod with;

pub use commented::Commented;
pub use comments::{AttachedComment, CommentPlacement};
pub use de::{from_document, from_str, from_stream};
#[cfg(feature = "editable")]
pub use editable::{Editable, EditableYaml};
pub use error::Error;
pub use ser::to_string;
pub use spanned::Spanned;
pub use tagged::Tagged;
