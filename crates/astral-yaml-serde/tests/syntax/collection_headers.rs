//! A block collection can carry properties on lines before its entries.
//! See <https://yaml.org/spec/1.2.2/#823-block-nodes>.

use super::support::{edited, normalize};
use astral_yaml::CollectionStyle;
use astral_yaml_dev::{edit_diff, indoc};

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

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_collection_headers_retain_inline_comments() {
    // Node properties: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            &root [old] # inline
        "},
        indoc! {r"
            &root {a: old} # inline
        "},
        indoc! {r"
            !!seq [old] # inline
        "},
        indoc! {r"
            !!seq # tag
            &root [old] # inline
        "},
        indoc! {r"
            # leading
            %YAML 1.2 # directive
            --- &root [old] # inline
            ... # end
            # trailing
        "},
    ] {
        let mut document = edited(source, |editor| {
            editor.set_collection_style(editor.root(), CollectionStyle::Block)
        });
        let once = document.source().to_owned();
        document
            .edit_syntax(|editor| {
                editor.set_collection_style(editor.root(), CollectionStyle::Block)
            })
            .unwrap();
        assert_eq!(document.source(), once);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1 +1,2 @@
    -&root [old] # inline
    +&root # inline
    +- old

    @@ -1 +1,2 @@
    -&root {a: old} # inline
    +&root # inline
    +a: old

    @@ -1 +1,2 @@
    -!!seq [old] # inline
    +!<tag:yaml.org,2002:seq> # inline
    +- old

    @@ -1,2 +1,3 @@
    -!!seq # tag
    -&root [old] # inline
    +!<tag:yaml.org,2002:seq> # tag
    +&root # inline
    +- old

    @@ -1,5 +1,7 @@
     # leading
     %YAML 1.2 # directive
    ---- &root [old] # inline
    +--- ⏎
    +&root # inline
    +- old
     ... # end
     # trailing
    ");
}
