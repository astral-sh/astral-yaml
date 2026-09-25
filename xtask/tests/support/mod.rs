use serde::Deserialize;
use yaml_serde::Value;

pub(crate) const FIXTURES: &[(&str, &str)] = &[
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

pub(crate) fn astral(source: &str) -> Vec<Value> {
    let stream = astral_yaml::parse(source).unwrap();
    (0..stream.documents().len())
        .map(|i| astral_yaml_serde::from_document(&stream, i).unwrap())
        .collect()
}

pub(crate) fn reference(source: &str) -> Vec<Value> {
    yaml_serde::Deserializer::from_str(source)
        .map(|doc| Value::deserialize(doc).unwrap())
        .collect()
}
