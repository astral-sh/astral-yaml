use astral_yaml::{ParseOptions, edit::EditableDocument};

#[test]
fn normalization_preserves_leading_comments_in_flow_collections() {
    // Comment separation: https://yaml.org/spec/1.2.2/#66-comments
    let mut outputs = Vec::new();
    for source in [
        "[\n  # first\n  one, two]\n",
        "[one,\n  # second\n  two]\n",
        "{\n  # first\n  one: 1, two: 2}\n",
        "{one: 1,\n  # second\n  two: 2}\n",
        "outer: [{\n  # nested\n  one: 1}]\n",
        "[one, # inline\n  # leading\n  two]\n",
    ] {
        let document = EditableDocument::parse(source).unwrap();
        outputs.push(document.normalized_yaml().unwrap());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "[\n  # first\n  one, two]\n",
        "[one, \n  # second\n  two]\n",
        "{\n  # first\n  one: 1, two: 2}\n",
        "{one: 1, \n  # second\n  two: 2}\n",
        "outer: [{\n    # nested\n    one: 1}]\n",
        "[one, # inline\n  # leading\n  two]\n",
    ]
    "#);
}

#[test]
fn normalization_preserves_empty_scalar_text() {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    for source in [
        "name:\n",
        "name: !custom\n",
        "{name: , other: null}\n",
        "-\n- !!str\n",
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        let scalars = |source: &str| {
            astral_yaml::parse(source)
                .unwrap()
                .nodes()
                .iter()
                .filter_map(|node| match &node.kind {
                    astral_yaml::NodeKind::Scalar(scalar) => Some(scalar.value.to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            scalars(&output),
            scalars(source),
            "{source:?} -> {output:?}"
        );
    }
}

#[test]
fn normalization_preserves_separate_header_comments() {
    // Node properties: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut outputs = Vec::new();
    for source in [
        "a: &a # anchor\n  | # header\n    text\n",
        "a: &a # anchor\n  !!str # tag\n  | # header\n    text\n",
        "a: # field\n  &a # anchor\n  !!seq # tag\n  - item\n",
        "a: &a # anchor\n  !!seq [ # opening\n    item] # closing\n",
        "a: [ # opening\n  item] # closing\n",
        "a: { # opening\n  key: value} # closing\n",
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        let comments = |source: &str| {
            let stream = astral_yaml::parse(source).unwrap();
            let index = astral_yaml::comments::CommentIndex::new(&stream, 0).unwrap();
            let mut comments = index
                .direct()
                .map(|(id, comment)| {
                    (
                        format!("{id:?}"),
                        comment.placement as u8,
                        comment.comment.text.to_owned(),
                    )
                })
                .collect::<Vec<_>>();
            comments.sort();
            comments
        };
        assert_eq!(
            comments(&output),
            comments(source),
            "{source:?} -> {output:?}"
        );
        outputs.push(output);
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "a: &a # anchor\n  |2 # header\n  text\n",
        "a: !<tag:yaml.org,2002:str> # anchor\n  &a # tag\n  |2 # header\n  text\n",
        "a: # field\n  !<tag:yaml.org,2002:seq> # anchor\n  &a # tag\n  - item\n",
        "a: !<tag:yaml.org,2002:seq> # anchor\n  &a # opening\n  [item] # closing\n\n",
        "a: [ # opening\n  item] # closing\n\n",
        "a: { # opening\n  key: value} # closing\n\n",
    ]
    "#);
}

#[test]
fn normalization_uses_explicit_keys_when_escaped_syntax_is_long() {
    // Implicit keys: https://yaml.org/spec/1.2.2/#832-block-mappings
    for key in [
        format!("\"{}\"", "\\u0001".repeat(103)),
        format!("&{} short", "a".repeat(1024)),
        format!("!<tag:example.org,2026:{}> short", "a".repeat(1024)),
        format!("\"{}\"", "a".repeat(1023)),
    ] {
        let source = format!("? {key}\n: value\n");
        let output = EditableDocument::parse(&source)
            .unwrap()
            .normalized_yaml()
            .unwrap();
        assert!(output.starts_with("? "), "{output:?}");
        assert_eq!(
            EditableDocument::parse(&output)
                .unwrap()
                .normalized_yaml()
                .unwrap(),
            output
        );
    }
}

#[test]
fn normalization_preserves_empty_scalar_resolution_and_separates_markers() {
    for source in ["---", "---\n", "!!str", "!", "[!!str , ! ]", "a:", "-"] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        let reparsed = EditableDocument::parse(&output).unwrap();
        assert_eq!(reparsed.normalized_yaml().unwrap(), output);
    }
    // Syntax inspection accepts invalid core payloads. Normalization retains
    // their text without interpreting or repairing it.
    let output = EditableDocument::parse("!!bool")
        .unwrap()
        .normalized_yaml()
        .unwrap();
    let parsed = astral_yaml::parse(&output).unwrap();
    assert!(
        matches!(&parsed.node(parsed.documents()[0].root).unwrap().kind,
        astral_yaml::NodeKind::Scalar(scalar) if scalar.value.is_empty())
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
