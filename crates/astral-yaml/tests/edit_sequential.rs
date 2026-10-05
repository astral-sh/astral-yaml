//! Transaction semantics for successive edits to the same syntax.

use astral_yaml::{ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
fn sequential_replacements_emit_the_final_candidate() {
    let source = "value: 1 # keep\nother : 'untouched'\n";
    let mut document = EditableDocument::parse(source).unwrap();
    let result = document
        .edit(|e| {
            let value = e.mapping_entries(&e.root())?.remove(0).1;
            e.replace(&value, "2")?;
            e.replace(&value, "[3, 4]")?;
            let first = e.sequence_items(&value)?.remove(0);
            e.replace(&first, "5")?;
            e.replace(&first, "6")?;
            Ok(42)
        })
        .unwrap();
    assert_eq!(result, 42);
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -value: 1 # keep
    -other : 'untouched'
    +value: [6, 4] # keep
    +other: 'untouched'
    ");
}

#[test]
fn removed_subtrees_discard_pending_rewrites_and_style_requests() {
    let mut diffs = Vec::new();
    for remove in [false, true] {
        let source = "items: [old, keep]\nother: retained\n";
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|e| {
                let items = e.mapping_entries(&e.root())?.remove(0).1;
                let old = e.sequence_items(&items)?.remove(0);
                e.replace(&old, "new")?;
                e.set_scalar_style(&old, ScalarStyle::SingleQuoted)?;
                if remove {
                    e.remove_item(&items, 0)?;
                } else {
                    e.replace(&items, "[replacement]")?;
                }
                Ok(())
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -items: [old, keep]
    +items: [replacement]
     other: retained

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -items: [old, keep]
    +items: [keep]
     other: retained
    ");
}

#[test]
fn inserted_nodes_can_be_edited_and_removed_before_commit() {
    let mut document = EditableDocument::parse("[]").unwrap();
    document
        .edit(|e| {
            let root = e.root();
            e.insert_item(&root, 0, "[one]")?;
            let inserted = e.sequence_items(&root)?.remove(0);
            let child = e.sequence_items(&inserted)?.remove(0);
            e.replace(&child, "two")?;
            e.set_scalar_style(&child, ScalarStyle::DoubleQuoted)?;
            e.remove_item(&root, 0)
        })
        .unwrap();
    assert_eq!(document.into_yaml(), "[]");
}

#[test]
fn replacement_keeps_compatible_styles_and_anchor_identity() {
    let mut diffs = Vec::new();
    for (source, replacement) in [
        ("old", "\"new\""),
        ("old", "\"false\""),
        ("'old'", "\"it's new\""),
        ("'old'", "\"line\\nbreak\""),
        ("\"old\"", "60"),
        ("|-\n  old\n", "\"new\\ntext\""),
        ("- old\n", "[new, values]"),
        ("old: 1\n", "{new: 2}"),
        ("[]", "42"),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|e| e.replace_preserving_style(&e.root(), replacement))
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    let source = "[&value 'old', *value]";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let value = e.sequence_items(&e.root())?.remove(0);
            e.replace_preserving_style(&value, "\"new\"")
        })
        .unwrap();
    diffs.push(edit_diff(source, document.source()));
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    \ No newline at end of file
    +new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    \ No newline at end of file
    +"false"
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -'old'
    \ No newline at end of file
    +'it''s new'
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -'old'
    \ No newline at end of file
    +"line\nbreak"
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -"old"
    \ No newline at end of file
    +60
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -|-
    -  old
    +|2-
    +  new
    +  text

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -- old
    +- new
    +- values

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old: 1
    +new: 2

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[]
    \ No newline at end of file
    +42
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[&value 'old', *value]
    \ No newline at end of file
    +[&value 'new', *value]
    \ No newline at end of file
    "#);
}

#[test]
fn application_errors_abort_transactions() {
    let mut document = EditableDocument::parse("1").unwrap();
    let result: Result<(), Box<dyn std::error::Error>> = document.try_edit(|e| {
        e.replace(&e.root(), "2")?;
        Err("application error".into())
    });
    assert_eq!(result.unwrap_err().to_string(), "application error");
    assert_eq!(document.source(), "1");
}
