use test_case::test_case;

use super::support::normalize;
use crate::syntax::DocumentSyntax;
use astral_yaml::ParseOptions;
use astral_yaml_dev::{edit_diff, formatdoc, indoc};

#[test]
fn normalization_preserves_header_comments_before_leading_comments() {
    // Multiline comments: https://yaml.org/spec/1.2.2/#66-comments
    let outputs = [
        indoc! {r"
            key: # header
              # leading
              value
        "},
        indoc! {r"
            - # header
              # leading
              value
        "},
        indoc! {r"
            {key: # header
              # leading
              value}
        "},
        indoc! {r"
            # before
            key: # header
              # leading
              value # inline
        "},
        indoc! {r"
            key: # header
              # leading
              | # scalar
                text
        "},
        indoc! {r"
            key: # header
              # leading
              &a # anchor
              value # inline
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"




    @@ -1,4 +1,4 @@
     key: # header
       # leading
       | # scalar
    -    text
    +  text

    @@ -1,4 +1,4 @@
     key: # header
       # leading
       &a # anchor
    -  value # inline
    +    value # inline
    ");
}

#[test]
fn normalization_retains_multiline_scalar_styles() {
    // Blank lines can encode line feeds in flow scalars. Keep chomping retains trailing breaks.
    // https://yaml.org/spec/1.2.2/#65-line-folding
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let outputs = [
        indoc! {r"
            a

            b
        "},
        indoc! {r"
            'a

            b'
        "},
        indoc! {r"
            |+

        "},
        indoc! {r"
            >+

        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"
    @@ -1,3 +1,3 @@
     a
     ⏎
    -b
    +  b

    @@ -1,3 +1,3 @@
     'a
     ⏎
    -b'
    +  b'

    @@ -1,2 +1,2 @@
    -|+
    -
    +|3+
    +  ⏎

    @@ -1,2 +1,2 @@
    ->+
    -
    +>3+
    +  ⏎
    ");
}

// Block indentation indicators: https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
#[test_case("|\n  hello\n"; "root literal")]
#[test_case("message: >-\n  \n  first\n    second\n"; "leading empty line")]
#[test_case("|-\n  \ttext\n"; "leading tab at the root")]
#[test_case("message: >2-\n  \n  \tfirst\n    second\n"; "leading tab in a nested scalar")]
#[test_case("- |\n  first\n   second\n"; "later content starts with a space")]
#[test_case("|3\n  \n   hello\n"; "leading space after an empty line")]
#[test_case("message: >2-\n   \n  next\n"; "space-only line before text")]
#[test_case("|3-\n# tail\n"; "empty scalar before a comment")]
#[test_case("|3+\n  \n# tail\n"; "only line breaks before a comment")]
fn normalized_block_scalar_indicators(source: &str) {
    assert_eq!(normalize(source), "");
}

#[test]
fn normalized_block_scalars_use_their_parent_indentation() {
    // The indentation indicator is relative to the indentation level of the block scalar.
    // https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
    let outputs = [
        r"  |2-
  first
  second
",
        r"  &anchor |2-
  first
  second
",
        indoc! {r"
            outer:
              inner: |1-
                first
                second
        "},
        indoc! {"
            outer:
              - |+
            \x20\x20\x20\x20
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"
    @@ -1,3 +1,3 @@
    -  |2-
    -  first
    -  second
    +  |3-
    +   first
    +   second

    @@ -1,3 +1,3 @@
    -  &anchor |2-
    -  first
    -  second
    +  &anchor |3-
    +   first
    +   second

    @@ -1,4 +1,4 @@
     outer:
    -  inner: |1-
    -    first
    -    second
    +  inner: |2-
    +     first
    +     second

    @@ -1,3 +1,3 @@
     outer:
    -  - |+
    +  - |2+
         ⏎
    ");
}

#[test]
fn normalization_preserves_leading_comments_in_flow_collections() {
    // Comment separation: https://yaml.org/spec/1.2.2/#66-comments
    let outputs = [
        indoc! {r"
            [
              # first
              one, two]
        "},
        indoc! {r"
            [one,
              # second
              two]
        "},
        indoc! {r"
            {
              # first
              one: 1, two: 2}
        "},
        indoc! {r"
            {one: 1,
              # second
              two: 2}
        "},
        indoc! {r"
            outer: [{
              # nested
              one: 1}]
        "},
        indoc! {r"
            [one, # inline
              # leading
              two]
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"

    @@ -1,3 +1,3 @@
    -[one,
    +[one, ⏎
       # second
       two]


    @@ -1,3 +1,3 @@
    -{one: 1,
    +{one: 1, ⏎
       # second
       two: 2}

    @@ -1,3 +1,3 @@
     outer: [{
    -  # nested
    -  one: 1}]
    +    # nested
    +    one: 1}]
    ");
}

#[test_case(
    indoc! {r"
        name:
    "};
    "implicit null"
)]
#[test_case(
    indoc! {r"
        name: !custom
    "};
    "application tagged null"
)]
#[test_case(
    indoc! {r"
        {name: , other: null}
    "};
    "flow omitted value"
)]
#[test_case(
    indoc! {r"
        -
        - !!str
    "};
    "sequence omitted and string"
)]
fn normalization_preserves_empty_scalar_text(source: &str) {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    let document = DocumentSyntax::parse(source).unwrap();
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

#[test]
fn normalization_preserves_separate_header_comments() {
    // Node properties: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut outputs = Vec::new();
    for source in [
        indoc! {r"
            a: &a # anchor
              | # header
                text
        "},
        indoc! {r"
            a: &a # anchor
              !!str # tag
              | # header
                text
        "},
        indoc! {r"
            a: # field
              &a # anchor
              !!seq # tag
              - item
        "},
        indoc! {r"
            a: &a # anchor
              !!seq [ # opening
                item] # closing
        "},
        indoc! {r"
            a: [ # opening
              item] # closing
        "},
        indoc! {r"
            a: { # opening
              key: value} # closing
        "},
    ] {
        let document = DocumentSyntax::parse(source).unwrap();
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
        outputs.push(edit_diff(source, &output));
    }
    insta::assert_snapshot!(outputs.join("\n"), @"
    @@ -1,3 +1,3 @@
     a: &a # anchor
       | # header
    -    text
    +  text

    @@ -1,4 +1,4 @@
    -a: &a # anchor
    -  !!str # tag
    +a: !<tag:yaml.org,2002:str> # anchor
    +  &a # tag
       | # header
    -    text
    +  text

    @@ -1,4 +1,4 @@
     a: # field
    -  &a # anchor
    -  !!seq # tag
    +  !<tag:yaml.org,2002:seq> # anchor
    +  &a # tag
       - item

    @@ -1,3 +1,3 @@
    -a: &a # anchor
    -  !!seq [ # opening
    -    item] # closing
    +a: !<tag:yaml.org,2002:seq> # anchor
    +  &a [ # opening
    +  item] # closing
    ");
}

#[test_case(&format!("\"{}\"", "\\x01".repeat(171)))]
#[test_case(&format!("&{} short", "a".repeat(1024)))]
#[test_case(&format!("!<tag:example.org,2026:{}> short", "a".repeat(1024)))]
#[test_case(&format!("\"{}\"", "a".repeat(1023)))]
fn normalization_uses_explicit_keys_when_escaped_syntax_is_long(key: &str) {
    // Implicit keys: https://yaml.org/spec/1.2.2/#822-block-mappings
    let source = formatdoc! {r"
        ? {key}
        : value
    "};
    let output = DocumentSyntax::parse(&source)
        .unwrap()
        .normalized_yaml()
        .unwrap();
    assert!(output.starts_with("? "), "{output:?}");
    assert_eq!(
        DocumentSyntax::parse(&output)
            .unwrap()
            .normalized_yaml()
            .unwrap(),
        output
    );
}

#[test_case("&a  "; "trailing spaces")]
#[test_case("&a\t#keep\n"; "tab before a comment")]
fn normalization_preserves_separation_after_a_property_only_scalar(source: &str) {
    // A node can have properties and empty content. An inline comment needs separation.
    // https://yaml.org/spec/1.2.2/#72-empty-nodes
    // https://yaml.org/spec/1.2.2/#66-comments
    let document = DocumentSyntax::parse(source).unwrap();
    assert_eq!(document.normalized_yaml().unwrap(), source);
}

#[test]
fn normalization_preserves_empty_scalar_resolution_and_separates_markers() {
    // Omitted content is an empty scalar. An explicit document may consist of its marker alone.
    // https://yaml.org/spec/1.2.2/#72-empty-nodes
    // https://yaml.org/spec/1.2.2/#914-explicit-documents
    for source in [
        "---",
        indoc! {r"
            ---
        "},
        "!!str",
        "!",
        "[!!str , ! ]",
        "a:",
        "-",
    ] {
        let document = DocumentSyntax::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        let reparsed = DocumentSyntax::parse(&output).unwrap();
        assert_eq!(reparsed.normalized_yaml().unwrap(), output);
    }
    // Syntax inspection accepts invalid core payloads. Normalization retains
    // their text without interpreting or repairing it.
    let output = DocumentSyntax::parse("!!bool")
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
    let document = DocumentSyntax::parse_with_options(
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

#[test_case("|+", "\n"; "literal")]
#[test_case(">+", "  x\n\n"; "folded")]
#[test_case("'", "x\n\n"; "single quoted")]
fn multiline_edits_budget_continuation_indentation(header: &str, line: &str) {
    let indent = " ".repeat(2048);
    let source = format!(
        "{indent}k: {header}\n{indent} old{}\n",
        if header == "'" { "'" } else { "" }
    );
    let mut document = DocumentSyntax::parse_with_options(
        &source,
        ParseOptions {
            max_input_bytes: 65_536,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let fragment = if header == "'" {
        format!("'{}end'", line.repeat(2048))
    } else {
        format!("{header}\n{}", line.repeat(2048))
    };
    assert_eq!(
        document
            .edit_syntax(|e| {
                let key = e.entry_ids(e.root())?[0].1;
                e.replace_preserving_style(key, &fragment)
            })
            .unwrap_err()
            .message,
        "emitted YAML exceeds input byte limit"
    );
    assert_eq!(document.source(), source);
}

#[test]
fn normalization_consumes_interleaved_property_comments_in_order() {
    // Node properties and comments: https://yaml.org/spec/1.2.2/#69-node-properties
    let outputs = [
        indoc! {r"
            key: # header
              &a # anchor
              # leading
              value
        "},
        indoc! {r"
            - # header
              &a # anchor
              # leading
              value
        "},
        indoc! {r"
            key: # header
              !!str # tag
              &a # anchor
              # leading
              value
        "},
        indoc! {r"
            key: # header
              # before anchor
              &a # anchor
              # before value
              value
        "},
        indoc! {r"
            key: # header
              &a # anchor
              # leading
              |
                value
        "},
        indoc! {r"
            {key: # header
              &a # anchor
              # leading
              [value]}
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"
    @@ -1,4 +1,4 @@
     key: # header
       &a # anchor
    -  # leading
    -  value
    +    # leading
    +    value

    @@ -1,4 +1,4 @@
     - # header
       &a # anchor
    -  # leading
    -  value
    +    # leading
    +    value

    @@ -1,5 +1,5 @@
     key: # header
    -  !!str # tag
    -  &a # anchor
    -  # leading
    -  value
    +  !<tag:yaml.org,2002:str> # tag
    +    &a # anchor
    +    # leading
    +    value

    @@ -1,5 +1,5 @@
     key: # header
       # before anchor
       &a # anchor
    -  # before value
    -  value
    +    # before value
    +    value

    @@ -1,5 +1,5 @@
     key: # header
       &a # anchor
    -  # leading
    -  |
    -    value
    +    # leading
    +    |
    +  value
    ");
}

#[test]
fn normalization_preserves_block_styles_in_explicit_keys() {
    // Explicit entries: https://yaml.org/spec/1.2.2/#822-block-mappings
    let outputs = [
        indoc! {r"
            ? |2-
              key
            : value
        "},
        indoc! {r"
            ? >2-
              key
            : value
        "},
        indoc! {r"
            ? - a
              - b
            : value
        "},
        indoc! {r"
            ? k: v
            : value
        "},
        indoc! {r"
            outer:
              ? |2-
                key
              : value
        "},
        indoc! {r"
            ? &key
              - a
            : value
        "},
        indoc! {r"
            ? # header
              # leading
              |2-
              key
            : value
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"
    @@ -1,3 +1,3 @@
    -? |2-
    +? |-
       key
     : value

    @@ -1,3 +1,3 @@
    -? >2-
    +? >-
       key
     : value

    @@ -1,3 +1,4 @@
    -? - a
    +?
    +  - a
       - b
     : value

    @@ -1,2 +1,3 @@
    -? k: v
    +?
    +  k: v
     : value

    @@ -1,4 +1,4 @@
     outer:
    -  ? |2-
    +  ? |-
         key
       : value

    @@ -1,3 +1,4 @@
    -? &key
    +?
    +  &key ⏎
       - a
     : value

    @@ -1,5 +1,5 @@
     ? # header
       # leading
    -  |2-
    +  |-
       key
     : value
    ");
}

#[test]
fn normalization_preserves_framing_and_collection_comment_positions() {
    // A marker line or a separation line can carry a comment. The editor preserves their order.
    // https://yaml.org/spec/1.2.2/#912-document-markers
    // https://yaml.org/spec/1.2.2/#67-separation-lines
    let outputs = [
        indoc! {r"
            # config
            %YAML 1.2
            ---
            key: old
            ...
              # after
        "},
        indoc! {r"
            # config
            --- &root
            - old
            ...
        "},
        indoc! {r"
            key: old # inline
            # tail
        "},
        indoc! {r"
            a: old

            # detached

            b: |+
              text

        "},
        indoc! {r"
            a: |+
              text

            # detached

            b: old
        "},
        indoc! {r"
            - old

            # detached

            - |+
              text

        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @"

    @@ -1,4 +1,5 @@
     # config
    ---- &root
    +--- ⏎
    +&root ⏎
     - old
     ...


    @@ -4,4 +4,4 @@
     ⏎
     b: |+
       text
    -
    +  ⏎

    @@ -1,6 +1,6 @@
     a: |+
       text
    -
    +  ⏎
     # detached
     ⏎
     b: old

    @@ -4,4 +4,4 @@
     ⏎
     - |+
       text
    -
    +  ⏎
    ");
}

#[test]
fn normalization_preserves_explicit_scalar_types() {
    // A shorthand tag and its expanded verbatim form identify the same tag.
    // https://yaml.org/spec/1.2.2/#691-node-tags
    let outputs = [
        "!!str 42",
        "!!null null",
        "!!bool true",
        "!!int '42'",
        "!!float 1.0",
    ]
    .map(normalize);
    insta::assert_snapshot!(outputs.join("\n"), @r"
    @@ -1 +1 @@
    -!!str 42
    \ No newline at end of file
    +!<tag:yaml.org,2002:str> 42
    \ No newline at end of file

    @@ -1 +1 @@
    -!!null null
    \ No newline at end of file
    +!<tag:yaml.org,2002:null> null
    \ No newline at end of file

    @@ -1 +1 @@
    -!!bool true
    \ No newline at end of file
    +!<tag:yaml.org,2002:bool> true
    \ No newline at end of file

    @@ -1 +1 @@
    -!!int '42'
    \ No newline at end of file
    +!<tag:yaml.org,2002:int> '42'
    \ No newline at end of file

    @@ -1 +1 @@
    -!!float 1.0
    \ No newline at end of file
    +!<tag:yaml.org,2002:float> 1.0
    \ No newline at end of file
    ");
}
