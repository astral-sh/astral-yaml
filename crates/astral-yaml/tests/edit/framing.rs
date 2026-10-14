//! Document framing, source separators, and comments between collection children.

use test_case::test_case;

use super::support::{edit, edited};
use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn descendant_inline_comments_are_separate_from_document_markers() {
    use astral_yaml::{comments::CommentPlacement, edit::NodeKind};

    let diff = super::support::edits(
        [
            indoc! {r"
                # root
                --- {key: value}
            "},
            indoc! {r"
                --- [item]
            "},
            indoc! {r"
                --- &root {key: value}
                ...
            "},
        ],
        |editor| {
            let root = editor.root();
            let child = match editor.node(&root)?.kind() {
                NodeKind::Mapping { .. } => editor.mapping_entries(&root)?[0].0.clone(),
                _ => editor.sequence_items(&root)?[0].clone(),
            };
            editor.insert_comment(&child, 0, " added", CommentPlacement::Inline)
        },
    );
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
     # root
    ---- {key: value}
    +--- ⏎
    +{? key # added
    +  : value}

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- [item]
    +--- ⏎
    +[item # added
    +  ]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    ---- &root {key: value}
    +--- ⏎
    +&root {? key # added
    +  : value}
     ...
    ");
}

#[test]
fn leading_comments_remain_adjacent_across_document_markers() {
    use astral_yaml::comments::CommentPlacement;

    let diff = super::support::edits(
        [
            indoc! {r"
                ---

                {key: value}
            "},
            indoc! {r"
                %YAML 1.2
                ---

                key: value
            "},
            indoc! {r"
                ---

                # retained
                &root {key: value}
            "},
        ],
        |editor| editor.insert_comment(&editor.root(), 0, " added", CommentPlacement::Leading),
    );
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    +# added
     ---
    -
     {key: value}

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     %YAML 1.2
    +# added
     ---
    -
     key: value

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    +# added
    +# retained
     ---
    -
    -# retained
     &root {key: value}
    ");
}

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

#[test_case(
    indoc! {r"
        # config
        %YAML 1.2
        ---
        key: old
        ...
          # tail
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    ");
    "version directive"
)]
#[test_case(
    indoc! {r"
        # config
        %TAG !e! tag:example.com,2026:
        ---
        key: !e!value old
        ... # end
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,5 +1,5 @@
         # config
         %TAG !e! tag:example.com,2026:
         ---
        -key: !e!value old
        +added: new
         ... # end
    ");
    "tag directive"
)]
#[test_case(
    indoc! {r"
        # config
        ---
        key:
          - old
        ...
          # tail
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    "nested sequence"
)]
fn structural_edits_preserve_framing_and_outer_comments(source: &str) -> String {
    // Document markers: https://yaml.org/spec/1.2.2/#922-document-markers
    edit(source, |editor| {
        let root = editor.root();
        editor.insert_entry(&root, 0, "added", "new")?;
        editor.remove_entry(&root, 1)
    })
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

#[test_case(
    indoc! {r"
        key: old # inline
        # tail
    "};
    "inline and trailing"
)]
#[test_case(
    indoc! {r"
        key: old # inline
    "};
    "inline only"
)]
#[test_case(
    indoc! {r"
        outer:
          key: old # inline
          # tail
        other: unchanged
    "};
    "nested inline and trailing"
)]
fn comment_terminators_compose_with_retained_suffixes(source: &str) {
    let mut document = EditableDocument::parse(source).unwrap();
    for _ in 0..3 {
        document
            .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
            .unwrap();
        assert_eq!(document.source(), source);
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
                    (true, 0) => editor.insert_item(&root, 0, "new").map(|_| ()),
                    (false, 0) => editor.insert_entry(&root, 0, "added", "new").map(|_| ()),
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
                            editor.insert_item(&root, 0, "new").map(|_| ())
                        } else {
                            editor.insert_entry(&root, 0, "added", "new").map(|_| ())
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

#[test_case(
    indoc! {r"
        a: old

        # detached

        b: removed
        c: kept
    "},
    false
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -2,5 +2,4 @@
         ⏎
         # detached
         ⏎
        -b: removed
         c: kept
    ");
    "mapping"
)]
#[test_case(
    indoc! {r"
        - old

        # detached

        - removed
        - kept
    "},
    true
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -2,5 +2,4 @@
         ⏎
         # detached
         ⏎
        -- removed
         - kept
    ");
    "sequence"
)]
fn removed_children_leave_their_comments_in_the_surviving_gap(
    source: &str,
    sequence: bool,
) -> String {
    edit(source, |editor| {
        let root = editor.root();
        if sequence {
            editor.remove_item(&root, 1)
        } else {
            editor.remove_entry(&root, 1)
        }
    })
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

#[test_case(
    indoc! {r"
        a: |+
          text

        # detached

        b: removed
    "},
    false
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    ");
    "mapping"
)]
#[test_case(
    indoc! {r"
        - |+
          text

        # detached

        - removed
    "},
    true
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    "sequence"
)]
fn removing_the_last_gap_preserves_block_scalar_content(source: &str, sequence: bool) -> String {
    // A document boundary keeps detached suffix separation outside the scalar.
    edit(source, |editor| {
        let root = editor.root();
        if sequence {
            editor.remove_item(&root, 1)
        } else {
            editor.remove_entry(&root, 1)
        }
    })
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
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            let count = editor.node(&root)?.comments().len();
            editor.insert_comment(&root, count, " added", CommentPlacement::Trailing)?;
            for _ in 0..count {
                editor.remove_comment(&root, 0)?;
            }
            Ok(())
        }));
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

#[test_case(
    "--- # header"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1,2 @@
        ---- # header
        \ No newline at end of file
        +---  # added
        +a: new
        \ No newline at end of file
    ");
    "header comment"
)]
#[test_case(
    indoc! {r"
        ---
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1,2 @@
        ----
        +--- # added
        +a: new
        \ No newline at end of file
    ");
    "empty root"
)]
#[test_case(
    indoc! {r"
        --- old
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1,2 @@
        ---- old
        +---  # added
        +a: new
    ");
    "scalar root"
)]
fn block_roots_can_use_the_document_marker_for_inline_comments(source: &str) -> String {
    use astral_yaml::comments::CommentPlacement;

    edit(source, |editor| {
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
}

#[test_case(
    indoc! {r"
        ---
        - old
        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,3 +1,4 @@
        +# added
         ---
         - old
         ...
    ");
    "sequence root"
)]
#[test_case(
    indoc! {r"
        --- # header
        key: old
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,3 @@
        +# added
         --- # header
         key: old
    ");
    "mapping root with header"
)]
#[test_case(
    indoc! {r"
        ---

        ... # end
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,3 +1,4 @@
         ---
         ⏎
        +# added
         ... # end
    ");
    "empty root with end comment"
)]
fn root_leading_comments_use_the_start_marker_boundary(source: &str) -> String {
    use astral_yaml::comments::CommentPlacement;

    edit(source, |editor| {
        editor.insert_comment(&editor.root(), 0, " added", CommentPlacement::Leading)
    })
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
        diffs.push(edit(source, |editor| {
            editor.replace(
                &editor.root(),
                indoc! {r"
                    ---
                "},
            )
        }));
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

#[test_case(
    indoc! {r"
        ---
        [ # open
          old
        ] # close
        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    ");
    "sequence"
)]
#[test_case(
    indoc! {r"
        ---
        { # open
          key: old
        } # close
        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
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
    "mapping"
)]
fn collection_comments_can_use_both_document_marker_headers(source: &str) -> String {
    edit(source, |editor| {
        editor.set_collection_style(&editor.root(), CollectionStyle::Block)
    })
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
            diffs.push(edit(source, |editor| {
                editor.set_scalar_style(&editor.root(), style)
            }));
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
    let source = indoc! {r"
        ---
        &root
        # header
        |
          old
        ...
        # tail
    "};
    let document = edited(source, |editor| {
        editor.replace_comment(&editor.root(), 1, " changed")
    });
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
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

#[test_case(
    indoc! {r"
        [] # old
        # trailing
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,2 +1,2 @@
        -[] # old
        +[] # new
         # trailing
    ");
    "empty sequence"
)]
#[test_case(
    indoc! {r"
        value # old
        # trailing

        # detached
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,4 +1,4 @@
        -value # old
        +value # new
         # trailing
         ⏎
         # detached
    ");
    "scalar with suffixes"
)]
#[test_case(
    indoc! {r"
        --- &root {key: value} # old
        # trailing
        ...
    "}
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1,3 +1,3 @@
        ---- &root {key: value} # old
        +--- &root {key: value} # new
         # trailing
         ...
    ");
    "anchored mapping"
)]
fn replaced_inline_comments_reuse_the_root_line_terminator(source: &str) -> String {
    use astral_yaml::comments::CommentPlacement;

    edit(source, |editor| {
        let root = editor.root();
        editor.remove_comment(&root, 0)?;
        editor.insert_comment(&root, 0, " new", CommentPlacement::Inline)
    })
}
