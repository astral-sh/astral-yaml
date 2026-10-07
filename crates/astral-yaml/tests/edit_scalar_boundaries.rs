//! Comment boundaries must not become block scalar content during an edit.

use astral_yaml::{comments::CommentPlacement, edit::EditableDocument};
use astral_yaml_dev::{edit_diff, indoc};

#[test]
fn relocated_inline_headers_keep_detached_suffix_separation() {
    let mut diffs = Vec::new();
    let source = indoc! {"
        --- # property
        &a [ # open
          old] # close
        # trailing

        # detached
        ...\x20

        # last
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            editor.replace(
                &editor.root(),
                indoc! {r"
                    - new
                "},
            )
        })
        .unwrap();
    diffs.push(edit_diff(source, document.source()));
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
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn detached_suffixes_survive_root_block_scalar_replacement() {
    // Blank lines before the dedented comment are subject to chomping:
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let mut diffs = Vec::new();
    for fragment in [
        indoc! {r"
            |+
              new

        "},
        indoc! {r"
            >+
              new

        "},
        indoc! {r"
            |
              new
        "},
        indoc! {r"
            |-
              new
        "},
        indoc! {r"
            - |+
                new

        "},
        indoc! {r"
            key: |+
              new

        "},
    ] {
        let source = indoc! {r"
            old
        "};
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| {
                let root = editor.root();
                editor.insert_comment(&root, 0, " detached", CommentPlacement::Detached)?;
                editor.replace(&root, fragment)
            })
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
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
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn removing_suffix_comments_does_not_extend_block_scalar_content() {
    let mut diffs = Vec::new();
    for source in [
        "|+\n  new\n\n# tail\n ",
        indoc! {r"
            |+
              new

            # tail

        "},
        indoc! {r"
            |+
              new

            # tail

            ...
        "},
        indoc! {r"
            - |+
                new

            # tail

        "},
        indoc! {r"
            key: |+
              new

            # tail

        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
        document
            .edit(|editor| editor.remove_comment(&editor.root(), 0))
            .unwrap();
        diffs.push(edit_diff(source, document.source()));
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
    ");
}

#[test]
fn trailing_comments_can_reclaim_blank_document_suffixes() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            old

            ... # inline
        "},
        indoc! {r"
            old
            ...

            ... # inline

        "},
    ] {
        let mut document = EditableDocument::parse(source).unwrap();
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
        diffs.push(edit_diff(source, document.source()));
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
    ");
}

#[test]
fn omitted_roots_keep_suffix_comments_between_their_fixed_neighbors() {
    let mut diffs = Vec::new();
    let source = indoc! {r"
        ---
        old
        # tail
        ...

        # detached
        ... # inline
        ... # last
    "};
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
    ");
}
