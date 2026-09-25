//! Development tooling shared with fuzz targets.

#[path = "../../crates/astral-yaml/tests/support/checks.rs"]
mod checks;

/// Formats semantic events using the YAML Test Suite notation.
///
/// # Panics
/// Panics if passed an internally inconsistent stream.
#[must_use]
pub fn events(stream: &astral_yaml::Stream<'_>) -> String {
    checks::events(stream)
}

/// Checks source boundaries, parent ranges, comments, and arena references.
///
/// # Errors
/// Returns a description of the first broken invariant.
pub fn validate_spans(stream: &astral_yaml::Stream<'_>) -> Result<(), String> {
    checks::validate_spans(stream)
}
