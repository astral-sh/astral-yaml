use test_case::test_case;

use super::support::edit;
use astral_yaml::comments::CommentPlacement;
use astral_yaml_dev::indoc;

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
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            let owner = match target {
                Target::Key => editor.mapping_entries(&root)?[0].0.clone(),
                Target::Value => editor.mapping_entries(&root)?[0].1.clone(),
                Target::Item => editor.sequence_items(&root)?[0].clone(),
            };
            editor.insert_comment(&owner, 0, " collection", CommentPlacement::Leading)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1 +1,4 @@
    -{? [item]: value}
    +{? ⏎
    +  # collection
    +  [item]
    +  : value}

    @@ -1,2 +1,4 @@
    -&root
    -key: value
    +&root ⏎
    +key:
    +  # collection
    +  value

    @@ -1,2 +1,4 @@
    -&root
    -- - child
    +&root ⏎
    +-
    +  # collection
    +    - child

    @@ -1,2 +1,4 @@
    -? - child
    +?
    +  # collection
    +    - child
     : value

    @@ -1,3 +1,4 @@
    -&root
    +&root ⏎
     key:
    -  - child
    +  # collection
    +    - child
    ");
}

#[test_case(
    indoc! {r"
        outer:
          # collection
            key: value
    "},
    Target::Value
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1,3 @@
        -outer:
        -  # collection
        -    key: value
        +# collection
        +outer: # inline
        +  key: value
    ");
    "mapping value"
)]
#[test_case(
    indoc! {r"
        -
          # collection
            - item
    "},
    Target::Item
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1,3 @@
        --
        -  # collection
        -    - item
        +# collection
        +- # inline
        +  - item
    ");
    "sequence item"
)]
#[test_case(
    indoc! {r"
        &root
        -
          # collection
            key: value
    "},
    Target::Item
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,4 +1,4 @@
        -&root
        +&root ⏎
         -
           # collection
        -    key: value
        +  {key: value} # inline
    ");
    "anchored sequence"
)]
#[test_case(
    indoc! {r"
        ?
          # collection
            - key
        : value
    "},
    Target::Key
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,4 +1,4 @@
         ?
           # collection
        -    - key
        +  [key] # inline
         : value
    ");
    "mapping key"
)]
fn collection_prefixes_and_inline_comments_get_distinct_headers(
    source: &str,
    target: Target,
) -> String {
    edit(source, |editor| {
        let root = editor.root();
        let owner = match target {
            Target::Key => editor.mapping_entries(&root)?[0].0.clone(),
            Target::Value => editor.mapping_entries(&root)?[0].1.clone(),
            Target::Item => editor.sequence_items(&root)?[0].clone(),
        };
        editor.insert_comment(&owner, 1, " inline", CommentPlacement::Inline)
    })
}

#[test_case(
    indoc! {r"
        outer: &inner
          key: value
          # trailing
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1,3 @@
        -outer: &inner
        -  key: value
        +outer: &inner {key: value,
           # trailing
        +  } # inline
    ");
    "mapping trailing"
)]
#[test_case(
    indoc! {r"
        outer: &inner
          - item

          # detached
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,4 +1,4 @@
        -outer: &inner
        -  - item
        -
        +outer: &inner [item,
        +  ⏎
           # detached
        +  ] # inline
    ");
    "sequence detached"
)]
#[test_case(
    indoc! {r"
        outer: !!map &inner
          key: value
          # trailing
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1,3 +1,3 @@
        -outer: !!map &inner
        -  key: value
        +outer: !<tag:yaml.org,2002:map> &inner {key: value,
           # trailing
        +  } # inline
    ");
    "tagged mapping trailing"
)]
fn collection_suffixes_can_precede_a_closing_inline_comment(source: &str) -> String {
    edit(source, |editor| {
        let inner = editor.mapping_entries(&editor.root())?[0].1.clone();
        editor.insert_comment(&inner, 1, " inline", CommentPlacement::Inline)
    })
}
