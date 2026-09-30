//! Pinned real-world inputs shared by parser and construction tests.

use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::Value;

#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub file: String,
    pub valid_yaml_1_2: Option<bool>,
    pub serde_error: Option<String>,
    pub python_error: Option<String>,
    pub saphyr_error: Option<String>,
    #[serde(default)]
    pub leading_zero_integers: usize,
    #[serde(default)]
    pub saphyr_null_documents: usize,
}

impl Fixture {
    /// Reads the fixture's unmodified UTF-8 input.
    ///
    /// # Panics
    /// Panics if the fixture cannot be read as UTF-8.
    #[must_use]
    pub fn source(&self) -> String {
        fs::read_to_string(root().join(&self.file)).unwrap()
    }

    #[must_use]
    pub fn valid_syntax(&self) -> bool {
        self.valid_yaml_1_2.unwrap_or(true)
    }
}

#[must_use]
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/real-world")
}

/// Loads the checked-in corpus manifest.
///
/// # Panics
/// Panics if the manifest does not match [`Fixture`].
#[must_use]
pub fn manifest() -> Vec<Fixture> {
    serde_json::from_str(include_str!("../../../fixtures/real-world/manifest.json")).unwrap()
}

/// Normalize YAML line breaks before exercising an alternate encoding.
#[must_use]
pub fn with_line_breaks(source: &str, line_break: &str) -> String {
    source
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', line_break)
}

/// Counts reviewed scalar-resolution differences in otherwise equal values.
///
/// # Panics
/// Panics on a structure difference or a scalar difference rejected by `accepts`.
pub fn scalar_differences(
    actual: &Value,
    reference: &Value,
    accepts: fn(&Value, &Value) -> bool,
) -> usize {
    if actual == reference {
        return 0;
    }

    match (actual, reference) {
        (Value::Sequence(actual), Value::Sequence(reference)) => {
            assert_eq!(actual.len(), reference.len());
            actual
                .iter()
                .zip(reference)
                .map(|(actual, reference)| scalar_differences(actual, reference, accepts))
                .sum()
        }
        (Value::Mapping(actual), Value::Mapping(reference)) => {
            assert_eq!(actual.len(), reference.len());
            actual
                .iter()
                .zip(reference)
                .map(|((key, value), (reference_key, reference_value))| {
                    scalar_differences(key, reference_key, accepts)
                        + scalar_differences(value, reference_value, accepts)
                })
                .sum()
        }
        _ => {
            assert!(accepts(actual, reference), "{actual:?} != {reference:?}");
            1
        }
    }
}
