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
