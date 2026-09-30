//! Pinned real-world inputs shared by parser and construction tests.

use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub file: String,
    pub valid_yaml_1_2: Option<bool>,
    pub serde_error: Option<String>,
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
