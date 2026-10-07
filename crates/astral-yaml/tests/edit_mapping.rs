use astral_yaml::{CollectionStyle, ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

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
            let mut document = EditableDocument::parse(source).unwrap();
            document
                .edit(|editor| {
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
                })
                .unwrap();
            diffs.push(edit_diff(source, document.source()));
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

#[test]
fn empty_key_insertions_do_not_copy_a_preceding_line_break() {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            a: b
            : c
        "},
        indoc! {r"
            a: b
            : c"},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let key = editor.mapping_entries(&editor.root())?.remove(1).0;
                editor.replace(&key, "next")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     a: b
    -: c
    +next: c

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     a: b
    -: c
    \ No newline at end of file
    +next: c
    \ No newline at end of file
    ");
}

#[test]
fn nested_flow_pair_keys_expand_the_enclosing_rewrite() {
    // Implicit flow pairs: https://yaml.org/spec/1.2.2/#741-flow-sequences
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            [[[a]: b]: c]
        "},
        indoc! {r"
            outer: [[[a]: b]: c]
            outside : untouched
        "},
        indoc! {r"
            [[[[a]: b]: c]: d]
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
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
                editor.insert_item(&sequence, 0, "new")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -[[[a]: b]: c]
    +[{? [{? [new, a]
    +      : b}]
    +  : c}]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -outer: [[[a]: b]: c]
    +outer: [{? [{? [new, a]
    +      : b}]
    +  : c}]
     outside : untouched

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -[[[[a]: b]: c]: d]
    +[{? [{? [{? [new, a]
    +          : b}]
    +      : c}]
    +  : d}]
    ");
}

#[test]
fn mapping_scalar_replacement_adds_omitted_value_indicators() {
    // Empty values: https://yaml.org/spec/1.2.2/#742-flow-mappings
    // Explicit entries: https://yaml.org/spec/1.2.2/#832-block-mappings
    let mut diffs = Vec::new();
    for source in [
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
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let value = editor.mapping_entries(&editor.root())?.remove(0).1;
                editor.replace(&value, "42")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
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

#[test]
fn mapping_scalar_key_replacement_preserves_required_separation() {
    // JSON-like keys: https://yaml.org/spec/1.2.2/#742-flow-mappings
    let mut diffs = Vec::new();
    for (source, replacement) in [
        (
            indoc! {r#"
                {"key":"value"}
            "#},
            "next",
        ),
        (
            indoc! {r#"
                {"key":42}
            "#},
            "next",
        ),
        (
            indoc! {r#"
                {"key":{}}
            "#},
            "next",
        ),
        (
            indoc! {r#"
                {"key" :"value"}
            "#},
            "next",
        ),
        (
            indoc! {r"
                key  :  value # note
                other : 'untouched'
            "},
            "next",
        ),
        (
            indoc! {r#"
                {"key":}
            "#},
            "next",
        ),
        (
            indoc! {r#"
                {"key":"value"}
            "#},
            "'next'",
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let key = editor.mapping_entries(&editor.root())?.remove(0).0;
                editor.replace(&key, replacement)
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key":"value"}
    +{next: "value"}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key":42}
    +{next: 42}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key":{}}
    +{next: {}}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key" :"value"}
    +{next: "value"}

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -key  :  value # note
    +next  :  value # note
     other : 'untouched'

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key":}
    +{next:}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{"key":"value"}
    +{'next':"value"}
    "#);
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
        .edit(|editor| {
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
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let mapping = editor.mapping_entries(&editor.root())?.remove(0).1;
            let key = editor.mapping_entries(&mapping)?.remove(0).0;
            editor.set_scalar_style(&key, ScalarStyle::Plain)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
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
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
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
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
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
    @@ -1,3 +1,4 @@
    -? - a
    +?
    +  - a
       - b
    -: value
    +: updated

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -? k: v
    -: value
    +?
    +  k: v
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
    ");
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
            let mut document = EditableDocument::parse(source).unwrap();
            document
                .edit(|editor| {
                    let mut mapping = editor.root();
                    if source.starts_with("outer:") {
                        mapping = editor.mapping_entries(&mapping)?.remove(0).1;
                    }
                    let key_handle = editor.mapping_entries(&mapping)?.remove(0).0;
                    editor.replace_preserving_style(&key_handle, key)
                })
                .unwrap();
            diffs.push(edit_diff(source, document.source()));
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
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
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
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
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
