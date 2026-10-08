//! Shared helpers for tests, fuzz targets, and benchmarks.

mod annotations;
pub mod benchmarks;
mod edits;
mod events;
pub mod fixtures;
mod spans;
mod value;

pub use annotations::span_snapshot;
pub use edits::edit_diff;
pub use events::events;
pub use indoc::{formatdoc, indoc};
pub use spans::validate_spans;
pub use value::Value;
