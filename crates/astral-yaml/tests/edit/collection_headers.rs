use super::support::normalize;
use astral_yaml_dev::indoc;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn first_child_comments_stay_inside_property_bearing_collections() {
    let diffs = [
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
    ]
    .map(normalize);
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,4 +1,4 @@
    -&mapping
    +&mapping ⏎
     key:
       # child
       value

    @@ -1,4 +1,4 @@
    -&sequence
    +&sequence ⏎
     -
       # child
       item

    @@ -1,4 +1,4 @@
    -!!map
    +!<tag:yaml.org,2002:map> ⏎
     key:
       # child
       [item]

    @@ -1,4 +1,4 @@
    -!!seq
    +!<tag:yaml.org,2002:seq> ⏎
     -
       # child
       {key: value}

    @@ -1,4 +1,4 @@
    -&mapping
    +&mapping ⏎
     ?
       # key
         - item

    @@ -1,4 +1,4 @@
    -&sequence
    +&sequence ⏎
     -
       # child
         - item
    ");
}
