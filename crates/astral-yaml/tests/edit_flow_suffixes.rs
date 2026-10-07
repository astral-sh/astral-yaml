//! Flow collection suffix comments belong before their closing delimiter.

use astral_yaml::edit::EditableDocument;
use astral_yaml_dev::edit_diff;
use astral_yaml_dev::{formatdoc, indoc};

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn normalization_keeps_flow_suffixes_inside_their_owner() {
    let mut diffs = Vec::new();
    for source in [
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
    ] {
        let document = EditableDocument::parse(source).unwrap();
        let output = document.normalized_yaml().unwrap();
        assert_eq!(
            EditableDocument::parse(&output)
                .unwrap()
                .normalized_yaml()
                .unwrap(),
            output
        );
        diffs.push(edit_diff(source, &output));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: [old
    -
    +  ⏎
       # detached
       ]

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     [one, {key: old
    -  # trailing
    -
    -  # detached
    -  }]
    +    # trailing
    +    ⏎
    +    # detached
    +    }]


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: {
    +  ⏎
       # detached
    -
       }
    ");
}

#[test]
fn flow_comment_layouts_normalize_idempotently() {
    // Delimiters enclose comments independently of block indentation:
    // https://yaml.org/spec/1.2.2/#74-flow-collection-styles
    for (open, body, close) in [
        ("[", "one", "]"),
        ("{", "key: value", "}"),
        ("[", "", "]"),
        ("{", "", "}"),
    ] {
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
                        let document = EditableDocument::parse(&source).unwrap();
                        let output = document
                            .normalized_yaml()
                            .unwrap_or_else(|error| panic!("{source:?}: {error}"));
                        assert_eq!(
                            EditableDocument::parse(&output)
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
}
