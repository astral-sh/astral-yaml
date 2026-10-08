use super::{EditableDocument, NodeKind, correspondence};
use crate::edit::draft::Draft;
use crate::edit::splice::{indent, region, render};
use astral_yaml_dev::indoc;

/// Render a frozen draft against its original source.
fn render_draft(original: &EditableDocument, draft: Draft) -> String {
    let candidate = draft.freeze().unwrap();
    let syntax = (&candidate).into();
    let source = original.source.clone();
    let region = region(original, original.source.root, false);
    let rendered = render(
        source.emission(syntax),
        original.source.root,
        region,
        indent(&original.source.text, region.start()),
        false,
    )
    .unwrap();
    let output = format!(
        "{}{}{}",
        &original.source.text[..rendered.span.start()],
        rendered.text,
        &original.source.text[rendered.span.end()..]
    );
    candidate.commit(&output).unwrap();
    output
}

#[test]
fn emission_accepts_syntax_without_source_locations() {
    use crate::edit::{
        Comment, CommentPosition, Node,
        view::{Emission, Syntax},
    };
    use crate::{NodeId, ScalarStyle, comments::CommentPlacement};

    let source = indoc! {"
        --- old
        ...
    "};
    let original = EditableDocument::parse(source).unwrap();
    let nodes = vec![Node {
        span: None,
        kind: NodeKind::Scalar {
            value: "new".to_owned(),
            style: ScalarStyle::SingleQuoted,
        },
        anchor: None,
        tag: None,
        comments: vec![Comment {
            span: None,
            text: " generated".to_owned(),
            placement: CommentPlacement::Inline,
            position: CommentPosition::After,
        }],
    }];
    let input = Emission::<Option<crate::Span>> {
        nodes: &nodes,
        root: NodeId(0),
        source,
        options: original.source.options,
        end_marker: original.source.end_marker,
    };
    let rendered = render(
        input,
        input.root,
        original.source.nodes[original.source.root.0].span,
        0,
        false,
    )
    .unwrap();
    let output = format!(
        "{}{}{}",
        &source[..rendered.span.start()],
        rendered.text,
        &source[rendered.span.end()..],
    );
    let parsed = EditableDocument::parse(&output).unwrap();
    let syntax = Syntax {
        nodes: &nodes,
        root: input.root,
    };
    let matched = correspondence(syntax, &parsed).unwrap();
    assert!(super::comments_preserved(syntax, &parsed, &matched));
    insta::assert_snapshot!(astral_yaml_dev::edit_diff(source, &output), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    ---- old
    +--- 'new' # generated
     ...
    ");
}

#[test]
fn emitted_content_comparison_checks_shapes_values_and_alias_targets() {
    for (left, right, expected) in [
        ("text", "'text'", true),
        ("null", "~", false),
        ("a:", "a: null", false),
        ("!custom", "!custom null", false),
        ("!!null null", "!!null null", true),
        ("!!bool true", "!!bool true", true),
        ("!!int '42'", "!!int 42", true),
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
            indoc! {r"
                # first
                # second
                value
            "},
            indoc! {r"
                # first
                # second
                value
            "},
            true,
        ),
        (
            indoc! {r"
                # first
                # second
                value
            "},
            indoc! {r"
                # second
                # first
                value
            "},
            false,
        ),
        (
            indoc! {r"
                value

                # first

                # second
            "},
            indoc! {r"
                value

                # second

                # first
            "},
            false,
        ),
        (
            indoc! {r"
                # first
                value
            "},
            indoc! {r"
                value # first
            "},
            false,
        ),
        (
            indoc! {r"
                # first
                value
            "},
            indoc! {r"
                value
            "},
            false,
        ),
        (
            indoc! {r"
                value
            "},
            indoc! {r"
                # first
                value
            "},
            false,
        ),
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
                        "{style:?}, {flow}, {indent}, {value:?} -> {text:?}"
                    );
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
            let text = super::scalar(value, style, super::ScalarPosition::Block, 2);
            let parsed = crate::parse(&text).unwrap();
            let node = parsed.node(parsed.documents()[0].root).unwrap();
            assert!(matches!(&node.kind, crate::NodeKind::Scalar(scalar)
                if scalar.value == value && scalar.style == ScalarStyle::DoubleQuoted));
        }
    }
    for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
        for value in [" a\nb", "a\n b\nc", "a\n\n b\n\nc", "a\n \nb", "\ta\nb"] {
            let text = super::scalar(value, style, super::ScalarPosition::Block, 2);
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
            indoc! {r"
                a: old

                # detached

                b: new
            "},
            100,
            10,
            "emitted YAML exceeds input byte limit",
        ),
        (
            indoc! {r"
                - old

                # detached

                - new
            "},
            100,
            10,
            "emitted YAML exceeds input byte limit",
        ),
        (
            indoc! {r"
                # leading
                a: value
            "},
            100,
            3,
            "emitted YAML exceeds input byte limit",
        ),
        (
            indoc! {r"
                a: # note
                  - value
            "},
            100,
            3,
            "emitted YAML exceeds input byte limit",
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document.source.options.max_nodes = nodes;
        document.source.options.max_input_bytes = bytes;
        let source = document.source.clone();
        let draft = Draft::from(&document).freeze().unwrap();
        let region = region(&document, document.source.root, false);
        // Both committed syntax and drafts obey the same rendering limits.
        for result in [
            render(&document, document.source.root, region, 0, false),
            render(
                source.emission((&draft).into()),
                document.source.root,
                region,
                0,
                false,
            ),
        ] {
            assert_eq!(result.unwrap_err().message, expected);
        }
    }
}

#[test]
fn plain_scalar_fallback_cannot_introduce_documents_or_collections() {
    for (value, flow) in [
        (
            indoc! {r"
                first
                ---
                second"},
            false,
        ),
        (
            indoc! {r"
                ]
                ---
                ["},
            true,
        ),
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
        );
        let parsed = crate::parse(&output).unwrap();
        assert_eq!(parsed.documents().len(), 1, "{output:?}");
        let root = parsed.node(parsed.documents()[0].root).unwrap();
        assert!(matches!(&root.kind, crate::NodeKind::Scalar(scalar) if scalar.value == value));
    }
}

#[test]
fn normalization_uses_the_target_anchor_and_rejects_an_unanchored_target() {
    let source = "[&old value, *old]";
    let mut document = EditableDocument::parse(source).unwrap();
    let target = document
        .source
        .nodes
        .iter()
        .find_map(|node| match node.kind {
            NodeKind::Alias { target } => Some(target),
            _ => None,
        })
        .unwrap();
    document.source.nodes[target.0].anchor = Some("new".to_owned());
    insta::assert_snapshot!(document.normalized_yaml().unwrap(), @"[&new value, *new]");

    document.source.nodes[target.0].anchor = None;
    insta::assert_snapshot!(document.normalized_yaml().unwrap_err().message, @"alias target has no anchor");
}

#[test]
fn normalization_rejects_an_alias_that_would_bind_to_a_different_definition() {
    let mut document = EditableDocument::parse("[&a 1, &a 2, *a]").unwrap();
    let first = crate::NodeId(
        document
            .source
            .nodes
            .iter()
            .position(|node| node.anchor.as_deref() == Some("a"))
            .unwrap(),
    );
    let target = document
        .source
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
    let mut document = EditableDocument::parse(indoc! {r"
        a: one
        # trailing
        b: two
    "})
    .unwrap();
    let mut comment = document
        .source
        .nodes
        .iter_mut()
        .find_map(|node| node.comments.pop())
        .unwrap();
    comment.placement = crate::comments::CommentPlacement::Trailing;
    comment.position = super::CommentPosition::After;
    let value = document
        .source
        .nodes
        .iter_mut()
        .find(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "one"))
        .unwrap();
    // This attachment would become a leading comment on the next field.
    value.comments.push(comment);
    assert_eq!(
        document.normalized_yaml().unwrap_err().message,
        "emission changed comment ownership or placement"
    );
}

#[test]
fn normalization_ignores_comments_on_unreachable_nodes() {
    let mut document = EditableDocument::parse("value").unwrap();
    let fragment = EditableDocument::parse("removed # comment").unwrap();
    let removed = fragment.source.nodes[fragment.source.root.0].clone();
    // Replaced nodes may remain in the arena without belonging to the document.
    document.source.nodes.push(removed);
    assert_eq!(document.normalized_yaml().unwrap(), "value");
}

#[test]
#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
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
        indoc! {r#"
        "a\nb\r\t\0\u0085\u2028\u2029""#},
        "{name: 'café', numbers: [1, 2]}",
        "{&key name: value}",
        "{? [a, b]: value}",
        "{? {key: value}: other}",
        "{!!str name: value}",
        r"  name: value
",
        "[{}, [], null]",
        indoc! {r"
            ---
            # start
            first: one # inline
            second:
              - two
              - three
            # end
            ...
        "},
        indoc! {r"
            [1, # one
             2]"},
        indoc! {r"
            {a: 1, # one
             b: 2}"},
        indoc! {r"
            a: [ # opening
              item] # closing
        "},
        indoc! {r"
            a: { # opening
              key: value} # closing
        "},
        indoc! {r"
            a: &a # anchor
              | # header
                text
        "},
        indoc! {r"
            ? [a, b]
            : value
        "},
        indoc! {r"
            ? |
              line
        "},
        "[&a [1, 2], *a]",
        indoc! {r"
            text: |-
              first
              second
        "},
        indoc! {r"
            text: >-
              first

              second
        "},
        indoc! {r"
            text: |+
              first

        "},
        indoc! {r"
            text: >
              first
              second
        "},
        indoc! {r"
            - one
            - two
        "},
        "!!str 42",
        "!<tag:yaml.org,2002:seq> [1]",
        "! text",
        indoc! {r"
            &root
            a: value
        "},
        indoc! {r"
            &root
            - value
        "},
        indoc! {r"
            ? key # key note
            : value
        "},
        indoc! {r"
            a: one

            # detached
            b: two
        "},
    ] {
        let document = EditableDocument::parse(source).unwrap();
        assert_eq!(document.to_yaml(), source);
        let output = document.normalized_yaml().expect(source);
        // Location types must not affect output.
        assert_eq!(render_draft(&document, Draft::from(&document)), output);
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
        indoc! {r"
            old
        "},
        indoc! {r"
            --- old
        "},
        r"  old
",
        "\u{feff}old\n",
        indoc! {r"
            &root old
        "},
        indoc! {r"
            ! old
        "},
        indoc! {r"
            # before
            old
        "},
    ] {
        for replacement in ["---", "... text"] {
            let mut document = EditableDocument::parse(source).unwrap();
            if let NodeKind::Scalar { value, .. } =
                &mut document.source.nodes[document.source.root.0].kind
            {
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
            indoc! {r"
                a
                - b"},
            "x: one, y",
        ] {
            let text = super::scalar(value, crate::ScalarStyle::Plain, position, 0);
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
    let source = indoc! {r"
        key: !!str # tag
          &a # anchor
          # before value
          old
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    let value = document
        .source
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

#[test]
fn detached_comments_preserve_their_structural_boundaries() {
    for (before, after, expected) in [
        (
            indoc! {r"
                # note

                value
            "},
            indoc! {r"
                value

                # note
            "},
            false,
        ),
        (
            indoc! {r"
                a: old

                # note

                b: old
                c: old
            "},
            indoc! {r"
                a: old
                b: old

                # note

                c: old
            "},
            false,
        ),
        (
            indoc! {r"
                # note

                - old
            "},
            indoc! {r"
                [
                # note

                old]
            "},
            true,
        ),
        (
            indoc! {r"
                # note

                a: old
            "},
            indoc! {r"
                {
                # note

                a: old}
            "},
            true,
        ),
        (
            indoc! {r"
                # note

                value
            "},
            indoc! {r"
                # note

                'value'
            "},
            true,
        ),
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
fn detached_prefixes_are_emitted_before_content() {
    let mut outputs = Vec::new();
    for source in [
        indoc! {r"
            # detached

            new
        "},
        indoc! {r"
            # detached

            # leading
            new
        "},
        indoc! {r"
            # detached

            - new
        "},
        indoc! {r"
            # first

            # detached

            new
        "},
    ] {
        let document = EditableDocument::parse(source).unwrap();
        // Including outer comments exercises the same emitter path used by
        // imported fragments, which have no retained source prefix.
        let span = region(&document, document.source.root, true);
        let rendered = render(&document, document.source.root, span, 0, false).unwrap();
        let parsed = EditableDocument::parse(&rendered.text).unwrap();
        let correspondence = correspondence(&document, &parsed).unwrap();
        assert!(super::comments_preserved(
            &document,
            &parsed,
            &correspondence
        ));
        outputs.push(rendered.text);
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "# detached\n\nnew",
        "# detached\n\n# leading\nnew",
        "# detached\n\n- new",
        "# first\n\n# detached\n\nnew",
    ]
    "##);
}

#[test]
fn flow_opening_comments_do_not_depend_on_visible_closing_comments() {
    let mut outputs = Vec::new();
    for source in [
        indoc! {r"
            [ # open
              old
            ] # close
        "},
        indoc! {r"
            { # open
              key: old
            } # close
        "},
        indoc! {r"
            &a [ # open
              old
            ] # close
        "},
        indoc! {r"
            !!seq # tag
            &a [ # open
              old
            ] # close
        "},
        indoc! {r"
            [ # open
            ]
        "},
        indoc! {r"
            key: [ # open
              old
              ]
        "},
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        assert_eq!(
            EditableDocument::parse(&output)
                .unwrap()
                .normalized_yaml()
                .unwrap(),
            output
        );
        outputs.push(output);
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "[ # open\n  old] # close\n",
        "{ # open\n  key: old} # close\n",
        "&a [ # open\n  old] # close\n",
        "!<tag:yaml.org,2002:seq> # tag\n  &a [ # open\n  old] # close\n",
        "[ # open\n  ]\n",
        "key: [ # open\n  old]\n",
    ]
    "#);
}

#[test]
fn omitted_roots_own_their_erased_line() {
    let mut outputs = Vec::new();
    for source in [
        indoc! {r"
            # leading
            ---
            old
            ...
        "},
        indoc! {r"
            # leading
            ---
              old

            ...
        "},
        "\u{feff}---\n  old\n...\n",
        "--- old",
        indoc! {r"
            --- old
        "},
        indoc! {r"
            --- old
            ...
        "},
        indoc! {r"
            # leading
            --- old

            ...
        "},
        indoc! {r"
            # leading
            --- |-
              old

            ...
        "},
        indoc! {r"
            # leading
            ---
            |+
              old

            ...
        "},
        indoc! {r"
            ---

            ...
        "},
    ] {
        let mut candidate = EditableDocument::parse(source).unwrap();
        candidate.source.nodes[candidate.source.root.0].kind = NodeKind::Scalar {
            value: String::new(),
            style: crate::ScalarStyle::Plain,
        };
        outputs.push(candidate.normalized_yaml().unwrap());
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "# leading\n---\n...\n",
        "# leading\n---\n...\n",
        "\u{feff}---\n...\n",
        "--- ",
        "--- \n",
        "--- \n...\n",
        "# leading\n--- \n...\n",
        "# leading\n--- \n...\n",
        "# leading\n---\n...\n",
        "---\n\n...\n",
    ]
    "##);
}

#[test]
fn emitted_comments_own_horizontal_whitespace_before_retained_breaks() {
    let mut outputs = Vec::new();
    for source in [
        indoc! {"
            old\x20
        "},
        "old \t \n",
        "[old] \t\n",
    ] {
        let original = EditableDocument::parse(source).unwrap();
        let mut candidate = Draft::from(&original);
        candidate
            .insert_comment(
                candidate.root(),
                0,
                " added",
                crate::comments::CommentPlacement::Trailing,
                None,
            )
            .unwrap();
        outputs.push(render_draft(&original, candidate));
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "old\n# added\n",
        "old\n# added\n",
        "[old]\n# added\n",
    ]
    "#);
}

#[test]
fn block_scalar_suffix_comments_use_the_parent_indentation() {
    let mut outputs = Vec::new();
    for source in [
        indoc! {r"
              old
            # tail
        "},
        indoc! {r"
                old
            # tail
        "},
        indoc! {r"
              old # inline
            # tail
        "},
        indoc! {r"
              &a old # inline
            # tail
        "},
    ] {
        let mut candidate = EditableDocument::parse(source).unwrap();
        candidate.source.nodes[candidate.source.root.0].kind = NodeKind::Scalar {
            value: "old".to_owned(),
            style: crate::ScalarStyle::Literal,
        };
        let region = region(&candidate, candidate.source.root, true);
        let rendered = render(
            &candidate,
            candidate.source.root,
            region,
            indent(source, region.start()),
            false,
        )
        .unwrap();
        let output = format!(
            "{}{}{}",
            &source[..rendered.span.start()],
            rendered.text,
            &source[rendered.span.end()..]
        );
        let parsed = EditableDocument::parse(&output).unwrap();
        let matched = correspondence(&candidate, &parsed).unwrap();
        assert!(super::comments_preserved(&candidate, &parsed, &matched));
        outputs.push(output);
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "  |2-\n  old\n# tail\n",
        "    |2-\n  old\n# tail\n",
        "  |2- # inline\n  old\n# tail\n",
        "  &a |2- # inline\n  old\n# tail\n",
    ]
    "#);
}

#[test]
fn omitted_roots_own_their_line_when_comments_are_emitted() {
    let mut outputs = Vec::new();
    for source in ["---\nold\n...\n", "--- old\n...\n"] {
        let original = EditableDocument::parse(source).unwrap();
        let mut candidate = Draft::from(&original);
        candidate.replace(candidate.root(), "---", false).unwrap();
        candidate
            .insert_comment(
                candidate.root(),
                0,
                " added",
                crate::comments::CommentPlacement::Leading,
                None,
            )
            .unwrap();
        outputs.push(render_draft(&original, candidate));
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "---\n# added\n...\n",
        "--- \n# added\n...\n",
    ]
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn full_line_suffixes_reserve_an_earlier_inline_header() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            outer: [old] # inline
        "},
        indoc! {r"
            outer: {key: old} # inline
        "},
    ] {
        for placement in [
            crate::comments::CommentPlacement::Trailing,
            crate::comments::CommentPlacement::Detached,
        ] {
            let original = EditableDocument::parse(source).unwrap();
            let mut candidate = Draft::from(&original);
            let mut children = Vec::new();
            super::children(&candidate.node(candidate.root()).kind, &mut children);
            let inner = children[1];
            candidate
                .insert_comment(
                    inner,
                    candidate.node(inner).comments.len(),
                    " added",
                    placement,
                    None,
                )
                .unwrap();
            diffs.push(astral_yaml_dev::edit_diff(
                source,
                &render_draft(&original, candidate),
            ));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -outer: [old] # inline
    +outer: [ # inline
    +  old,
    +  # added
    +  ]

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -outer: [old] # inline
    +outer: [ # inline
    +  old,
    +  ⏎
    +  # added
    +  ]

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -outer: {key: old} # inline
    +outer: { # inline
    +  key: old,
    +  # added
    +  }

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -outer: {key: old} # inline
    +outer: { # inline
    +  key: old,
    +  ⏎
    +  # added
    +  }
    ");
}
