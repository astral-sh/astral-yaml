//! Owned syntax retains YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.
use astral_yaml::{
    ParseOptions,
    comments::{CommentIndex, CommentPlacement},
    edit::{EditableDocument, NodeKind},
};
use astral_yaml_dev::indoc;

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
fn editable_sources_normalize_line_breaks_before_assigning_spans() {
    // YAML line breaks normalize to LF: https://yaml.org/spec/1.2.2/#54-line-break-characters
    let source = indoc! {r#"
        # before
        key: &a 'é' # inline

        copy: *a
        escaped: "\r\n"
    "#};
    let expected = EditableDocument::parse(source).unwrap();
    // Line-ending variation belongs at the input boundary, rather than in every
    // structural edit test. Include mixed and consecutive physical line breaks.
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
