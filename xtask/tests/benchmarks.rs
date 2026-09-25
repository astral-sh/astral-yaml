mod support;

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/real-world")
}

#[test]
fn benchmark_inputs_match_reference() {
    // Keep the historical benchmark inputs fixed while the test corpus grows.
    for &(name, source) in support::FIXTURES {
        assert_eq!(
            fs::read_to_string(root().join(format!("{name}.yaml"))).unwrap(),
            source
        );
        assert_eq!(
            support::astral(source),
            support::reference(source),
            "{name}"
        );
    }
}
