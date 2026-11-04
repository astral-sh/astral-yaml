//! The editor retains presentation details alongside the serialization tree.
//! See <https://yaml.org/spec/1.2.2/#323-presentation-stream>.

use crate::syntax::{DocumentSyntax, NodeKind};
use astral_yaml::ParseOptions;
use astral_yaml::comments::{CommentIndex, CommentPlacement};
use astral_yaml_dev::{indoc, span_snapshot};

#[test]
fn owned_document_retains_syntax_and_direct_comments() {
    // A node can have a tag, an anchor, or both. An alias refers back to an anchored node.
    // https://yaml.org/spec/1.2.2/#69-node-properties
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let source = indoc! {r"
        %YAML 1.2
        ---
        # before
        items: &items [!!str 'é', 2] # inline
        copy: *items
        ...
    "};
    let document = DocumentSyntax::parse(source).unwrap();
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
}

#[test]
fn alias_edges_distinguish_recursive_and_shadowed_definitions() {
    // Alias nodes refer to the most recent preceding definition of an anchor:
    // https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    let source = indoc! {r"
        - &item [first, *item]
        - *item
        - &item second
        - &item second
        - *item
    "};
    let document = DocumentSyntax::parse(source).unwrap();
    let NodeKind::Sequence { items, .. } = document.node(document.root()).unwrap().kind() else {
        panic!("sequence")
    };
    let NodeKind::Sequence { items: first, .. } = document.node(items[0]).unwrap().kind() else {
        panic!("sequence")
    };

    let target = |id| {
        let NodeKind::Alias { target } = document.node(id).unwrap().kind() else {
            panic!("alias")
        };
        *target
    };
    let before = items[1];
    let after = items[4];
    assert_eq!(target(first[1]), target(before));

    let span = |id| document.node(id).unwrap().span();
    let spans = [
        ("target before rebinding", span(target(before))),
        ("alias before rebinding", span(before)),
        ("target after rebinding", span(target(after))),
        ("alias after rebinding", span(after)),
    ];
    insta::assert_snapshot!(span_snapshot(source, spans), @"
      |
    1 | - &item [first, *item]
      |   ^^^^^^^^^^^^^^^^^^^^ target before rebinding
    2 | - *item
      |   ^^^^^ alias before rebinding
    3 | - &item second
    4 | - &item second
      |   ^^^^^^^^^^^^ target after rebinding
    5 | - *item
      |   ^^^^^ alias after rebinding
    ");
}

#[test]
fn construction_rejects_invalid_documents_and_limits() {
    // YAML streams permit zero or more documents. This editor requires exactly one.
    // https://yaml.org/spec/1.2.2/#92-streams
    for source in [
        "[",
        "",
        indoc! {r"
            ---
            1
            ---
            2"},
    ] {
        assert!(DocumentSyntax::parse(source).is_err());
    }
    let options = ParseOptions {
        max_input_bytes: 1,
        ..ParseOptions::default()
    };
    assert!(DocumentSyntax::parse_with_options("42", options).is_err());
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
fn editable_sources_normalize_line_breaks_before_assigning_spans() {
    // YAML normalizes physical breaks inside scalar content. The editor also stores LF source.
    // https://yaml.org/spec/1.2.2/#54-line-break-characters
    let source = indoc! {r#"
        # before
        key: &a 'é' # inline

        copy: *a
        escaped: "\r\n"
    "#};
    let expected = DocumentSyntax::parse(source).unwrap();
    // Include mixed and consecutive physical line breaks.
    for input in [
        source.to_owned(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
        source
            .replace("# before\n", "# before\r\n")
            .replace("copy: *a\n", "copy: *a\r"),
    ] {
        let document = DocumentSyntax::parse(&input).unwrap();
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
    let limited = DocumentSyntax::parse_with_options(
        "key: value\r\n",
        ParseOptions {
            max_input_bytes: "key: value\n".len(),
            ..ParseOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(limited.message, "input byte limit exceeded");
    let error = DocumentSyntax::parse("key: value\r\ninvalid: [\r\n").unwrap_err();
    assert_eq!(
        error,
        DocumentSyntax::parse("key: value\ninvalid: [\n").unwrap_err()
    );
}
