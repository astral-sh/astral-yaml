//! Comment boundaries must not become block scalar content during an edit.

use astral_yaml::{comments::CommentPlacement, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
fn relocated_inline_headers_keep_detached_suffix_separation() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "--- # property\n&a [ # open\n  old] # close\n# trailing\n\n# detached\n... \n\n# last\n".replace('\n', newline);
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| editor.replace(&editor.root(), "- new\n"))
            .unwrap();
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,9 +1,9 @@
     --- # property
    -&a [ # open
    -  old] # close
    +- new
    +...  # open
    +... # close
     # trailing
     ⏎
     # detached
    -... ⏎
     ⏎
     # last

    --- before.yaml
    +++ after.yaml
    @@ -1,9 +1,9 @@
     --- # property␍
    -&a [ # open␍
    -  old] # close␍
    +- new␍
    +...  # open␍
    +... # close␍
     # trailing␍
     ␍
     # detached␍
    -... ␍
     ␍
     # last␍

    --- before.yaml
    +++ after.yaml
    @@ -1,9 +1,9 @@
     --- # property␍-&a [ # open␍-  old] # close␍+- new␍+...  # open␍+... # close␍ # trailing␍ ␍ # detached␍-... ␍ ␍ # last␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn detached_suffixes_survive_root_block_scalar_replacement() {
    // Blank lines before the dedented comment are subject to chomping:
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for fragment in [
            "|+\n  new\n\n",
            ">+\n  new\n\n",
            "|\n  new\n",
            "|-\n  new\n",
            "- |+\n    new\n\n",
            "key: |+\n  new\n\n",
        ] {
            let source = "old\n".replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let root = editor.root();
                    editor.insert_comment(&root, 0, " detached", CommentPlacement::Detached)?;
                    editor.replace(&root, &fragment.replace('\n', newline))
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old
    +|2+
    +  new
    +  ⏎
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old
    +>2+
    +  new
    +  ⏎
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old
    +|2
    +  new
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old
    +|2-
    +  new
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old
    +- |2+
    +  new
    +  ⏎
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old
    +key: |2+
    +  new
    +  ⏎
    +...
    +
    +# detached

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍
    +|2+␍
    +  new␍
    +  ␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍
    +>2+␍
    +  new␍
    +  ␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old␍
    +|2␍
    +  new␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old␍
    +|2-␍
    +  new␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍
    +- |2+␍
    +  new␍
    +  ␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍
    +key: |2+␍
    +  new␍
    +  ␍
    +...␍
    +␍
    +# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍+|2+␍+  new␍+  ␍+...␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍+>2+␍+  new␍+  ␍+...␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old␍+|2␍+  new␍+...␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,5 @@
    -old␍+|2-␍+  new␍+...␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍+- |2+␍+  new␍+  ␍+...␍+␍+# detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,6 @@
    -old␍+key: |2+␍+  new␍+  ␍+...␍+␍+# detached␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn removing_suffix_comments_does_not_extend_block_scalar_content() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "|+\n  new\n\n# tail\n ",
            "|+\n  new\n\n# tail\n\n",
            "|+\n  new\n\n# tail\n\n...\n",
            "- |+\n    new\n\n# tail\n\n",
            "key: |+\n  new\n\n# tail\n\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.remove_comment(&editor.root(), 0))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+
       new
     ⏎
    -# tail
    +...
      ⏎
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+
       new
     ⏎
    -# tail
    +...
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     |+
       new
     ⏎
    -# tail
    +...
     ⏎
     ...

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     - |+
         new
     ⏎
    -# tail
    +...
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     key: |+
       new
     ⏎
    -# tail
    +...
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+␍
       new␍
     ␍
    -# tail␍
    +...␍
      ⏎
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+␍
       new␍
     ␍
    -# tail␍
    +...␍
     ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     |+␍
       new␍
     ␍
    -# tail␍
    +...␍
     ␍
     ...␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     - |+␍
         new␍
     ␍
    -# tail␍
    +...␍
     ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     key: |+␍
       new␍
     ␍
    -# tail␍
    +...␍
     ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+␍   new␍ ␍-# tail␍+...␍  ⏎
    \ No newline at end of file

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     |+␍   new␍ ␍-# tail␍+...␍ ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,6 @@
     |+␍   new␍ ␍-# tail␍+...␍ ␍ ...␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     - |+␍     new␍ ␍-# tail␍+...␍ ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     key: |+␍   new␍ ␍-# tail␍+...␍ ␍
    ");
}

#[test]
fn trailing_comments_can_reclaim_blank_document_suffixes() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in ["old\n\n... # inline\n", "old\n...\n\n... # inline\n\n"] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let root = editor.root();
                    editor.insert_comment(
                        &root,
                        editor.node(&root)?.comments().len(),
                        " tail",
                        CommentPlacement::Trailing,
                    )
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     old
    -
     ... # inline
    +# tail

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     old
     ...
    -
     ... # inline
    +# tail
     ⏎

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     old␍
    -␍
     ... # inline␍
    +# tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     old␍
     ...␍
    -␍
     ... # inline␍
    +# tail␍
     ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
     old␍-␍ ... # inline␍+# tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     old␍ ...␍-␍ ... # inline␍+# tail␍ ␍
    ");
}

#[test]
fn omitted_roots_keep_suffix_comments_between_their_fixed_neighbors() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "---\nold\n# tail\n...\n\n# detached\n... # inline\n... # last\n"
            .replace('\n', newline);
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| editor.replace(&editor.root(), "---\n"))
            .unwrap();
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,6 @@
     ---
    -old
    -# tail
     ...
    +# tail
     ⏎
     # detached
     ... # inline

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,6 @@
     ---␍
    -old␍
    -# tail␍
     ...␍
    +# tail␍
     ␍
     # detached␍
     ... # inline␍

    --- before.yaml
    +++ after.yaml
    @@ -1,7 +1,6 @@
     ---␍-old␍-# tail␍ ...␍+# tail␍ ␍ # detached␍ ... # inline␍
    ");
}
