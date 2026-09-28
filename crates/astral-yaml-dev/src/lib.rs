//! Shared helpers for tests, fuzz targets, and benchmarks.

pub mod benchmarks;
mod events;
#[cfg(feature = "saphyr")]
pub mod saphyr;
mod spans;
mod value;

pub use events::events;
pub use spans::validate_spans;
pub use value::{Value, assert_eq_with_nan};

#[cfg(test)]
mod tests;
