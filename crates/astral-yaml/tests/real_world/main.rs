//! Sources and revisions for the snapshot inputs are recorded in `fixtures/real-world/manifest.json`.

mod display;
mod fixtures;

use std::fs;

use astral_yaml_dev::fixtures::{manifest, root, with_line_breaks};
use astral_yaml_dev::{diagnostic_snapshot, validate_spans};

// Each alias resolves to the most recent preceding anchor with the same name.
// https://yaml.org/spec/1.2.2/#71-alias-nodes
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
    validate_spans(&stream).unwrap();
    assert_alias_targets(&stream);
    let tree = display::tree(&stream);

    // Scalar content normalizes LF, CRLF, and CR to line feeds.
    // https://yaml.org/spec/1.2.2/#54-line-break-characters
    for line_break in ["\r\n", "\r"] {
        let converted = with_line_breaks(&source, line_break);
        let converted_stream = astral_yaml::parse(&converted).unwrap();
        validate_spans(&converted_stream).unwrap();
        assert_eq!(
            display::tree(&converted_stream),
            tree,
            "line endings: {name}"
        );
    }

    tree
}

// The rejected fixtures exercise flow continuation indentation and document-scoped tag handles.
// https://yaml.org/spec/1.2.2/#rule-s-flow-line-prefix
// https://yaml.org/spec/1.2.2/#rule-s-l+flow-in-block
// https://yaml.org/spec/1.2.2/#6821-tag-handles
fn check_rejection(name: &str) -> String {
    let source = fs::read_to_string(root().join(format!("{name}.yaml"))).unwrap();
    let error = astral_yaml::parse(&source).unwrap_err();

    diagnostic_snapshot(&source, error)
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
        let source = fixture.source();
        assert_eq!(
            astral_yaml::parse(&source).is_ok(),
            fixture.valid_syntax(),
            "{}",
            fixture.file
        );
    }
}

// https://yaml.org/spec/1.2.2/#733-plain-style
// https://yaml.org/spec/1.2.2/#812-literal-style
// https://yaml.org/spec/1.2.2/#66-comments
#[test]
fn workflow_scalars_retain_shell_syntax_and_yaml_styles() {
    use astral_yaml::{NodeKind, ScalarStyle};

    let source = include_str!("../../../../fixtures/real-world/black-diff-shades.yaml");
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

    let source = include_str!("../../../../fixtures/real-world/compose-ci.yaml");
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
