use std::{fs, path::Path};

use astral_yaml_dev::benchmarks::{astral, reference};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    file: String,
    valid_yaml_1_2: Option<bool>,
}

#[test]
fn real_world_values_and_serialization_match_reference() {
    let fixtures: Vec<Fixture> = serde_json::from_str(include_str!(
        "../../../../fixtures/real-world/manifest.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/real-world");

    for fixture in fixtures {
        let source = fs::read_to_string(root.join(&fixture.file)).unwrap();

        if !fixture.valid_yaml_1_2.unwrap_or(true) {
            // LibYAML accepts Sentry's under-indented flow delimiter.
            assert!(astral_yaml::parse(&source).is_err());
            assert!(!reference(&source).is_empty());
            continue;
        }

        let values = astral(&source);
        assert_eq!(values, reference(&source), "{}", fixture.file);

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(reference(&yaml), vec![value], "{}", fixture.file);
        }
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn flow_key_and_value_separation_matches_reference() {
    for source in ["{a: [b]}", "{'a':[b]}", "{[]:value}"] {
        assert_eq!(astral(source), reference(source), "{source}");
    }
}
