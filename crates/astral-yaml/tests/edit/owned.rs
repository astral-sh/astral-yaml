//! Owned syntax retains YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.
use astral_yaml::{
    ParseOptions,
    comments::{CommentIndex, CommentPlacement},
    edit::{EditableDocument, NodeKind},
};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn owned_document_retains_syntax_and_direct_comments() {
    let source = indoc! {r"
        %YAML 1.2
        ---
        # before
        items: &items [!!str 'é', 2] # inline
        copy: *items
        ...
    "};
    let document = EditableDocument::parse(source).unwrap();
    assert_eq!(document.source(), source);
    assert_eq!(
        document.options().max_depth,
        ParseOptions::default().max_depth
    );
    let root = document.node(document.root()).unwrap();
    let NodeKind::Mapping { entries, .. } = root.kind() else {
        panic!("mapping")
    };
    let sequence = document.node(entries[0].1).unwrap();
    assert_eq!(sequence.anchor(), Some("items"));
    assert_eq!(sequence.tag(), None);
    let NodeKind::Sequence { items, .. } = sequence.kind() else {
        panic!("sequence")
    };
    let scalar = document.node(items[0]).unwrap();
    assert_eq!(scalar.tag(), Some("tag:yaml.org,2002:str"));
    assert_eq!(scalar.span().slice(source), Some("!!str 'é'"));
    assert!(matches!(
        document.node(entries[1].1).unwrap().kind(),
        NodeKind::Alias { .. }
    ));
    let comment = sequence
        .comments()
        .iter()
        .find(|c| c.placement() == CommentPlacement::Inline)
        .unwrap();
    assert_eq!(comment.text(), " inline");
    assert_eq!(comment.span().slice(source), Some("# inline"));

    let owned = {
        let source = String::from("key: value");
        EditableDocument::parse(&source).unwrap()
    };
    assert_eq!(owned.source(), "key: value");
}

#[test]
fn alias_edges_distinguish_recursive_and_shadowed_definitions() {
    // Alias nodes refer to the most recent preceding definition of an anchor:
    // https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    let source = indoc! {r"
        - &item [first, *item]
        - *item
        - &item second
        - *item
    "};
    let document = EditableDocument::parse(source).unwrap();
    let NodeKind::Sequence { items, .. } = document.node(document.root()).unwrap().kind() else {
        panic!("sequence")
    };
    let NodeKind::Sequence { items: first, .. } = document.node(items[0]).unwrap().kind() else {
        panic!("sequence")
    };
    let targets: Vec<_> = [first[1], items[1], items[3]]
        .into_iter()
        .map(|id| {
            let NodeKind::Alias { target } = document.node(id).unwrap().kind() else {
                panic!("alias")
            };
            document
                .node(*target)
                .unwrap()
                .span()
                .slice(source)
                .unwrap()
        })
        .collect();
    insta::assert_debug_snapshot!(targets, @r#"
    [
        "&item [first, *item]",
        "&item [first, *item]",
        "&item second",
    ]
    "#);
}

#[test]
fn construction_rejects_invalid_documents_and_limits() {
    for source in [
        "[",
        "",
        indoc! {r"
            ---
            1
            ---
            2"},
    ] {
        assert!(EditableDocument::parse(source).is_err());
    }
    let options = ParseOptions {
        max_input_bytes: 1,
        ..ParseOptions::default()
    };
    assert!(EditableDocument::parse_with_options("42", options).is_err());
    let stream = astral_yaml::parse("42 # note").unwrap();
    assert_eq!(
        CommentIndex::new(&stream, 1).unwrap_err().message,
        "document index out of bounds"
    );
    let index = CommentIndex::new(&stream, 0).unwrap();
    assert_eq!(index.direct().count(), 1);
    assert_eq!(index.subtree(stream.documents()[0].root).len(), 1);
}

#[test]
fn editable_sources_normalize_line_breaks_before_assigning_spans() {
    // YAML line breaks normalize to LF: https://yaml.org/spec/1.2.2/#54-line-break-characters
    let source = indoc! {r#"
        # before
        key: &a 'é' # inline

        copy: *a
        escaped: "\r\n"
    "#};
    let expected = EditableDocument::parse(source).unwrap();
    // Include mixed and consecutive physical line breaks.
    for input in [
        source.to_owned(),
        source.replace('\n', "\r\n"),
        source.replace('\n', "\r"),
        source
            .replace("# before\n", "# before\r\n")
            .replace("copy: *a\n", "copy: *a\r"),
    ] {
        let document = EditableDocument::parse(&input).unwrap();
        assert_eq!(format!("{document:?}"), format!("{expected:?}"));
    }
    // Escaped carriage returns are scalar data, not physical line breaks.
    let NodeKind::Mapping { entries, .. } = expected.node(expected.root()).unwrap().kind() else {
        panic!("mapping")
    };
    insta::assert_debug_snapshot!(expected.node(entries[2].1).unwrap().kind(), @r#"
    Scalar {
        value: "\r\n",
        style: DoubleQuoted,
    }
    "#);
}

#[test]
fn normalization_keeps_limits_on_the_original_input_and_locates_errors_in_lf() {
    let limited = EditableDocument::parse_with_options(
        "key: value\r\n",
        ParseOptions {
            max_input_bytes: "key: value\n".len(),
            ..ParseOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(limited.message, "input byte limit exceeded");
    let error = EditableDocument::parse("key: value\r\ninvalid: [\r\n").unwrap_err();
    assert_eq!(
        error,
        EditableDocument::parse("key: value\ninvalid: [\n").unwrap_err()
    );
}

#[test]
fn scalar_replacements_preserve_surrounding_source_and_refresh_spans() {
    let source = "# config\r\ntimeout : 30   # seconds\r\nother:    'untouched'\r\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let entries = editor.mapping_entries(&editor.root())?;
            editor.replace(&entries[0].1, "600")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     # config␍
    -timeout : 30   # seconds␍
    +timeout : 600   # seconds␍
     other:    'untouched'␍
    ");
    let before = document.to_yaml();
    document
        .edit(|editor| {
            let entries = editor.mapping_entries(&editor.root())?;
            editor.replace(&entries[0].1, "1000")?;
            editor.replace(&entries[1].1, "'updated'")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(&before, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     # config␍
    -timeout : 600   # seconds␍
    -other:    'untouched'␍
    +timeout : 1000   # seconds␍
    +other:    'updated'␍
    ");
}

#[test]
fn structural_edits_preserve_comments_and_accept_complex_keys() {
    let source = "# config\na: 1 # remove\nb: [2, 3] # keep\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let root = editor.root();
            let entries = editor.mapping_entries(&root)?;
            editor.remove_entry(&root, 0)?;
            editor.remove_item(&entries[1].1, 0)?;
            editor.insert_item(&entries[1].1, 1, "4")?;
            editor.insert_entry(&root, 1, "[x, y]", "{enabled: true}")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
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
    use astral_yaml::{CollectionStyle, ScalarStyle};
    let source = "text: 'hello'\nitems: [1, 2]\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let entries = editor.mapping_entries(&editor.root())?;
            editor.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)?;
            editor.set_collection_style(&entries[1].1, CollectionStyle::Block)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
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
                .edit(|e| e.set_scalar_style(&e.root(), style))
                .is_err()
        );
        assert_eq!(document.source(), source);
    }
    let mut document = EditableDocument::parse("[one]").unwrap();
    assert!(
        document
            .edit(|e| {
                let item = e.sequence_items(&e.root())?.remove(0);
                e.set_scalar_style(&item, ScalarStyle::Literal)
            })
            .is_err()
    );
}

#[test]
fn edits_are_atomic_and_handles_expire() {
    let mut document = EditableDocument::parse("[1, 2]").unwrap();
    let mut stale = None;
    document
        .edit(|editor| {
            stale = Some(editor.root());
            Ok(())
        })
        .unwrap();
    assert!(
        document
            .edit(|editor| editor.insert_item(stale.as_ref().unwrap(), 0, "3"))
            .is_err()
    );
    assert!(
        document
            .edit(|editor| {
                let root = editor.root();
                let first = editor.sequence_items(&root)?.remove(0);
                editor.remove_item(&root, 0)?;
                editor.replace(&first, "3")
            })
            .is_err()
    );
    assert!(
        document
            .edit(|editor| {
                let root = editor.root();
                editor.insert_item(&root, 0, "3")?;
                assert!(editor.remove_item(&root, 20).is_err());
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
        assert!(document.edit(|e| e.remove_item(&e.root(), index)).is_err());
        assert_eq!(document.source(), source);
    }
    let source = "[&a 1, *a]";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let first = e.sequence_items(&e.root())?.remove(0);
            e.replace(&first, "&a 2")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[&a 1, *a]
    \ No newline at end of file
    +[&a 2, *a]
    \ No newline at end of file
    ");
    let before = document.to_yaml();
    document
        .edit(|e| e.insert_item(&e.root(), 2, "&b [*b]"))
        .unwrap();
    insta::assert_snapshot!(edit_diff(&before, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[&a 2, *a]
    \ No newline at end of file
    +[&a 2, *a, &b [*b]]
    \ No newline at end of file
    ");
}

#[test]
fn comment_edits_keep_document_markers_and_source_outside_the_node() {
    let source = "%YAML 1.2 # version\n--- # marker\n42 # number\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let root = e.root();
            e.replace_comment(&root, 0, " updated")?;
            e.remove_comment(&root, 1)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -%YAML 1.2 # version
    ---- # marker
    +%YAML 1.2 # updated
    +--- ⏎
     42 # number
    ");
    let source = "a: 1\nb: 2\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.insert_comment(&entries[0].1, 0, " first", CommentPlacement::Leading)?;
            e.insert_comment(&entries[1].1, 0, " second", CommentPlacement::Inline)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    +# first
     a: 1
    -b: 2
    +b: 2 # second
    ");
}

#[test]
fn invalid_operations_roll_back() {
    use astral_yaml::{CollectionStyle, ScalarStyle};
    for (source, operation) in [
        ("42", 0),
        ("[]", 1),
        ("[]", 2),
        ("[]", 3),
        ("42", 4),
        ("[]", 5),
        ("{}", 6),
        ("{}", 7),
        ("{}", 8),
        ("42", 9),
        ("{}", 10),
        ("[]", 11),
        ("42", 12),
        ("42", 13),
        ("42", 14),
        ("42", 15),
        ("42 # old", 16),
        ("42", 17),
        ("42", 18),
        ("42", 19),
        ("42", 20),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        let result = document.edit(|e| {
            let root = e.root();
            match operation {
                0 => e.insert_item(&root, 0, "1"),
                1 => e.insert_item(&root, 1, "1"),
                2 => e.insert_item(&root, 0, "["),
                3 | 4 => e.remove_item(&root, 0),
                5 => e.insert_entry(&root, 0, "key", "value"),
                6 => e.insert_entry(&root, 1, "key", "value"),
                7 => e.insert_entry(&root, 0, "[", "value"),
                8 => e.insert_entry(&root, 0, "key", "["),
                9 | 10 => e.remove_entry(&root, 0),
                11 => e.set_scalar_style(&root, ScalarStyle::Plain),
                12 => e.set_collection_style(&root, CollectionStyle::Flow),
                13 => e.insert_comment(&root, 1, " note", CommentPlacement::Inline),
                14 => e.replace_comment(&root, 0, " note"),
                15 => e.remove_comment(&root, 0),
                16 => e.insert_comment(&root, 0, " note", CommentPlacement::Inline),
                17 => e.insert_comment(&root, 0, "bad\ncomment", CommentPlacement::Leading),
                18 => {
                    e.replace(&root, "[1]")?;
                    let old = e.sequence_items(&root)?.remove(0);
                    e.replace(&root, "[2]")?;
                    e.replace(&old, "3")
                }
                19 => {
                    e.mapping_entries(&root)?;
                    Ok(())
                }
                _ => {
                    e.sequence_items(&root)?;
                    Ok(())
                }
            }
        });
        assert!(result.is_err(), "{source:?} operation {operation}");
        assert_eq!(document.source(), source);
    }
}

#[test]
fn local_collection_edits_leave_siblings_untouched() {
    let mut diffs = Vec::new();
    for source in [
        "a:  'untouched'\nb: [1, 2]\nc : true\n",
        "a:  'untouched'\nb:\n  - 1\n  - 2\nc : true\n",
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|e| {
                let entries = e.mapping_entries(&e.root())?;
                e.insert_item(&entries[1].1, 2, "3")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     a:  'untouched'
    -b: [1, 2]
    +b: [1, 2, 3]
     c : true

    --- before.yaml
    +++ after.yaml
    @@ -2,4 +2,5 @@
     b:
       - 1
       - 2
    +  - 3
     c : true
    ");
    let source = "- a: 1\n  b: 2\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let item = e.sequence_items(&e.root())?.remove(0);
            e.insert_entry(&item, 2, "c", "3")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
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
        .edit(|e| {
            let root = e.root();
            e.insert_item(&root, 0, "{a: 1}")?;
            let item = e.sequence_items(&root)?.remove(0);
            e.insert_entry(&item, 1, "b", "2")?;
            let entries = e.mapping_entries(&item)?;
            e.replace(&entries[0].1, "3")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("[]", document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[]
    \ No newline at end of file
    +[{a: 3, b: 2}]
    \ No newline at end of file
    ");
}

#[test]
fn scalar_styles_can_be_changed() {
    use astral_yaml::ScalarStyle;

    let mut diffs = Vec::new();
    for style in [
        ScalarStyle::Plain,
        ScalarStyle::SingleQuoted,
        ScalarStyle::DoubleQuoted,
        ScalarStyle::Literal,
        ScalarStyle::Folded,
    ] {
        let mut document = EditableDocument::parse("'hello'").unwrap();
        document
            .edit(|e| e.set_scalar_style(&e.root(), style))
            .unwrap();
        diffs.push(format!(
            "{style:?}\n{}",
            edit_diff("'hello'", document.source())
        ));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    Plain
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -'hello'
    \ No newline at end of file
    +hello
    \ No newline at end of file

    SingleQuoted

    DoubleQuoted
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -'hello'
    \ No newline at end of file
    +"hello"
    \ No newline at end of file

    Literal
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -'hello'
    \ No newline at end of file
    +|2-
    +  hello

    Folded
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -'hello'
    \ No newline at end of file
    +>2-
    +  hello
    "#);
}

#[test]
fn empty_collections_remain_in_flow_style() {
    use astral_yaml::CollectionStyle;

    for source in ["[]", "{}"] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|e| e.set_collection_style(&e.root(), CollectionStyle::Block))
            .unwrap();
        assert_eq!(document.source(), source);
    }
}

#[test]
fn scalar_style_changes_retain_header_comments() {
    use astral_yaml::ScalarStyle;

    let source = "value: | # header\n  hello\nother: true\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
    -value: | # header
    -  hello
    +value: "hello\n" # header
     other: true
    "#);
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
    assert!(document.edit(|e| e.insert_item(&e.root(), 0, "1")).is_err());
    let mut document = EditableDocument::parse_with_options(
        "[]",
        ParseOptions {
            max_depth: 1,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    assert!(document.edit(|e| e.insert_item(&e.root(), 0, "1")).is_err());
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
                .edit(|e| {
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
fn poisoned_transactions_reject_subsequent_mutations() {
    let mut document = EditableDocument::parse("[]").unwrap();
    assert!(
        document
            .edit(|e| {
                let root = e.root();
                assert!(e.remove_item(&root, 0).is_err());
                assert!(e.insert_item(&root, 0, "1").is_err());
                Ok(())
            })
            .is_err()
    );
}

#[test]
fn fragment_comments_and_nested_comment_updates_are_retained() {
    let source = "a: 1 # old\nb: 2 # remove\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.replace_comment(&entries[0].1, 0, " new")?;
            e.remove_comment(&entries[1].1, 0)?;
            e.insert_entry(&e.root(), 2, "c", "3 # imported")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -a: 1 # old
    -b: 2 # remove
    +a: 1 # new
    +b: 2
    +c: 3 # imported
    ");
    let source = "--- # marker\na: 1\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|e| {
            e.replace_comment(&e.root(), 0, " updated")?;
            let entries = e.mapping_entries(&e.root())?;
            e.replace(&entries[0].1, "2")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
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
    let mut document = EditableDocument::parse("|\n  hello\n").unwrap();
    document
        .edit(|e| e.replace(&e.root(), "'updated'"))
        .unwrap();
    insta::assert_snapshot!(edit_diff("|\n  hello\n", document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -|
    -  hello
    +'updated'
    ");
    let mut document = EditableDocument::parse("{a: text}").unwrap();
    document
        .edit(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.set_scalar_style(&entries[0].1, ScalarStyle::DoubleQuoted)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("{a: text}", document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{a: text}
    \ No newline at end of file
    +{a: "text"}
    \ No newline at end of file
    "#);
    let mut document = EditableDocument::parse("[one]").unwrap();
    document
        .edit(|e| {
            let root = e.root();
            let item = e.sequence_items(&root)?.remove(0);
            e.set_scalar_style(&item, ScalarStyle::SingleQuoted)?;
            e.remove_item(&root, 0)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff("[one]", document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[one]
    \ No newline at end of file
    +[]
    \ No newline at end of file
    ");
    let mut document = EditableDocument::parse("[{a: 1}]").unwrap();
    assert!(
        document
            .edit(|e| {
                let item = e.sequence_items(&e.root())?.remove(0);
                e.set_collection_style(&item, CollectionStyle::Block)
            })
            .is_err()
    );
}

#[test]
fn impossible_comment_placement_and_combined_output_overflow_roll_back() {
    let source = "a: 1\nb: 2\n";
    let mut document = EditableDocument::parse(source).unwrap();
    assert!(
        document
            .edit(|e| {
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
            .edit(|e| {
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
            .edit(|e| {
                let root = e.root();
                e.node(&root)?;
                e.node(&root)?;
                Ok(())
            })
            .is_err()
    );
}

#[test]
fn replacing_implicit_nulls_adds_required_separation() {
    let mut diffs = Vec::new();
    for source in ["---", "a:", "-", "{a:}"] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|e| {
                let root = e.root();
                let target = match e.node(&root)?.kind() {
                    NodeKind::Mapping { .. } => e.mapping_entries(&root)?.remove(0).1,
                    NodeKind::Sequence { .. } => e.sequence_items(&root)?.remove(0),
                    _ => root,
                };
                e.replace(&target, "42")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    ----
    \ No newline at end of file
    +--- 42
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -a:
    \ No newline at end of file
    +a: 42
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    --
    \ No newline at end of file
    +- 42
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{a:}
    \ No newline at end of file
    +{a: 42}
    \ No newline at end of file
    ");
}
