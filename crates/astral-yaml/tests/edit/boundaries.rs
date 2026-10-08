//! Root edits may require document boundaries to preserve comment ownership.

use super::support::{edit, edited};
use astral_yaml::{ParseOptions, comments::CommentPlacement, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_edits_create_missing_document_boundaries() {
    // Repeated document suffixes remain part of a single-document stream:
    // https://yaml.org/spec/1.2.2/#92-streams
    let cases = [
        (
            indoc! {r"
                # leading
                old
            "},
            indoc! {r"
                - new
            "},
        ),
        (
            indoc! {r"
                old # inline
            "},
            indoc! {r"
                - new
            "},
        ),
        (
            "old",
            indoc! {r"
                ---
            "},
        ),
        (
            indoc! {r"
                old
                # tail
            "},
            indoc! {r"
                ---
            "},
        ),
        (
            indoc! {r"
                old

                # gap
            "},
            indoc! {r"
                ---
            "},
        ),
        (
            indoc! {r"
                [ # open
                  old
                ] # close
            "},
            indoc! {r"
                - new
            "},
        ),
        (
            indoc! {r"
                &a # property
                [ # open
                  old
                ] # close
                # tail
            "},
            indoc! {r"
                - new
            "},
        ),
        (
            indoc! {r"
                &a # property
                [ # open
                  old
                ] # close
                ... # end
            "},
            indoc! {r"
                - new
            "},
        ),
        (
            "\u{feff}\n# leading\nold # inline",
            indoc! {r"
                - new
            "},
        ),
    ];
    let mut diffs = Vec::new();
    for (source, fragment) in cases {
        diffs.push(edit(source, |editor| {
            editor.replace(&editor.root(), fragment)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     # leading
    -old
    +---
    +- new

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline
    +--- # inline
    +- new

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    \ No newline at end of file
    +---

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -old
    +---
    +...
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old
    +---
    +...
     ⏎
     # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -[ # open
    -  old
    -] # close
    +--- # open
    +- new
    +... # close

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property
    -[ # open
    -  old
    -] # close
    +--- # property
    +- new
    +... # open
    +... # close
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property
    -[ # open
    -  old
    -] # close
    +--- # property
    +- new
    +... # open
    +... # close
     ... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ﻿
     # leading
    -old # inline
    \ No newline at end of file
    +--- # inline
    +- new
    \ No newline at end of file
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn inserted_comments_can_create_document_boundaries() {
    let mut diffs = Vec::new();
    for (source, placement) in [
        (
            indoc! {r"
                - old
            "},
            CommentPlacement::Leading,
        ),
        (
            indoc! {r"
                - old
            "},
            CommentPlacement::Inline,
        ),
        (
            indoc! {r"
                ---
            "},
            CommentPlacement::Trailing,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            editor.insert_comment(&editor.root(), 0, " added", placement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    +# added
    +---
     - old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    +--- # added
     - old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
     ---
    +...
    +# added
    ");
}

#[test]
fn generated_boundaries_obey_the_output_limit_and_roll_back() {
    let source = indoc! {r"
        # lead
        x"};
    let mut document = EditableDocument::parse_with_options(
        source,
        ParseOptions {
            max_input_bytes: 12,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let error = document
        .edit(|editor| editor.replace(&editor.root(), "- a"))
        .unwrap_err();
    assert_eq!(document.source(), source);
    insta::assert_snapshot!(error.message, @"emitted YAML exceeds input byte limit");
}

#[test]
fn nested_edits_leave_sufficient_root_framing_unchanged() {
    let source = indoc! {r"
        # prefix
        [old, keep]
    "};
    let diff = edit(source, |editor| {
        let items = editor.sequence_items(&editor.root())?;
        editor.replace(&items[0], "new")
    });
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     # prefix
    -[old, keep]
    +[new, keep]
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_inline_comments_can_use_additional_marker_lines() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            old # first
        "},
        indoc! {r"
            --- old # first
        "},
        indoc! {r"
            &a [ # open
              old
            ] # close
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        for text in [" added 1", " added 2", " added 3"] {
            document
                .edit(|editor| {
                    let root = editor.root();
                    let index = editor.node(&root)?.comments().len();
                    editor.insert_comment(&root, index, text, CommentPlacement::Inline)
                })
                .unwrap();
        }
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -old # first
    +--- # first
    +old # added 1
    +... # added 2
    +... # added 3

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    ---- old # first
    +---  # first
    +old # added 1
    +... # added 2
    +... # added 3

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     &a [ # open
    -  old
    -] # close
    +  old] # close
    +... # added 1
    +... # added 2
    +... # added 3
    ");
}

#[test]
fn nested_inline_comments_can_use_available_syntax_headers() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            value: [old] # first
        "},
        indoc! {r"
            value: &a old # first
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let value = editor.mapping_entries(&editor.root())?.remove(0).1;
            editor.insert_comment(&value, 1, " second", CommentPlacement::Inline)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: [old] # first
    +value: [ # first
    +  old] # second

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: &a old # first
    +value: &a # first
    +  old # second
    ");
}

#[test]
fn detached_suffix_comments_follow_inserted_root_content() {
    let source = indoc! {r"
        ---
        ...
    "};
    let document = edited(source, |editor| {
        let root = editor.root();
        editor.insert_comment(&root, 0, " gap", CommentPlacement::Detached)?;
        editor.insert_comment(&root, 1, " end", CommentPlacement::Inline)?;
        editor.replace(&root, "new")
    });
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,5 @@
     ---
    -...
    +new
    +
    +# gap
    +... # end
    ");
}

#[test]
fn generated_start_markers_preserve_root_prefixes() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            # child
            [ # open
              old
            ] # close
        "},
        indoc! {r"
            # child
            { # open
              key: old
            } # close
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            let index = editor.node(&root)?.comments().len();
            editor.insert_comment(&root, index, " added", CommentPlacement::Inline)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     # child
    -[ # open
    -  old
    -] # close
    +--- # open
    +[ # close
    +  old] # added

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     # child
    -{ # open
    -  key: old
    -} # close
    +--- # open
    +{ # close
    +  key: old} # added
    ");
}
