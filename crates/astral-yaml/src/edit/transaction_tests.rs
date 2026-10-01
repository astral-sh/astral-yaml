use super::{EditableDocument, Editor, NodeKind, Span, same_style};

#[test]
fn overlapping_rewrite_regions_expand_to_their_common_ancestor() {
    let mut document = EditableDocument::parse("[[a, b], c]").unwrap();
    let NodeKind::Sequence { items: outer, .. } = document.nodes[document.root.0].kind.clone()
    else {
        panic!("sequence")
    };
    let NodeKind::Sequence { items: inner, .. } = document.nodes[outer[0].0].kind.clone() else {
        panic!("sequence")
    };
    // The patch planner must coalesce overlapping retained ranges, including
    // ranges expanded by trivia ownership, before slicing the original source.
    document.nodes[inner[0].0].span = Span::new(2, 10).unwrap();
    let mut editor = Editor::new(&document);
    editor.rewrites.extend([inner[0], outer[1]]);
    assert_eq!(
        editor.rewrite_regions().unwrap(),
        [(document.root, document.nodes[document.root.0].span)]
    );
}

#[test]
fn requested_style_comparison_rejects_a_different_node_kind() {
    let scalar = EditableDocument::parse("one").unwrap();
    let sequence = EditableDocument::parse("[one]").unwrap();
    assert!(!same_style(
        &scalar.nodes[scalar.root.0].kind,
        &sequence.nodes[sequence.root.0].kind
    ));
}
