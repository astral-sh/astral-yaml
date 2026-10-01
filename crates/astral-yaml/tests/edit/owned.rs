//! Owned syntax retains YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.
use astral_yaml::{
    ParseOptions,
    comments::{CommentIndex, CommentPlacement},
    edit::{EditableDocument, NodeKind},
};
use astral_yaml_dev::indoc;

#[test]
fn editable_sources_normalize_line_breaks_before_assigning_spans() {
    // YAML line breaks normalize to LF: https://yaml.org/spec/1.2.2/#54-line-break-characters
    let source = indoc! {r#"
        # before
        key: &a 'é' # inline

        copy: *a
        escaped: "\r\n"
    "#};
    let expected = EditableDocument::parse(source).unwrap();
    // Include mixed and consecutive physical line breaks.
    for input in [
        source.to_owned(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
        source
            .replace("# before\n", "# before\r\n")
            .replace("copy: *a\n", "copy: *a\r"),
    ] {
        let document = EditableDocument::parse(&input).unwrap();
        assert_eq!(format!("{document:?}"), format!("{expected:?}"));
    }
    // Escaped carriage returns are scalar data, not physical line breaks.
    let NodeKind::Mapping { entries, .. } = expected.node(expected.root()).unwrap().kind() else {
        panic!("mapping")
    };
    insta::assert_debug_snapshot!(expected.node(entries[2].1).unwrap().kind(), @r#"
    Scalar {
        value: "\r\n",
        style: DoubleQuoted,
    }
    "#);
}

#[test]
fn normalization_keeps_limits_on_the_original_input_and_locates_errors_in_lf() {
    let limited = EditableDocument::parse_with_options(
        "key: value\r\n",
        ParseOptions {
            max_input_bytes: "key: value\n".len(),
            ..ParseOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(limited.message, "input byte limit exceeded");
    let error = EditableDocument::parse("key: value\r\ninvalid: [\r\n").unwrap_err();
    assert_eq!(
        error,
        EditableDocument::parse("key: value\ninvalid: [\n").unwrap_err()
    );
}

#[test]
fn owned_document_retains_syntax_and_direct_comments() {
    let source = indoc! {r"
        %YAML 1.2
        ---
        # before
        items: &items [!!str 'é', 2] # inline
        copy: *items
        ...
    "};
    let document = EditableDocument::parse(source).unwrap();
    assert_eq!(document.source(), source);
    assert_eq!(
        document.options().max_depth,
        ParseOptions::default().max_depth
    );
    let root = document.node(document.root()).unwrap();
    let NodeKind::Mapping { entries, .. } = root.kind() else {
        panic!("mapping")
    };
    let sequence = document.node(entries[0].1).unwrap();
    assert_eq!(sequence.anchor(), Some("items"));
    assert_eq!(sequence.tag(), None);
    let NodeKind::Sequence { items, .. } = sequence.kind() else {
        panic!("sequence")
    };
    let scalar = document.node(items[0]).unwrap();
    assert_eq!(scalar.tag(), Some("tag:yaml.org,2002:str"));
    assert_eq!(scalar.span().slice(source), Some("!!str 'é'"));
    assert!(matches!(
        document.node(entries[1].1).unwrap().kind(),
        NodeKind::Alias { .. }
    ));
    let comment = sequence
        .comments()
        .iter()
        .find(|c| c.placement() == CommentPlacement::Inline)
        .unwrap();
    assert_eq!(comment.text(), " inline");
    assert_eq!(comment.span().slice(source), Some("# inline"));

    let owned = {
        let source = String::from("key: value");
        EditableDocument::parse(&source).unwrap()
    };
    assert_eq!(owned.source(), "key: value");
}

#[test]
fn alias_edges_distinguish_recursive_and_shadowed_definitions() {
    // Alias nodes refer to the most recent preceding definition of an anchor:
    // https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    let source = indoc! {r"
        - &item [first, *item]
        - *item
        - &item second
        - *item
    "};
    let document = EditableDocument::parse(source).unwrap();
    let NodeKind::Sequence { items, .. } = document.node(document.root()).unwrap().kind() else {
        panic!("sequence")
    };
    let NodeKind::Sequence { items: first, .. } = document.node(items[0]).unwrap().kind() else {
        panic!("sequence")
    };
    let targets: Vec<_> = [first[1], items[1], items[3]]
        .into_iter()
        .map(|id| {
            let NodeKind::Alias { target } = document.node(id).unwrap().kind() else {
                panic!("alias")
            };
            document
                .node(*target)
                .unwrap()
                .span()
                .slice(source)
                .unwrap()
        })
        .collect();
    insta::assert_debug_snapshot!(targets, @r#"
    [
        "&item [first, *item]",
        "&item [first, *item]",
        "&item second",
    ]
    "#);
}

#[test]
fn construction_rejects_invalid_documents_and_limits() {
    for source in [
        "[",
        "",
        indoc! {r"
            ---
            1
            ---
            2"},
    ] {
        assert!(EditableDocument::parse(source).is_err());
    }
    let options = ParseOptions {
        max_input_bytes: 1,
        ..ParseOptions::default()
    };
    assert!(EditableDocument::parse_with_options("42", options).is_err());
    let stream = astral_yaml::parse("42 # note").unwrap();
    assert_eq!(
        CommentIndex::new(&stream, 1).unwrap_err().message,
        "document index out of bounds"
    );
    let index = CommentIndex::new(&stream, 0).unwrap();
    assert_eq!(index.direct().count(), 1);
    assert_eq!(index.subtree(stream.documents()[0].root).len(), 1);
}

#[test]
fn normalized_syntax_preserves_values_and_comments() {
    use astral_yaml_dev::Value;
    for source in [
        "42",
        "'null'",
        "null",
        "''",
        "true",
        "0xff",
        "'a''b'",
        "\"a\\nb\\r\\t\\0\\u0085\\u2028\\u2029\"",
        "{name: 'café', numbers: [1, 2]}",
        "[{}, [], null]",
        "---\n# start\nfirst: one # inline\nsecond:\n  - two\n  - three\n# end\n...\n",
        "[1, # one\n 2]",
        "{a: 1, # one\n b: 2}",
        "? [a, b]\n: value\n",
        "[&a [1, 2], *a]",
        "text: |-\n  first\n  second\n",
        "text: >-\n  first\n\n  second\n",
        "text: |+\n  first\n\n",
        "text: >\n  first\n  second\n",
        "first: one\r\nsecond: two\r\n",
        "first: one\rsecond: two\r",
        "- one\n- two\n",
        "!!str 42",
        "!<tag:yaml.org,2002:seq> [1]",
        "! text",
        "&root\na: value\n",
        "&root\n- value\n",
        "? key # key note\n: value\n",
        "a: one\n\n# detached\nb: two\n",
    ] {
        let document = EditableDocument::parse(source).unwrap();
        assert_eq!(document.to_yaml(), source);
        let output = document
            .normalized_yaml()
            .unwrap_or_else(|e| panic!("{source:?}: {e}"));
        assert_eq!(
            astral_yaml_dev::benchmarks::documents::<Value>(&output),
            astral_yaml_dev::benchmarks::documents::<Value>(source),
            "{source:?} -> {output:?}"
        );
        let before = astral_yaml::parse(source).unwrap();
        let after = astral_yaml::parse(&output).unwrap();
        let mut before: Vec<_> = before.comments().iter().map(|c| c.text).collect();
        let mut after: Vec<_> = after.comments().iter().map(|c| c.text).collect();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(before, after, "{source:?} -> {output:?}");
    }
}

#[test]
fn real_world_documents_can_be_normalized() {
    for fixture in astral_yaml_dev::fixtures::manifest() {
        if !fixture.valid_syntax() {
            continue;
        }
        let source = fixture.source();
        let stream = astral_yaml::parse(&source).unwrap();
        if stream.documents().len() != 1 {
            continue;
        }
        let document = EditableDocument::parse(&source).unwrap();
        document
            .normalized_yaml()
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.file));
    }
}
