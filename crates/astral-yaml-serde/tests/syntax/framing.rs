//! Document framing, source separators, and comments between collection children.

use test_case::test_case;

use super::support::{edit, edited};
use crate::syntax::DocumentSyntax;
use astral_yaml::CollectionStyle;
use astral_yaml_dev::{edit_diff, indoc};

// https://yaml.org/spec/1.2.2/#92-streams
#[test_case("|")]
#[test_case(">")]
fn a_stream_suffix_before_the_document_keeps_its_inline_comment(style: &str) {
    use crate::Editable;
    use astral_yaml::comments::CommentPlacement;

    let prefix = "... # first\n... # second\n";
    let source = format!("{prefix}{style}\n  old\n");
    let mut document = Editable::<String>::parse(&source).unwrap();
    let root: [&str; 0] = [];
    document.set_at(root, "new").unwrap();
    assert_eq!(document.source(), format!("{prefix}{style}-\n  new\n"));

    document
        .edit(|mut root| root.append_comment(" after", CommentPlacement::Detached))
        .unwrap();
    assert_eq!(
        document.source(),
        format!("{prefix}{style}-\n  new\n...\n\n# after\n")
    );
}

#[test]
fn a_marked_stream_suffix_after_the_document_keeps_its_inline_comment() {
    let source = "|\n  old\n\u{feff}... # end\n... # extra\n";
    let mut document = crate::Editable::<String>::parse(source).unwrap();
    let root: [&str; 0] = [];
    document.set_at(root, "new").unwrap();
    assert_eq!(
        document.source(),
        "|-\n  new\n\u{feff}... # end\n... # extra\n"
    );
}

// https://yaml.org/spec/1.2.2/#911-document-prefix
#[test_case("\u{feff}")]
#[test_case("\n\u{feff}")]
#[test_case("# before\n\u{feff}\u{feff}")]
fn inserting_a_document_marker_preserves_all_prefix_marks(prefix: &str) {
    let mut document = DocumentSyntax::parse(&format!("{prefix}old")).unwrap();
    document
        .edit_syntax(|e| e.replace(e.root(), "---"))
        .unwrap();
    assert_eq!(document.source(), format!("{prefix}---\n"));
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_case("|-\n  x\n\u{feff}\n# tail\n", "[a, b]" => "[a, b]\n\u{feff}# tail\n"; "trailing attachment after a marked blank line")]
#[test_case("---\n\u{feff}", "[a, b]" => "---\n[a, b]\n\u{feff}"; "empty document before a suffix mark")]
#[test_case("---\n\u{feff}... # tail\n", "'new'" => "---\n'new'\n\u{feff}... # tail\n"; "empty document before a marked end marker")]
#[test_case("# first\n\u{feff}old", "[x, y]" => "# first\n\u{feff}[x, y]"; "prefix on replaced line")]
#[test_case("\u{feff}old\n\u{feff}# tail\n", "|-\n  x" => "\u{feff}|-\n  x\n\u{feff}# tail\n"; "suffix after block scalar")]
#[test_case("# first\n\u{feff}# second\n\u{feff}\u{feff}old\n\u{feff}# tail\n", "[x, y]" => "# first\n\u{feff}# second\n\u{feff}\u{feff}[x, y]\n\u{feff}# tail\n"; "marks between comments")]
#[test_case("# first\n\u{feff}'x\u{feff}'", "new" => "# first\n\u{feff}new"; "removed quoted content")]
#[test_case("# first\n\u{feff}old", "'y\u{feff}'" => "# first\n\u{feff}\"y\\uFEFF\""; "imported quoted content")]
fn replacing_a_root_keeps_its_stream_marks(source: &str, replacement: &str) -> String {
    let mut document = DocumentSyntax::parse(source).unwrap();
    document
        .edit_syntax(|e| e.replace(e.root(), replacement))
        .unwrap();
    document.into_yaml()
}

#[test]
fn normalization_keeps_the_mark_before_a_collection() {
    let source = "# first\n\u{feff}-   old\n";
    let normalized = DocumentSyntax::parse(source)
        .unwrap()
        .normalized_yaml()
        .unwrap();
    assert_eq!(normalized, "# first\n\u{feff}- old\n");
    assert_eq!(
        DocumentSyntax::parse(&normalized)
            .unwrap()
            .normalized_yaml()
            .unwrap(),
        normalized
    );
}

#[test_case(" # c\na: b\n" => " # c\n\n{}\n"; "adjacent")]
#[test_case(" # c\n\na: b\n" => " # c\n\n{}\n"; "already separated")]
#[test_case(" # first\n # last\na: b\n" => " # first\n # last\n\n{}\n"; "multiple comments")]
#[test_case("---\n # c\na: b\n" => "---\n # c\n\n{}\n"; "explicit document")]
fn removing_the_last_entry_keeps_detached_prefix_comments(source: &str) -> String {
    let mut document = DocumentSyntax::parse(source).unwrap();
    document
        .edit(|root| root.into_mapping()?.remove("a"))
        .unwrap();
    document.into_yaml()
}

#[test_case("{a: b}" => " # c\n\n{a: b}\n"; "flow mapping")]
#[test_case("new" => " # c\n\nnew\n"; "scalar")]
fn replacing_a_block_mapping_keeps_detached_prefix_comments(replacement: &str) -> String {
    let source = " # c\na: b\n";
    edited(source, |editor| editor.replace(editor.root(), replacement)).into_yaml()
}

#[test]
fn an_inserted_end_marker_shares_a_line_with_the_stream_mark() {
    use astral_yaml::comments::CommentPlacement;

    let source = "\u{feff}# first\n\n\u{feff}\u{feff}old # inline\n\u{feff}";
    let document = edited(source, |editor| {
        let root = editor.root();
        let count = editor.node(root)?.comments().len();
        editor.insert_comment(root, count, " audit", CommentPlacement::Inline)
    });
    assert_eq!(
        document.source(),
        "\u{feff}# first\n\n\u{feff}\u{feff}old # inline\n\u{feff}... # audit\n"
    );
}

#[test]
fn a_marked_blank_line_can_be_removed_between_comment_headers() {
    use astral_yaml::comments::CommentPlacement;

    let source = "o\n... # first\n\u{feff}\n... # second\n";
    let mut document = DocumentSyntax::parse(source).unwrap();
    document
        .edit(|mut root| root.append_comment(" added", CommentPlacement::Trailing))
        .unwrap();
    assert_eq!(
        document.source(),
        "o\n... # first\n\u{feff}... # second\n# added\n"
    );
}

#[test_case(0 => (None, "# first\n\u{feff}[x, y]".into()); "exact limit")]
#[test_case(1 => (Some("emitted YAML exceeds input byte limit"), "# first\n\u{feff}x".into()); "one byte short")]
fn retained_stream_marks_count_toward_the_edit_limit(
    shortfall: usize,
) -> (Option<&'static str>, String) {
    let mut document = DocumentSyntax::parse_with_options(
        "# first\n\u{feff}x",
        astral_yaml::ParseOptions {
            max_input_bytes: "# first\n\u{feff}[x, y]".len() - shortfall,
            ..Default::default()
        },
    )
    .unwrap();
    let error = document
        .edit_syntax(|e| e.replace(e.root(), "[x, y]"))
        .err()
        .map(|err| err.message);
    (error, document.into_yaml())
}

#[test]
fn descendant_inline_comments_are_separate_from_document_markers() {
    use crate::syntax::NodeKind;
    use astral_yaml::comments::CommentPlacement;

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
            let child = match editor.node(root)?.kind() {
                NodeKind::Mapping { .. } => editor.mapping_entries(root)?[0].0,
                _ => editor.sequence_items(root)?[0],
            };
            editor.insert_comment(child, 0, " added", CommentPlacement::Inline)
        },
    );
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,4 @@
     # root
    ---- {key: value}
    +--- ⏎
    +{? key # added
    +  : value}

    @@ -1 +1,3 @@
    ---- [item]
    +--- ⏎
    +[item # added
    +  ]

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
        |editor| editor.insert_comment(editor.root(), 0, " added", CommentPlacement::Leading),
    );
    insta::assert_snapshot!(diff, @"
    @@ -1,3 +1,3 @@
    +# added
     ---
    -
     {key: value}

    @@ -1,4 +1,4 @@
     %YAML 1.2
    +# added
     ---
    -
     key: value

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
                        root,
                        indoc! {r"
                            ---
                        "},
                    )
                } else {
                    let index = editor.node(root)?.comments().len() - 1;
                    editor.insert_comment(root, index, " added", CommentPlacement::Trailing)
                }
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,5 +1,6 @@
     # prefix
     --- old
    -
     ...
    +# added
    +
     # gap

    @@ -1,5 +1,5 @@
     # prefix
    ---- old
    +--- ⏎
    +...
     ⏎
    -...
     # gap

    @@ -1,6 +1,6 @@
     # prefix
     --- old
    -
     ...
    +# added
      \t ⏎
       # gap

    @@ -1,6 +1,5 @@
     # prefix
    ---- old
    -
    +--- ⏎
     ...
      \t ⏎
       # gap

    @@ -1,5 +1,6 @@
     # prefix
     --- old
    -
     ... # end
    +# added
    +
     # gap

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
        editor.insert_entry(root, 0, "added", "new")?;
        editor.remove_entry(root, 1)
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
            editor.replace(editor.root(), replacement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,3 +1,4 @@
     ---
    +new
     ... # end
     # tail

    @@ -1,3 +1,4 @@
     ---
    +[a, b]
     ... # end
     # tail

    @@ -1,3 +1,4 @@
     ---
    +{a: b}
     ... # end
     # tail

    @@ -1,3 +1,5 @@
     ---
    +- a
    +- b
     ... # end
     # tail

    @@ -1,3 +1,4 @@
     ---
    +a: b
     ... # end
     # tail

    @@ -1,3 +1,5 @@
     ---
    +|
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
    let mut document = DocumentSyntax::parse(source).unwrap();
    for _ in 0..3 {
        document
            .edit_syntax(|editor| {
                editor.set_collection_style(editor.root(), CollectionStyle::Block)
            })
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
        for operation in 0..3 {
            diffs.push(edit(source, |editor| {
                let root = editor.root();
                match (sequence, operation) {
                    (true, 0) => editor.insert_item(root, 0, "new").map(|_| ()),
                    (false, 0) => editor.insert_entry(root, 0, "added", "new").map(|_| ()),
                    (true, 1) => editor.remove_item(root, 1),
                    (false, 1) => editor.remove_entry(root, 1),
                    (true, _) => editor.replace(
                        root,
                        indoc! {r"
                            - new
                            - |+
                              other

                        "},
                    ),
                    (false, _) => editor.replace(
                        root,
                        indoc! {r"
                            a: new
                            b: |+
                              other

                        "},
                    ),
                }
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,7 +1,8 @@
    +added: new
     a: old
     ⏎
     # detached
     ⏎
     b: |+
       text
    -
    +  ⏎

    @@ -1,7 +1,3 @@
     a: old
     ⏎
     # detached
    -
    -b: |+
    -  text
    -

    @@ -1,7 +1,7 @@
    -a: old
    +a: new
     ⏎
     # detached
     ⏎
     b: |+
    -  text
    -
    +  other
    +  ⏎

    @@ -1,7 +1,8 @@
    +- new
     - old
     ⏎
     # detached
     ⏎
     - |+
       text
    -
    +  ⏎

    @@ -1,7 +1,3 @@
     - old
     ⏎
     # detached
    -
    -- |+
    -  text
    -

    @@ -1,7 +1,7 @@
    -- old
    +- new
     ⏎
     # detached
     ⏎
     - |+
    -  text
    -
    +  other
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
            editor.remove_item(root, 1)
        } else {
            editor.remove_entry(root, 1)
        }
    })
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
        @@ -1,6 +1,6 @@
         a: |+
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
        @@ -1,6 +1,6 @@
         - |+
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
            editor.remove_item(root, 1)
        } else {
            editor.remove_entry(root, 1)
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
            let count = editor.node(root)?.comments().len();
            editor.insert_comment(root, count, " added", CommentPlacement::Trailing)?;
            for _ in 0..count {
                editor.remove_comment(root, 0)?;
            }
            Ok(())
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,2 +1,3 @@
     ---
     ...
    +# added

    @@ -1,2 +1,3 @@
    ---- # header
    -... # end
    +--- ⏎
    +... ⏎
    +# added

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
        while !editor.node(root)?.comments().is_empty() {
            editor.remove_comment(root, 0)?;
        }
        editor.insert_comment(root, 0, " added", CommentPlacement::Inline)?;
        editor.replace(
            root,
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
        editor.insert_comment(editor.root(), 0, " added", CommentPlacement::Leading)
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
                editor.root(),
                indoc! {r"
                    ---
                "},
            )
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,4 +1,3 @@
     # prefix
     ---
    -old
     ...

    @@ -1,4 +1,3 @@
     # prefix
    ---- old
    -
    +--- ⏎
     ...

    @@ -1,5 +1,3 @@
     # prefix
    ---- |-
    -  old
    -
    +--- ⏎
     ...

    @@ -1,3 +1,2 @@
    ----
    -old # inline
    +--- # inline
     ...

    @@ -1,4 +1,3 @@
     ---
    -old
    -# tail
     ...
    +# tail

    @@ -1,5 +1,4 @@
     ---
    -old
    +...
     ⏎
     # gap
    -...

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
        editor.set_collection_style(editor.root(), CollectionStyle::Block)
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
                editor.set_scalar_style(editor.root(), style)
            }));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,2 +1,3 @@
    +  |-
       old
     # tail

    @@ -1,2 +1,3 @@
    +  >-
       old
     # tail

    @@ -1,2 +1,3 @@
    -old
    -  # tail
    +|-
    +  old
    +# tail

    @@ -1,2 +1,3 @@
    -old
    -  # tail
    +>-
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
        editor.replace_comment(editor.root(), 1, " changed")
    });
    let diff = edit_diff(source, document.source());
    insta::assert_snapshot!(diff, @"
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
        editor.remove_comment(root, 0)?;
        editor.insert_comment(root, 0, " new", CommentPlacement::Inline)
    })
}
