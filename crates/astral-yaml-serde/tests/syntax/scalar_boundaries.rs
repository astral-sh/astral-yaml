//! Chomping determines which trailing line breaks belong to block scalar content.
//! The first trailing comment must be less indented than that content.
//! See <https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator>.

use super::support::{edit, edits};
use crate::syntax::DocumentSyntax;
use astral_yaml::comments::CommentPlacement;
use astral_yaml_dev::indoc;
use test_case::test_case;

#[test_case("|\n   x\n  # after\n" => "|\n  x\n# after\n"; "root")]
#[test_case("a: |\n   x\n  # after\n" => "a: |\n  x\n# after\n"; "mapping")]
#[test_case("- >-\n   x\n  # after\n" => "- >-\n  x\n# after\n"; "sequence")]
#[test_case("a:\n     b: |+\n        x\n\n    # first\n    # second\n" => "a:\n  b: |+\n    x\n    \n# first\n# second\n"; "nested keep")]
#[test_case("a: |\n   x\n  # after\n... # end\n" => "a: |\n  x\n# after\n... # end\n"; "end marker")]
#[test_case("|\n   x\n...\n  # after\n" => "|\n  x\n...\n  # after\n"; "comment after end marker")]
#[test_case("|\n   x\n # after\n" => "|\n  x\n # after\n"; "safe indentation")]
fn normalization_keeps_trailing_comments_outside_block_scalars(source: &str) -> String {
    let document = DocumentSyntax::parse(source).unwrap();
    let output = document.normalized_yaml().unwrap();
    let reparsed = DocumentSyntax::parse(&output).unwrap();
    assert_eq!(reparsed.normalized_yaml().unwrap(), output);
    output
}

#[test]
fn relocated_inline_headers_keep_detached_suffix_separation() {
    let source = indoc! {"
        --- # property
        !a [ # open
          old] # close
        # trailing

        # detached
        ...\x20

        # last
    "};
    let diff = edit(source, |editor| {
        editor.replace(
            editor.root(),
            indoc! {r"
                - new
            "},
        )
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,9 +1,9 @@
     --- # property
    -!a [ # open
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
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            editor.insert_comment(root, 0, " detached", CommentPlacement::Detached)?;
            editor.replace(root, fragment)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1 +1,6 @@
    -old
    +|+
    +  new
    +  ⏎
    +...
    +
    +# detached

    @@ -1 +1,6 @@
    -old
    +>+
    +  new
    +  ⏎
    +...
    +
    +# detached

    @@ -1 +1,5 @@
    -old
    +|
    +  new
    +...
    +
    +# detached

    @@ -1 +1,5 @@
    -old
    +|-
    +  new
    +...
    +
    +# detached

    @@ -1 +1,6 @@
    -old
    +- |+
    +  new
    +  ⏎
    +...
    +
    +# detached

    @@ -1 +1,6 @@
    -old
    +key: |+
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
    let diffs = edits(
        [
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
        ],
        |editor| editor.remove_comment(editor.root(), 0),
    );
    insta::assert_snapshot!(diffs, @r"
    @@ -1,5 +1,5 @@
     |+
       new
     ⏎
    -# tail
    +...
      ⏎
    \ No newline at end of file

    @@ -1,5 +1,5 @@
     |+
       new
     ⏎
    -# tail
    +...
     ⏎

    @@ -1,6 +1,6 @@
     |+
       new
     ⏎
    -# tail
    +...
     ⏎
     ...

    @@ -1,5 +1,5 @@
     - |+
         new
     ⏎
    -# tail
    +...
     ⏎

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
    // A document end marker can carry an inline comment.
    // https://yaml.org/spec/1.2.2/#912-document-markers
    let diffs = edits(
        [
            indoc! {r"
                old

                ... # inline
            "},
            indoc! {r"
                old
                ...

                ... # inline

            "},
        ],
        |editor| {
            let root = editor.root();
            editor.insert_comment(
                root,
                editor.node(root)?.comments().len(),
                " tail",
                CommentPlacement::Trailing,
            )
        },
    );
    insta::assert_snapshot!(diffs, @"
    @@ -1,3 +1,3 @@
     old
    -
     ... # inline
    +# tail

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
    // An explicit document may omit its root content. A stream permits repeated suffixes.
    // https://yaml.org/spec/1.2.2/#914-explicit-documents
    // https://yaml.org/spec/1.2.2/#92-streams
    let source = indoc! {r"
        ---
        old
        # tail
        ...

        # detached
        ... # inline
        ... # last
    "};
    let diff = edit(source, |editor| {
        editor.replace(
            editor.root(),
            indoc! {r"
                ---
            "},
        )
    });
    insta::assert_snapshot!(diff, @"
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
