#[path = "support/checks.rs"]
mod checks;
#[path = "real_world/display.rs"]
mod display;
#[path = "real_world/fixtures/mod.rs"]
mod fixtures;

use std::{fs, path::PathBuf, process::Command};

use serde::Deserialize;

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

fn assert_alias_targets(stream: &astral_yaml::Stream<'_>) {
    for node in stream.nodes() {
        let astral_yaml::NodeKind::Alias { name, target } = &node.kind else {
            continue;
        };

        let expected = stream
            .nodes()
            .iter()
            .filter(|candidate| {
                candidate.span.start() < node.span.start()
                    && candidate
                        .anchor
                        .as_ref()
                        .is_some_and(|anchor| anchor.value == *name)
            })
            .max_by_key(|candidate| candidate.span.start())
            .unwrap();

        assert_eq!(stream.node(*target).unwrap(), expected, "alias {name}");
        assert_eq!(
            node.span.slice(stream.source()),
            Some(format!("*{name}").as_str())
        );
    }
}

fn check_fixture(name: &str) -> String {
    let source = fs::read_to_string(root().join(format!("{name}.yaml"))).unwrap();
    let stream = astral_yaml::parse(&source).unwrap();
    assert_alias_targets(&stream);
    let tree = display::tree(&stream);

    for line_break in ["\r\n", "\r"] {
        let converted = source.replace('\n', line_break);
        let converted_stream = astral_yaml::parse(&converted).unwrap();
        assert_eq!(
            display::tree(&converted_stream),
            tree,
            "line endings: {name}"
        );
    }

    tree
}

fn check_rejection(name: &str) -> String {
    let source = fs::read_to_string(root().join(format!("{name}.yaml"))).unwrap();
    let error = astral_yaml::parse(&source).unwrap_err();

    display::diagnostic(name, &source, &error)
}

#[test]
fn every_fixture_has_an_inline_snapshot() {
    let mut expected: Vec<_> = manifest().into_iter().map(|fixture| fixture.file).collect();
    expected.sort();
    let mut actual: Vec<_> = fixtures::NAMES
        .iter()
        .map(|name| format!("{}.yaml", name.replace('_', "-")))
        .collect();
    actual.sort();
    assert_eq!(actual, expected);

    for fixture in manifest() {
        let source = fs::read_to_string(root().join(&fixture.file)).unwrap();
        assert_eq!(
            astral_yaml::parse(&source).is_ok(),
            fixture.valid_yaml_1_2.unwrap_or(true),
            "{}",
            fixture.file
        );
    }
}

#[test]
fn workflow_scalars_retain_shell_syntax_and_yaml_styles() {
    use astral_yaml::{NodeKind, ScalarStyle};

    let source = include_str!("../../../fixtures/real-world/black-diff-shades.yaml");
    let stream = astral_yaml::parse(source).unwrap();
    let scalars: Vec<_> = stream
        .nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Scalar(scalar) => Some(scalar),
            _ => None,
        })
        .collect();

    for (value, style) in [
        ("on", ScalarStyle::Plain),
        ("1", ScalarStyle::DoubleQuoted),
        (
            "${{ github.workflow }}-${{ github.event.pull_request.number || github.run_id }}",
            ScalarStyle::Plain,
        ),
        (
            "diff-shades analyze ${{ matrix.baseline-analysis }} --work-dir projects-cache/ --force-${{ matrix.style }}-style -v",
            ScalarStyle::Plain,
        ),
        (
            "git config user.name \"diff-shades-gha\"\ngit config user.email \"diff-shades-gha@example.com\"\n",
            ScalarStyle::Literal,
        ),
    ] {
        assert!(
            scalars
                .iter()
                .any(|scalar| scalar.value == value && scalar.style == style),
            "{value:?}"
        );
    }

    let source = include_str!("../../../fixtures/real-world/compose-ci.yaml");
    let stream = astral_yaml::parse(source).unwrap();
    let shell_hash = source.find("s# ").unwrap() + 1;
    assert!(
        !stream
            .comments()
            .iter()
            .any(|comment| comment.span.start() <= shell_hash && shell_hash < comment.span.end())
    );
    assert!(
        stream
            .comments()
            .iter()
            .any(|comment| comment.text == " to fetch code (actions/checkout)")
    );
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
        checks::validate_spans(&stream).unwrap();
        assert_eq!(checks::events(&stream), oracle.events, "{}", fixture.file);

        for line_break in ["\r\n", "\r"] {
            let converted = source.replace('\n', line_break);
            let converted_stream = astral_yaml::parse(&converted).unwrap();
            checks::validate_spans(&converted_stream).unwrap();
            assert_eq!(
                checks::events(&converted_stream),
                oracle.events,
                "{}",
                fixture.file
            );
        }

        assert_eq!(scalar_spans(&stream), oracle.scalars, "{}", fixture.file);
    }
}
