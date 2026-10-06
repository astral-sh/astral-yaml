use astral_yaml::{ParseOptions, edit::EditableDocument};

#[test]
fn normalization_preserves_header_comments_before_leading_comments() {
    // Multiline comments: https://yaml.org/spec/1.2.2/#66-comments
    let mut outputs = Vec::new();
    for source in [
        "key: # header\n  # leading\n  value\n",
        "- # header\n  # leading\n  value\n",
        "{key: # header\n  # leading\n  value}\n",
        "# before\nkey: # header\n  # leading\n  value # inline\n",
        "key: # header\n  # leading\n  | # scalar\n    text\n",
        "key: # header\n  # leading\n  &a # anchor\n  value # inline\n",
    ] {
        outputs.push(
            EditableDocument::parse(source)
                .unwrap()
                .normalized_yaml()
                .unwrap(),
        );
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "key: # header\n  # leading\n  value\n",
        "- # header\n  # leading\n  value\n",
        "{key: # header\n  # leading\n  value}\n",
        "# before\nkey: # header\n  # leading\n  value # inline\n",
        "key: # header\n  # leading\n  |2 # scalar\n  text\n",
        "key: # header\n  # leading\n  &a # anchor\n    value # inline\n",
    ]
    "##);
}

#[test]
fn normalization_retains_multiline_scalar_styles() {
    let outputs: Vec<_> = ["a\n\nb\n", "'a\n\nb'\n", "|+\n\n", ">+\n\n"]
        .into_iter()
        .map(|source| {
            EditableDocument::parse(source)
                .unwrap()
                .normalized_yaml()
                .unwrap()
        })
        .collect();
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "a\n\n  b\n",
        "'a\n\n  b'\n",
        "|2+\n  \n",
        ">2+\n  \n",
    ]
    "#);
}

#[test]
fn normalized_block_scalars_use_their_parent_indentation() {
    let outputs: Vec<_> = [
        "  |2-\n  first\n  second\n",
        "  &anchor |2-\n  first\n  second\n",
        "outer:\n  inner: |-\n    first\n    second\n",
        "outer:\n  - |+\n    \n",
    ]
    .into_iter()
    .map(|source| {
        EditableDocument::parse(source)
            .unwrap()
            .normalized_yaml()
            .unwrap()
    })
    .collect();
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "  |2-\n  first\n  second\n",
        "  &anchor |2-\n  first\n  second\n",
        "outer:\n  inner: |2-\n    first\n    second\n",
        "outer:\n  - |2+\n    \n",
    ]
    "#);
}

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
        "a: !<tag:yaml.org,2002:seq> # anchor\n  &a # opening\n  [item] # closing\n",
        "a: [ # opening\n  item] # closing\n",
        "a: { # opening\n  key: value} # closing\n",
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

#[test]
fn normalization_consumes_interleaved_property_comments_in_order() {
    // Node properties and comments: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut outputs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "key: # header\n  &a # anchor\n  # leading\n  value\n",
            "- # header\n  &a # anchor\n  # leading\n  value\n",
            "key: # header\n  !!str # tag\n  &a # anchor\n  # leading\n  value\n",
            "key: # header\n  # before anchor\n  &a # anchor\n  # before value\n  value\n",
            "key: # header\n  &a # anchor\n  # leading\n  |\n    value\n",
            "{key: # header\n  &a # anchor\n  # leading\n  [value]}\n",
        ] {
            let document = EditableDocument::parse(&source.replace('\n', newline)).unwrap();
            outputs.push(document.normalized_yaml().unwrap());
        }
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "key: # header\n  &a # anchor\n    # leading\n    value\n",
        "- # header\n  &a # anchor\n    # leading\n    value\n",
        "key: # header\n  !<tag:yaml.org,2002:str> # tag\n    &a # anchor\n    # leading\n    value\n",
        "key: # header\n  # before anchor\n  &a # anchor\n    # before value\n    value\n",
        "key: # header\n  &a # anchor\n    # leading\n    |2\n  value\n",
        "{key: # header\n  &a # anchor\n  # leading\n  [value]}\n",
        "key: # header\r\n  &a # anchor\r\n    # leading\r\n    value\r\n",
        "- # header\r\n  &a # anchor\r\n    # leading\r\n    value\r\n",
        "key: # header\r\n  !<tag:yaml.org,2002:str> # tag\r\n    &a # anchor\r\n    # leading\r\n    value\r\n",
        "key: # header\r\n  # before anchor\r\n  &a # anchor\r\n    # before value\r\n    value\r\n",
        "key: # header\r\n  &a # anchor\r\n    # leading\r\n    |2\r\n  value\r\n",
        "{key: # header\r\n  &a # anchor\r\n  # leading\r\n  [value]}\r\n",
        "key: # header\r  &a # anchor\r    # leading\r    value\r",
        "- # header\r  &a # anchor\r    # leading\r    value\r",
        "key: # header\r  !<tag:yaml.org,2002:str> # tag\r    &a # anchor\r    # leading\r    value\r",
        "key: # header\r  # before anchor\r  &a # anchor\r    # before value\r    value\r",
        "key: # header\r  &a # anchor\r    # leading\r    |2\r  value\r",
        "{key: # header\r  &a # anchor\r  # leading\r  [value]}\r",
    ]
    "#);
}

#[test]
fn normalization_preserves_block_styles_in_explicit_keys() {
    // Explicit entries: https://yaml.org/spec/1.2.2/#822-block-mappings
    let mut outputs = Vec::new();
    for source in [
        "? |2-\n  key\n: value\n",
        "? >2-\n  key\n: value\n",
        "? - a\n  - b\n: value\n",
        "? k: v\n: value\n",
        "outer:\n  ? |2-\n    key\n  : value\n",
        "? &key\n  - a\n: value\n",
        "? # header\n  # leading\n  |2-\n  key\n: value\n",
    ] {
        let document = EditableDocument::parse(source).unwrap();
        outputs.push(document.normalized_yaml().unwrap());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "? |2-\n  key\n: value\n",
        "? >2-\n  key\n: value\n",
        "?\n  - a\n  - b\n: value\n",
        "?\n  k: v\n: value\n",
        "outer:\n  ? |2-\n    key\n  : value\n",
        "?\n  &key \n  - a\n: value\n",
        "? # header\n  # leading\n  |2-\n  key\n: value\n",
    ]
    "#);
}

#[test]
fn normalization_preserves_framing_and_collection_comment_positions() {
    let mut outputs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "# config\n%YAML 1.2\n---\nkey: old\n...\n  # after\n",
            "# config\n--- &root\n- old\n...\n",
            "key: old # inline\n# tail\n",
            "a: old\n\n# detached\n\nb: |+\n  text\n\n",
            "a: |+\n  text\n\n# detached\n\nb: old\n",
            "- old\n\n# detached\n\n- |+\n  text\n\n",
        ] {
            let source = source.replace('\n', newline);
            let document = EditableDocument::parse(&source).unwrap();
            let output = document.normalized_yaml().unwrap();
            // Normalization reaches a fixed point, including source separators.
            assert_eq!(
                EditableDocument::parse(&output)
                    .unwrap()
                    .normalized_yaml()
                    .unwrap(),
                output
            );
            outputs.push(output);
        }
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "# config\n%YAML 1.2\n---\nkey: old\n...\n  # after\n",
        "# config\n--- &root \n- old\n...\n",
        "key: old # inline\n# tail\n",
        "a: old\n\n# detached\n\nb: |2+\n  text\n  \n",
        "a: |2+\n  text\n  \n# detached\n\nb: old\n",
        "- old\n\n# detached\n\n- |2+\n  text\n  \n",
        "# config\r\n%YAML 1.2\r\n---\r\nkey: old\r\n...\r\n  # after\r\n",
        "# config\r\n--- &root \r\n- old\r\n...\r\n",
        "key: old # inline\r\n# tail\r\n",
        "a: old\r\n\r\n# detached\r\n\r\nb: |2+\r\n  text\r\n  \r\n",
        "a: |2+\r\n  text\r\n  \r\n# detached\r\n\r\nb: old\r\n",
        "- old\r\n\r\n# detached\r\n\r\n- |2+\r\n  text\r\n  \r\n",
        "# config\r%YAML 1.2\r---\rkey: old\r...\r  # after\r",
        "# config\r--- &root \r- old\r...\r",
        "key: old # inline\r# tail\r",
        "a: old\r\r# detached\r\rb: |2+\r  text\r  \r",
        "a: |2+\r  text\r  \r# detached\r\rb: old\r",
        "- old\r\r# detached\r\r- |2+\r  text\r  \r",
    ]
    "##);
}
