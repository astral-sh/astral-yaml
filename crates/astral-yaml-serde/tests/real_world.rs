use std::{fs, path::PathBuf};

use serde::Deserialize;
use yaml_serde::Value;

#[derive(Deserialize)]
struct Fixture {
    file: String,
    valid_yaml_1_2: Option<bool>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/real-world")
}

fn manifest() -> Vec<Fixture> {
    serde_json::from_str(include_str!("../../../fixtures/real-world/manifest.json")).unwrap()
}

fn astral(source: &str) -> Vec<Value> {
    let stream = astral_yaml::parse(source).unwrap();
    (0..stream.documents().len())
        .map(|index| astral_yaml_serde::from_document(&stream, index).unwrap())
        .collect()
}

fn reference(source: &str) -> Vec<Value> {
    yaml_serde::Deserializer::from_str(source)
        .map(|document| Value::deserialize(document).unwrap())
        .collect()
}

#[test]
fn real_world_values_comments_and_serialization() {
    for fixture in manifest() {
        let name = fixture.file;
        let source = fs::read_to_string(root().join(&name)).unwrap();

        if !fixture.valid_yaml_1_2.unwrap_or(true) {
            // The reference reader accepts Sentry's under-indented flow delimiter.
            assert!(astral_yaml_serde::from_str::<Value>(&source).is_err());
            assert!(!reference(&source).is_empty());
            continue;
        }

        let stream = astral_yaml::parse(&source).unwrap();
        let values = astral(&source);
        assert_eq!(values, reference(&source), "{name}");

        if stream.documents().len() == 1 {
            let wrapped: astral_yaml_serde::Spanned<'_, Value> =
                astral_yaml_serde::from_document(&stream, 0).unwrap();
            let comments: Vec<_> = wrapped
                .comments
                .iter()
                .map(|attached| attached.comment.clone())
                .collect();
            assert_eq!(comments, stream.comments(), "root comments: {name}");
        }

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<Value>(&yaml).unwrap(),
                value,
                "{name}"
            );
            assert_eq!(
                yaml_serde::from_str::<Value>(&yaml).unwrap(),
                value,
                "reference reader: {name}"
            );
        }

        assert!(
            astral_yaml_serde::from_document::<Value>(&stream, stream.documents().len()).is_err()
        );
    }
}

#[test]
fn flow_key_and_value_separation_matches_reference() {
    for source in ["{a: [b]}", "{'a':[b]}", "{[]:value}"] {
        assert_eq!(astral(source), reference(source), "{source}");
    }
}
