use test_case::test_case;

use super::{DocumentSyntax, NodeKind, correspondence};
use crate::syntax::draft::{Draft, ReplacementStyle};
use crate::syntax::splice::{self, OuterComments};
use astral_yaml::ScalarStyle;
use astral_yaml_dev::indoc;

/// Render a frozen draft against its original source.
fn render_draft(original: &DocumentSyntax, draft: Draft) -> String {
    let candidate = draft.freeze().unwrap();
    let syntax = (&candidate).into();
    let source = &original.source;
    let region = splice::region(original, source.root, OuterComments::Exclude);
    let rendered = splice::render(source.emission(syntax), source.root, region).unwrap();
    let output = format!(
        "{}{}{}",
        &source.text[..rendered.span.start()],
        rendered.text,
        &source.text[rendered.span.end()..]
    );
    candidate.commit(&output).unwrap();
    output
}

#[test]
fn emission_accepts_syntax_without_source_locations() {
    use crate::syntax::view::Syntax;
    use crate::syntax::{Comment, CommentPosition, Node};
    use astral_yaml::comments::CommentPlacement;
    use astral_yaml::{NodeId, ScalarStyle};

    let source = indoc! {"
        --- old
        ...
    "};
    let original = DocumentSyntax::parse(source).unwrap();
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
    let syntax = Syntax::<Option<astral_yaml::Span>> {
        nodes: &nodes,
        root: NodeId::from_index(0),
    };
    let rendered = splice::render(
        original.source.emission(syntax),
        syntax.root,
        original.source.nodes[original.source.root.index()].span,
    )
    .unwrap();
    let output = format!(
        "{}{}{}",
        &source[..rendered.span.start()],
        rendered.text,
        &source[rendered.span.end()..],
    );
    let parsed = DocumentSyntax::parse(&output).unwrap();
    let matched = correspondence(syntax, &parsed).unwrap();
    assert!(super::comments_preserved(syntax, &parsed, &matched));
    insta::assert_snapshot!(astral_yaml_dev::edit_diff(source, &output), @"
    @@ -1,2 +1,2 @@
    ---- old
    +--- 'new' # generated
     ...
    ");
}

#[test_case("text", "'text'", true)]
#[test_case("null", "~", false)]
#[test_case("a:", "a: null", false)]
#[test_case("!custom", "!custom null", false)]
#[test_case("! 'true'", "! true", true)]
#[test_case("!!str '42'", "!!str 42", true)]
#[test_case("!!null 'null'", "!!null null", true)]
#[test_case("!!bool 'true'", "!!bool true", true)]
#[test_case("!!int '42'", "!!int 42", true)]
#[test_case("!!float '1.0'", "!!float 1.0", true)]
#[test_case("!custom 'true'", "!custom true", false)]
#[test_case("!!str 42", "!!int 42", false)]
#[test_case("1", "2", false)]
#[test_case("[1]", "[]", false)]
#[test_case("{a: b}", "{}", false)]
#[test_case("{a: b}", "{a: b}", true)]
#[test_case("[]", "{}", false)]
#[test_case("[&a 1, &b 1, *a]", "[&a 1, &b 1, *b]", false)]
fn emitted_content_comparison_checks_shapes_values_and_alias_targets(
    left: &str,
    right: &str,
    expected: bool,
) {
    let left = DocumentSyntax::parse(left).unwrap();
    let right = DocumentSyntax::parse(right).unwrap();
    assert_eq!(correspondence(&left, &right).is_some(), expected);
}

#[test_case(
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
    true;
    "same order"
)]
#[test_case(
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
    false;
    "reversed leading comments"
)]
#[test_case(
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
    false;
    "reversed detached comments"
)]
#[test_case(
    indoc! {r"
        # first
        value
    "},
    indoc! {r"
        value # first
    "},
    false;
    "changed placement"
)]
#[test_case(
    indoc! {r"
        # first
        value
    "},
    indoc! {r"
        value
    "},
    false;
    "removed comment"
)]
#[test_case(
    indoc! {r"
        value
    "},
    indoc! {r"
        # first
        value
    "},
    false;
    "added comment"
)]
fn comment_comparison_preserves_each_owners_order(before: &str, after: &str, expected: bool) {
    let before = DocumentSyntax::parse(before).unwrap();
    let after = DocumentSyntax::parse(after).unwrap();
    let correspondence = correspondence(&before, &after).unwrap();
    assert_eq!(
        super::comments_preserved(&before, &after, &correspondence),
        expected
    );
}

fn scalar(
    value: &str,
    style: ScalarStyle,
    position: super::ScalarPosition,
    indent: usize,
) -> String {
    super::scalar(value, style, position, indent, usize::MAX).unwrap()
}

#[test_case(ScalarStyle::Plain)]
#[test_case(ScalarStyle::SingleQuoted)]
#[test_case(ScalarStyle::Literal)]
#[test_case(ScalarStyle::Folded)]
fn scalar_emission_preserves_representable_line_breaks(style: ScalarStyle) {
    // Flow folding and block chomping:
    // https://yaml.org/spec/1.2.2/#65-line-folding
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
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
                let text = scalar(
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
                let stream = astral_yaml::parse(&source).unwrap();
                let mut node = stream.node(stream.documents()[0].root).unwrap();
                if let astral_yaml::NodeKind::Sequence { items, .. } = &node.kind {
                    node = stream.node(items[0]).unwrap();
                } else if let astral_yaml::NodeKind::Mapping { entries, .. } = &node.kind {
                    node = stream.node(entries[0].value).unwrap();
                }
                assert!(
                    matches!(&node.kind, astral_yaml::NodeKind::Scalar(scalar) if scalar.value == value && scalar.style == style),
                    "{style:?}, {flow}, {indent}, {value:?} -> {text:?}"
                );
            }
        }
    }
}

#[test_case(ScalarStyle::Plain)]
#[test_case(ScalarStyle::SingleQuoted)]
fn flow_scalars_quote_whitespace_that_cannot_be_folded(style: ScalarStyle) {
    for value in ["a \nb", "a\n b", "a\t\nb", "a\n\tb", "a\rb", "a\0b"] {
        let text = scalar(value, style, super::ScalarPosition::Block, 2);
        let parsed = astral_yaml::parse(&text).unwrap();
        let node = parsed.node(parsed.documents()[0].root).unwrap();
        assert!(matches!(&node.kind, astral_yaml::NodeKind::Scalar(scalar)
            if scalar.value == value && scalar.style == ScalarStyle::DoubleQuoted));
    }
}

#[test_case(ScalarStyle::Literal)]
#[test_case(ScalarStyle::Folded)]
fn block_scalars_preserve_indented_lines(style: ScalarStyle) {
    for value in [" a\nb", "a\n b\nc", "a\n\n b\n\nc", "a\n \nb", "\ta\nb"] {
        let text = scalar(value, style, super::ScalarPosition::Block, 2);
        let source = format!("  value: {text}");
        let parsed = astral_yaml::parse(&source).unwrap();
        let node = parsed
            .nodes()
            .iter()
            .find(|node| {
                matches!(&node.kind,
            astral_yaml::NodeKind::Scalar(scalar) if scalar.value == value)
            })
            .unwrap();
        assert!(
            matches!(&node.kind, astral_yaml::NodeKind::Scalar(scalar)
            if scalar.style == style),
            "{value:?} -> {text:?}"
        );
    }
}

#[test_case("42", 0, 100, "emission resource limit exceeded")]
#[test_case("[42]", 1, 100, "emission resource limit exceeded")]
#[test_case("42", 100, 1, "emitted YAML exceeds input byte limit")]
#[test_case(
    indoc! {r"
        a: old

        # detached

        b: new
    "},
    100,
    10,
    "emitted YAML exceeds input byte limit";
    "mapping gap bytes"
)]
#[test_case(
    indoc! {r"
        - old

        # detached

        - new
    "},
    100,
    10,
    "emitted YAML exceeds input byte limit";
    "sequence gap bytes"
)]
#[test_case(
    indoc! {r"
        # leading
        a: value
    "},
    100,
    3,
    "emitted YAML exceeds input byte limit";
    "leading comment bytes"
)]
#[test_case(
    indoc! {r"
        a: # note
          - value
    "},
    100,
    3,
    "emitted YAML exceeds input byte limit";
    "nested comment bytes"
)]
fn rendering_obeys_work_and_output_budgets(
    source: &str,
    nodes: usize,
    bytes: usize,
    expected: &str,
) {
    let mut document = DocumentSyntax::parse(source).unwrap();
    document.source.options.max_nodes = nodes;
    document.source.options.max_input_bytes = bytes;
    let source = &document.source;
    let draft = Draft::from(&document).freeze().unwrap();
    let region = splice::region(&document, source.root, OuterComments::Exclude);
    // Both committed syntax and drafts obey the same rendering limits.
    for result in [
        splice::render(&document, source.root, region),
        splice::render(source.emission((&draft).into()), source.root, region),
    ] {
        assert_eq!(result.unwrap_err().message, expected);
    }
}

#[test_case(
    indoc! {r"
        first
        ---
        second"},
    false;
    "document start in block value"
)]
#[test_case(
    indoc! {r"
        ]
        ---
        ["},
    true;
    "brackets in flow value"
)]
#[test_case("key]: value #", true)]
#[test_case("first, second", true)]
#[test_case("] # empty sequence", true)]
#[test_case("first\r- second", false)]
#[test_case("first\r---\rsecond", false)]
fn plain_scalar_fallback_cannot_introduce_documents_or_collections(value: &str, flow: bool) {
    let output = scalar(
        value,
        ScalarStyle::Plain,
        if flow {
            super::ScalarPosition::Flow
        } else {
            super::ScalarPosition::Block
        },
        0,
    );
    let parsed = astral_yaml::parse(&output).unwrap();
    assert_eq!(parsed.documents().len(), 1, "{output:?}");
    let root = parsed.node(parsed.documents()[0].root).unwrap();
    assert!(matches!(&root.kind, astral_yaml::NodeKind::Scalar(scalar) if scalar.value == value));
}

#[test]
fn normalization_uses_the_target_anchor_and_rejects_an_unanchored_target() {
    let source = "[&old value, *old]";
    let mut document = DocumentSyntax::parse(source).unwrap();
    let target = document
        .source
        .nodes
        .iter()
        .find_map(|node| match node.kind {
            NodeKind::Alias { target } => Some(target),
            _ => None,
        })
        .unwrap();
    document.source.nodes[target.index()].anchor = Some("new".to_owned());
    insta::assert_snapshot!(document.normalized_yaml().unwrap(), @"[&new value, *new]");

    document.source.nodes[target.index()].anchor = None;
    insta::assert_snapshot!(document.normalized_yaml().unwrap_err().message, @"alias target has no anchor");
}

#[test]
fn normalization_rejects_an_alias_that_would_bind_to_a_different_definition() {
    let mut document = DocumentSyntax::parse("[&a 1, &a 2, *a]").unwrap();
    let first = astral_yaml::NodeId::from_index(
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
    let mut document = DocumentSyntax::parse(indoc! {r"
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
    comment.placement = astral_yaml::comments::CommentPlacement::Trailing;
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
    let mut document = DocumentSyntax::parse("value").unwrap();
    let fragment = DocumentSyntax::parse("removed # comment").unwrap();
    let removed = fragment.source.nodes[fragment.source.root.index()].clone();
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
        let document = DocumentSyntax::parse(source).unwrap();
        assert_eq!(document.to_yaml(), source);
        let output = document.normalized_yaml().expect(source);
        // Location types must not affect output.
        assert_eq!(render_draft(&document, Draft::from(&document)), output);
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
            let mut document = DocumentSyntax::parse(source).unwrap();
            if let NodeKind::Scalar { value, .. } =
                &mut document.source.nodes[document.source.root.index()].kind
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
            let text = scalar(value, ScalarStyle::Plain, position, 0);
            let source = if position == ScalarPosition::BlockKey {
                format!("{text}: value")
            } else {
                format!("{{{text}: value}}")
            };
            let parsed = astral_yaml::parse(&source).unwrap();
            let root = parsed.node(parsed.documents()[0].root).unwrap();
            assert!(
                matches!(&root.kind, astral_yaml::NodeKind::Mapping { entries, .. }
                if entries.len() == 1)
            );
            let key = parsed
                .nodes()
                .iter()
                .find_map(|node| {
                    if let astral_yaml::NodeKind::Scalar(key) = &node.kind {
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
    let mut document = DocumentSyntax::parse(source).unwrap();
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

#[test_case(
    indoc! {r"
        # note

        value
    "},
    indoc! {r"
        value

        # note
    "},
    false;
    "prefix versus suffix"
)]
#[test_case(
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
    false;
    "moved mapping gap"
)]
#[test_case(
    indoc! {r"
        # note

        - old
    "},
    indoc! {r"
        [
        # note

        old]
    "},
    true;
    "sequence prefix across styles"
)]
#[test_case(
    indoc! {r"
        # note

        a: old
    "},
    indoc! {r"
        {
        # note

        a: old}
    "},
    true;
    "mapping prefix across styles"
)]
#[test_case(
    indoc! {r"
        # note

        value
    "},
    indoc! {r"
        # note

        'value'
    "},
    true;
    "scalar prefix across styles"
)]
fn detached_comments_preserve_their_structural_boundaries(
    before: &str,
    after: &str,
    expected: bool,
) {
    let before = DocumentSyntax::parse(before).unwrap();
    let after = DocumentSyntax::parse(after).unwrap();
    let correspondence = correspondence(&before, &after).unwrap();
    assert_eq!(
        super::comments_preserved(&before, &after, &correspondence),
        expected
    );
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
        let document = DocumentSyntax::parse(source).unwrap();
        // Including outer comments exercises the same emitter path used by
        // imported fragments, which have no retained source prefix.
        let span = splice::region(&document, document.source.root, OuterComments::Include);
        let rendered = splice::render(&document, document.source.root, span).unwrap();
        let parsed = DocumentSyntax::parse(&rendered.text).unwrap();
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
        let document = DocumentSyntax::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        assert_eq!(
            DocumentSyntax::parse(&output)
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
        let mut candidate = DocumentSyntax::parse(source).unwrap();
        candidate.source.nodes[candidate.source.root.index()].kind = NodeKind::Scalar {
            value: String::new(),
            style: ScalarStyle::Plain,
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
        let original = DocumentSyntax::parse(source).unwrap();
        let mut candidate = Draft::from(&original);
        candidate
            .insert_comment(
                candidate.root(),
                0,
                " added",
                astral_yaml::comments::CommentPlacement::Trailing,
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
        let mut candidate = DocumentSyntax::parse(source).unwrap();
        candidate.source.nodes[candidate.source.root.index()].kind = NodeKind::Scalar {
            value: "old".to_owned(),
            style: ScalarStyle::Literal,
        };
        let region = splice::region(&candidate, candidate.source.root, OuterComments::Include);
        let rendered = splice::render(&candidate, candidate.source.root, region).unwrap();
        let output = format!(
            "{}{}{}",
            &source[..rendered.span.start()],
            rendered.text,
            &source[rendered.span.end()..]
        );
        let parsed = DocumentSyntax::parse(&output).unwrap();
        let matched = correspondence(&candidate, &parsed).unwrap();
        assert!(super::comments_preserved(&candidate, &parsed, &matched));
        outputs.push(output);
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "  |3-\n  old\n# tail\n",
        "    |3-\n  old\n# tail\n",
        "  |3- # inline\n  old\n# tail\n",
        "  &a |3- # inline\n  old\n# tail\n",
    ]
    "#);
}

#[test]
fn omitted_roots_own_their_line_when_comments_are_emitted() {
    let mut outputs = Vec::new();
    for source in ["---\nold\n...\n", "--- old\n...\n"] {
        let original = DocumentSyntax::parse(source).unwrap();
        let mut candidate = Draft::from(&original);
        candidate
            .replace(candidate.root(), "---", ReplacementStyle::FromValue)
            .unwrap();
        candidate
            .insert_comment(
                candidate.root(),
                0,
                " added",
                astral_yaml::comments::CommentPlacement::Leading,
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
            astral_yaml::comments::CommentPlacement::Trailing,
            astral_yaml::comments::CommentPlacement::Detached,
        ] {
            let original = DocumentSyntax::parse(source).unwrap();
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
                )
                .unwrap();
            diffs.push(astral_yaml_dev::edit_diff(
                source,
                &render_draft(&original, candidate),
            ));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1 +1,4 @@
    -outer: [old] # inline
    +outer: [ # inline
    +  old,
    +  # added
    +  ]

    @@ -1 +1,5 @@
    -outer: [old] # inline
    +outer: [ # inline
    +  old,
    +  ⏎
    +  # added
    +  ]

    @@ -1 +1,4 @@
    -outer: {key: old} # inline
    +outer: { # inline
    +  key: old,
    +  # added
    +  }

    @@ -1 +1,5 @@
    -outer: {key: old} # inline
    +outer: { # inline
    +  key: old,
    +  ⏎
    +  # added
    +  }
    ");
}
