mod support;

use std::{fs, path::PathBuf, process::Command};

use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    file: String,
    valid_yaml_1_2: Option<bool>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/real-world")
}

fn manifest() -> Vec<Fixture> {
    serde_json::from_str(include_str!("../../fixtures/real-world/manifest.json")).unwrap()
}

fn scalar_spans(stream: &astral_yaml::Stream<'_>) -> Vec<(usize, usize)> {
    let mut spans: Vec<_> = stream
        .nodes()
        .iter()
        .filter_map(|node| {
            if let astral_yaml::NodeKind::Scalar(scalar) = &node.kind {
                (!scalar.span.is_empty()).then_some((scalar.span.start(), scalar.span.end()))
            } else {
                None
            }
        })
        .collect();
    spans.sort_unstable();

    spans
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

#[test]
#[ignore = "requires uv and pinned PyYAML; run by scripts/check.sh"]
fn syntax_and_scalar_spans_match_pyyaml() {
    #[derive(Deserialize)]
    struct Oracle {
        file: String,
        events: String,
        scalars: Vec<(usize, usize)>,
    }

    let output = Command::new("uv")
        .args(["run", "--script"])
        .arg(root().join("../../scripts/real-world-oracle.py"))
        .output()
        .expect("run the independent oracle with uv");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let oracles: Vec<Oracle> = serde_json::from_slice(&output.stdout).unwrap();
    let fixtures = manifest();
    assert_eq!(oracles.len(), fixtures.len());

    for (oracle, fixture) in oracles.iter().zip(fixtures) {
        assert_eq!(oracle.file, fixture.file);
        let source = fs::read_to_string(root().join(&fixture.file)).unwrap();

        if !fixture.valid_yaml_1_2.unwrap_or(true) {
            // PyYAML accepts this input; the inline diagnostic pins our rejection.
            assert!(astral_yaml::parse(&source).is_err());
            continue;
        }

        let stream = astral_yaml::parse(&source).unwrap();
        xtask::validate_spans(&stream).unwrap();
        assert_eq!(xtask::events(&stream), oracle.events, "{}", fixture.file);

        for line_break in ["\r\n", "\r"] {
            let converted = source.replace('\n', line_break);
            let converted_stream = astral_yaml::parse(&converted).unwrap();
            xtask::validate_spans(&converted_stream).unwrap();
            assert_eq!(
                xtask::events(&converted_stream),
                oracle.events,
                "{}",
                fixture.file
            );
        }

        assert_eq!(scalar_spans(&stream), oracle.scalars, "{}", fixture.file);
    }
}
