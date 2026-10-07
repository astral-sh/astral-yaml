use astral_yaml::{comments::CommentPlacement, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[derive(Clone, Copy)]
enum Target {
    Key,
    Value,
    Item,
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn generated_collection_prefixes_keep_their_direct_owner() {
    let mut diffs = Vec::new();
    for (source, target) in [
        (
            indoc! {r"
                {? [item]: value}
            "},
            Target::Key,
        ),
        (
            indoc! {r"
                &root
                key: value
            "},
            Target::Value,
        ),
        (
            indoc! {r"
                &root
                - - child
            "},
            Target::Item,
        ),
        (
            indoc! {r"
                ? - child
                : value
            "},
            Target::Key,
        ),
        (
            indoc! {r"
                &root
                key:
                  - child
            "},
            Target::Value,
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                let owner = match target {
                    Target::Key => editor.mapping_entries(&root)?[0].0.clone(),
                    Target::Value => editor.mapping_entries(&root)?[0].1.clone(),
                    Target::Item => editor.sequence_items(&root)?[0].clone(),
                };
                editor.insert_comment(&owner, 0, " collection", CommentPlacement::Leading)
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -{? [item]: value}
    +{? ⏎
    +  # collection
    +  [item]
    +  : value}

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -&root
    -key: value
    +&root ⏎
    +key:
    +  # collection
    +  value

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -&root
    -- - child
    +&root ⏎
    +-
    +  # collection
    +    - child

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -? - child
    +?
    +  # collection
    +    - child
     : value

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -&root
    +&root ⏎
     key:
    -  - child
    +  # collection
    +    - child
    ");
}
