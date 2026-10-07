use astral_yaml::edit::EditableDocument;
use astral_yaml_dev::{edit_diff, indoc};

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn first_child_comments_stay_inside_property_bearing_collections() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            &mapping
            key:
              # child
              value
        "},
        indoc! {r"
            &sequence
            -
              # child
              item
        "},
        indoc! {r"
            !!map
            key:
              # child
              [item]
        "},
        indoc! {r"
            !!seq
            -
              # child
              {key: value}
        "},
        indoc! {r"
            &mapping
            ?
              # key
                - item
            :
              # value
                child: value
        "},
        indoc! {r"
            &sequence
            -
              # child
                - item
        "},
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        diffs.push(edit_diff(source, &output));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -&mapping
    +&mapping ⏎
     key:
       # child
       value

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -&sequence
    +&sequence ⏎
     -
       # child
       item

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -!!map
    +!<tag:yaml.org,2002:map> ⏎
     key:
       # child
       [item]

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -!!seq
    +!<tag:yaml.org,2002:seq> ⏎
     -
       # child
       {key: value}

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -&mapping
    +&mapping ⏎
     ?
       # key
         - item

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -&sequence
    +&sequence ⏎
     -
       # child
         - item
    ");
}
