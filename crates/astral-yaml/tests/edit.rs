//! Owned syntax retains YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.
use astral_yaml::{
    ParseOptions,
    comments::{CommentIndex, CommentPlacement},
    edit::{EditableDocument, NodeKind},
};

#[test]
fn owned_document_retains_syntax_and_direct_comments() {
    let source =
        "%YAML 1.2\n---\n# before\nitems: &items [!!str 'é', 2] # inline\ncopy: *items\n...\n";
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
    for source in ["[", "", "---\n1\n---\n2"] {
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
