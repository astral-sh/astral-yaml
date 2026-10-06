use astral_yaml::{CollectionStyle, ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn indentless_sequences_can_become_empty_or_flow_values() {
    // Block sequences: https://yaml.org/spec/1.2.2/#821-block-sequences
    let mut diffs = Vec::new();
    for source in [
        "items:\n- x\nother : kept\n",
        "outer:\n  items:\n  - x\n  other : kept\noutside : untouched\n",
        "- items:\n  - x\n  other : kept\n",
        "&mapping\nitems:\n- x\nother : kept\n",
        "items:\r\n- x\r\n",
        "items:\n- x",
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
    -items:␍
    -- x␍
    +items: []␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -items:␍
    -- x␍
    +items: [x]␍

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
    for source in ["a: b\n: c\n", "a: b\r\n: c\r\n", "a: b\r: c\r", "a: b\n: c"] {
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
     a: b␍
    -: c␍
    +next: c␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     a: b␍-: c␍+next: c␍

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
        "[[[a]: b]: c]\n",
        "outer: [[[a]: b]: c]\noutside : untouched\n",
        "[[[[a]: b]: c]: d]\n",
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
        "{key}\n",
        "{? key}\n",
        "? key\n",
        "?\n",
        "? key # note\n",
        "{key # a colon in a comment: \n}\n",
        "key:  # note\nother : 'untouched'\n",
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
        ("{\"key\":\"value\"}\n", "next"),
        ("{\"key\":42}\n", "next"),
        ("{\"key\":{}}\n", "next"),
        ("{\"key\" :\"value\"}\n", "next"),
        ("key  :  value # note\nother : 'untouched'\n", "next"),
        ("{\"key\":}\n", "next"),
        ("{\"key\":\"value\"}\n", "'next'"),
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
    let source = "key: value\n";
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
    let source = "outer: {\"key\":\"value\"}\nother : 'untouched'\n";
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
fn explicit_keys_accept_block_style_requests() {
    let mut diffs = Vec::new();
    for source in [
        "? |2-\n  key\n: value\n",
        "? >2-\n  key\n: value\n",
        "? - a\n  - b\n: value\n",
        "? k: v\n: value\n",
        "? [a, b]\n: value\n",
        "? {k: v}\n: value\n",
        "outer:\n  ? [a, b]\n  : value\n",
        "? |2-\n  key\n: [value]\n",
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
    for source in ["old: value\n", "{old: value}\n", "outer:\n  old: value\n"] {
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
