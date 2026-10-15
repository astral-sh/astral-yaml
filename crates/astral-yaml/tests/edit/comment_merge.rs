//! Comment placement across fragment replacement and presentation changes.

use test_case::test_case;

use super::support::{edit, edited};
use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn replacement_merges_comments_at_their_structural_positions() {
    let mut diffs = Vec::new();
    for (source, replacement) in [
        (
            indoc! {r"
                old # tail
            "},
            indoc! {r"
                # leading
                new
            "},
        ),
        (
            indoc! {r"
                a: old

                # tail
            "},
            indoc! {r"
                a: one

                # middle

                b: two
            "},
        ),
        (
            indoc! {r"
                a: old
                b: old

                # retained

                c: old
            "},
            indoc! {r"
                a: one

                # imported

                b: two
                c: three
            "},
        ),
        (
            indoc! {r"
                - old

                # retained

                - old
            "},
            indoc! {r"
                # leading
                - one

                # imported

                - two
            "},
        ),
        (
            indoc! {r"
                old
                # trailing
            "},
            indoc! {r"
                new # inline
            "},
        ),
    ] {
        diffs.push(edit(source, |editor| {
            editor.replace(&editor.root(), replacement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail
    +# leading
    +new # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old
    +a: one
    +
    +# middle
    +
    +b: two
     ⏎
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old
    -b: old
    +a: one
    +
    +# imported
    +
    +b: two
     ⏎
     # retained
     ⏎
    -c: old
    +c: three

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old
    +# leading
    +- one
     ⏎
     # retained
     ⏎
    -- old
    +
    +# imported
    +
    +- two

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old
    +new # inline
     # trailing
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
            editor.set_collection_style(&editor.root(), CollectionStyle::Block)
        });
        let once = document.source().to_owned();
        document
            .edit_syntax(|editor| {
                editor.set_collection_style(&editor.root(), CollectionStyle::Block)
            })
            .unwrap();
        assert_eq!(document.source(), once);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline
    +&root # inline
    +a: old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline
    +!<tag:yaml.org,2002:seq> # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag
    -&root [old] # inline
    +!<tag:yaml.org,2002:seq> # tag
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
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

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn imported_prefixes_and_suffixes_preserve_comment_adjacency() {
    let mut diffs = Vec::new();
    for (source, replacement) in [
        (
            indoc! {r"
                old
            "},
            indoc! {r"
                # detached

                new
            "},
        ),
        (
            indoc! {r"
                # retained
                old
            "},
            indoc! {r"
                # detached

                new
            "},
        ),
        (
            indoc! {r"
                # retained

                old
            "},
            indoc! {r"
                # leading
                new
            "},
        ),
        (
            indoc! {r"
                --- # marker
                # retained
                old
            "},
            indoc! {r"
                # detached

                new
            "},
        ),
        (
            indoc! {r"
                old # inline
            "},
            indoc! {r"
                new
                # trailing
            "},
        ),
        (
            indoc! {r"
                old # inline
            "},
            indoc! {r"
                new

                # detached
            "},
        ),
        (
            indoc! {r"
                old
                # trailing
            "},
            indoc! {r"
                new

                # detached
            "},
        ),
        (
            indoc! {r"
                old

                # detached
            "},
            indoc! {r"
                new
                # trailing
            "},
        ),
        (
            indoc! {r"
                &root # retained
                old
            "},
            indoc! {r"
                # leading
                &next # imported
                new
            "},
        ),
    ] {
        diffs.push(edit(source, |editor| {
            editor.replace(&editor.root(), replacement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old
    +# detached
    +
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    +# detached
    +
     # retained
    -old
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     # retained
     ⏎
    -old
    +# leading
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     --- # marker
    +# detached
    +
     # retained
    -old
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline
    +new # inline
    +# trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # inline
    +new # inline
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -old
    +new
     # trailing
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old
    +new
    +# trailing
     ⏎
     # detached

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -&root # retained
    -old
    +# leading
    +&next # retained
    +  new # imported
    ");
}

#[test_case(
    indoc! {r"
        outer: old
        other: keep
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,6 @@
        -outer: old
        +outer:
        +  # detached
        +
        +  # leading
        +  new
         other: keep
    ");
    "mapping"
)]
#[test_case(
    indoc! {r"
        - old
        - keep
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,6 @@
        -- old
        +-
        +  # detached
        +
        +  # leading
        +  new
         - keep
    ");
    "sequence"
)]
fn nested_fragment_prefixes_belong_to_the_selected_value(source: &str) -> String {
    edit(source, |editor| {
        let root = editor.root();
        let value = if source.starts_with("outer") {
            editor.mapping_entries(&root)?.remove(0).1
        } else {
            editor.sequence_items(&root)?.remove(0)
        };
        editor.replace(
            &value,
            indoc! {r"
                # detached

                # leading
                new
            "},
        )
    })
}

#[test]
fn replacement_does_not_reorder_an_explicitly_edited_comment_list() {
    use astral_yaml::comments::CommentPlacement;
    let source = indoc! {r"
        old # inline
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    let result = document.edit_syntax(|editor| {
        let root = editor.root();
        // This explicit order cannot keep the new comment leading. Replacement
        // must preserve that request and reject it, rather than sort it away.
        editor.insert_comment(&root, 1, " leading", CommentPlacement::Leading)?;
        editor.replace(
            &root,
            indoc! {r"
                # imported
                new
            "},
        )
    });
    assert!(result.is_err());
    assert_eq!(document.source(), source);
}

#[test]
fn merged_comment_gaps_survive_further_replacement_and_removal() {
    let source = indoc! {r"
        a: old

        # tail
    "};
    let diff = edit(source, |editor| {
        let root = editor.root();
        editor.replace(
            &root,
            indoc! {r"
                a: one

                # first

                b: two
            "},
        )?;
        editor.replace(
            &root,
            indoc! {r"
                a: first

                # second

                b: middle
                c: last
            "},
        )?;
        editor.remove_entry(&root, 1)
    });
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,10 @@
    -a: old
    +a: first
    +
    +# first
    +
    +
    +# second
    +
    +c: last
     ⏎
     # tail
    ");
}
