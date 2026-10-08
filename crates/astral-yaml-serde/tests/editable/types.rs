//! YAML node kinds: <https://yaml.org/spec/1.2.2/#3211-nodes>.

use super::support::edit_syntax;
use astral_yaml::{
    NodeId,
    edit::{EditableDocument, NodeKind},
};
use astral_yaml_dev::{edit_diff, formatdoc, indoc};
use astral_yaml_serde::{Editable, Spanned, from_str};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum Shape {
    Scalar(u8),
    Sequence(Vec<u8>),
    Mapping(BTreeMap<String, u8>),
}

fn kind_changes() -> [(&'static str, &'static str, &'static str, Shape); 6] {
    [
        ("sequence to scalar", "[1, 2]", "3", Shape::Scalar(3)),
        (
            "scalar to sequence",
            "1",
            indoc! {r"
                - 2
                - 3"},
            Shape::Sequence(vec![2, 3]),
        ),
        ("mapping to scalar", "{one: 1}", "2", Shape::Scalar(2)),
        (
            "scalar to mapping",
            "1",
            "two: 2",
            Shape::Mapping(BTreeMap::from([("two".to_owned(), 2)])),
        ),
        (
            "sequence to mapping",
            indoc! {r"
                - 1
                - 2"},
            "{three: 3}",
            Shape::Mapping(BTreeMap::from([("three".to_owned(), 3)])),
        ),
        (
            "mapping to sequence",
            indoc! {r"
                one: 1
                two: 2"},
            "[3, 4]",
            Shape::Sequence(vec![3, 4]),
        ),
    ]
}

fn assert_shape(
    actual: &Spanned<Shape>,
    syntax: &EditableDocument,
    node: NodeId,
    expected: &Shape,
) {
    assert_eq!(&actual.value, expected);
    assert_eq!(actual.span, syntax.node(node).unwrap().span());
}

#[test]
fn root_kind_changes_refresh_the_untagged_variant_and_span() {
    let mut diffs = Vec::new();
    for (name, before, replacement, expected) in kind_changes() {
        let source = formatdoc! {r"
            {before}
        "};
        let document = edit_syntax::<Spanned<Shape>>(&source, |editor| {
            editor.replace(&editor.root(), replacement)
        });

        assert_shape(
            document.value(),
            document.syntax(),
            document.syntax().root(),
            &expected,
        );
        diffs.push(formatdoc! {r"
            {name}
            {}",
            edit_diff(&source, document.syntax().source())
        });
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    sequence to scalar
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[1, 2]
    +3

    scalar to sequence
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -1
    +- 2
    +- 3

    mapping to scalar
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{one: 1}
    +2

    scalar to mapping
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -1
    +two: 2

    sequence to mapping
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -- 1
    -- 2
    +{three: 3}

    mapping to sequence
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -one: 1
    -two: 2
    +[3, 4]
    ");
}

#[derive(Debug, Deserialize)]
struct ShapeConfig {
    value: Spanned<Shape>,
    untouched: String,
}

#[test]
fn nested_kind_changes_refresh_the_variant_and_preserve_siblings() {
    let mut diffs = Vec::new();
    for (name, before, replacement, expected) in kind_changes() {
        let source = formatdoc! {r"
            # config
            value:
              {}
            untouched: keep # sibling
        ",
            before.replace('\n', "\n  ")
        };
        let document = edit_syntax::<ShapeConfig>(&source, |editor| {
            let entries = editor.mapping_entries(&editor.root())?;
            editor.replace(&entries[0].1, replacement)
        });

        let syntax = document.syntax();
        let NodeKind::Mapping { entries, .. } = syntax.node(syntax.root()).unwrap().kind() else {
            panic!("expected a mapping");
        };
        assert_shape(&document.value().value, syntax, entries[0].1, &expected);
        assert_eq!(document.value().untouched, "keep");
        diffs.push(formatdoc! {r"
            {name}
            {}",
            edit_diff(&source, document.syntax().source())
        });
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    sequence to scalar
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
     # config
    -value:
    -  [1, 2]
    +value: 3
     untouched: keep # sibling

    scalar to sequence
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,5 @@
     # config
     value:
    -  1
    +  - 2
    +  - 3
     untouched: keep # sibling

    mapping to scalar
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,3 @@
     # config
    -value:
    -  {one: 1}
    +value: 2
     untouched: keep # sibling

    scalar to mapping
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     # config
     value:
    -  1
    +  two: 2
     untouched: keep # sibling

    sequence to mapping
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,3 @@
     # config
    -value:
    -  - 1
    -  - 2
    +value: {three: 3}
     untouched: keep # sibling

    mapping to sequence
    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,3 @@
     # config
    -value:
    -  one: 1
    -  two: 2
    +value: [3, 4]
     untouched: keep # sibling
    ");
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn scalar_type_changes_refresh_the_typed_value() {
    let mut diffs = Vec::new();
    for (before, replacement, expected) in [
        ("null", "true", json!(true)),
        ("true", "42", json!(42)),
        ("42", "1.5", json!(1.5)),
        ("1.5", "'text'", json!("text")),
        ("'text'", "null", Value::Null),
    ] {
        let source = formatdoc! {r"
            {before} # value
        "};
        let document = edit_syntax::<Value>(&source, |editor| {
            editor.replace(&editor.root(), replacement)
        });

        assert_eq!(document.value(), &expected);
        diffs.push(edit_diff(&source, document.syntax().source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -null # value
    +true # value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -true # value
    +42 # value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -42 # value
    +1.5 # value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -1.5 # value
    +'text' # value

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -'text' # value
    +null # value
    ");
}

#[test]
fn null_and_empty_collections_can_replace_each_other() {
    let mut diffs = Vec::new();
    for (before, replacement, expected) in [
        ("null", "[]", Some(Shape::Sequence(Vec::new()))),
        ("[]", "null", None),
        ("null", "{}", Some(Shape::Mapping(BTreeMap::new()))),
        ("{}", "null", None),
    ] {
        let source = formatdoc! {r"
            {before} # nullable
        "};
        let document = edit_syntax::<Option<Shape>>(&source, |editor| {
            editor.replace(&editor.root(), replacement)
        });

        assert_eq!(document.value(), &expected);
        diffs.push(edit_diff(&source, document.syntax().source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -null # nullable
    +[] # nullable

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -[] # nullable
    +null # nullable

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -null # nullable
    +{} # nullable

    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -{} # nullable
    +null # nullable
    ");
}

fn assert_rejected_kind_changes<T>(source: &str, expected: &T, replacements: &[&str])
where
    T: Debug + DeserializeOwned + PartialEq,
{
    for replacement in replacements {
        let mut document = Editable::<Spanned<T>>::parse(source).unwrap();
        let span = document.value().span;
        let mut syntax = document.syntax().clone();
        syntax
            .edit(|editor| editor.replace(&editor.root(), replacement))
            .unwrap();
        assert!(
            document
                .edit_syntax(|editor| editor.replace(&editor.root(), replacement))
                .is_err()
        );
        assert_eq!(document.syntax().source(), source);
        assert_eq!(&document.value().value, expected);
        assert_eq!(document.value().span, span);
    }
}

#[test]
fn incompatible_kind_changes_roll_back_source_value_and_span() {
    assert_rejected_kind_changes(
        indoc! {r"
            1 # scalar
        "},
        &1u8,
        &["[2, 3]", "{two: 2}"],
    );
    assert_rejected_kind_changes(
        indoc! {r"
            [1, 2] # sequence
        "},
        &vec![1u8, 2],
        &["3", "{three: 3}"],
    );
    assert_rejected_kind_changes(
        indoc! {r"
            one: 1 # mapping
        "},
        &BTreeMap::from([("one".to_owned(), 1u8)]),
        &["2", "[2, 3]"],
    );
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn kind_changes_create_boundaries_for_outer_comments() {
    let mut diffs = Vec::new();
    let source = indoc! {r"
        # attached
        1
    "};
    for replacement in [
        indoc! {r"
            - 2
            - 3"},
        "[2, 3]",
        "two: 2",
        "{two: 2}",
    ] {
        let expected = from_str::<Shape>(replacement).unwrap();
        let document = edit_syntax::<Spanned<Shape>>(source, |editor| {
            editor.replace(&editor.root(), replacement)
        });
        assert_eq!(document.value().value, expected);
        let syntax = document.syntax();
        assert_eq!(
            document.value().span,
            syntax.node(syntax.root()).unwrap().span()
        );
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
     # attached
    -1
    +---
    +- 2
    +- 3

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     # attached
    -1
    +[2, 3]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
     # attached
    -1
    +---
    +two: 2

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     # attached
    -1
    +{two: 2}
    ");
}
