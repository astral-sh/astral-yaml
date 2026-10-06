//! Comment placement across fragment replacement and presentation changes.

use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn replacement_merges_comments_at_their_structural_positions() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, replacement) in [
            ("old # tail\n", "# leading\nnew\n"),
            ("a: old\n\n# tail\n", "a: one\n\n# middle\n\nb: two\n"),
            (
                "a: old\nb: old\n\n# retained\n\nc: old\n",
                "a: one\n\n# imported\n\nb: two\nc: three\n",
            ),
            (
                "- old\n\n# retained\n\n- old\n",
                "# leading\n- one\n\n# imported\n\n- two\n",
            ),
            ("old\n# trailing\n", "new # inline\n"),
        ] {
            let source = source.replace('\n', newline);
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
    @@ -1 +1,2 @@
    -old # tail
    +# leading
    +new # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old
    +a: one
    +
    +# middle
    +
    +b: two
     ⏎
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old
    -b: old
    +a: one
    +
    +# imported
    +
    +b: two
     ⏎
     # retained
     ⏎
    -c: old
    +c: three

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old
    +# leading
    +- one
     ⏎
     # retained
     ⏎
    -- old
    +
    +# imported
    +
    +- two

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old
    +new # inline
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail␍
    +# leading␍
    +new # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old␍
    +a: one␍
    +␍
    +# middle␍
    +␍
    +b: two␍
     ␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old␍
    -b: old␍
    +a: one␍
    +␍
    +# imported␍
    +␍
    +b: two␍
     ␍
     # retained␍
     ␍
    -c: old␍
    +c: three␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old␍
    +# leading␍
    +- one␍
     ␍
     # retained␍
     ␍
    -- old␍
    +␍
    +# imported␍
    +␍
    +- two␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old␍
    +new # inline␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail␍+# leading␍+new # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old␍+a: one␍+␍+# middle␍+␍+b: two␍ ␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old␍-b: old␍+a: one␍+␍+# imported␍+␍+b: two␍ ␍ # retained␍ ␍-c: old␍+c: three␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old␍+# leading␍+- one␍ ␍ # retained␍ ␍-- old␍+␍+# imported␍+␍+- two␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old␍+new # inline␍ # trailing␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_collection_headers_retain_inline_comments() {
    // Node properties: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "&root [old] # inline\n",
            "&root {a: old} # inline\n",
            "!!seq [old] # inline\n",
            "!!seq # tag\n&root [old] # inline\n",
            "# leading\n%YAML 1.2 # directive\n--- &root [old] # inline\n... # end\n# trailing\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
                .unwrap();
            let once = document.source().to_owned();
            document
                .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
                .unwrap();
            assert_eq!(document.source(), once);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline
    +&root # inline
    +a: old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline
    +!<tag:yaml.org,2002:seq> # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag
    -&root [old] # inline
    +!<tag:yaml.org,2002:seq> # tag
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading
     %YAML 1.2 # directive
    ---- &root [old] # inline
    +--- ⏎
    +&root # inline
    +- old
     ... # end
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline␍
    +&root # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline␍
    +&root # inline␍
    +a: old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline␍
    +!<tag:yaml.org,2002:seq> # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag␍
    -&root [old] # inline␍
    +!<tag:yaml.org,2002:seq> # tag␍
    +&root # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading␍
     %YAML 1.2 # directive␍
    ---- &root [old] # inline␍
    +--- ␍
    +&root # inline␍
    +- old␍
     ... # end␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline␍+&root # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline␍+&root # inline␍+a: old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline␍+!<tag:yaml.org,2002:seq> # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag␍-&root [old] # inline␍+!<tag:yaml.org,2002:seq> # tag␍+&root # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading␍ %YAML 1.2 # directive␍---- &root [old] # inline␍+--- ␍+&root # inline␍+- old␍ ... # end␍ # trailing␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn imported_prefixes_and_suffixes_preserve_comment_adjacency() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, replacement) in [
            ("old\n", "# detached\n\nnew\n"),
            ("# retained\nold\n", "# detached\n\nnew\n"),
            ("# retained\n\nold\n", "# leading\nnew\n"),
            ("--- # marker\n# retained\nold\n", "# detached\n\nnew\n"),
            ("old # inline\n", "new\n# trailing\n"),
            ("old # inline\n", "new\n\n# detached\n"),
            ("old\n# trailing\n", "new\n\n# detached\n"),
            ("old\n\n# detached\n", "new\n# trailing\n"),
            (
                "&root # retained\nold\n",
                "# leading\n&next # imported\nnew\n",
            ),
        ] {
            let source = source.replace('\n', newline);
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
    @@ -1 +1,3 @@
    -old
    +# detached
    +
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    +# detached
    +
     # retained
    -old
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     # retained
     ⏎
    -old
    +# leading
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     --- # marker
    +# detached
    +
     # retained
    -old
    +new

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline
    +new # inline
    +# trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # inline
    +new # inline
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -old
    +new
     # trailing
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old
    +new
    +# trailing
     ⏎
     # detached

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -&root # retained
    -old
    +# leading
    +&next # retained
    +  new # imported

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old␍
    +# detached␍
    +␍
    +new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    +# detached␍
    +␍
     # retained␍
    -old␍
    +new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     # retained␍
     ␍
    -old␍
    +# leading␍
    +new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     --- # marker␍
    +# detached␍
    +␍
     # retained␍
    -old␍
    +new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline␍
    +new # inline␍
    +# trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # inline␍
    +new # inline␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -old␍
    +new␍
     # trailing␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old␍
    +new␍
    +# trailing␍
     ␍
     # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -&root # retained␍
    -old␍
    +# leading␍
    +&next # retained␍
    +  new # imported␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old␍+# detached␍+␍+new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    +# detached␍+␍ # retained␍-old␍+new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     # retained␍ ␍-old␍+# leading␍+new␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
     --- # marker␍+# detached␍+␍ # retained␍-old␍+new␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # inline␍+new # inline␍+# trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old # inline␍+new # inline␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -old␍+new␍ # trailing␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -old␍+new␍+# trailing␍ ␍ # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -&root # retained␍-old␍+# leading␍+&next # retained␍+  new # imported␍
    ");
}

#[test]
fn nested_fragment_prefixes_belong_to_the_selected_value() {
    let mut diffs = Vec::new();
    for source in ["outer: old\nother: keep\n", "- old\n- keep\n"] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                let value = if source.starts_with("outer") {
                    editor.mapping_entries(&root)?.remove(0).1
                } else {
                    editor.sequence_items(&root)?.remove(0)
                };
                editor.replace(&value, "# detached\n\n# leading\nnew\n")
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,6 @@
    -outer: old
    +outer:
    +  # detached
    +
    +  # leading
    +  new
     other: keep

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,6 @@
    -- old
    +-
    +  # detached
    +
    +  # leading
    +  new
     - keep
    ");
}

#[test]
fn replacement_does_not_reorder_an_explicitly_edited_comment_list() {
    use astral_yaml::comments::CommentPlacement;
    let source = "old # inline\n";
    let mut document = EditableDocument::parse(source).unwrap();
    let result = document.edit(|editor| {
        let root = editor.root();
        // This explicit order cannot keep the new comment leading. Replacement
        // must preserve that request and reject it, rather than sort it away.
        editor.insert_comment(&root, 1, " leading", CommentPlacement::Leading)?;
        editor.replace(&root, "# imported\nnew\n")
    });
    assert!(result.is_err());
    assert_eq!(document.source(), source);
}

#[test]
fn merged_comment_gaps_survive_further_replacement_and_removal() {
    let source = "a: old\n\n# tail\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let root = editor.root();
            editor.replace(&root, "a: one\n\n# first\n\nb: two\n")?;
            editor.replace(&root, "a: first\n\n# second\n\nb: middle\nc: last\n")?;
            editor.remove_entry(&root, 1)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,10 @@
    -a: old
    +a: first
    +
    +# first
    +
    +
    +# second
    +
    +c: last
     ⏎
     # tail
    ");
}
