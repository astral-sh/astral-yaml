use astral_yaml::{ParseOptions, edit::EditableDocument};

#[test]
fn normalization_preserves_empty_scalar_resolution_and_separates_markers() {
    for source in ["---", "!!str", "!", "[!!str , ! ]", "a:", "-"] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        let reparsed = EditableDocument::parse(&output).unwrap();
        assert_eq!(reparsed.normalized_yaml().unwrap(), output);
    }
    // Syntax inspection accepts an explicit tag with an invalid core payload.
    // Normalization must still reject a change to that payload's contents.
    assert!(
        EditableDocument::parse("!!bool")
            .unwrap()
            .normalized_yaml()
            .is_err()
    );
}

#[test]
fn normalization_respects_limits_on_output_expansion() {
    let source = "{a: 1,b: 2}";
    let document = EditableDocument::parse_with_options(
        source,
        ParseOptions {
            max_input_bytes: source.len(),
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        document.normalized_yaml().unwrap_err().message,
        "emitted YAML exceeds input byte limit"
    );
}
