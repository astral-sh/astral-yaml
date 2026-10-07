//! Comments and separation: <https://yaml.org/spec/1.2.2/#66-comments>.

use astral_yaml::{
    CollectionStyle,
    edit::{EditableDocument, Editor, Handle, NodeKind},
};
use astral_yaml_dev::{edit_diff, indoc};

fn collection(editor: &Editor<'_>, nested: bool) -> Handle {
    if nested {
        editor.mapping_entries(&editor.root()).unwrap()[0].1.clone()
    } else {
        editor.root()
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn flow_delimiter_comments_survive_collection_edits() {
    let mut diffs = Vec::new();
    for (source, nested) in [
        (
            indoc! {r"
                [ # open
                  old
                ] # close
            "},
            false,
        ),
        (
            indoc! {r"
                { # open
                  key: old
                } # close
            "},
            false,
        ),
        (
            indoc! {r"
                outer: [ # open
                  old
                  ] # close
            "},
            true,
        ),
        (
            indoc! {r"
                outer: { # open
                  key: old
                  } # close
            "},
            true,
        ),
        (
            indoc! {r"
                &a [ # open
                  *a
                ] # close
            "},
            false,
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let target = collection(editor, nested);
                match editor.node(&target)?.kind() {
                    NodeKind::Sequence { .. } => editor.insert_item(&target, 1, "new"),
                    _ => editor.insert_entry(&target, 1, "added", "new"),
                }
            })
            .unwrap();
        let committed = document.to_yaml();
        document
            .edit(|editor| {
                let target = collection(editor, nested);
                editor.set_collection_style(&target, CollectionStyle::Flow)
            })
            .unwrap();
        assert_eq!(document.source(), committed);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     [ # open
    -  old
    -] # close
    +  old, new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     { # open
    -  key: old
    -} # close
    +  key: old, added: new} # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open
    -  old
    -  ] # close
    +  old, new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open
    -  key: old
    -  } # close
    +  key: old, added: new} # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     &a [ # open
    -  *a
    -] # close
    +  *a, new] # close
    ");
}

#[test]
fn shrinking_collections_retain_their_comment_boundaries() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            outer: [ # open
              old
              ] # close
        "},
        indoc! {r"
            outer: { # open
              key: old
              } # close
        "},
        indoc! {r"
            outer:
            # leading
              {key: old}
        "},
        indoc! {r"
            outer:
            # detached
              key: old
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let target = collection(editor, true);
                match editor.node(&target)?.kind() {
                    NodeKind::Sequence { .. } => editor.remove_item(&target, 0),
                    _ => editor.remove_entry(&target, 0),
                }
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open
    -  old
       ] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open
    -  key: old
       } # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1 @@
    -outer:
    -# leading
    -  {key: old}
    +outer: {}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  key: old
    +  # detached
    +
    +  {}
    ");
}

#[test]
fn rewrite_context_is_selected_after_all_mutations() {
    let source = indoc! {r"
        outer:   [old]
        untouched:    keep
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let target = collection(editor, true);
            // The transient block layout needs its parent. The final flow layout
            // fits in the original value region and keeps the surrounding source.
            editor.set_collection_style(&target, CollectionStyle::Block)?;
            editor.insert_item(&target, 1, "new")?;
            editor.set_collection_style(&target, CollectionStyle::Flow)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -outer:   [old]
    +outer:   [old, new]
     untouched:    keep
    ");
}

#[test]
fn retained_opening_comments_merge_with_fragment_prefixes() {
    let mut diffs = Vec::new();
    let source = indoc! {r"
        [ # open
          old
        ] # close
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            editor.replace(
                &editor.root(),
                indoc! {r"
                    # detached

                    [new]
                "},
            )?;
            editor.replace_comment(&editor.root(), 1, " updated")
        })
        .unwrap();
    diffs.push(edit_diff(source, document.source()));
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[ # open
    -  old
    -] # close
    +# detached
    +
    +[ # updated
    +  new] # close
    ");
}
