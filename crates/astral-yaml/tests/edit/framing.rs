//! Document framing, source separators, and comments between collection children.

use super::support::edit;
use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn suffix_boundaries_preserve_detached_comments() {
    use astral_yaml::comments::CommentPlacement;

    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            # prefix
            --- old

            ...
            # gap
        "},
        "# prefix\n--- old\n\n...\n \t \n  # gap\n",
        indoc! {r"
            # prefix
            --- old

            ... # end
            # gap
        "},
    ] {
        for omit in [false, true] {
            diffs.push(edit(source, |editor| {
                let root = editor.root();
                if omit {
                    editor.replace(
                        &root,
                        indoc! {r"
                            ---
                        "},
                    )
                } else {
                    let index = editor.node(&root)?.comments().len() - 1;
                    editor.insert_comment(&root, index, " added", CommentPlacement::Trailing)
                }
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,6 @@
     # prefix
     --- old
    -
     ...
    +# added
    +
     # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     # prefix
    ---- old
    +--- ⏎
    +...
     ⏎
    -...
     # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     # prefix
     --- old
    -
     ...
    +# added
      \t ⏎
       # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,5 @@
     # prefix
    ---- old
    -
    +--- ⏎
     ...
      \t ⏎
       # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,6 @@
     # prefix
     --- old
    -
     ... # end
    +# added
    +
     # gap

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     # prefix
    ---- old
    +--- ⏎
    +... # end
     ⏎
    -... # end
     # gap
    ");
}

#[test]
fn structural_edits_preserve_framing_and_outer_comments() {
    // Document markers: https://yaml.org/spec/1.2.2/#922-document-markers
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            # config
            %YAML 1.2
            ---
            key: old
            ...
              # tail
        "},
        indoc! {r"
            # config
            %TAG !e! tag:example.com,2026:
            ---
            key: !e!value old
            ... # end
        "},
        indoc! {r"
            # config
            ---
            key:
              - old
            ...
              # tail
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            editor.insert_entry(&root, 0, "added", "new")?;
            editor.remove_entry(&root, 1)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     # config
     %YAML 1.2
     ---
    -key: old
    +added: new
     ...
       # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     # config
     %TAG !e! tag:example.com,2026:
     ---
    -key: !e!value old
    +added: new
     ... # end

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,5 @@
     # config
     ---
    -key:
    -  - old
    +added: new
     ...
       # tail
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn inserted_roots_are_separated_from_end_markers() {
    let mut diffs = Vec::new();
    for replacement in [
        "new",
        "[a, b]",
        "{a: b}",
        indoc! {r"
            - a
            - b
        "},
        indoc! {r"
            a: b
        "},
        indoc! {r"
            |
              text
        "},
    ] {
        let source = indoc! {r"
            ---
            ... # end
            # tail
        "};

        diffs.push(edit(source, |editor| {
            editor.replace(&editor.root(), replacement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---
    +new
     ... # end
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---
    +[a, b]
     ... # end
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---
    +{a: b}
     ... # end
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---
    +- a
    +- b
     ... # end
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---
    +a: b
     ... # end
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---
    +|2
    +  text
     ... # end
     # tail
    ");
}

#[test]
fn comment_terminators_compose_with_retained_suffixes() {
    for source in [
        indoc! {r"
            key: old # inline
            # tail
        "},
        indoc! {r"
            key: old # inline
        "},
        indoc! {r"
            outer:
              key: old # inline
              # tail
            other: unchanged
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        for _ in 0..3 {
            document
                .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
                .unwrap();
            assert_eq!(document.source(), source);
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn detached_comment_gaps_survive_collection_mutations() {
    let mut diffs = Vec::new();
    for (source, sequence) in [
        (
            indoc! {r"
                a: old

                # detached

                b: |+
                  text

            "},
            false,
        ),
        (
            indoc! {r"
                - old

                # detached

                - |+
                  text

            "},
            true,
        ),
    ] {
        for operation in 0..4 {
            diffs.push(edit(source, |editor| {
                let root = editor.root();
                match (sequence, operation) {
                    (true, 0) => editor.insert_item(&root, 0, "new"),
                    (false, 0) => editor.insert_entry(&root, 0, "added", "new"),
                    (true, 1) => editor.remove_item(&root, 1),
                    (false, 1) => editor.remove_entry(&root, 1),
                    (true, 2) => editor.replace(
                        &root,
                        indoc! {r"
                            - new
                            - |+
                              other

                        "},
                    ),
                    (false, 2) => editor.replace(
                        &root,
                        indoc! {r"
                            a: new
                            b: |+
                              other

                        "},
                    ),
                    _ => {
                        // Import into a nonzero arena offset, then mutate the
                        // imported collection before committing it.
                        editor.replace(&root, source)?;
                        if sequence {
                            editor.insert_item(&root, 0, "new")
                        } else {
                            editor.insert_entry(&root, 0, "added", "new")
                        }
                    }
                }
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +added: new
     a: old
     ⏎
     # detached
     ⏎
    -b: |+
    +b: |2+
       text
    -
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     a: old
     ⏎
     # detached
    -
    -b: |+
    -  text
    -

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -a: old
    +a: new
     ⏎
     # detached
     ⏎
    -b: |+
    -  text
    -
    +b: |2+
    +  other
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +added: new
     a: old
     ⏎
     # detached
     ⏎
    -b: |+
    +
    +# detached
    +
    +b: |2+
       text
    -
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +- new
     - old
     ⏎
     # detached
     ⏎
    -- |+
    +- |2+
       text
    -
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     - old
     ⏎
     # detached
    -
    -- |+
    -  text
    -

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -- old
    +- new
     ⏎
     # detached
     ⏎
    -- |+
    -  text
    -
    +- |2+
    +  other
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +- new
     - old
     ⏎
     # detached
     ⏎
    -- |+
    +
    +# detached
    +
    +- |2+
       text
    -
    +  ⏎
    ");
}

#[test]
fn removed_children_leave_their_comments_in_the_surviving_gap() {
    let mut diffs = Vec::new();
    for (source, sequence) in [
        (
            indoc! {r"
                a: old

                # detached

                b: removed
                c: kept
            "},
            false,
        ),
        (
            indoc! {r"
                - old

                # detached

                - removed
                - kept
            "},
            true,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            if sequence {
                editor.remove_item(&root, 1)
            } else {
                editor.remove_entry(&root, 1)
            }
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -2,5 +2,4 @@
     ⏎
     # detached
     ⏎
    -b: removed
     c: kept

    --- before.yaml
    +++ after.yaml
    @@ -2,5 +2,4 @@
     ⏎
     # detached
     ⏎
    -- removed
     - kept
    ");
}

#[test]
fn new_detached_comments_share_the_following_comment_gap() {
    use astral_yaml::comments::CommentPlacement;
    let source = indoc! {r"
        a: old

        # retained

        b: |+
          text

    "};
    let diff = edit(source, |editor| {
        editor.insert_comment(&editor.root(), 0, " added", CommentPlacement::Detached)
    });
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,10 @@
     a: old
     ⏎
    +# added
    +
    +
     # retained
     ⏎
    -b: |+
    +b: |2+
       text
    -
    +  ⏎
    ");
}

#[test]
fn removing_the_last_gap_preserves_block_scalar_content() {
    // A document boundary keeps detached suffix separation outside the scalar.
    let mut diffs = Vec::new();
    for (source, sequence) in [
        (
            indoc! {r"
                a: |+
                  text

                # detached

                b: removed
            "},
            false,
        ),
        (
            indoc! {r"
                - |+
                  text

                # detached

                - removed
            "},
            true,
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                if sequence {
                    editor.remove_item(&root, 1)
                } else {
                    editor.remove_entry(&root, 1)
                }
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
    -a: |+
    +a: |2+
       text
    +  ⏎
    +...
     ⏎
     # detached
    -
    -b: removed

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
    -- |+
    +- |2+
       text
    +  ⏎
    +...
     ⏎
     # detached
    -
    -- removed
    ");
}

#[test]
fn root_comments_can_use_framing_without_retained_comment_anchors() {
    use astral_yaml::comments::CommentPlacement;

    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            ---
            ...
        "},
        indoc! {r"
            --- # header
            ... # end
        "},
        indoc! {r"
            ---

            ...
            # tail
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                let count = editor.node(&root)?.comments().len();
                editor.insert_comment(&root, count, " added", CommentPlacement::Trailing)?;
                for _ in 0..count {
                    editor.remove_comment(&root, 0)?;
                }
                Ok(())
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     ---
     ...
    +# added

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    ---- # header
    -... # end
    +--- ⏎
    +... ⏎
    +# added

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     ---
     ⏎
     ...
    -# tail
    +# added
    ");
}

#[test]
fn block_roots_can_use_the_document_marker_for_inline_comments() {
    use astral_yaml::comments::CommentPlacement;

    let mut diffs = Vec::new();
    for source in [
        "--- # header",
        indoc! {r"
            ---
        "},
        indoc! {r"
            --- old
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                while !editor.node(&root)?.comments().is_empty() {
                    editor.remove_comment(&root, 0)?;
                }
                editor.insert_comment(&root, 0, " added", CommentPlacement::Inline)?;
                editor.replace(
                    &root,
                    indoc! {r"
                        a: new
                    "},
                )
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # header
    \ No newline at end of file
    +---  # added
    +a: new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ----
    +--- # added
    +a: new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- old
    +---  # added
    +a: new
    ");
}

#[test]
fn root_leading_comments_use_the_start_marker_boundary() {
    use astral_yaml::comments::CommentPlacement;

    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            ---
            - old
            ...
        "},
        indoc! {r"
            --- # header
            key: old
        "},
        indoc! {r"
            ---

            ... # end
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                editor.insert_comment(&editor.root(), 0, " added", CommentPlacement::Leading)
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    +# added
     ---
     - old
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    +# added
     --- # header
     key: old

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---
     ⏎
    +# added
     ... # end
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_omission_retains_comments_at_document_boundaries() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            # prefix
            ---
            old
            ...
        "},
        indoc! {r"
            # prefix
            --- old

            ...
        "},
        indoc! {r"
            # prefix
            --- |-
              old

            ...
        "},
        indoc! {r"
            ---
            old # inline
            ...
        "},
        indoc! {r"
            ---
            old
            # tail
            ...
        "},
        indoc! {r"
            ---
            old

            # gap
            ...
        "},
        indoc! {r"
            ---
            # leading
            old # inline
            ...
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                editor.replace(
                    &editor.root(),
                    indoc! {r"
                        ---
                    "},
                )
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
     # prefix
     ---
    -old
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
     # prefix
    ---- old
    -
    +--- ⏎
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,3 @@
     # prefix
    ---- |-
    -  old
    -
    +--- ⏎
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    ----
    -old # inline
    +--- # inline
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
     ---
    -old
    -# tail
     ...
    +# tail

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,4 @@
     ---
    -old
    +...
     ⏎
     # gap
    -...

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
    ----
     # leading
    -old # inline
    +--- # inline
     ...
    ");
}

#[test]
fn collection_comments_can_use_both_document_marker_headers() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            ---
            [ # open
              old
            ] # close
            ...
        "},
        indoc! {r"
            ---
            { # open
              key: old
            } # close
            ...
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,3 @@
    ----
    -[ # open
    -  old
    -] # close
    -...
    +--- # open
    +- old
    +... # close

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,3 @@
    ----
    -{ # open
    -  key: old
    -} # close
    -...
    +--- # open
    +key: old
    +... # close
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn scalar_style_edits_keep_comments_outside_the_content() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
              old
            # tail
        "},
        indoc! {r"
            old
              # tail
        "},
    ] {
        for style in [
            astral_yaml::ScalarStyle::Literal,
            astral_yaml::ScalarStyle::Folded,
        ] {
            let mut document = EditableDocument::parse(source).unwrap();
            document
                .edit(|editor| editor.set_scalar_style(&editor.root(), style))
                .unwrap();
            diffs.push(edit_diff(source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    +  |2-
       old
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    +  >2-
       old
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -old
    -  # tail
    +|2-
    +  old
    +# tail

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -old
    -  # tail
    +>2-
    +  old
    +# tail
    ");
}

#[test]
fn framing_text_edits_retain_the_unchanged_body_layout() {
    let mut diffs = Vec::new();
    let source = indoc! {r"
        ---
        &root
        # header
        |
          old
        ...
        # tail
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| editor.replace_comment(&editor.root(), 1, " changed"))
        .unwrap();
    diffs.push(edit_diff(source, document.source()));
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -4,4 +4,4 @@
     |
       old
     ...
    -# tail
    +# changed
    ");
}
