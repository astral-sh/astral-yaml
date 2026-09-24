use std::{path::Path, process::Command};

#[test]
fn official_yaml_test_suite() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_astral-yaml-dev"))
        .arg("conformance")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("402/402 cases passed"));
}

#[test]
fn fuzz_regressions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/regressions");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let source = std::fs::read_to_string(&path).unwrap();
            let error =
                astral_yaml::parse(&source).expect_err("malformed regression must be rejected");
            assert!(error.span.slice(&source).is_some(), "{}", path.display());
        }
    }
}

#[test]
fn every_corpus_prefix_has_valid_spans() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/yaml-test-suite");
    let mut directories = vec![root];
    let mut inputs = 0;
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else if path.file_name().is_some_and(|name| name == "in.yaml") {
                inputs += 1;
                let source = std::fs::read_to_string(&path).unwrap();
                for end in source.char_indices().map(|(i, _)| i).chain([source.len()]) {
                    let prefix = &source[..end];
                    match astral_yaml::parse(prefix) {
                        Ok(stream) => {
                            astral_yaml_dev::validate_spans(&stream).unwrap_or_else(|e| {
                                panic!("{} truncated at {end}: {e}", path.display())
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
    }
    assert_eq!(inputs, 402);
}

#[test]
fn events_preserve_escaped_controls_and_opaque_tag_bytes() {
    let stream = astral_yaml::parse("!<tag:example,%FF> \"\\0\\b\\t\\r\\n\\\\\"").unwrap();
    assert!(
        astral_yaml_dev::events(&stream).contains("=VAL <tag:example,%FF> \"\\0\\b\\t\\r\\n\\\\\n")
    );
}
