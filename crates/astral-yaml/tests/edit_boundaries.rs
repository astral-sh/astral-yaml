//! Root edits may require document boundaries to preserve comment ownership.

use astral_yaml::{ParseOptions, comments::CommentPlacement, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_edits_create_missing_document_boundaries() {
    // Repeated document suffixes remain part of a single-document stream:
    // https://yaml.org/spec/1.2.2/#92-streams
    let cases = [
        ("# leading\nold\n", "- new\n"),
        ("old # inline\n", "- new\n"),
        ("old", "---\n"),
        ("old\n# tail\n", "---\n"),
        ("old\n\n# gap\n", "---\n"),
        ("[ # open\n  old\n] # close\n", "- new\n"),
        (
            "&a # property\n[ # open\n  old\n] # close\n# tail\n",
            "- new\n",
        ),
        (
            "&a # property\n[ # open\n  old\n] # close\n... # end\n",
            "- new\n",
        ),
        ("\u{feff}\n# leading\nold # inline", "- new\n"),
    ];
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, fragment) in cases {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.replace(&editor.root(), fragment))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     # leading␍
    -old␍
    +---␍
    +- new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline␍
    +--- # inline␍
    +- new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    \ No newline at end of file
    +---

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -old␍
    +---␍
    +...␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old␍
    +---␍
    +...␍
     ␍
     # gap␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -[ # open␍
    -  old␍
    -] # close␍
    +--- # open␍
    +- new␍
    +... # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property␍
    -[ # open␍
    -  old␍
    -] # close␍
    +--- # property␍
    +- new␍
    +... # open␍
    +... # close␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property␍
    -[ # open␍
    -  old␍
    -] # close␍
    +--- # property␍
    +- new␍
    +... # open␍
    +... # close␍
     ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ﻿␍
     # leading␍
    -old # inline
    \ No newline at end of file
    +--- # inline␍
    +- new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     # leading␍-old␍+---␍+- new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline␍+--- # inline␍+- new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    \ No newline at end of file
    +---

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -old␍+---␍+...␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old␍+---␍+...␍ ␍ # gap␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -[ # open␍-  old␍-] # close␍+--- # open␍+- new␍+... # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property␍-[ # open␍-  old␍-] # close␍+--- # property␍+- new␍+... # open␍+... # close␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -&a # property␍-[ # open␍-  old␍-] # close␍+--- # property␍+- new␍+... # open␍+... # close␍ ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ﻿␍ # leading␍-old # inline
    \ No newline at end of file
    +--- # inline␍+- new
    \ No newline at end of file
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn inserted_comments_can_create_document_boundaries() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, placement) in [
            ("- old\n", CommentPlacement::Leading),
            ("- old\n", CommentPlacement::Inline),
            ("---\n", CommentPlacement::Trailing),
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.insert_comment(&editor.root(), 0, " added", placement))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    +# added␍
    +---␍
     - old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    +--- # added␍
     - old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
     ---␍
    +...␍
    +# added␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    +# added␍+---␍ - old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    +--- # added␍ - old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
     ---␍+...␍+# added␍
    ");
}

#[test]
fn generated_boundaries_obey_the_output_limit_and_roll_back() {
    let source = "# lead\nx";
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
    let source = "# prefix\n[old, keep]\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let items = editor.sequence_items(&editor.root())?;
            editor.replace(&items[0], "new")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
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
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "old # first\n",
            "--- old # first\n",
            "&a [ # open\n  old\n] # close\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            for text in [" added 1", " added 2", " added 3"] {
                document
                    .edit(|editor| {
                        let root = editor.root();
                        let index = editor.node(&root)?.comments().len();
                        editor.insert_comment(&root, index, text, CommentPlacement::Inline)
                    })
                    .unwrap();
            }
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -old # first␍
    +--- # first␍
    +old # added 1␍
    +... # added 2␍
    +... # added 3␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    ---- old # first␍
    +---  # first␍
    +old # added 1␍
    +... # added 2␍
    +... # added 3␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     &a [ # open␍
    -  old␍
    -] # close␍
    +  old] # close␍
    +... # added 1␍
    +... # added 2␍
    +... # added 3␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    -old # first␍+--- # first␍+old # added 1␍+... # added 2␍+... # added 3␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,4 @@
    ---- old # first␍+---  # first␍+old # added 1␍+... # added 2␍+... # added 3␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     &a [ # open␍-  old␍-] # close␍+  old] # close␍+... # added 1␍+... # added 2␍+... # added 3␍
    ");
}

#[test]
fn nested_inline_comments_can_use_available_syntax_headers() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in ["value: [old] # first\n", "value: &a old # first\n"] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let value = editor.mapping_entries(&editor.root())?.remove(0).1;
                    editor.insert_comment(&value, 1, " second", CommentPlacement::Inline)
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: [old] # first␍
    +value: [ # first␍
    +  old] # second␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: &a old # first␍
    +value: &a # first␍
    +  old # second␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: [old] # first␍+value: [ # first␍+  old] # second␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -value: &a old # first␍+value: &a # first␍+  old # second␍
    ");
}

#[test]
fn detached_suffix_comments_follow_inserted_root_content() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "---\n...\n".replace('\n', newline);
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                editor.insert_comment(&root, 0, " gap", CommentPlacement::Detached)?;
                editor.insert_comment(&root, 1, " end", CommentPlacement::Inline)?;
                editor.replace(&root, "new")
            })
            .unwrap();
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,5 @@
     ---
    -...
    +new
    +
    +# gap
    +... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,5 @@
     ---␍
    -...␍
    +new␍
    +␍
    +# gap␍
    +... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,5 @@
     ---␍-...␍+new␍+␍+# gap␍+... # end␍
    ");
}

#[test]
fn generated_start_markers_precede_descendant_prefixes() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "# child\n[ # open\n  old\n] # close\n",
            "# child\n{ # open\n  key: old\n} # close\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let root = editor.root();
                    let index = editor.node(&root)?.comments().len();
                    editor.insert_comment(&root, index, " added", CommentPlacement::Inline)
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child
    -[ # open
    -  old
    -] # close
    +--- # open
    +[ # close
    +  # child
    +  old] # added

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child
    -{ # open
    -  key: old
    -} # close
    +--- # open
    +{ # close
    +  # child
    +  key: old} # added

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child␍
    -[ # open␍
    -  old␍
    -] # close␍
    +--- # open␍
    +[ # close␍
    +  # child␍
    +  old] # added␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child␍
    -{ # open␍
    -  key: old␍
    -} # close␍
    +--- # open␍
    +{ # close␍
    +  # child␍
    +  key: old} # added␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child␍-[ # open␍-  old␍-] # close␍+--- # open␍+[ # close␍+  # child␍+  old] # added␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -# child␍-{ # open␍-  key: old␍-} # close␍+--- # open␍+{ # close␍+  # child␍+  key: old} # added␍
    ");
}
