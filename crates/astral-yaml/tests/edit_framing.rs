//! Document framing, source separators, and comments between collection children.

use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
fn structural_edits_preserve_framing_and_outer_comments() {
    // Document markers: https://yaml.org/spec/1.2.2/#922-document-markers
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "# config\n%YAML 1.2\n---\nkey: old\n...\n  # tail\n",
            "# config\n%TAG !e! tag:example.com,2026:\n---\nkey: !e!value old\n... # end\n",
            "# config\n---\nkey:\n  - old\n...\n  # tail\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let root = editor.root();
                    editor.insert_entry(&root, 0, "added", "new")?;
                    editor.remove_entry(&root, 1)
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     # config␍
     %YAML 1.2␍
     ---␍
    -key: old␍
    +added: new␍
     ...␍
       # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     # config␍
     %TAG !e! tag:example.com,2026:␍
     ---␍
    -key: !e!value old␍
    +added: new␍
     ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,5 @@
     # config␍
     ---␍
    -key:␍
    -  - old␍
    +added: new␍
     ...␍
       # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     # config␍ %YAML 1.2␍ ---␍-key: old␍+added: new␍ ...␍   # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     # config␍ %TAG !e! tag:example.com,2026:␍ ---␍-key: !e!value old␍+added: new␍ ... # end␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,5 @@
     # config␍ ---␍-key:␍-  - old␍+added: new␍ ...␍   # tail␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn inserted_roots_are_separated_from_end_markers() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for replacement in [
            "new",
            "[a, b]",
            "{a: b}",
            "- a\n- b\n",
            "a: b\n",
            "|\n  text\n",
        ] {
            let source = "---\n... # end\n# tail\n".replace('\n', newline);
            let replacement = replacement.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.replace(&editor.root(), &replacement))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
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

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍
    +new␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍
    +[a, b]␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍
    +{a: b}␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---␍
    +- a␍
    +- b␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍
    +a: b␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---␍
    +|2␍
    +  text␍
     ... # end␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍+new␍ ... # end␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍+[a, b]␍ ... # end␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍+{a: b}␍ ... # end␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---␍+- a␍+- b␍ ... # end␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     ---␍+a: b␍ ... # end␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     ---␍+|2␍+  text␍ ... # end␍ # tail␍
    ");
}

#[test]
fn comment_terminators_compose_with_retained_suffixes() {
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "key: old # inline\n# tail\n",
            "key: old # inline\n",
            "outer:\n  key: old # inline\n  # tail\nother: unchanged\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            for _ in 0..3 {
                document
                    .edit(|editor| {
                        editor.set_collection_style(&editor.root(), CollectionStyle::Block)
                    })
                    .unwrap();
                assert_eq!(document.source(), source);
            }
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn detached_comment_gaps_survive_collection_mutations() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, sequence) in [
            ("a: old\n\n# detached\n\nb: |+\n  text\n\n", false),
            ("- old\n\n# detached\n\n- |+\n  text\n\n", true),
        ] {
            for operation in 0..4 {
                let source = source.replace('\n', newline);
                let mut document = EditableDocument::parse(&source).unwrap();
                document
                    .edit(|editor| {
                        let root = editor.root();
                        match (sequence, operation) {
                            (true, 0) => editor.insert_item(&root, 0, "new"),
                            (false, 0) => editor.insert_entry(&root, 0, "added", "new"),
                            (true, 1) => editor.remove_item(&root, 1),
                            (false, 1) => editor.remove_entry(&root, 1),
                            (true, 2) => editor.replace(&root, "- new\n- |+\n  other\n\n"),
                            (false, 2) => editor.replace(&root, "a: new\nb: |+\n  other\n\n"),
                            _ => {
                                // Import into a nonzero arena offset, then mutate the
                                // imported collection before committing it.
                                editor.replace(&root, &source)?;
                                if sequence {
                                    editor.insert_item(&root, 0, "new")
                                } else {
                                    editor.insert_entry(&root, 0, "added", "new")
                                }
                            }
                        }
                    })
                    .unwrap();
                diffs.push(edit_diff(&source, document.source()));
            }
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

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +added: new␍
     a: old␍
     ␍
     # detached␍
     ␍
    -b: |+␍
    +b: |2+␍
       text␍
    -␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     a: old␍
     ␍
     # detached␍
    -␍
    -b: |+␍
    -  text␍
    -␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -a: old␍
    +a: new␍
     ␍
     # detached␍
     ␍
    -b: |+␍
    -  text␍
    -␍
    +b: |2+␍
    +  other␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +added: new␍
     a: old␍
     ␍
     # detached␍
     ␍
    -b: |+␍
    +␍
    +# detached␍
    +␍
    +b: |2+␍
       text␍
    -␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +- new␍
     - old␍
     ␍
     # detached␍
     ␍
    -- |+␍
    +- |2+␍
       text␍
    -␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     - old␍
     ␍
     # detached␍
    -␍
    -- |+␍
    -  text␍
    -␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -- old␍
    +- new␍
     ␍
     # detached␍
     ␍
    -- |+␍
    -  text␍
    -␍
    +- |2+␍
    +  other␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +- new␍
     - old␍
     ␍
     # detached␍
     ␍
    -- |+␍
    +␍
    +# detached␍
    +␍
    +- |2+␍
       text␍
    -␍
    +  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +added: new␍ a: old␍ ␍ # detached␍ ␍-b: |+␍+b: |2+␍   text␍-␍+  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     a: old␍ ␍ # detached␍-␍-b: |+␍-  text␍-␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -a: old␍+a: new␍ ␍ # detached␍ ␍-b: |+␍-  text␍-␍+b: |2+␍+  other␍+  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +added: new␍ a: old␍ ␍ # detached␍ ␍-b: |+␍+␍+# detached␍+␍+b: |2+␍   text␍-␍+  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,8 @@
    +- new␍ - old␍ ␍ # detached␍ ␍-- |+␍+- |2+␍   text␍-␍+  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,3 @@
     - old␍ ␍ # detached␍-␍-- |+␍-  text␍-␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,7 @@
    -- old␍+- new␍ ␍ # detached␍ ␍-- |+␍-  text␍-␍+- |2+␍+  other␍+  ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,11 @@
    +- new␍ - old␍ ␍ # detached␍ ␍-- |+␍+␍+# detached␍+␍+- |2+␍   text␍-␍+  ␍
    ");
}

#[test]
fn removed_children_leave_their_comments_in_the_surviving_gap() {
    let mut diffs = Vec::new();
    for (source, sequence) in [
        ("a: old\n\n# detached\n\nb: removed\nc: kept\n", false),
        ("- old\n\n# detached\n\n- removed\n- kept\n", true),
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
    let source = "a: old\n\n# retained\n\nb: |+\n  text\n\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            editor.insert_comment(&editor.root(), 0, " added", CommentPlacement::Detached)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
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
fn removing_the_last_gap_cannot_change_block_scalar_content() {
    // After removing the last entry, preserving this comment's detached
    // placement would require a blank line that changes the keep scalar.
    for (source, sequence) in [
        ("a: |+\n  text\n\n# detached\n\nb: removed\n", false),
        ("- |+\n  text\n\n# detached\n\n- removed\n", true),
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        assert!(
            document
                .edit(|editor| {
                    let root = editor.root();
                    if sequence {
                        editor.remove_item(&root, 1)
                    } else {
                        editor.remove_entry(&root, 1)
                    }
                })
                .is_err()
        );
        assert_eq!(document.source(), source);
    }
}
