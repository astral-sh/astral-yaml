//! Fixed benchmark fixtures and equivalent deserialization workloads.

use crate::Value;
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write};

/// Fixed benchmark inputs, independent of changes to the test corpus.
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
    documents(source)
}

/// Parses and deserializes every document into the requested type.
///
/// # Panics
/// Panics if the input cannot be parsed or deserialized.
#[must_use]
pub fn documents<'a, T: Deserialize<'a>>(source: &'a str) -> Vec<T> {
    let stream = astral_yaml::parse(source).unwrap();

    (0..stream.documents().len())
        .map(|i| astral_yaml_serde::from_document(&stream, i).unwrap())
        .collect()
}

/// Returns a fixed benchmark fixture by name.
///
/// # Panics
/// Panics if the name is not in [`FIXTURES`].
#[must_use]
pub fn fixture(name: &str) -> &'static str {
    FIXTURES
        .iter()
        .find(|&&(candidate, _)| candidate == name)
        .unwrap()
        .1
}

/// Typed output for the generated service configuration benchmark.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "compare", derive(deser::Deserialize))]
#[cfg_attr(feature = "compare", deser(deny_unknown_fields))]
pub struct ServiceConfig {
    services: Vec<Service>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "compare", derive(deser::Deserialize))]
#[cfg_attr(feature = "compare", deser(deny_unknown_fields))]
struct Service {
    name: String,
    image: String,
    enabled: bool,
    replicas: u32,
    ports: Vec<u16>,
    environment: BTreeMap<String, String>,
    command: Option<Vec<String>>,
}

/// Generates a configuration of 128 services for typed deserialization.
#[must_use]
pub fn services_fixture() -> String {
    let mut source = String::from("# Generated service configuration\nservices:\n");

    for index in 0..128 {
        writeln!(source, "  - name: service-{index}").unwrap();
        source.push_str("    image: example/app:v1\n    enabled: true\n    replicas: 3\n");
        source.push_str(
            "    ports: [8080, 8443]\n    environment: {LOG_LEVEL: info, REGION: west}\n",
        );
        source.push_str("    command: [serve, '--verbose'] # process arguments\n");
    }

    source
}
