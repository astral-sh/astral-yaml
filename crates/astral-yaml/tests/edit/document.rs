use super::support::{edit, edited, edits};
use astral_yaml::{
    CollectionStyle, ParseOptions, ScalarStyle, comments::CommentPlacement, edit::EditableDocument,
};
use astral_yaml_dev::{edit_diff, formatdoc, indoc};

#[test]
fn emission_limits_roll_back_root_and_subtree_rewrites() {
    let quoted = format!("\"{}\"", "\\x01".repeat(8));
    let replacement = format!("\"\\x02{}\"", "\\x01".repeat(7));
    for nested in [false, true] {
        let source = if nested {
            formatdoc! {r"
                key: {quoted}
            "}
        } else {
            quoted.clone()
        };
        let mut document = EditableDocument::parse_with_options(
            &source,
            ParseOptions {
                max_input_bytes: source.len(),
                ..ParseOptions::default()
            },
        )
        .unwrap();
        // Emission canonicalizes the short escapes, exceeding the source limit
        // inside the renderer, before a replacement patch can be constructed.
        let error = document
            .edit(|editor| {
                let value = if nested {
                    editor.mapping_entries(&editor.root())?.remove(0).1
                } else {
                    editor.root()
                };
                editor.replace(&value, &replacement)
            })
            .unwrap_err();
        assert_eq!(error.message, "emitted YAML exceeds input byte limit");
        assert_eq!(document.source(), source);
    }
}

#[test]
fn root_comments_retain_the_requested_order() {
    let mut diffs = Vec::new();
    for (source, placement) in [
        (
            indoc! {r"
                # old
                value
            "},
            CommentPlacement::Leading,
        ),
        (
            indoc! {r"
                value
                # old
            "},
            CommentPlacement::Trailing,
        ),
        (
            indoc! {r"
                value

                # old
            "},
            CommentPlacement::Detached,
        ),
        (
            indoc! {r"
                # old
                --- value
                ...
            "},
            CommentPlacement::Leading,
        ),
        (
            indoc! {r"
                value
                ...

                # old
            "},
            CommentPlacement::Detached,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            editor.insert_comment(&root, 0, " first", placement)?;
            editor.insert_comment(&root, 2, " last", placement)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    +# first
     # old
    +# last
     value

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
     value
    +# first
     # old
    +# last

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     value
     ⏎
    +# first
     # old
    +# last

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    +# first
     # old
    +# last
     --- value
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,6 @@
     value
     ...
     ⏎
    +# first
     # old
    +# last
    ");
}

#[test]
fn successive_root_comment_edits_keep_indices_and_attachments() {
    let mut diffs = Vec::new();
    let source = formatdoc! {r"
          # old
        value
    "};
    let mut document = edited(&source, |editor| {
        let root = editor.root();
        editor.insert_comment(&root, 0, " discarded", CommentPlacement::Leading)?;
        editor.replace_comment(&root, 1, " renamed")?;
        editor.insert_comment(&root, 0, " first", CommentPlacement::Leading)?;
        editor.remove_comment(&root, 1)?;
        editor.insert_comment(&root, 2, " last", CommentPlacement::Leading)?;
        editor.replace(&root, "changed")
    });
    diffs.push(formatdoc! {r"
        initial edits
        {}",
        edit_diff(&source, document.source())
    });
    let before = document.to_yaml();
    document
        .edit(|editor| editor.remove_comment(&editor.root(), 2))
        .unwrap();
    document
        .edit(|editor| editor.replace_comment(&editor.root(), 1, " final"))
        .unwrap();
    diffs.push(formatdoc! {r"
        subsequent edits
        {}",
        edit_diff(&before, document.source())
    });
    insta::assert_snapshot!(diffs.join("\n"), @"
    initial edits
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -  # old
    -value
    +  # first
    +  # renamed
    +  # last
    +changed

    subsequent edits
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
       # first
    -  # renamed
    -  # last
    +  # final
     changed
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
fn block_roots_can_move_off_a_document_marker_line() {
    // Document prefixes and markers: https://yaml.org/spec/1.2.2/#92-documents
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            --- old
        "},
        indoc! {r"
            --- [old]
        "},
        "---",
        indoc! {r"
            %YAML 1.2
            --- old
            ...
        "},
    ] {
        diffs.push(edit(source, |editor| {
            editor.replace(
                &editor.root(),
                indoc! {r"
                    - new
                    - values
                "},
            )
        }));
    }
    for source in [
        indoc! {r"
            --- [old]
        "},
        indoc! {r"
            --- {old: value}
        "},
        indoc! {r"
            --- &anchor [old]
        "},
        indoc! {r"
            --- !!map {old: value}
        "},
    ] {
        diffs.push(edit(source, |editor| {
            editor.set_collection_style(&editor.root(), CollectionStyle::Block)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- old
    +--- ⏎
    +- new
    +- values

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- [old]
    +--- ⏎
    +- new
    +- values

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ----
    \ No newline at end of file
    +---
    +- new
    +- values
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     %YAML 1.2
    ---- old
    +--- ⏎
    +- new
    +- values
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- [old]
    +--- ⏎
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- {old: value}
    +--- ⏎
    +old: value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- &anchor [old]
    +--- ⏎
    +&anchor ⏎
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- !!map {old: value}
    +--- ⏎
    +!<tag:yaml.org,2002:map> ⏎
    +old: value
    ");
}

#[test]
fn root_comment_placement_respects_the_document_prefix() {
    let source = indoc! {r"
        %YAML 1.2
        --- value
        ...
    "};
    let diff = edit(source, |editor| {
        editor.insert_comment(&editor.root(), 0, " leading", CommentPlacement::Leading)
    });
    insta::assert_snapshot!(diff, @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     %YAML 1.2
    +# leading
     --- value
     ...
    ");

    let source = indoc! {r"
        value # inline
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    // A leading comment cannot follow an inline comment in source order.
    let error = document
        .edit(|editor| {
            editor.insert_comment(&editor.root(), 1, " leading", CommentPlacement::Leading)
        })
        .unwrap_err();
    assert_eq!(
        error.message,
        "emission changed comment ownership or placement"
    );
    assert_eq!(document.source(), source);
}

#[test]
fn exact_scalar_style_requests_preserve_multiline_values() {
    let mut diffs = Vec::new();
    for (source, style) in [
        (
            indoc! {r#"
                "a\nb"
            "#},
            ScalarStyle::SingleQuoted,
        ),
        (
            indoc! {r#"
                "a\nb"
            "#},
            ScalarStyle::Plain,
        ),
        (
            indoc! {r#"
                "\n"
            "#},
            ScalarStyle::Literal,
        ),
        (
            indoc! {r#"
                "\n"
            "#},
            ScalarStyle::Folded,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            editor.set_scalar_style(&editor.root(), style)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -"a\nb"
    +'a
    +
    +  b'

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -"a\nb"
    +a
    +
    +  b

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -"\n"
    +|2+
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -"\n"
    +>2+
    +  ⏎
    "#);
}

#[test]
fn block_scalar_rewrites_own_their_trailing_line_breaks() {
    let diffs = edits(
        [
            indoc! {r"
                'old'
            "},
            indoc! {r"
                'old'

            "},
            "'old'\n  \n  ",
            indoc! {r"
                'old' # keep

            "},
            indoc! {r"
                'old'

                ...
            "},
            indoc! {r"
                %YAML 1.2 # directive
                --- 'old' # inline
                ... # end
            "},
        ],
        |editor| {
            let root = editor.root();
            editor.replace(&root, "\"\\n\"")?;
            editor.set_scalar_style(&root, ScalarStyle::Literal)
        },
    );
    insta::assert_snapshot!(diffs, @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -'old'
    +|2+
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -'old'
    -
    +|2+
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -'old'
    +|2+
       ⏎
    -  ⏎
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -'old' # keep
    -
    +|2+ # keep
    +  ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -'old'
    -
    +|2+
    +  ⏎
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     %YAML 1.2 # directive
    ---- 'old' # inline
    +--- |2+ # inline
    +  ⏎
     ... # end
    ");
}

#[test]
fn nested_block_scalar_boundaries_stop_before_following_syntax() {
    let mut diffs = Vec::new();
    for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
        let source = indoc! {r"
            outer:
              value: old # keep

            next: retained
        "};
        diffs.push(edit(source, |editor| {
            let outer = editor.mapping_entries(&editor.root())?.remove(0).1;
            let value = editor.mapping_entries(&outer)?.remove(0).1;
            editor.replace(&value, "\"\\n\"")?;
            editor.set_scalar_style(&value, style)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
    -  value: old # keep
    -
    +  value: |2+ # keep
    +    ⏎
     next: retained

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
    -  value: old # keep
    -
    +  value: >2+ # keep
    +    ⏎
     next: retained
    ");
}

#[test]
fn comment_transactions_match_an_ordered_list_model() {
    let mut state = 0x93ca_ee76_1963_7651u64;
    for (source, placement) in [
        (
            indoc! {r"
                # old
                --- value
                ...
            "},
            CommentPlacement::Leading,
        ),
        (
            indoc! {r"
                value
                # old
            "},
            CommentPlacement::Trailing,
        ),
        (
            indoc! {r"
                value

                # old
            "},
            CommentPlacement::Detached,
        ),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        let mut model = vec![" old".to_owned()];
        for round in 0..100 {
            let before = document.to_yaml();
            document
                .edit(|editor| {
                    let root = editor.root();
                    for step in 0..3 {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        let operation = state % 4;
                        let text = format!(" comment {round}/{step}");
                        if operation == 0 || model.is_empty() {
                            let index = usize::try_from(state % (model.len() as u64 + 1)).unwrap();
                            editor.insert_comment(&root, index, &text, placement)?;
                            model.insert(index, text);
                        } else {
                            let index = usize::try_from(state % model.len() as u64).unwrap();
                            match operation {
                                1 => {
                                    editor.remove_comment(&root, index)?;
                                    model.remove(index);
                                }
                                2 => {
                                    editor.replace_comment(&root, index, &text)?;
                                    model[index] = text;
                                }
                                _ => editor.replace(&root, "updated")?,
                            }
                        }
                    }
                    Ok(())
                })
                .unwrap_or_else(|error| panic!("{before:?}, {placement:?}, {model:?}: {error}"));
            let comments = document.node(document.root()).unwrap().comments();
            assert_eq!(
                comments
                    .iter()
                    .map(astral_yaml::edit::Comment::text)
                    .collect::<Vec<_>>(),
                model
            );
            assert!(
                comments
                    .iter()
                    .all(|comment| comment.placement() == placement)
            );
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn empty_roots_end_retained_header_comments_before_inserting_content() {
    let mut diffs = Vec::new();
    for source in [
        "--- # marker",
        indoc! {r"
            %YAML 1.2
            --- # marker"},
    ] {
        for fragment in [
            "new",
            "[new]",
            "{key: new}",
            indoc! {r"
                |2-
                  new
            "},
        ] {
            let mut document = EditableDocument::parse(source).unwrap();
            document
                .edit(|editor| editor.replace(&editor.root(), fragment))
                .unwrap_or_else(|error| panic!("{source:?}, {fragment:?}: {error}"));
            diffs.push(edit_diff(source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +[new]
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +{key: new}
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +|2-
    +  new

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     %YAML 1.2
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +new
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     %YAML 1.2
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +[new]
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     %YAML 1.2
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +{key: new}
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
     %YAML 1.2
    ---- # marker
    \ No newline at end of file
    +--- # marker
    +|2-
    +  new
    ");
}

#[test]
fn edits_preserve_comments_between_headers_properties_and_content() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            key: # header
              &a # anchor
              # before value
              old
        "},
        indoc! {r"
            key: !!str # tag
              &a # anchor
              # before value
              old
        "},
        indoc! {r"
            key: # header
              # before anchor
              &a # anchor
              # before value
              old
        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let value = editor.mapping_entries(&editor.root())?.remove(0).1;
                editor.replace_preserving_style(&value, "new")
            })
            .unwrap_or_else(|error| panic!("{source:?}: {error}"));
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     key: # header
       &a # anchor
    -  # before value
    -  old
    +    # before value
    +    new

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
    -key: !!str # tag
    +key: # tag
       &a # anchor
    -  # before value
    -  old
    +    # before value
    +    new

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     key: # header
       # before anchor
       &a # anchor
    -  # before value
    -  old
    +    # before value
    +    new
    ");
}
