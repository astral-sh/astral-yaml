use super::{Editor, Error};
use astral_yaml::{ParseOptions, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
fn string_key_updates_leave_application_tagged_entries_intact() {
    let mut diffs = Vec::new();
    // Exercise mixed key tags without imposing a Serde key type on the syntax.
    for source in [
        "!custom name: tagged\n",
        "!custom name: tagged\nname: old\n",
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .try_edit(|syntax| {
                let mut editor = Editor::new(syntax);
                editor.root_mut().set_entry("name", "new")?;
                editor.finish()
            })
            .unwrap();
        diffs.push(format!(
            "set_entry\n{}",
            edit_diff(source, document.source())
        ));
        let before = document.to_yaml();
        document
            .try_edit(|syntax| {
                let mut editor = Editor::new(syntax);
                assert!(editor.root_mut().remove_field("name")?);
                editor.finish()
            })
            .unwrap();
        diffs.push(format!(
            "remove_field\n{}",
            edit_diff(&before, document.source())
        ));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    set_entry
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!custom name: tagged
    +!<!custom> name: tagged
    +"name": "new"

    remove_field
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
     !<!custom> name: tagged
    -"name": "new"

    set_entry
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     !custom name: tagged
    -name: old
    +name: new

    remove_field
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -!custom name: tagged
    -name: new
    +!<!custom> name: tagged
    "#);
}

#[test]
fn exhausted_mutations_poison_subsequent_setters() {
    let mut document = EditableDocument::parse_with_options(
        "1",
        ParseOptions {
            max_nodes: 3,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result: Result<(), Error> = document.try_edit(|syntax| {
        let mut editor = Editor::new(syntax);
        let mut root = editor.root_mut();
        root.set(2)?;
        let error = root.set(3).unwrap_err();
        assert_eq!(root.set(4).unwrap_err(), error);
        editor.finish()
    });
    assert!(result.is_err());
    assert_eq!(document.source(), "1");
}

#[test]
fn cursor_reads_and_mutations_respect_the_syntax_work_budget() {
    for mutate in [false, true] {
        let mut document = EditableDocument::parse_with_options(
            "1",
            ParseOptions {
                max_nodes: 1,
                ..ParseOptions::default()
            },
        )
        .unwrap();
        let result: Result<(), Error> = document.try_edit(|syntax| {
            let mut editor = Editor::new(syntax);
            let mut root = editor.root_mut();
            root.node()?;
            if mutate {
                root.set(2)
            } else {
                root.node().map(|_| ())
            }
        });
        assert!(
            result
                .unwrap_err()
                .message
                .contains("editing resource limit")
        );
        assert_eq!(document.source(), "1");
    }
}
