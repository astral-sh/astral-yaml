use crate::syntax::DocumentSyntax;
use astral_yaml::CollectionStyle;
use astral_yaml_dev::{edit_diff, indoc};

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
                    let outer = editor.mapping_entries(&editor.root())?[0].1.clone();
                    editor.set_collection_style(&outer, style)
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
