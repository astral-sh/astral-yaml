use std::fs;
use std::path::PathBuf;

use serde_json::Value;

// JSON cannot represent YAML's non-core tags. The deserializer rejects them.
const CASES_WITH_NON_CORE_TAGS: &[&str] = &[
    "2XXW", "565N", "5TYM", "6CK3", "6WLZ", "7FWL", "9WXW", "C4HZ", "CC74", "CUP7", "J7PZ", "M5C3",
    "P76L", "UGM3", "Z67P", "Z9M4",
];

fn corpus_paths() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/yaml-test-suite");
    let mut directories = vec![root];
    let mut inputs = Vec::new();

    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                directories.push(path);
            } else if path.file_name().is_some_and(|name| name == "in.yaml") {
                inputs.push(path);
            }
        }
    }

    inputs.sort();
    assert_eq!(inputs.len(), 402);
    inputs
}

#[test]
fn yaml_test_suite_json_values() {
    let mut count = 0;
    let mut rejected = Vec::new();

    for path in corpus_paths() {
        let directory = path.parent().unwrap();
        let json_path = directory.join("in.json");

        if directory.join("error").exists() || !json_path.exists() {
            continue;
        }

        let case = directory.file_name().unwrap().to_str().unwrap();
        let source = fs::read_to_string(&path).unwrap();
        let json = fs::read_to_string(json_path).unwrap();
        let expected = serde_json::Deserializer::from_str(&json)
            .into_iter::<Value>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let stream = astral_yaml::parse(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let actual: Result<Vec<Value>, _> = (0..stream.documents().len())
            .map(|index| astral_yaml_serde::from_document(&stream, index))
            .collect();

        if CASES_WITH_NON_CORE_TAGS.contains(&case) {
            let error = actual.expect_err(&format!("{} must reject its tag", path.display()));
            assert_eq!(
                error.message,
                "non-core YAML tags require Tagged<T> or with::yaml_tag",
                "{}",
                path.display()
            );
            rejected.push(case.to_owned());
        } else {
            assert_eq!(
                actual.unwrap_or_else(|error| panic!("{}: {error}", path.display())),
                expected,
                "{}",
                path.display()
            );
        }

        count += 1;
    }

    assert_eq!(count, 279);
    assert_eq!(rejected, CASES_WITH_NON_CORE_TAGS);
}

#[test]
fn yaml_test_suite_values_round_trip() {
    use astral_yaml_dev::Value;

    let mut documents = 0;
    let mut tagged = Vec::new();
    let mut duplicate_keys = Vec::new();

    for path in corpus_paths() {
        let directory = path.parent().unwrap();
        if directory.join("error").exists() {
            continue;
        }

        let case = directory.file_name().unwrap().to_str().unwrap();
        let source = fs::read_to_string(&path).unwrap();
        let stream = astral_yaml::parse(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let values = (0..stream.documents().len())
            .map(|index| astral_yaml_serde::from_document::<Value>(&stream, index))
            .collect::<Result<Vec<_>, _>>();

        let values = match values {
            Ok(values) => values,
            Err(error) if CASES_WITH_NON_CORE_TAGS.contains(&case) => {
                assert_eq!(
                    error.message,
                    "non-core YAML tags require Tagged<T> or with::yaml_tag",
                    "{}",
                    path.display()
                );
                tagged.push(case.to_owned());
                continue;
            }
            // https://yaml.org/spec/1.2.2/#3213-node-comparison
            Err(error) if ["2JQS", "X38W"].contains(&case) => {
                assert_eq!(error.message, "duplicate mapping key", "{}", path.display());
                duplicate_keys.push(case.to_owned());
                continue;
            }
            Err(error) => panic!("{}: {error}", path.display()),
        };

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let actual: Value = astral_yaml_serde::from_str(&yaml)
                .unwrap_or_else(|error| panic!("{}: {error}\n{yaml}", path.display()));
            assert_eq!(actual, value, "{}\n{yaml}", path.display());
            documents += 1;
        }
    }

    assert_eq!(documents, 313);
    assert_eq!(tagged, CASES_WITH_NON_CORE_TAGS);
    assert_eq!(duplicate_keys, ["2JQS", "X38W"]);
}
