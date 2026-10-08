use super::support::{edit, edited, edits};
use astral_yaml::edit::EditableDocument;
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn imported_aliases_follow_the_retained_or_fragment_anchor() {
    let fragment = indoc! {r"
        &fragment
        self: *fragment
        nested: &item [*fragment, *item]
    "};
    let mut diffs = Vec::new();
    for (source, nested) in [
        (
            indoc! {r"
                &old [before]
            "},
            false,
        ),
        (
            indoc! {r"
                [before]
            "},
            false,
        ),
        (
            indoc! {r"
                definition: &old [before]
                copy: *old
            "},
            true,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            let target = if nested {
                editor.mapping_entries(&editor.root())?[0].1.clone()
            } else {
                editor.root()
            };
            editor.replace_preserving_style(&target, fragment)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -&old [before]
    +&old ⏎
    +self: *old
    +nested: &item [*old, *item]

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -[before]
    +&fragment ⏎
    +self: *fragment
    +nested: &item [*fragment, *item]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,5 @@
    -definition: &old [before]
    +definition:
    +  &old ⏎
    +  self: *old
    +  nested: &item [*old, *item]
     copy: *old
    ");
}

#[test]
fn retained_aliases_follow_anchor_changes_outside_the_definition() {
    let diffs = edits(
        [
            indoc! {r"
                definition: &old before
                first: *old # keep
                second: *old
                untouched:   [a,b]
            "},
            indoc! {r"
                {definition: &old before, first: *old, second: *old, untouched:   [a,b]}
            "},
            indoc! {r"
                definition: &old before
                ? *old
                : value
            "},
        ],
        |editor| {
            let target = editor.mapping_entries(&editor.root())?[0].1.clone();
            editor.replace(&target, "&new after")
        },
    );
    insta::assert_snapshot!(diffs, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -definition: &old before
    -first: *old # keep
    -second: *old
    +definition: &new after
    +first: *new # keep
    +second: *new
     untouched:   [a,b]

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{definition: &old before, first: *old, second: *old, untouched:   [a,b]}
    +{definition: &new after, first: *new, second: *new, untouched:   [a,b]}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -definition: &old before
    -? *old
    +definition: &new after
    +? *new
     : value
    ");
}

#[test]
fn canceled_anchor_changes_do_not_rewrite_alias_occurrences() {
    let source = indoc! {r"
        definition: &old 'before'
        first:   *old
        second: *old # keep
    "};
    let document = edited(source, |editor| {
        let target = editor.mapping_entries(&editor.root())?[0].1.clone();
        editor.replace(&target, "&temporary 'before'")?;
        editor.replace(&target, "&old 'before'")
    });
    assert_eq!(document.source(), source);
}

#[test]
fn invalid_alias_bindings_roll_back_the_transaction() {
    let mut errors = Vec::new();
    for (source, replacement) in [
        ("[&old before, *old]", None),
        ("[&old before, *old]", Some("after")),
        ("[&old before, &shadow other, *old]", Some("&shadow after")),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        let error = document
            .edit(|editor| {
                let target = editor.sequence_items(&editor.root())?[0].clone();
                if let Some(replacement) = replacement {
                    editor.replace(&target, replacement)
                } else {
                    editor.remove_item(&editor.root(), 0)
                }
            })
            .unwrap_err();
        assert_eq!(document.source(), source);
        errors.push(error.message);
    }
    insta::assert_debug_snapshot!(errors, @r#"
    [
        "alias target was removed",
        "alias target has no anchor",
        "emission changed YAML content or alias identity",
    ]
    "#);
}

#[test]
fn alias_bindings_only_need_to_be_complete_at_the_end_of_mutation() {
    let source = indoc! {r"
        - &old before
        - *old
        - &retained value
        - *retained
    "};
    let document = edited(source, |editor| {
        let root = editor.root();
        editor.remove_item(&root, 0)?;
        editor.remove_item(&root, 0)?;
        let target = editor.sequence_items(&root)?[0].clone();
        editor.replace(&target, "changed")?;
        editor.replace(&target, "&retained changed")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,2 @@
    -- &old before
    -- *old
    -- &retained value
    +- &retained changed
     - *retained
    ");
}
