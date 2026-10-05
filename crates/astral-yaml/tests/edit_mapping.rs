use astral_yaml::{CollectionStyle, ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
fn indentless_sequences_can_become_empty_or_flow_values() {
    // Block sequences: https://yaml.org/spec/1.2.2/#821-block-sequences
    let mut outputs = Vec::new();
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
            outputs.push(document.to_yaml());
        }
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "items: []\nother: kept\n",
        "items: [x]\nother: kept\n",
        "outer:\n  items: []\n  other: kept\noutside : untouched\n",
        "outer:\n  items: [x]\n  other: kept\noutside : untouched\n",
        "-\n  items: []\n  other: kept\n",
        "-\n  items: [x]\n  other: kept\n",
        "&mapping \nitems: []\nother: kept\n",
        "&mapping \nitems: [x]\nother: kept\n",
        "items: []\r\n",
        "items: [x]\r\n",
        "items: []",
        "items: [x]",
    ]
    "#);
}

#[test]
fn empty_key_insertions_do_not_copy_a_preceding_line_break() {
    // Empty nodes: https://yaml.org/spec/1.2.2/#72-empty-nodes
    let mut outputs = Vec::new();
    for source in ["a: b\n: c\n", "a: b\r\n: c\r\n", "a: b\r: c\r", "a: b\n: c"] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let key = editor.mapping_entries(&editor.root())?.remove(1).0;
                editor.replace(&key, "next")
            })
            .unwrap();
        outputs.push(document.to_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "a: b\nnext: c\n",
        "a: b\r\nnext: c\r\n",
        "a: b\rnext: c\r",
        "a: b\nnext: c",
    ]
    "#);
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
    @@ -1 +1,2 @@
    -? key # note
    +key: 42 # note
    +

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -{key # a colon in a comment: ⏎
    -}
    +{key: 42 # a colon in a comment: ⏎
    +  }

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
