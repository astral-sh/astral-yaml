//! Owned syntax retains YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.
use test_case::test_case;

use super::support::{edit, edited, edits};
use astral_yaml::{
    CollectionStyle, Error, ParseOptions, ScalarStyle,
    comments::CommentPlacement,
    edit::{EditableDocument, Editor, Handle, NodeKind},
};
use astral_yaml_dev::{edit_diff, formatdoc, indoc};

#[test]
fn draft_locations_distinguish_original_empty_values_from_generated_syntax() {
    let source = indoc! {"
        existing: # keep
        items: []
    "};
    let document = edited(source, |editor| {
        let entries = editor.mapping_entries(&editor.root())?;
        let existing = &entries[0].1;
        let items = &entries[1].1;
        let empty_origin = editor.node(existing)?.span();
        editor.replace(existing, "new")?;
        editor.insert_item(items, 0, "'' # imported")?;
        editor.set_collection_style(items, astral_yaml::CollectionStyle::Block)?;
        let item = &editor.sequence_items(items)?[0];
        editor.insert_comment(item, 0, " inserted", CommentPlacement::Leading)?;

        let replacement = editor.node(existing)?;
        let item = editor.node(item)?;
        insta::assert_snapshot!(formatdoc! {"
            original empty value: {:?}
            replacement origin: {:?}
            retained comment: {:?}
            generated value: {:?}
            inserted comment: {:?}
            imported comment: {:?}
        ",
            empty_origin.and_then(|span| span.slice(source)),
            replacement.span().and_then(|span| span.slice(source)),
            replacement.comments()[0].span().and_then(|span| span.slice(source)),
            item.span(),
            item.comments()[0].span(),
            item.comments()[1].span(),
        }, @r##"
        original empty value: Some("")
        replacement origin: Some("")
        retained comment: Some("# keep")
        generated value: None
        inserted comment: None
        imported comment: None
        "##);
        Ok(())
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,2 +1,4 @@
    -existing: # keep
    -items: []
    +existing: new # keep
    +items:
    +  # inserted
    +  - '' # imported
    ");
}

#[test]
fn scalar_replacements_preserve_surrounding_source_and_refresh_spans() {
    let source = indoc! {r"
        # config
        timeout : 30   # seconds
        other:    'untouched'
    "};
    let mut document = edited(source, |editor| {
        let entries = editor.mapping_entries(&editor.root())?;
        editor.replace(&entries[0].1, "600")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,3 +1,3 @@
     # config
    -timeout : 30   # seconds
    +timeout : 600   # seconds
     other:    'untouched'
    ");
    let before = document.to_yaml();
    document
        .edit_syntax(|editor| {
            let entries = editor.mapping_entries(&editor.root())?;
            editor.replace(&entries[0].1, "1000")?;
            editor.replace(&entries[1].1, "'updated'")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(&before, document.source()), @"
    @@ -1,3 +1,3 @@
     # config
    -timeout : 600   # seconds
    -other:    'untouched'
    +timeout : 1000   # seconds
    +other:    'updated'
    ");
}

#[test]
fn structural_edits_preserve_comments_and_accept_complex_keys() {
    let source = indoc! {r"
        # config
        a: 1 # remove
        b: [2, 3] # keep
    "};
    let diff = edit(source, |editor| {
        let root = editor.root();
        let entries = editor.mapping_entries(&root)?;
        editor.remove_entry(&root, 0)?;
        editor.remove_item(&entries[1].1, 0)?;
        editor.insert_item(&entries[1].1, 1, "4")?;
        editor
            .insert_entry(&root, 1, "[x, y]", "{enabled: true}")
            .map(|_| ())
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,3 +1,3 @@
    -# config
    -a: 1 # remove
    -b: [2, 3] # keep
    +b: [3, 4] # keep
    +? [x, y]
    +: {enabled: true}
    ");
}

#[test]
fn style_changes_preserve_meaning_and_reject_impossible_requests() {
    let source = indoc! {r"
        text: 'hello'
        items: [1, 2]
    "};
    let diff = edit(source, |editor| {
        let entries = editor.mapping_entries(&editor.root())?;
        editor.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)?;
        editor.set_collection_style(&entries[1].1, CollectionStyle::Block)
    });
    insta::assert_snapshot!(diff, @r#"
    @@ -1,2 +1,4 @@
    -text: 'hello'
    -items: [1, 2]
    +text: "hello"
    +items:
    +  - 1
    +  - 2
    "#);
    for (source, style) in [
        ("42", ScalarStyle::SingleQuoted),
        ("'null'", ScalarStyle::Plain),
        ("'a: b'", ScalarStyle::Plain),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        assert!(
            document
                .edit_syntax(|e| e.set_scalar_style(&e.root(), style))
                .is_err()
        );
        assert_eq!(document.source(), source);
    }
    let mut document = EditableDocument::parse("[one]").unwrap();
    assert!(
        document
            .edit_syntax(|e| {
                let item = e.sequence_items(&e.root())?.remove(0);
                e.set_scalar_style(&item, ScalarStyle::Literal)
            })
            .is_err()
    );
}

#[test]
fn edits_are_atomic_and_handles_expire() {
    let mut document = EditableDocument::parse("[1, 2]").unwrap();
    let stale = document.edit_syntax(|editor| Ok(editor.root())).unwrap();
    assert!(
        document
            .edit_syntax(|editor| editor.insert_item(&stale, 0, "3"))
            .is_err()
    );
    assert!(
        document
            .edit_syntax(|editor| {
                let root = editor.root();
                let first = editor.sequence_items(&root)?.remove(0);
                editor.remove_item(&root, 0)?;
                editor.replace(&first, "3")
            })
            .is_err()
    );
    assert!(
        document
            .edit_syntax(|editor| {
                let root = editor.root();
                editor.insert_item(&root, 0, "3")?;
                let error = editor.remove_item(&root, 20).unwrap_err();
                assert_eq!(editor.insert_item(&root, 0, "1").unwrap_err(), error);
                Ok(())
            })
            .is_err()
    );
    assert_eq!(document.source(), "[1, 2]");
}

#[test]
fn aliases_cannot_be_dangled_or_silently_rebound() {
    for (source, index) in [("[&a 1, *a]", 0), ("[&a 1, &a 2, *a]", 1)] {
        let mut document = EditableDocument::parse(source).unwrap();
        assert!(
            document
                .edit_syntax(|e| e.remove_item(&e.root(), index))
                .is_err()
        );
        assert_eq!(document.source(), source);
    }
    let source = "[&a 1, *a]";
    let mut document = edited(source, |e| {
        let first = e.sequence_items(&e.root())?.remove(0);
        e.replace(&first, "&a 2")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    @@ -1 +1 @@
    -[&a 1, *a]
    \ No newline at end of file
    +[&a 2, *a]
    \ No newline at end of file
    ");
    let before = document.to_yaml();
    document
        .edit_syntax(|e| e.insert_item(&e.root(), 2, "&b [*b]"))
        .unwrap();
    insta::assert_snapshot!(edit_diff(&before, document.source()), @r"
    @@ -1 +1 @@
    -[&a 2, *a]
    \ No newline at end of file
    +[&a 2, *a, &b [*b]]
    \ No newline at end of file
    ");
}

#[test]
fn comment_edits_keep_document_markers_and_source_outside_the_node() {
    let source = indoc! {r"
        %YAML 1.2 # version
        --- # marker
        42 # number
    "};
    let document = edited(source, |e| {
        let root = e.root();
        e.replace_comment(&root, 0, " updated")?;
        e.remove_comment(&root, 1)
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,3 +1,3 @@
    -%YAML 1.2 # version
    ---- # marker
    +%YAML 1.2 # updated
    +--- ⏎
     42 # number
    ");
    let source = indoc! {r"
        a: 1
        b: 2
    "};
    let document = edited(source, |e| {
        let entries = e.mapping_entries(&e.root())?;
        e.insert_comment(&entries[0].1, 0, " first", CommentPlacement::Leading)?;
        e.insert_comment(&entries[1].1, 0, " second", CommentPlacement::Inline)
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,2 +1,3 @@
    +# first
     a: 1
    -b: 2
    +b: 2 # second
    ");
}

#[test_case("42", |e, h| e.insert_item(h, 0, "1").map(|_| ()); "insert item into scalar")]
#[test_case("[]", |e, h| e.insert_item(h, 1, "1").map(|_| ()); "insert item beyond end")]
#[test_case("[]", |e, h| e.insert_item(h, 0, "[").map(|_| ()); "insert malformed item")]
#[test_case("[]", |e, h| e.remove_item(h, 0); "remove absent item")]
#[test_case("42", |e, h| e.remove_item(h, 0); "remove item from scalar")]
#[test_case("[]", |e, h| e.insert_entry(h, 0, "key", "value").map(|_| ()); "insert entry into sequence")]
#[test_case("{}", |e, h| e.insert_entry(h, 1, "key", "value").map(|_| ()); "insert entry beyond end")]
#[test_case("{}", |e, h| e.insert_entry(h, 0, "[", "value").map(|_| ()); "insert malformed key")]
#[test_case("{}", |e, h| e.insert_entry(h, 0, "key", "[").map(|_| ()); "insert malformed value")]
#[test_case("42", |e, h| e.remove_entry(h, 0); "remove entry from scalar")]
#[test_case("{}", |e, h| e.remove_entry(h, 0); "remove absent entry")]
#[test_case("[]", |e, h| e.set_scalar_style(h, ScalarStyle::Plain); "set scalar style on sequence")]
#[test_case("42", |e, h| e.set_collection_style(h, CollectionStyle::Flow); "set collection style on scalar")]
#[test_case("42", |e, h| e.insert_comment(h, 1, " note", CommentPlacement::Inline); "insert comment beyond end")]
#[test_case("42", |e, h| e.replace_comment(h, 0, " note"); "replace absent comment")]
#[test_case("42", |e, h| e.remove_comment(h, 0); "remove absent comment")]
#[test_case("value: 42 # old", |e, h| {
    let value = e.mapping_entries(h)?.remove(0).1;
    e.insert_comment(&value, 0, " note", CommentPlacement::Inline)
}; "insert excess inline comment")]
#[test_case("42", |e, h| e.insert_comment(h, 0, "bad\ncomment", CommentPlacement::Leading); "insert multiline comment")]
#[test_case("42", |e, h| {
    e.replace(h, "[1]")?;
    let old = e.sequence_items(h)?.remove(0);
    e.replace(h, "[2]")?;
    e.replace(&old, "3")
}; "replace child of replaced node")]
#[test_case("42", |e, h| e.mapping_entries(h).map(|_| ()); "read mapping entries from scalar")]
#[test_case("42", |e, h| e.sequence_items(h).map(|_| ()); "read sequence items from scalar")]
fn invalid_operations_roll_back(
    source: &str,
    operation: fn(&mut Editor<'_>, &Handle) -> Result<(), Error>,
) {
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit_syntax(|editor| operation(editor, &editor.root()))
        .unwrap_err();
    assert_eq!(document.source(), source);
}

#[test]
fn local_collection_edits_leave_siblings_untouched() {
    let diffs = edits(
        [
            indoc! {r"
                a:  'untouched'
                b: [1, 2]
                c : true
            "},
            indoc! {r"
                a:  'untouched'
                b:
                  - 1
                  - 2
                c : true
            "},
        ],
        |e| {
            let entries = e.mapping_entries(&e.root())?;
            e.insert_item(&entries[1].1, 2, "3").map(|_| ())
        },
    );
    insta::assert_snapshot!(diffs, @"
    @@ -1,3 +1,3 @@
     a:  'untouched'
    -b: [1, 2]
    +b: [1, 2, 3]
     c : true

    @@ -2,4 +2,5 @@
     b:
       - 1
       - 2
    +  - 3
     c : true
    ");
    let source = indoc! {r"
        - a: 1
          b: 2
    "};
    let diff = edit(source, |e| {
        let item = e.sequence_items(&e.root())?.remove(0);
        e.insert_entry(&item, 2, "c", "3").map(|_| ())
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,4 @@
    -- a: 1
    +-
    +  a: 1
       b: 2
    +  c: 3
    ");
}

#[test]
fn inserted_nodes_can_be_edited() {
    let mut document = EditableDocument::parse("[]").unwrap();
    document
        .edit_syntax(|e| {
            let root = e.root();
            e.insert_item(&root, 0, "{a: 1}")?;
            let item = e.sequence_items(&root)?.remove(0);
            e.insert_entry(&item, 1, "b", "2")?;
            let entries = e.mapping_entries(&item)?;
            e.replace(&entries[0].1, "3")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("[]", document.source()), @r"
    @@ -1 +1 @@
    -[]
    \ No newline at end of file
    +[{a: 3, b: 2}]
    \ No newline at end of file
    ");
}

#[test_case(ScalarStyle::Plain, "hello"; "plain")]
#[test_case(ScalarStyle::SingleQuoted, "'hello'"; "single quoted")]
#[test_case(ScalarStyle::DoubleQuoted, r#""hello""#; "double quoted")]
#[test_case(ScalarStyle::Literal, "|3-\n  hello\n"; "literal")]
#[test_case(ScalarStyle::Folded, ">3-\n  hello\n"; "folded")]
fn scalar_styles_can_be_changed(style: ScalarStyle, expected: &str) {
    let document = edited("'hello'", |e| e.set_scalar_style(&e.root(), style));
    assert_eq!(document.source(), expected);
}

#[test_case("[]")]
#[test_case("{}")]
fn empty_collections_remain_in_flow_style(source: &str) {
    use astral_yaml::CollectionStyle;

    let document = edited(source, |e| {
        e.set_collection_style(&e.root(), CollectionStyle::Block)
    });
    assert_eq!(document.source(), source);
}

#[test]
fn scalar_style_changes_retain_header_comments() {
    use astral_yaml::ScalarStyle;

    let source = indoc! {r"
        value: | # header
          hello
        other: true
    "};
    let diff = edit(source, |e| {
        let entries = e.mapping_entries(&e.root())?;
        e.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)
    });
    insta::assert_snapshot!(diff, @r#"
    @@ -1,3 +1,2 @@
    -value: | # header
    -  hello
    +value: "hello\n" # header
     other: true
    "#);
}

#[test]
fn imported_fragments_share_an_expanded_tag_byte_limit() {
    let fragment = "%TAG !e! tag:example.com,2026:long-prefix/\n--- [!e!x a, !e!x b, !e!x c]";
    let options = ParseOptions {
        max_input_bytes: fragment.len() * 2,
        ..ParseOptions::default()
    };
    let mut document = EditableDocument::parse_with_options("[]", options).unwrap();
    let error = document
        .edit(|root| {
            let mut sequence = root.into_sequence()?;
            sequence.push_yaml(fragment)?;
            sequence.push_yaml(fragment)?;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.message, "expanded tag byte limit exceeded");
    assert_eq!(document.source(), "[]");
}

#[test]
fn small_resource_limits_are_rejected() {
    let mut document = EditableDocument::parse_with_options(
        "[]",
        ParseOptions {
            max_nodes: 1,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert!(
        document
            .edit_syntax(|e| e.insert_item(&e.root(), 0, "1"))
            .is_err()
    );
    let mut document = EditableDocument::parse_with_options(
        "[]",
        ParseOptions {
            max_depth: 1,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert!(
        document
            .edit_syntax(|e| e.insert_item(&e.root(), 0, "1"))
            .is_err()
    );
    for comment in [false, true] {
        let mut document = EditableDocument::parse_with_options(
            "[]",
            ParseOptions {
                max_input_bytes: 2,
                ..ParseOptions::default()
            },
        )
        .unwrap();
        assert!(
            document
                .edit_syntax(|e| {
                    if comment {
                        e.insert_comment(&e.root(), 0, "long", CommentPlacement::Leading)
                    } else {
                        e.replace(&e.root(), "long")
                    }
                })
                .is_err()
        );
    }
}

#[test]
fn fragment_comments_and_nested_comment_updates_are_retained() {
    let source = indoc! {r"
        a: 1 # old
        b: 2 # remove
    "};
    let diff = edit(source, |e| {
        let entries = e.mapping_entries(&e.root())?;
        e.replace_comment(&entries[0].1, 0, " new")?;
        e.remove_comment(&entries[1].1, 0)?;
        e.insert_entry(&e.root(), 2, "c", "3 # imported")
            .map(|_| ())
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,3 @@
    -a: 1 # old
    -b: 2 # remove
    +a: 1 # new
    +b: 2
    +c: 3 # imported
    ");
    let source = indoc! {r"
        --- # marker
        a: 1
    "};
    let diff = edit(source, |e| {
        e.replace_comment(&e.root(), 0, " updated")?;
        let entries = e.mapping_entries(&e.root())?;
        e.replace(&entries[0].1, "2")
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,2 @@
    ---- # marker
    -a: 1
    +--- # updated
    +a: 2
    ");
}

#[test]
fn changes_preserve_terminators_and_discard_removed_style_requests() {
    use astral_yaml::{CollectionStyle, ScalarStyle};
    let mut document = EditableDocument::parse(indoc! {r"
        |
          hello
    "})
    .unwrap();
    document
        .edit_syntax(|e| e.replace(&e.root(), "'updated'"))
        .unwrap();
    insta::assert_snapshot!(edit_diff(indoc! {r"
        |
          hello
    "}, document.source()), @"
    @@ -1,2 +1 @@
    -|
    -  hello
    +'updated'
    ");
    let mut document = EditableDocument::parse("{a: text}").unwrap();
    document
        .edit_syntax(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("{a: text}", document.source()), @r#"
    @@ -1 +1 @@
    -{a: text}
    \ No newline at end of file
    +{a: "text"}
    \ No newline at end of file
    "#);
    let mut document = EditableDocument::parse("[one]").unwrap();
    document
        .edit_syntax(|e| {
            let root = e.root();
            let item = e.sequence_items(&root)?.remove(0);
            e.set_scalar_style(&item, ScalarStyle::SingleQuoted)?;
            e.remove_item(&root, 0)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("[one]", document.source()), @r"
    @@ -1 +1 @@
    -[one]
    \ No newline at end of file
    +[]
    \ No newline at end of file
    ");
    let mut document = EditableDocument::parse("[{a: 1}]").unwrap();
    assert!(
        document
            .edit_syntax(|e| {
                let item = e.sequence_items(&e.root())?.remove(0);
                e.set_collection_style(&item, CollectionStyle::Block)
            })
            .is_err()
    );
}

#[test]
fn impossible_comment_placement_and_combined_output_overflow_roll_back() {
    let source = indoc! {r"
        a: 1
        b: 2
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    assert!(
        document
            .edit_syntax(|e| {
                let entries = e.mapping_entries(&e.root())?;
                e.insert_comment(&entries[0].1, 0, " trailing", CommentPlacement::Trailing)
            })
            .is_err()
    );
    let source = "[0,0]";
    let mut document = EditableDocument::parse_with_options(
        source,
        ParseOptions {
            max_input_bytes: 5,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert!(
        document
            .edit_syntax(|e| {
                let items = e.sequence_items(&e.root())?;
                e.replace(&items[1], "12345")
            })
            .is_err()
    );
    assert_eq!(document.source(), source);
}

#[test]
fn repeated_navigation_is_bounded() {
    let mut document = EditableDocument::parse_with_options(
        "42",
        ParseOptions {
            max_nodes: 1,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert!(
        document
            .edit_syntax(|e| {
                let root = e.root();
                e.node(&root)?;
                e.node(&root)?;
                Ok(())
            })
            .is_err()
    );
}

#[test_case(
    "---"
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1 +1 @@
        ----
        \ No newline at end of file
        +--- 42
        \ No newline at end of file
    ");
    "document"
)]
#[test_case(
    "a:"
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1 +1 @@
        -a:
        \ No newline at end of file
        +a: 42
        \ No newline at end of file
    ");
    "block mapping"
)]
#[test_case(
    "-"
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1 +1 @@
        --
        \ No newline at end of file
        +- 42
        \ No newline at end of file
    ");
    "sequence"
)]
#[test_case(
    "{a:}"
    => with |diff| insta::assert_snapshot!(diff, @r"
        @@ -1 +1 @@
        -{a:}
        \ No newline at end of file
        +{a: 42}
        \ No newline at end of file
    ");
    "flow mapping"
)]
fn replacing_implicit_nulls_adds_required_separation(source: &str) -> String {
    edit(source, |e| {
        let root = e.root();
        let target = match e.node(&root)?.kind() {
            NodeKind::Mapping { .. } => e.mapping_entries(&root)?.remove(0).1,
            NodeKind::Sequence { .. } => e.sequence_items(&root)?.remove(0),
            _ => root,
        };
        e.replace(&target, "42")
    })
}

#[test]
fn transactions_keep_lf_source_for_noops_fragments_and_rollback() {
    let source = indoc! {r"
        # retained
        key: old # inline
        other: retained
    "};
    let fragment = indoc! {r"
        |-
          first
          second
    "};
    let mut document = EditableDocument::parse(&source.replace('\n', "\r\n")).unwrap();
    document.edit_syntax(|_| Ok(())).unwrap();
    assert_eq!(document.to_yaml(), source);

    let result = document.edit_syntax(|editor| {
        let value = editor.mapping_entries(&editor.root())?.remove(0).1;
        editor.replace(&value, &fragment.replace('\n', "\r"))?;
        editor.replace(&editor.root(), "[")
    });
    assert!(result.is_err());
    assert_eq!(document.source(), source);

    document
        .edit_syntax(|editor| {
            let value = editor.mapping_entries(&editor.root())?.remove(0).1;
            editor.replace(&value, &fragment.replace('\n', "\r"))
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,3 +1,5 @@
     # retained
    -key: old # inline
    +key: |2- # inline
    +  first
    +  second
     other: retained
    ");
}

#[test]
fn replacement_handles_survive_composed_structural_edits() {
    use astral_yaml::{CollectionStyle, ScalarStyle};

    let source = indoc! {r"
        selected: [old, removed] # keep
        retained: untouched
    "};
    let diff = edit(source, |editor| {
        let selected = editor.mapping_entries(&editor.root())?[0].1.clone();
        let old_children = editor.sequence_items(&selected)?;
        for child in &old_children {
            editor.set_scalar_style(child, ScalarStyle::SingleQuoted)?;
        }
        editor.replace(
            &selected,
            "&root {first: [&leaf value, *leaf], self: *root}",
        )?;
        let first = editor.mapping_entries(&selected)?[0].1.clone();
        let leaf = editor.sequence_items(&first)?[0].clone();
        editor.replace(&leaf, "&leaf {nested: value}")?;
        editor.set_collection_style(&first, CollectionStyle::Block)?;
        editor.remove_entry(&selected, 0)?;
        editor.insert_entry(&selected, 1, "tail", "&tail [*tail]")?;
        let tail = editor.mapping_entries(&selected)?[1].1.clone();
        editor.insert_item(&tail, 1, "temporary")?;
        let temporary = editor.sequence_items(&tail)?[1].clone();
        editor.remove_item(&tail, 1)?;
        editor.set_collection_style(&tail, CollectionStyle::Flow)?;
        for removed in old_children.into_iter().chain([first, leaf, temporary]) {
            assert!(
                editor.node(&removed).is_err(),
                "a retired handle cannot revive"
            );
        }
        Ok(())
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,2 +1,2 @@
    -selected: [old, removed] # keep
    +selected: &root {self: *root, tail: &tail [*tail]} # keep
     retained: untouched
    ");
}

#[test]
fn canceled_edits_do_not_expand_the_final_rewrite() {
    use astral_yaml::ScalarStyle;

    let source = indoc! {r"
        edited: old
        untouched:   [x,y]
    "};
    let mut diffs = Vec::new();
    for replacement in [None, Some("new")] {
        let document = edited(source, |editor| {
            let value = editor.mapping_entries(&editor.root())?[0].1.clone();
            editor.set_scalar_style(&value, ScalarStyle::Literal)?;
            editor.set_scalar_style(&value, ScalarStyle::Plain)?;
            editor.insert_comment(&value, 0, "temporary", CommentPlacement::Leading)?;
            editor.remove_comment(&value, 0)?;
            let items = editor.mapping_entries(&editor.root())?[1].1.clone();
            editor.insert_item(&items, 1, "temporary")?;
            editor.remove_item(&items, 1)?;
            editor.set_collection_style(&items, astral_yaml::CollectionStyle::Block)?;
            editor.set_collection_style(&items, astral_yaml::CollectionStyle::Flow)?;
            if let Some(replacement) = replacement {
                editor.replace(&value, replacement)?;
            }
            Ok(())
        });
        let diff = edit_diff(source, document.source());
        diffs.push(format!(
            "Final replacement: {replacement:?}\n{}",
            if diff.is_empty() {
                "unchanged\n"
            } else {
                &diff
            }
        ));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    Final replacement: None
    unchanged

    Final replacement: Some("new")
    @@ -1,2 +1,2 @@
    -edited: old
    +edited: new
     untouched:   [x,y]
    "#);
}
