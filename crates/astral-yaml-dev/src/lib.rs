//! Shared helpers for tests, fuzz targets, and benchmarks.

pub mod benchmarks;
mod events;
mod spans;

pub use events::events;
pub use spans::validate_spans;

#[cfg(test)]
mod tests;
