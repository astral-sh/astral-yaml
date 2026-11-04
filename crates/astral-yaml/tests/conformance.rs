use std::fs;
use std::path::{Path, PathBuf};

use astral_yaml_dev::{events, validate_spans};

fn corpus_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/yaml-test-suite");
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

// The upstream corpus supplies the expected events and rejections.
// https://github.com/yaml/yaml-test-suite/tree/6ad3d2c62885d82fc349026c136ef560838fdf3d
// https://yaml.org/spec/1.2.2/#rule-l-yaml-stream
#[test]
fn official_yaml_test_suite() {
    let mut valid = 0;
    let mut invalid = 0;

    for path in corpus_paths() {
        let directory = path.parent().unwrap();
        let source = fs::read_to_string(&path).unwrap();

        if directory.join("error").exists() {
            let error = astral_yaml::parse(&source)
                .expect_err(&format!("{} must be rejected", path.display()));
            assert!(error.span.slice(&source).is_some(), "{}", path.display());
            invalid += 1;
            continue;
        }

        let stream = astral_yaml::parse(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        validate_spans(&stream).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let expected = fs::read_to_string(directory.join("test.event")).unwrap();
        assert_eq!(events(&stream), expected, "{}", path.display());
        valid += 1;
    }

    assert_eq!((valid, invalid), (308, 94));
}

// The plain-scalar regressions exercise colons and flow collection indicators.
// https://yaml.org/spec/1.2.2/#733-plain-style
#[test]
fn fuzz_regressions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/regressions");

    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let source = fs::read_to_string(&path).unwrap();
            let error =
                astral_yaml::parse(&source).expect_err("malformed regression must be rejected");
            assert!(error.span.slice(&source).is_some(), "{}", path.display());
        }
    }
}

#[test]
fn every_corpus_prefix_has_valid_spans() {
    for path in corpus_paths() {
        let source = fs::read_to_string(&path).unwrap();

        for end in source
            .char_indices()
            .map(|(index, _)| index)
            .chain([source.len()])
        {
            let prefix = &source[..end];

            match astral_yaml::parse(prefix) {
                Ok(stream) => {
                    validate_spans(&stream).unwrap_or_else(|error| {
                        panic!("{} truncated at {end}: {error}", path.display())
                    });
                }
                Err(error) => assert!(
                    error.span.slice(prefix).is_some(),
                    "{} truncated at {end}: {error}",
                    path.display()
                ),
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#56-miscellaneous-characters
// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test]
fn events_preserve_escaped_controls_and_opaque_tag_bytes() {
    let stream = astral_yaml::parse("!<tag:example,%FF> \"\\0\\b\\t\\r\\n\\\\\"").unwrap();
    assert!(events(&stream).contains("=VAL <tag:example,%FF> \"\\0\\b\\t\\r\\n\\\\\n"));
}
