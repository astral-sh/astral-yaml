mod support;

use std::{fmt::Write, fs, path::Path};

fn scalar_spans(stream: &astral_yaml::Stream<'_>) -> String {
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
    let mut actual = String::new();

    for (start, end) in spans {
        writeln!(actual, "{start} {end}").unwrap();
    }

    actual
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

#[test]
fn real_world_syntax_values_and_serialization_match_independent_oracles() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/real-world");
    let mut paths: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    paths.sort();

    // Keep the historical benchmark inputs fixed while the test corpus grows.
    for &(name, source) in support::FIXTURES {
        assert_eq!(
            fs::read_to_string(root.join(format!("{name}.yaml"))).unwrap(),
            source
        );
    }

    assert!(paths.len() >= 12);

    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let source = fs::read_to_string(&path).unwrap();
        if path.with_extension("error").exists() {
            // Real applications sometimes accept YAML outside the 1.2 grammar.
            // Keep these inputs and verify the documented rejection explicitly.
            let error = astral_yaml::parse(&source).unwrap_err();
            let actual = format!(
                "{} {}\n{}\n",
                error.span.start(),
                error.span.end(),
                error.message
            );
            assert_eq!(
                actual,
                fs::read_to_string(path.with_extension("error")).unwrap(),
                "{name}"
            );
            assert!(!support::reference(&source).is_empty());
            continue;
        }

        let values = support::astral(&source);
        assert_eq!(values, support::reference(&source), "{name}");

        let stream = astral_yaml::parse(&source).unwrap();
        xtask::validate_spans(&stream).unwrap();
        assert_alias_targets(&stream);

        // The snapshots come from PyYAML's Python and C parsers, never this
        // parser. They retain collection/scalar styles, order, tags and aliases.
        assert_eq!(
            xtask::events(&stream),
            fs::read_to_string(path.with_extension("event")).unwrap(),
            "syntax tree: {name}"
        );

        for line_break in ["\r\n", "\r"] {
            let converted = source.replace('\n', line_break);
            let converted_stream = astral_yaml::parse(&converted).unwrap();
            xtask::validate_spans(&converted_stream).unwrap();
            assert_eq!(
                xtask::events(&converted_stream),
                xtask::events(&stream),
                "line endings: {name}"
            );
        }

        if stream.documents().len() == 1 {
            let wrapped: astral_yaml_serde::Spanned<'_, yaml_serde::Value> =
                astral_yaml_serde::from_document(&stream, 0).unwrap();
            let comments: Vec<_> = wrapped
                .comments
                .iter()
                .map(|attached| attached.comment.clone())
                .collect();
            assert_eq!(comments, stream.comments(), "root comments: {name}");
        }

        let actual = scalar_spans(&stream);

        assert_eq!(
            actual,
            fs::read_to_string(path.with_extension("scalars")).unwrap(),
            "scalar spans: {name}"
        );

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<yaml_serde::Value>(&yaml).unwrap(),
                value,
                "{name}"
            );
            assert_eq!(
                yaml_serde::from_str::<yaml_serde::Value>(&yaml).unwrap(),
                value,
                "reference reader: {name}"
            );
        }

        assert!(
            astral_yaml_serde::from_document::<yaml_serde::Value>(
                &stream,
                stream.documents().len()
            )
            .is_err()
        );
    }
}

#[test]
fn workflow_scalars_retain_shell_syntax_and_yaml_styles() {
    use astral_yaml::{NodeKind, ScalarStyle};

    let source = include_str!("../../fixtures/real-world/black-diff-shades.yaml");
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

    let source = include_str!("../../fixtures/real-world/compose-ci.yaml");
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

#[test]
fn flow_key_and_value_separation_matches_reference() {
    for source in ["{a: [b]}", "{'a':[b]}", "{[]:value}"] {
        assert_eq!(
            support::astral(source),
            support::reference(source),
            "{source}"
        );
    }
}
