use astral_yaml::{
    CollectionStyle, ParseOptions, ScalarStyle, comments::CommentPlacement, edit::EditableDocument,
};

#[test]
fn emission_limits_roll_back_root_and_subtree_rewrites() {
    let quoted = format!("\"{}\"", "\\x01".repeat(8));
    for nested in [false, true] {
        let source = if nested {
            format!("key: {quoted}\n")
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
                editor.set_scalar_style(&value, ScalarStyle::DoubleQuoted)
            })
            .unwrap_err();
        assert_eq!(error.message, "emitted YAML exceeds input byte limit");
        assert_eq!(document.source(), source);
    }
}

#[test]
fn root_comments_retain_the_requested_order() {
    let mut outputs = Vec::new();
    for (source, placement) in [
        ("# old\nvalue\n", CommentPlacement::Leading),
        ("value\n# old\n", CommentPlacement::Trailing),
        ("value\n\n# old\n", CommentPlacement::Detached),
        ("# old\n--- value\n...\n", CommentPlacement::Leading),
        ("value\n...\n\n# old\n", CommentPlacement::Detached),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                editor.insert_comment(&root, 0, " first", placement)?;
                editor.insert_comment(&root, 2, " last", placement)
            })
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r##"
    [
        "# first\n# old\n# last\nvalue\n",
        "value\n# first\n# old\n# last\n",
        "value\n\n# first\n# old\n# last\n",
        "# first\n# old\n# last\n--- value\n...\n",
        "value\n...\n\n# first\n# old\n# last\n",
    ]
    "##);
}

#[test]
fn successive_root_comment_edits_keep_indices_and_attachments() {
    let mut outputs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!("  # old{newline}value{newline}");
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                editor.insert_comment(&root, 0, " discarded", CommentPlacement::Leading)?;
                editor.replace_comment(&root, 1, " renamed")?;
                editor.insert_comment(&root, 0, " first", CommentPlacement::Leading)?;
                editor.remove_comment(&root, 1)?;
                editor.insert_comment(&root, 2, " last", CommentPlacement::Leading)?;
                editor.replace(&root, "changed")
            })
            .unwrap();
        outputs.push(document.to_yaml());
        document
            .edit(|editor| editor.remove_comment(&editor.root(), 2))
            .unwrap();
        document
            .edit(|editor| editor.replace_comment(&editor.root(), 1, " final"))
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "  # first\n  # renamed\n  # last\nchanged\n",
        "  # first\n  # final\nchanged\n",
        "  # first\r\n  # renamed\r\n  # last\r\nchanged\r\n",
        "  # first\r\n  # final\r\nchanged\r\n",
        "  # first\r  # renamed\r  # last\rchanged\r",
        "  # first\r  # final\rchanged\r",
    ]
    "#);
}

#[test]
fn block_roots_can_move_off_a_document_marker_line() {
    // Document prefixes and markers: https://yaml.org/spec/1.2.2/#92-documents
    let mut outputs = Vec::new();
    for source in [
        "--- old\n",
        "--- [old]\n",
        "---",
        "%YAML 1.2\r\n--- old\r\n...\r\n",
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| editor.replace(&editor.root(), "- new\n- values\n"))
            .unwrap();
        outputs.push(document.into_yaml());
    }
    for source in [
        "--- [old]\n",
        "--- {old: value}\n",
        "--- &anchor [old]\n",
        "--- !!map {old: value}\n",
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "--- \n- new\n- values\n",
        "--- \n- new\n- values\n",
        "---\n- new\n- values",
        "%YAML 1.2\r\n--- \r\n- new\r\n- values\r\n...\r\n",
        "--- \n- old\n",
        "--- \nold: value\n",
        "--- \n&anchor \n- old\n",
        "--- \n!<tag:yaml.org,2002:map> \nold: value\n",
    ]
    "#);
}

#[test]
fn root_comment_placement_respects_the_document_prefix() {
    let mut document = EditableDocument::parse("%YAML 1.2\n--- value\n...\n").unwrap();
    document
        .edit(|editor| {
            editor.insert_comment(&editor.root(), 0, " leading", CommentPlacement::Leading)
        })
        .unwrap();
    insta::assert_debug_snapshot!(document.source(), @r#""%YAML 1.2\n--- \n# leading\nvalue\n...\n""#);

    let source = "value # inline\n";
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
    let mut outputs = Vec::new();
    for (source, style) in [
        ("'a\n\nb'\n", ScalarStyle::SingleQuoted),
        ("a\n\nb\n", ScalarStyle::Plain),
        ("|+\n\n", ScalarStyle::Literal),
        (">+\n\n", ScalarStyle::Folded),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| editor.set_scalar_style(&editor.root(), style))
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "'a\n\n  b'\n",
        "a\n\n  b\n",
        "|2+\n  \n",
        ">2+\n  \n",
    ]
    "#);
}

#[test]
fn block_scalar_rewrites_own_their_trailing_line_breaks() {
    let mut outputs = Vec::new();
    for source in [
        "'old'\n",
        "'old'\n\n",
        "'old'\n  \n  ",
        "'old' # keep\n\n",
        "'old'\n\n...\n",
        "%YAML 1.2 # directive\n--- 'old' # inline\n... # end\n",
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                editor.replace(&root, "\"\\n\"")?;
                editor.set_scalar_style(&root, ScalarStyle::Literal)
            })
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "|2+\n  \n",
        "|2+\n  \n",
        "|2+\n  \n",
        "|2+ # keep\n  \n",
        "|2+\n  \n...\n",
        "%YAML 1.2 # directive\n--- |2+ # inline\n  \n... # end\n",
    ]
    "#);
}

#[test]
fn nested_block_scalar_boundaries_stop_before_following_syntax() {
    let mut outputs = Vec::new();
    for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
        let mut document =
            EditableDocument::parse("outer:\n  value: old # keep\n\nnext: retained\n").unwrap();
        document
            .edit(|editor| {
                let outer = editor.mapping_entries(&editor.root())?.remove(0).1;
                let value = editor.mapping_entries(&outer)?.remove(0).1;
                editor.replace(&value, "\"\\n\"")?;
                editor.set_scalar_style(&value, style)
            })
            .unwrap();
        outputs.push(document.into_yaml());
    }
    insta::assert_debug_snapshot!(outputs, @r#"
    [
        "outer:\n  value: |2+ # keep\n    \nnext: retained\n",
        "outer:\n  value: >2+ # keep\n    \nnext: retained\n",
    ]
    "#);
}

#[test]
fn comment_transactions_match_an_ordered_list_model() {
    let mut state = 0x93ca_ee76_1963_7651u64;
    for newline in ["\n", "\r\n", "\r"] {
        for (source, placement) in [
            ("# old\n--- value\n...\n", CommentPlacement::Leading),
            ("value\n# old\n", CommentPlacement::Trailing),
            ("value\n\n# old\n", CommentPlacement::Detached),
        ] {
            let mut document = EditableDocument::parse(&source.replace('\n', newline)).unwrap();
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
                                let index =
                                    usize::try_from(state % (model.len() as u64 + 1)).unwrap();
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
                    .unwrap_or_else(|error| {
                        panic!("{before:?}, {placement:?}, {model:?}: {error}")
                    });
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
}
