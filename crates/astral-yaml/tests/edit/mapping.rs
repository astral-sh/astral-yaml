use test_case::test_case;

use super::support::{edit, edits};
use astral_yaml::{CollectionStyle, ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn flow_conversion_uses_content_indentation_after_properties() {
    let results = edits(
        [
            indoc! {r"
                outer: &mapping
                  items: &sequence
                    - x
                  untouched:   [a,b]
                copy: *sequence
            "},
            indoc! {r"
                outer: &mapping
                  items: &sequence
                  - x
                  untouched:   [a,b]
                copy: *sequence
            "},
        ],
        |editor| {
            let mapping = editor.mapping_entries(&editor.root())?[0].1.clone();
            let sequence = editor.mapping_entries(&mapping)?[0].1.clone();
            editor.set_collection_style(&sequence, CollectionStyle::Flow)
        },
    );
    insta::assert_snapshot!(results, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,4 @@
     outer: &mapping
    -  items: &sequence
    -    - x
    +  items: &sequence [x]
       untouched:   [a,b]
     copy: *sequence

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -outer: &mapping
    -  items: &sequence
    -  - x
    -  untouched:   [a,b]
    +outer:
    +  &mapping ⏎
    +  items: &sequence [x]
    +  untouched: [a, b]
     copy: *sequence
    ");
}

#[test]
fn edits_inside_complex_keys_share_the_enclosing_mapping_context() {
    let results = edits(
        [
            indoc! {r"
                ? - {nested: &a [old]}
                : *a
                outside:   [x,y]
            "},
            indoc! {r"
                {? [{nested: &a [old]}]: *a, outside:   [x,y]}
            "},
        ],
        |editor| {
            let key = editor.mapping_entries(&editor.root())?[0].0.clone();
            let mapping = editor.sequence_items(&key)?[0].clone();
            let sequence = editor.mapping_entries(&mapping)?[0].1.clone();
            let value = editor.sequence_items(&sequence)?[0].clone();
            editor.replace(&value, "changed")?;
            editor.insert_item(&sequence, 1, "{child: yes}").map(|_| ())
        },
    );
    insta::assert_snapshot!(results, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -? - {nested: &a [old]}
    +?
    +  - {nested: &a [changed, {child: yes}]}
     : *a
    -outside:   [x,y]
    +outside: [x, y]

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -{? [{nested: &a [old]}]: *a, outside:   [x,y]}
    +{? [{nested: &a [changed, {child: yes}]}]
    +  : *a, outside: [x, y]}
    ");
}

#[test]
fn fallback_quoted_keys_can_retain_adjacent_value_indicators() {
    let source = indoc! {r#"
        {"old":value, untouched:   [a,b]}
    "#};
    let diff = edit(source, |editor| {
        let key = editor.mapping_entries(&editor.root())?[0].0.clone();
        editor.replace(&key, "new, key")
    });
    insta::assert_snapshot!(diff, @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"old":value, untouched:   [a,b]}
    +{"new, key":value, untouched:   [a,b]}
    "#);
}

#[test]
fn omitted_keys_retain_entries_without_value_indicators() {
    // Empty flow keys need an indicator when their key token disappears:
    // https://yaml.org/spec/1.2.2/#742-flow-mappings
    let results = edits(
        [
            indoc! {"
                {old}
            "},
            indoc! {"
                {old, retained: untouched}
            "},
        ],
        |editor| {
            let key = editor.mapping_entries(&editor.root())?[0].0.clone();
            editor.replace(&key, "---")
        },
    );
    insta::assert_snapshot!(results, @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old}
    +{: }

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old, retained: untouched}
    +{: , retained: untouched}
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn indentless_sequences_can_become_empty_or_flow_values() {
    // Block sequences: https://yaml.org/spec/1.2.2/#821-block-sequences
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            items:
            - x
            other : kept
        "},
        indoc! {r"
            outer:
              items:
              - x
              other : kept
            outside : untouched
        "},
        indoc! {r"
            - items:
              - x
              other : kept
        "},
        indoc! {r"
            &mapping
            items:
            - x
            other : kept
        "},
        indoc! {r"
            items:
            - x
        "},
        indoc! {r"
            items:
            - x"},
    ] {
        for remove in [true, false] {
            diffs.push(edit(source, |editor| {
                let mut mapping = editor.root();
                if source.starts_with("outer:") {
                    mapping = editor.mapping_entries(&mapping)?.remove(0).1;
                } else if source.starts_with('-') {
                    mapping = editor.sequence_items(&mapping)?.remove(0);
                }
                let sequence = editor.mapping_entries(&mapping)?.remove(0).1;
                if remove {
                    editor.remove_item(&sequence, 0)
                } else {
                    editor.set_collection_style(&sequence, CollectionStyle::Flow)
                }
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -items:
    -- x
    -other : kept
    +items: []
    +other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -items:
    -- x
    -other : kept
    +items: [x]
    +other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,4 @@
     outer:
    -  items:
    -  - x
    -  other : kept
    +  items: []
    +  other: kept
     outside : untouched

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,4 @@
     outer:
    -  items:
    -  - x
    -  other : kept
    +  items: [x]
    +  other: kept
     outside : untouched

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -- items:
    -  - x
    -  other : kept
    +-
    +  items: []
    +  other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -- items:
    -  - x
    -  other : kept
    +-
    +  items: [x]
    +  other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    -&mapping
    -items:
    -- x
    -other : kept
    +&mapping ⏎
    +items: []
    +other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    -&mapping
    -items:
    -- x
    -other : kept
    +&mapping ⏎
    +items: [x]
    +other: kept

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -items:
    -- x
    +items: []

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -items:
    -- x
    +items: [x]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -items:
    -- x
    \ No newline at end of file
    +items: []
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -items:
    -- x
    \ No newline at end of file
    +items: [x]
    \ No newline at end of file
    ");
}

#[test_case(
    indoc! {r"
        a: b
        : c
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
         a: b
        -: c
        +next: c
    ");
    "final newline"
)]
#[test_case(
    indoc! {r"
        a: b
        : c"}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
         a: b
        -: c
        \ No newline at end of file
        +next: c
        \ No newline at end of file
    ");
    "no final newline"
)]
fn empty_key_insertions_do_not_copy_a_preceding_line_break(source: &str) -> String {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    edit(source, |editor| {
        let key = editor.mapping_entries(&editor.root())?.remove(1).0;
        editor.replace(&key, "next")
    })
}

#[test_case(
    indoc! {r"
        [[[a]: b]: c]
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1,3 @@
        -[[[a]: b]: c]
        +[{? [{? [new, a]
        +      : b}]
        +  : c}]
    ");
    "root pair"
)]
#[test_case(
    indoc! {r"
        outer: [[[a]: b]: c]
        outside : untouched
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,4 @@
        -outer: [[[a]: b]: c]
        +outer: [{? [{? [new, a]
        +      : b}]
        +  : c}]
         outside : untouched
    ");
    "nested pair"
)]
#[test_case(
    indoc! {r"
        [[[[a]: b]: c]: d]
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1,4 @@
        -[[[[a]: b]: c]: d]
        +[{? [{? [{? [new, a]
        +          : b}]
        +      : c}]
        +  : d}]
    ");
    "recursive pair"
)]
fn nested_flow_pair_keys_expand_the_enclosing_rewrite(source: &str) -> String {
    // Implicit flow pairs: https://yaml.org/spec/1.2.2/#741-flow-sequences
    edit(source, |editor| {
        let mut sequence = editor.root();
        if source.starts_with("outer:") {
            sequence = editor.mapping_entries(&sequence)?.remove(0).1;
        }
        while let Ok(items) = editor.sequence_items(&sequence) {
            let item = &items[0];
            let Ok(entries) = editor.mapping_entries(item) else {
                break;
            };
            sequence = entries[0].0.clone();
        }
        editor.insert_item(&sequence, 0, "new").map(|_| ())
    })
}

#[test]
fn mapping_scalar_replacement_adds_omitted_value_indicators() {
    // Empty values: https://yaml.org/spec/1.2.2/#742-flow-mappings
    // Explicit entries: https://yaml.org/spec/1.2.2/#832-block-mappings
    let diffs = edits(
        [
            indoc! {r"
                {key}
            "},
            indoc! {r"
                {? key}
            "},
            indoc! {r"
                ? key
            "},
            indoc! {r"
                ?
            "},
            indoc! {r"
                ? key # note
            "},
            indoc! {"
                {key # a colon in a comment:\x20
                }
            "},
            indoc! {r"
                key:  # note
                other : 'untouched'
            "},
        ],
        |editor| {
            let value = editor.mapping_entries(&editor.root())?.remove(0).1;
            editor.replace(&value, "42")
        },
    );
    insta::assert_snapshot!(diffs, @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{key}
    +{key: 42}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{? key}
    +{key: 42}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -? key
    +key: 42

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -?
    +: 42

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -? key # note
    +key: 42 # note

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -{key # a colon in a comment: ⏎
    -}
    +{key: # a colon in a comment: ⏎
    +  42}

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -key:  # note
    +key:  42 # note
     other : 'untouched'
    ");
}

#[test_case(
    indoc! {r#"
        {"key":"value"}
    "#},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key":"value"}
        +{next: "value"}
    "#);
    "string value"
)]
#[test_case(
    indoc! {r#"
        {"key":42}
    "#},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key":42}
        +{next: 42}
    "#);
    "numeric value"
)]
#[test_case(
    indoc! {r#"
        {"key":{}}
    "#},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key":{}}
        +{next: {}}
    "#);
    "mapping value"
)]
#[test_case(
    indoc! {r#"
        {"key" :"value"}
    "#},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key" :"value"}
        +{next: "value"}
    "#);
    "spaced colon"
)]
#[test_case(
    indoc! {r"
        key  :  value # note
        other : 'untouched'
    "},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
        -key  :  value # note
        +next  :  value # note
         other : 'untouched'
    ");
    "block mapping"
)]
#[test_case(
    indoc! {r#"
        {"key":}
    "#},
    "next"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key":}
        +{next:}
    "#);
    "omitted value"
)]
#[test_case(
    indoc! {r#"
        {"key":"value"}
    "#},
    "'next'"
    => with |diff| insta::assert_snapshot!(diff, @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{"key":"value"}
        +{'next':"value"}
    "#);
    "quoted replacement"
)]
fn mapping_scalar_key_replacement_preserves_required_separation(
    source: &str,
    replacement: &str,
) -> String {
    // JSON-like keys: https://yaml.org/spec/1.2.2/#742-flow-mappings
    edit(source, |editor| {
        let key = editor.mapping_entries(&editor.root())?.remove(0).0;
        editor.replace(&key, replacement)
    })
}

#[test]
fn mapping_scalar_key_replacement_can_require_explicit_notation() {
    // Implicit keys: https://yaml.org/spec/1.2.2/#832-block-mappings
    let source = indoc! {r"
        key: value
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    let replacement = format!("\"{}\"", "\\u0001".repeat(103));
    document
        .edit_syntax(|editor| {
            let key = editor.mapping_entries(&editor.root())?.remove(0).0;
            editor.replace(&key, &replacement)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -key: value
    +? "\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001\U00000001"
    +: value
    "#);
}

#[test]
fn mapping_scalar_key_style_changes_preserve_required_separation() {
    let source = indoc! {r#"
        outer: {"key":"value"}
        other : 'untouched'
    "#};
    let diff = edit(source, |editor| {
        let mapping = editor.mapping_entries(&editor.root())?.remove(0).1;
        let key = editor.mapping_entries(&mapping)?.remove(0).0;
        editor.set_scalar_style(&key, ScalarStyle::Plain)
    });
    insta::assert_snapshot!(diff, @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -outer: {"key":"value"}
    +outer: {key: "value"}
     other : 'untouched'
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
fn explicit_keys_accept_block_style_requests() {
    let mut diffs = Vec::new();
    for source in [
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
            ? [a, b]
            : value
        "},
        indoc! {r"
            ? {k: v}
            : value
        "},
        indoc! {r"
            outer:
              ? [a, b]
              : value
        "},
        indoc! {r"
            ? |2-
              key
            : [value]
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let mut mapping = editor.root();
            if source.starts_with("outer:") {
                mapping = editor.mapping_entries(&mapping)?.remove(0).1;
            }
            let (key, value) = editor.mapping_entries(&mapping)?.remove(0);
            match editor.node(&key)?.kind() {
                astral_yaml::edit::NodeKind::Scalar { style, .. } => {
                    editor.set_scalar_style(&key, *style)?;
                }
                _ => editor.set_collection_style(&key, CollectionStyle::Block)?,
            }
            // Force a visible change even when the key already has its requested style.
            editor.replace(&value, "updated")
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     ? |2-
       key
    -: value
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     ? >2-
       key
    -: value
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     ? - a
       - b
    -: value
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     ? k: v
    -: value
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -? [a, b]
    -: value
    +?
    +  - a
    +  - b
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -? {k: v}
    -: value
    +?
    +  k: v
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     outer:
    -  ? [a, b]
    -  : value
    +  ?
    +    - a
    +    - b
    +  : updated

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     ? |2-
       key
    -: [value]
    +: updated
    "#);
}

#[test]
fn implicit_block_keys_keep_their_own_scalar_context() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            old: value
        "},
        indoc! {r"
            {old: value}
        "},
        indoc! {r"
            outer:
              old: value
        "},
    ] {
        for key in ["'---'", "'...'", "'a,b'", "'a[b]'"] {
            diffs.push(edit(source, |editor| {
                let mut mapping = editor.root();
                if source.starts_with("outer:") {
                    mapping = editor.mapping_entries(&mapping)?.remove(0).1;
                }
                let key_handle = editor.mapping_entries(&mapping)?.remove(0).0;
                editor.replace_preserving_style(&key_handle, key)
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old: value
    +---: value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old: value
    +...: value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old: value
    +a,b: value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old: value
    +a[b]: value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old: value}
    +{---: value}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old: value}
    +{...: value}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old: value}
    +{"a,b": value}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{old: value}
    +{"a[b]": value}

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     outer:
    -  old: value
    +  ---: value

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     outer:
    -  old: value
    +  ...: value

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     outer:
    -  old: value
    +  a,b: value

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     outer:
    -  old: value
    +  a[b]: value
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn multiline_scalar_rewrites_use_enclosing_syntax_context() {
    use astral_yaml::edit::NodeKind;

    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            - key: 'old'
        "},
        indoc! {r"
            - key: ['old']
        "},
        indoc! {r"
            outer:
              - key: [['old']]
            untouched : value
        "},
        indoc! {r"
            - - key: 'old'
        "},
        indoc! {r#"
            - key: ["new\n\n"]
        "#},
    ] {
        diffs.push(edit(source, |editor| {
            let mut node = editor.root();
            loop {
                node = match editor.node(&node)?.kind() {
                    NodeKind::Sequence { .. } => editor.sequence_items(&node)?[0].clone(),
                    NodeKind::Mapping { .. } => editor.mapping_entries(&node)?[0].1.clone(),
                    _ => break,
                };
            }
            if matches!(
                editor.node(&node)?.kind(),
                NodeKind::Scalar {
                    style: ScalarStyle::DoubleQuoted,
                    ..
                }
            ) {
                return editor.set_scalar_style(&node, ScalarStyle::SingleQuoted);
            }
            editor.replace_preserving_style(
                &node,
                indoc! {r"
                    |+
                      new

                "},
            )
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -- key: 'old'
    +-
    +  key: 'new
    +
    +
    +    '

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -- key: ['old']
    +- key: ['new
    +
    +
    +    ']

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,6 @@
     outer:
    -  - key: [['old']]
    +  - key: [['new
    +
    +
    +      ']]
     untouched : value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -- - key: 'old'
    +-
    +  -
    +    key: 'new
    +
    +
    +      '

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -- key: ["new\n\n"]
    +- key: ['new
    +
    +
    +    ']
    "#);
}
