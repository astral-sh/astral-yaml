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
    let NodeKind::Sequence { items, .. } = document.nodes[document.root.0].kind.clone() else {
        panic!("sequence")
    };
    let NodeKind::Alias { target, .. } = &mut document.nodes[items[2].0].kind else {
        panic!("alias")
    };
    // Model retaining the first definition while a later definition shadows it.
    // The renderer must fail rather than emit an alias with different meaning.
    *target = items[0];
    assert_eq!(
        document.normalized_yaml().unwrap_err().message,
        "emission changed YAML content or alias identity"
    );
}
