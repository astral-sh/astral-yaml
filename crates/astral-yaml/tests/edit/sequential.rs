//! Transaction semantics for successive edits to the same syntax.

use test_case::test_case;

use super::support::edit;
use astral_yaml::{ScalarStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn sequential_replacements_emit_the_final_candidate() {
    let source = indoc! {r"
        value: 1 # keep
        other : 'untouched'
    "};
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

#[test_case(
    false
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
        -items: [old, keep]
        +items: [replacement]
         other: retained
    ");
    "replace"
)]
#[test_case(
    true
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
        -items: [old, keep]
        +items: [keep]
         other: retained
    ");
    "remove"
)]
fn removed_subtrees_discard_pending_rewrites_and_style_requests(remove: bool) -> String {
    let source = indoc! {r"
        items: [old, keep]
        other: retained
    "};
    edit(source, |e| {
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
#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
fn replacement_keeps_compatible_styles_and_anchor_identity() {
    let mut diffs = Vec::new();
    for (source, replacement) in [
        ("old", "\"new\""),
        ("old", "\"false\""),
        ("'old'", "\"it's new\""),
        ("'old'", "\"line\\nbreak\""),
        ("\"old\"", "60"),
        (
            indoc! {r"
                |-
                  old
            "},
            "\"new\\ntext\"",
        ),
        (
            indoc! {r"
                - old
            "},
            "[new, values]",
        ),
        (
            indoc! {r"
                old: 1
            "},
            "{new: 2}",
        ),
        ("[]", "42"),
    ] {
        diffs.push(edit(source, |e| {
            e.replace_preserving_style(&e.root(), replacement)
        }));
    }
    let source = "[&value 'old', *value]";
    diffs.push(edit(source, |e| {
        let value = e.sequence_items(&e.root())?.remove(0);
        e.replace_preserving_style(&value, "\"new\"")
    }));
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
    @@ -1 +1,3 @@
    -'old'
    \ No newline at end of file
    +'line
    +
    +  break'
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
