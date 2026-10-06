use super::{EditableDocument, NodeKind, correspondence, region, render};

#[test]
fn emitted_content_comparison_checks_shapes_values_and_alias_targets() {
    for (left, right, expected) in [
        ("text", "'text'", true),
        ("null", "~", false),
        ("a:", "a: null", false),
        ("!custom", "!custom null", false),
        ("!!null null", "!!null null", true),
        ("!!bool true", "!!bool true", true),
        ("!!float 1.0", "!!float 1.0", true),
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
        assert_eq!(correspondence(&left, &right).is_some(), expected);
    }
}

#[test]
fn comment_comparison_preserves_each_owners_order() {
    for (before, after, expected) in [
        (
            "# first\n# second\nvalue\n",
            "# first\n# second\nvalue\n",
            true,
        ),
        (
            "# first\n# second\nvalue\n",
            "# second\n# first\nvalue\n",
            false,
        ),
        (
            "value\n\n# first\n\n# second\n",
            "value\n\n# second\n\n# first\n",
            false,
        ),
        ("# first\nvalue\n", "value # first\n", false),
        ("# first\nvalue\n", "value\n", false),
        ("value\n", "# first\nvalue\n", false),
    ] {
        let before = EditableDocument::parse(before).unwrap();
        let after = EditableDocument::parse(after).unwrap();
        let correspondence = correspondence(&before, &after).unwrap();
        assert_eq!(
            super::comments_preserved(&before, &after, &correspondence),
            expected
        );
    }
}

#[test]
fn scalar_emission_preserves_representable_line_breaks() {
    // Flow folding and block chomping:
    // https://yaml.org/spec/1.2.2/#65-line-folding
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    use crate::ScalarStyle;
    for style in [
        ScalarStyle::Plain,
        ScalarStyle::SingleQuoted,
        ScalarStyle::Literal,
        ScalarStyle::Folded,
    ] {
        for value in [
            "a\nb", "a\n\nb", "\nb", "\n\nb", "b\n", "b\n\n", "\n", "\n\n",
        ] {
            if style == ScalarStyle::Plain && (value.starts_with('\n') || value.ends_with('\n')) {
                continue;
            }
            for indent in [0, 2, 4] {
                for newline in ["\n", "\r\n", "\r"] {
                    for flow in [false, true] {
                        if flow && matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) {
                            continue;
                        }
                        let text = super::scalar(
                            value,
                            style,
                            if flow {
                                super::ScalarPosition::Flow
                            } else {
                                super::ScalarPosition::Block
                            },
                            indent,
                            newline,
                        );
                        let source = if flow {
                            format!("[{text}]")
                        } else {
                            format!("{}value: {text}", " ".repeat(indent))
                        };
                        let stream = crate::parse(&source).unwrap();
                        let mut node = stream.node(stream.documents()[0].root).unwrap();
                        if let crate::NodeKind::Sequence { items, .. } = &node.kind {
                            node = stream.node(items[0]).unwrap();
                        } else if let crate::NodeKind::Mapping { entries, .. } = &node.kind {
                            node = stream.node(entries[0].value).unwrap();
                        }
                        assert!(
                            matches!(&node.kind, crate::NodeKind::Scalar(scalar) if scalar.value == value && scalar.style == style),
                            "{style:?}, {flow}, {indent}, {newline:?}, {value:?} -> {text:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn scalar_emission_retains_whitespace_that_cannot_be_folded() {
    use crate::ScalarStyle;
    for style in [ScalarStyle::Plain, ScalarStyle::SingleQuoted] {
        for value in ["a \nb", "a\n b", "a\t\nb", "a\n\tb", "a\rb", "a\0b"] {
            let text = super::scalar(value, style, super::ScalarPosition::Block, 2, "\n");
            let parsed = crate::parse(&text).unwrap();
            let node = parsed.node(parsed.documents()[0].root).unwrap();
            assert!(matches!(&node.kind, crate::NodeKind::Scalar(scalar)
                if scalar.value == value && scalar.style == ScalarStyle::DoubleQuoted));
        }
    }
    for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
        for value in [" a\nb", "a\n b\nc", "a\n\n b\n\nc", "a\n \nb", "\ta\nb"] {
            let text = super::scalar(value, style, super::ScalarPosition::Block, 2, "\n");
            let source = format!("  value: {text}");
            let parsed = crate::parse(&source).unwrap();
            let node = parsed
                .nodes()
                .iter()
                .find(|node| {
                    matches!(&node.kind,
                crate::NodeKind::Scalar(scalar) if scalar.value == value)
                })
                .unwrap();
            assert!(
                matches!(&node.kind, crate::NodeKind::Scalar(scalar)
                if scalar.style == style),
                "{value:?} -> {text:?}"
            );
        }
    }
}

#[test]
fn rendering_obeys_work_and_output_budgets() {
    for (source, nodes, bytes, expected) in [
        ("42", 0, 100, "emission resource limit exceeded"),
        ("[42]", 1, 100, "emission resource limit exceeded"),
        ("42", 100, 1, "emitted YAML exceeds input byte limit"),
        (
            "a: old\n\n# detached\n\nb: new\n",
            100,
            10,
            "emitted YAML exceeds input byte limit",
        ),
        (
            "- old\n\n# detached\n\n- new\n",
            100,
            10,
            "emitted YAML exceeds input byte limit",
        ),
        (
            "# leading\na: value\n",
            100,
            3,
            "emitted YAML exceeds input byte limit",
        ),
        (
            "a: # note\n  - value\n",
            100,
            3,
            "emitted YAML exceeds input byte limit",
        ),
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
fn plain_scalar_fallback_cannot_introduce_documents_or_collections() {
    for (value, flow) in [
        ("first\n---\nsecond", false),
        ("]\n---\n[", true),
        ("key]: value #", true),
        ("first, second", true),
        ("] # empty sequence", true),
        ("first\r- second", false),
        ("first\r---\rsecond", false),
    ] {
        let output = super::scalar(
            value,
            crate::ScalarStyle::Plain,
            if flow {
                super::ScalarPosition::Flow
            } else {
                super::ScalarPosition::Block
            },
            0,
            "\n",
        );
        let parsed = crate::parse(&output).unwrap();
        assert_eq!(parsed.documents().len(), 1, "{output:?}");
        let root = parsed.node(parsed.documents()[0].root).unwrap();
        assert!(matches!(&root.kind, crate::NodeKind::Scalar(scalar) if scalar.value == value));
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
fn normalization_rejects_reassigned_comments() {
    let mut document = EditableDocument::parse("a: one\nb: two\n").unwrap();
    let value = document
        .nodes
        .iter_mut()
        .find(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "one"))
        .unwrap();
    // This attachment would become a leading comment on the next field.
    value.comments.push(crate::edit::Comment {
        text: " trailing".to_owned(),
        placement: crate::comments::CommentPlacement::Trailing,
        position: super::CommentPosition::After,
        span: crate::Span::new(0, 0).unwrap(),
    });
    assert_eq!(
        document.normalized_yaml().unwrap_err().message,
        "emission changed comment ownership or placement"
    );
}

#[test]
fn normalization_ignores_comments_on_unreachable_nodes() {
    let mut document = EditableDocument::parse("value").unwrap();
    let mut removed = document.nodes[document.root.0].clone();
    removed.comments.push(crate::edit::Comment {
        text: " removed".to_owned(),
        placement: crate::comments::CommentPlacement::Inline,
        position: super::CommentPosition::After,
        span: crate::Span::new(0, 0).unwrap(),
    });
    // Replaced nodes may remain in the arena without belonging to the document.
    document.nodes.push(removed);
    assert_eq!(document.normalized_yaml().unwrap(), "value");
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
        r#""a\"b\\c""#,
        "\"a\\nb\\r\\t\\0\\u0085\\u2028\\u2029\"",
        "{name: 'café', numbers: [1, 2]}",
        "{&key name: value}",
        "{!!str name: value}",
        "  name: value\n",
        "[{}, [], null]",
        "---\n# start\nfirst: one # inline\nsecond:\n  - two\n  - three\n# end\n...\n",
        "[1, # one\n 2]",
        "{a: 1, # one\n b: 2}",
        "a: [ # opening\n  item] # closing\n",
        "a: { # opening\n  key: value} # closing\n",
        "a: &a # anchor\n  | # header\n    text\n",
        "? [a, b]\n: value\n",
        "? |\n  line\n",
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

#[test]
fn root_scalar_candidates_respect_document_position() {
    let mut outputs = Vec::new();
    for source in [
        "old\n",
        "--- old\n",
        "  old\n",
        "\u{feff}old\n",
        "&root old\n",
        "! old\n",
        "# before\nold\n",
    ] {
        for replacement in ["---", "... text"] {
            let mut document = EditableDocument::parse(source).unwrap();
            if let NodeKind::Scalar { value, .. } = &mut document.nodes[document.root.0].kind {
                replacement.clone_into(value);
            }
            outputs.push(document.normalized_yaml().unwrap());
        }
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "\"---\"\n",
        "\"... text\"\n",
        "--- ---\n",
        "--- ... text\n",
        "  ---\n",
        "  ... text\n",
        "\u{feff}\"---\"\n",
        "\u{feff}\"... text\"\n",
        "&root ---\n",
        "&root ... text\n",
        "! ---\n",
        "! ... text\n",
        "# before\n\"---\"\n",
        "# before\n\"... text\"\n",
    ]
    "##);
}

#[test]
fn scalar_validation_distinguishes_implicit_keys_from_values() {
    use super::ScalarPosition;
    let mut outputs = Vec::new();
    for position in [ScalarPosition::BlockKey, ScalarPosition::FlowKey] {
        for value in [
            "---",
            "...",
            "a,b",
            "a[b]",
            "- item",
            "x: value",
            "first\rsecond",
            "a\n- b",
            "x: one, y",
        ] {
            let text = super::scalar(value, crate::ScalarStyle::Plain, position, 0, "\n");
            let source = if position == ScalarPosition::BlockKey {
                format!("{text}: value")
            } else {
                format!("{{{text}: value}}")
            };
            let parsed = crate::parse(&source).unwrap();
            let root = parsed.node(parsed.documents()[0].root).unwrap();
            assert!(
                matches!(&root.kind, crate::NodeKind::Mapping { entries, .. }
                if entries.len() == 1)
            );
            let key = parsed
                .nodes()
                .iter()
                .find_map(|node| {
                    if let crate::NodeKind::Scalar(key) = &node.kind {
                        Some(key)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(key.value, value);
            outputs.push(source);
        }
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "---: value",
        "...: value",
        "a,b: value",
        "a[b]: value",
        "\"- item\": value",
        "\"x: value\": value",
        "\"first\\rsecond\": value",
        "\"a\\n- b\": value",
        "\"x: one, y\": value",
        "{---: value}",
        "{...: value}",
        "{\"a,b\": value}",
        "{\"a[b]\": value}",
        "{\"- item\": value}",
        "{\"x: value\": value}",
        "{\"first\\rsecond\": value}",
        "{a\n\n  - b: value}",
        "{\"x: one, y\": value}",
    ]
    "#);
}

#[test]
fn removed_properties_leave_room_for_retained_header_comments() {
    let source = "key: !!str # tag\n  &a # anchor\n  # before value\n  old\n";
    let mut document = EditableDocument::parse(source).unwrap();
    let value = document
        .nodes
        .iter_mut()
        .find(|node| node.anchor.as_deref() == Some("a"))
        .unwrap();
    value.tag = None;
    insta::assert_snapshot!(document.normalized_yaml().unwrap(), @"
    key: # tag
      &a # anchor
        # before value
        old
    ");
}
