//! Flow collection suffix comments belong before their closing delimiter.

use test_case::test_case;

use super::support::normalize;
use crate::syntax::DocumentSyntax;
use astral_yaml_dev::{formatdoc, indoc};

#[test]
fn omitted_final_values_keep_collection_suffix_comments() {
    let diffs = [
        indoc! {r"
            outer: {empty:,
              # trailing
              }
        "},
        indoc! {r"
            outer: {? : ,

              # detached
              }
        "},
        indoc! {r"
            [{empty: ,
              # trailing
              }]
        "},
        indoc! {r"
            outer: {empty:
              # value
              ,
              # collection
              }
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,3 +1,3 @@
    -outer: {empty:,
    +outer: {empty: ,
       # trailing
       }

    @@ -1,4 +1,4 @@
    -outer: {? : ,
    -
    +outer: {: ,
    +  ⏎
       # detached
       }

    @@ -1,3 +1,3 @@
     [{empty: ,
    -  # trailing
    -  }]
    +    # trailing
    +    }]

    @@ -1,5 +1,5 @@
    -outer: {empty:
    +outer: {
       # value
    -  ,
    +  empty: ,
       # collection
       }
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn normalization_keeps_flow_suffixes_inside_their_owner() {
    let diffs = [
        indoc! {r"
            outer: {key: old
              # trailing
              }
        "},
        indoc! {r"
            outer: [old

              # detached
              ]
        "},
        indoc! {r"
            [one, {key: old
              # trailing

              # detached
              }]
        "},
        indoc! {r"
            outer: [ # opening
              old
              # trailing
              ] # closing
        "},
        indoc! {r"
            outer: {
              # detached

              }
        "},
    ]
    .map(normalize);
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,3 +1,3 @@
    -outer: {key: old
    +outer: {key: old,
       # trailing
       }

    @@ -1,4 +1,4 @@
    -outer: [old
    -
    +outer: [old,
    +  ⏎
       # detached
       ]

    @@ -1,5 +1,5 @@
    -[one, {key: old
    -  # trailing
    -
    -  # detached
    -  }]
    +[one, {key: old,
    +    # trailing
    +    ⏎
    +    # detached
    +    }]

    @@ -1,4 +1,4 @@
     outer: [ # opening
    -  old
    +  old,
       # trailing
       ] # closing

    @@ -1,4 +1,4 @@
     outer: {
    +  ⏎
       # detached
    -
       }
    ");
}

#[test_case("[", "one", "]")]
#[test_case("{", "key: value", "}")]
#[test_case("{", "key: ,", "}")]
#[test_case("[", "", "]")]
#[test_case("{", "", "}")]
fn flow_comment_layouts_normalize_idempotently(open: &str, body: &str, close: &str) {
    // Delimiters enclose comments independently of block indentation:
    // https://yaml.org/spec/1.2.2/#74-flow-collection-styles
    for prefix in [
        "",
        r" # open
  ",
        r"
  # leading
  ",
        r"

  # detached

  ",
    ] {
        for suffix in [
            "",
            r"
  # trailing
  ",
            r"

  # detached
  ",
            r"
  # trailing

  # detached
  ",
        ] {
            for end in ["", " # close"] {
                let inner = format!("{open}{prefix}{body}{suffix}{close}{end}");
                for source in [
                    formatdoc! {r"
                        {inner}
                    "},
                    formatdoc! {r"
                        outer: {inner}
                    "},
                    formatdoc! {r"
                        - {inner}
                    "},
                    formatdoc! {r"
                        [{inner}
                        ]
                    "},
                    formatdoc! {r"
                        {{outer: {inner}
                        }}
                    "},
                ] {
                    let document = DocumentSyntax::parse(&source).unwrap();
                    let output = document
                        .normalized_yaml()
                        .unwrap_or_else(|error| panic!("{source:?}: {error}"));
                    assert_eq!(
                        DocumentSyntax::parse(&output)
                            .unwrap()
                            .normalized_yaml()
                            .unwrap(),
                        output
                    );
                }
            }
        }
    }
}
