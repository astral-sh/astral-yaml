//! Shared helpers for tests, fuzz targets, and benchmarks.

pub mod benchmarks;
mod events;
#[cfg(feature = "saphyr")]
pub mod saphyr;
mod spans;
mod value;

pub use events::events;
pub use spans::validate_spans;
pub use value::Value;

#[cfg(test)]
mod tests;
