use crate::syntax::DocumentSyntax;
use astral_yaml::CollectionStyle;
use astral_yaml_dev::{edit_diff, indoc};
use test_case::test_case;

#[test_case("a:\n    b: x\n  # root\n" => "a:\n  b: x\n# root\n"; "mapping")]
#[test_case("-\n    - x\n  # root\n" => "-\n  - x\n# root\n"; "sequence")]
#[test_case("-   a: x\n  # root\n" => "-\n  a: x\n# root\n"; "compact mapping")]
#[test_case("a:\n    b: x\n  # first\n # safe\n\n  # detached\n" => "a:\n  b: x\n# first\n # safe\n\n# detached\n"; "mixed suffixes")]
#[test_case("a:\n    b: x\nz: [q]\n   # root\n" => "a:\n  b: x\nz: [q]\n   # root\n"; "final flow collection")]
#[test_case("a:\n    b: x\nz: q\n   # root\n" => "a:\n  b: x\nz: q\n   # root\n"; "final scalar")]
#[test_case("a:\n    b: x\n  # root\n... # end\n    # framed\n" => "a:\n  b: x\n# root\n... # end\n    # framed\n"; "end marker")]
#[test_case("a:\n    b: |\n       x\n    # inner\n  # root\n" => "a:\n  b: |\n    x\n  # inner\n# root\n"; "nested scalar with comment")]
#[test_case("&root\na:\n  # before\n    b: x\n  # root\n" => "&root \na:\n  # before\n    b: x\n  # root\n"; "leading comment keeps child deeper")]
#[test_case("  a:\n      b: x\n    # root\n" => "  a:\n    b: x\n  # root\n"; "indented root")]
fn normalization_keeps_suffix_comments_on_the_root(source: &str) -> String {
    let document = DocumentSyntax::parse(source).unwrap();
    let output = document.normalized_yaml().unwrap();
    let reparsed = DocumentSyntax::parse(&output).unwrap();
    assert_eq!(reparsed.normalized_yaml().unwrap(), output);
    output
}

#[test]
fn layout_changes_keep_comments_after_omitted_mapping_values() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            outer:
              empty:
                # trailing
            next: retained
        "},
        indoc! {r"
            outer:
              empty:

              # detached
            next: retained
        "},
        indoc! {r"
            outer:
              empty:
                # trailing

              # detached
            next: retained
        "},
    ] {
        let mut document = DocumentSyntax::parse(source).unwrap();
        for style in [CollectionStyle::Flow, CollectionStyle::Block] {
            let before = document.source().to_owned();
            document
                .edit_syntax(|editor| {
                    let outer = editor.mapping_entries(editor.root())?[0].1;
                    editor.set_collection_style(outer, style)
                })
                .unwrap();
            diffs.push(edit_diff(&before, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,4 +1,4 @@
    -outer:
    -  empty:
    -    # trailing
    +outer: {empty: ,
    +  # trailing
    +  }
     next: retained

    @@ -1,4 +1,4 @@
    -outer: {empty: ,
    +outer:
    +  empty: ⏎
       # trailing
    -  }
     next: retained

    @@ -1,5 +1,5 @@
    -outer:
    -  empty:
    -
    +outer: {empty: ,
    +  ⏎
       # detached
    +  }
     next: retained

    @@ -1,5 +1,5 @@
    -outer: {empty: ,
    +outer:
    +  empty: ⏎
       ⏎
       # detached
    -  }
     next: retained

    @@ -1,6 +1,6 @@
    -outer:
    -  empty:
    -    # trailing
    -
    +outer: {empty: ,
    +  # trailing
    +  ⏎
       # detached
    +  }
     next: retained

    @@ -1,6 +1,6 @@
    -outer: {empty: ,
    +outer:
    +  empty: ⏎
       # trailing
       ⏎
       # detached
    -  }
     next: retained
    ");
}
