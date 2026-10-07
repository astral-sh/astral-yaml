//! Serde support for YAML 1.2, with source-located errors and bounded alias expansion.
//! Enable `editable` for source-preserving editing with a typed view.

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
pub use editable::Editable;
pub use error::Error;
pub use ser::to_string;
pub use spanned::Spanned;
pub use tagged::Tagged;
