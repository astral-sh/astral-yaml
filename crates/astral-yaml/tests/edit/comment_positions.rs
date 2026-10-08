use super::support::{edit, edited, edits};
use astral_yaml::{
    comments::CommentPlacement,
    edit::{CommentPosition, EditableDocument, NodeKind},
};
use astral_yaml_dev::indoc;

#[test]
fn unchanged_reinsertion_preserves_source() {
    for source in [
        indoc! {r"
            # detached

            item
        "},
        indoc! {r"
            &node
            # property
            item
        "},
        indoc! {r"
            - one

            # gap

            - two
        "},
    ] {
        let document = edited(source, |editor| {
            let owner = editor.root();
            let comment = &editor.node(&owner)?.comments()[0];
            let text = comment.text().to_owned();
            let placement = comment.placement();
            let position = editor.comment_position(&owner, 0)?;
            editor.remove_comment(&owner, 0)?;
            editor.insert_comment_at(&owner, 0, &text, placement, position)
        });
        assert_eq!(document.source(), source);
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn reinsertion_retains_headers_and_collection_gaps() {
    let diff = edits(
        [
            indoc! {r"
                - !local
                  # property
                  item
            "},
            indoc! {r"
                # leading
                [ # opening
                  item
                ] # closing
            "},
            indoc! {r"
                - one

                # gap

                - two
            "},
            indoc! {r"
                # detached

                # leading
                ---
                key: value
            "},
            indoc! {r"
                # detached

                # leading
                ---
                &root
                key: value
            "},
        ],
        |editor| {
            let root = editor.root();
            let owner = if editor.node(&root)?.comments().is_empty() {
                editor.sequence_items(&root)?[0].clone()
            } else {
                root
            };
            for index in (0..editor.node(&owner)?.comments().len()).rev() {
                let comment = &editor.node(&owner)?.comments()[index];
                let text = format!("{} edited", comment.text());
                let placement = comment.placement();
                let position = editor.comment_position(&owner, index)?;
                editor.remove_comment(&owner, index)?;
                editor.insert_comment_at(&owner, index, &text, placement, position)?;
            }
            Ok(())
        },
    );
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -- !local
    -  # property
    +- !<!local> ⏎
    +  # property edited
       item

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    -# leading
    -[ # opening
    -  item
    -] # closing
    +# leading edited
    +[ # opening edited
    +  item] # closing edited

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     - one
     ⏎
    -# gap
    +# gap edited
     ⏎
     - two

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,6 @@
    -# detached
     ⏎
    -# leading
    +# detached edited
    +
    +# leading edited
     ---
     key: value

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +3,5 @@
    -# detached
     ⏎
    -# leading
    +# detached edited
    +
    +# leading edited
     ---
    -&root
    +&root ⏎
     key: value
    ");
}

#[test]
fn inserted_gaps_follow_children_through_collection_edits() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            - one
            - two
            - three
        "},
        indoc! {r"
            one: 1
            two: 2
            three: 3
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            let sequence = matches!(editor.node(&root)?.kind(), NodeKind::Sequence { .. });
            let before = if sequence {
                editor.sequence_items(&root)?[1].clone()
            } else {
                editor.mapping_entries(&root)?[1].0.clone()
            };
            editor.insert_comment_at(
                &root,
                0,
                " section",
                CommentPlacement::Detached,
                CommentPosition::BeforeChild(before),
            )?;
            if sequence {
                editor.insert_item(&root, 0, "zero")?;
                editor.remove_item(&root, 2)?;
            } else {
                editor.insert_entry(&root, 0, "zero", "0")?;
                editor.remove_entry(&root, 2)?;
            }
            Ok(())
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,6 @@
    +- zero
     - one
    -- two
    +
    +# section
    +
     - three

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,6 @@
    +zero: 0
     one: 1
    -two: 2
    +
    +# section
    +
     three: 3
    ");
}

#[test]
fn invalid_comment_boundaries_poison_the_transaction() {
    let source = indoc! {r"
        items: [one, two]
        other: [three]
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    let foreign = document.edit(|editor| Ok(editor.root())).unwrap();
    let mut errors = Vec::new();
    for case in 0..6 {
        let error = document
            .edit(|editor| {
                let root = editor.root();
                let entries = editor.mapping_entries(&root)?;
                let items = entries[0].1.clone();
                let child = editor.sequence_items(&items)?[0].clone();
                let (owner, before, placement) = match case {
                    0 => (items, foreign.clone(), CommentPlacement::Detached),
                    1 => {
                        editor.remove_item(&items, 0)?;
                        (items, child, CommentPlacement::Detached)
                    }
                    2 => (entries[1].1.clone(), child, CommentPlacement::Detached),
                    3 => (root, items, CommentPlacement::Detached),
                    4 => (child.clone(), child, CommentPlacement::Detached),
                    _ => (items, child, CommentPlacement::Leading),
                };
                assert!(
                    editor
                        .insert_comment_at(
                            &owner,
                            0,
                            " invalid",
                            placement,
                            CommentPosition::BeforeChild(before)
                        )
                        .is_err()
                );
                Ok(())
            })
            .unwrap_err();
        assert_eq!(document.source(), source);
        errors.push(error.to_string());
    }
    insta::assert_debug_snapshot!(errors, @r#"
    [
        "foreign or deleted edit handle at bytes 0..32",
        "foreign or deleted edit handle at bytes 0..32",
        "comment boundary is not a collection child at bytes 0..32",
        "comment boundary is not a collection child at bytes 0..32",
        "comment boundary is not a collection child at bytes 0..32",
        "collection gaps require detached comments at bytes 0..32",
    ]
    "#);
}

#[test]
fn querying_a_missing_comment_does_not_poison_the_transaction() {
    let diff = edit("old\n", |editor| {
        assert!(editor.comment_position(&editor.root(), 0).is_err());
        editor.replace(&editor.root(), "new")
    });
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +new
    ");
}
