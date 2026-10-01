use super::{EditableDocument, NodeKind, equivalent, region, render};

#[test]
fn emitted_content_comparison_checks_shapes_values_and_alias_targets() {
    for (left, right, expected) in [
        ("text", "'text'", true),
        ("null", "~", true),
        ("!!str 42", "!!int 42", false),
        ("1", "2", false),
        ("[1]", "[]", false),
        ("{a: b}", "{}", false),
        ("{a: b}", "{a: b}", true),
        ("[]", "{}", false),
        ("[&a 1, &b 1, *a]", "[&a 1, &b 1, *b]", false),
    ] {
        let left = EditableDocument::parse(left).unwrap();
        let right = EditableDocument::parse(right).unwrap();
        assert_eq!(equivalent(&left, &right), expected);
    }
}

#[test]
fn rendering_obeys_work_and_output_budgets() {
    for (source, nodes, bytes, expected) in [
        ("42", 0, 100, "emission resource limit exceeded"),
        ("[42]", 1, 100, "emission resource limit exceeded"),
        ("42", 100, 1, "emitted YAML exceeds input byte limit"),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document.options.max_nodes = nodes;
        document.options.max_input_bytes = bytes;
        let error = render(
            &document,
            document.root,
            region(&document, document.root, false),
            0,
            false,
        )
        .unwrap_err();
        assert_eq!(error.message, expected);
    }
}

#[test]
fn normalization_rejects_an_alias_that_would_bind_to_a_different_definition() {
    let mut document = EditableDocument::parse("[&a 1, &a 2, *a]").unwrap();
    let first = crate::NodeId(
        document
            .nodes
            .iter()
            .position(|node| node.anchor.as_deref() == Some("a"))
            .unwrap(),
    );
    let target = document
        .nodes
        .iter_mut()
        .find_map(|node| {
            if let NodeKind::Alias { target, .. } = &mut node.kind {
                Some(target)
            } else {
                None
            }
        })
        .unwrap();
    // Model retaining the first definition while a later definition shadows it.
    // The renderer must fail rather than emit an alias with different meaning.
    *target = first;
    assert_eq!(
        document.normalized_yaml().unwrap_err().message,
        "emission changed YAML content or alias identity"
    );
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
        let output = document.normalized_yaml().expect(source);
        assert_eq!(
            astral_yaml_dev::benchmarks::documents::<Value>(&output),
            astral_yaml_dev::benchmarks::documents::<Value>(source),
            "{source:?} -> {output:?}"
        );
        let before = crate::parse(source).unwrap();
        let after = crate::parse(&output).unwrap();
        let mut before: Vec<_> = before.comments().iter().map(|c| c.text).collect();
        let mut after: Vec<_> = after.comments().iter().map(|c| c.text).collect();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(before, after, "{source:?} -> {output:?}");
    }
}
