//! Fixed benchmark fixtures and equivalent deserialization workloads.

use serde::Deserialize;
use yaml_serde::Value;

/// Historical benchmark inputs, kept fixed as the test corpus grows.
pub const FIXTURES: &[(&str, &str)] = &[
    (
        "uv-ci",
        include_str!("../../../fixtures/real-world/uv-ci.yaml"),
    ),
    (
        "kubernetes-guestbook",
        include_str!("../../../fixtures/real-world/kubernetes-guestbook.yaml"),
    ),
    (
        "openapi-petstore",
        include_str!("../../../fixtures/real-world/openapi-petstore.yaml"),
    ),
];

/// Deserializes every document with astral-yaml.
///
/// # Panics
/// Panics if the input cannot be deserialized.
#[must_use]
pub fn astral(source: &str) -> Vec<Value> {
    let stream = astral_yaml::parse(source).unwrap();

    (0..stream.documents().len())
        .map(|i| astral_yaml_serde::from_document(&stream, i).unwrap())
        .collect()
}

/// Deserializes every document with `yaml_serde`.
///
/// # Panics
/// Panics if the input cannot be deserialized.
#[must_use]
pub fn reference(source: &str) -> Vec<Value> {
    yaml_serde::Deserializer::from_str(source)
        .map(|doc| Value::deserialize(doc).unwrap())
        .collect()
}
