use astral_yaml::{ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

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
