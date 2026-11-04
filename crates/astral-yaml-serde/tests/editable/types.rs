//! YAML node kinds: <https://yaml.org/spec/1.2.2/#3211-nodes>.

use test_case::test_case;

use super::support::edit;
use astral_yaml::NodeId;
use astral_yaml_serde::edit::{CollectionStyle, DocumentSyntax, NodeKind};
use astral_yaml_serde::{Editable, Spanned, from_str};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum Shape {
    Scalar(u8),
    Sequence(Vec<u8>),
    Mapping(BTreeMap<String, u8>),
}

fn assert_shape(actual: &Spanned<Shape>, syntax: &DocumentSyntax, node: NodeId, expected: &Shape) {
    assert_eq!(&actual.value, expected);
    assert_eq!(actual.span, syntax.node(node).unwrap().span());
}

#[derive(Debug, Deserialize)]
struct ShapeConfig {
    value: Spanned<Shape>,
    untouched: String,
}

#[test_case("[1, 2]", &Shape::Scalar(3); "sequence to scalar")]
#[test_case("1", &Shape::Sequence(vec![2, 3]); "scalar to sequence")]
#[test_case("{one: 1}", &Shape::Scalar(2); "mapping to scalar")]
#[test_case("1", &Shape::Mapping(BTreeMap::from([("two".into(), 2)])); "scalar to mapping")]
#[test_case("- 1\n- 2", &Shape::Mapping(BTreeMap::from([("three".into(), 3)])); "sequence to mapping")]
#[test_case("one: 1\ntwo: 2", &Shape::Sequence(vec![3, 4]); "mapping to sequence")]
fn kind_changes_refresh_the_variant_and_span(before: &str, expected: &Shape) {
    let document = edit::<Spanned<Shape>>(&format!("{before}\n"), |mut root| root.set(expected));
    let syntax = document.syntax();
    assert_shape(document.value(), syntax, syntax.root(), expected);

    let source = format!(
        "# config\nvalue:\n  {}\nuntouched: keep # sibling\n",
        before.replace('\n', "\n  ")
    );
    let document = edit::<ShapeConfig>(&source, |root| root.field("value")?.set(expected));
    let syntax = document.syntax();
    let NodeKind::Mapping { entries, .. } = syntax.node(syntax.root()).unwrap().kind() else {
        panic!("expected a mapping");
    };
    assert_shape(&document.value().value, syntax, entries[0].1, expected);
    assert_eq!(document.value().untouched, "keep");
    assert!(document.source().ends_with("\nuntouched: keep # sibling\n"));
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test_case("null", "true", &json!(true); "null to boolean")]
#[test_case("true", "42", &json!(42); "boolean to integer")]
#[test_case("42", "1.5", &json!(1.5); "integer to float")]
#[test_case("1.5", "text", &json!("text"); "float to string")]
#[test_case("'text'", "null", &Value::Null; "string to null")]
fn scalar_type_changes_refresh_the_typed_value(before: &str, replacement: &str, expected: &Value) {
    let document = edit::<Value>(&format!("{before} # value\n"), |mut root| {
        root.set(expected)
    });
    assert_eq!(document.value(), expected);
    assert_eq!(document.source(), format!("{replacement} # value\n"));
}

#[test_case("null", "[]", Some(&Shape::Sequence(Vec::new())); "null to sequence")]
#[test_case("[]", "null", None; "sequence to null")]
#[test_case("null", "{}", Some(&Shape::Mapping(BTreeMap::new())); "null to mapping")]
#[test_case("{}", "null", None; "mapping to null")]
fn null_and_empty_collections_can_replace_each_other(
    before: &str,
    replacement: &str,
    expected: Option<&Shape>,
) {
    // A null scalar is distinct from an empty sequence or mapping.
    // https://yaml.org/spec/1.2.2/#10211-null
    // https://yaml.org/spec/1.2.2/#3211-nodes
    let document = edit::<Option<Shape>>(&format!("{before} # nullable\n"), |mut root| {
        root.set(expected)
    });
    assert_eq!(document.value().as_ref(), expected);
    assert_eq!(document.source(), format!("{replacement} # nullable\n"));
}

#[test_case("1 # scalar\n", &1u8, ["[2, 3]", "{two: 2}"]; "scalar")]
#[test_case("[1, 2] # sequence\n", &vec![1u8, 2], ["3", "{three: 3}"]; "sequence")]
#[test_case(
    "one: 1 # mapping\n", &BTreeMap::from([("one".to_owned(), 1u8)]),
    ["2", "[2, 3]"]; "mapping"
)]
fn incompatible_kind_changes_roll_back_source_value_and_span<T>(
    source: &str,
    expected: &T,
    replacements: [&str; 2],
) where
    T: DeserializeOwned + Debug + PartialEq,
{
    let mut document = Editable::<Spanned<T>>::parse(source).unwrap();
    let span = document.value().span;
    for replacement in replacements {
        let value: Value = from_str(replacement).unwrap();
        assert!(document.edit(|mut root| root.set(&value)).is_err());
        assert_eq!(document.source(), source);
        assert_eq!(&document.value().value, expected);
        assert_eq!(document.value().span, span);
    }
}

#[test_case("- 2\n- 3", "---\n"; "block sequence")]
#[test_case("[2, 3]", ""; "flow sequence")]
#[test_case("two: 2", "---\n"; "block mapping")]
#[test_case("{two: 2}", ""; "flow mapping")]
fn kind_changes_create_boundaries_for_outer_comments(replacement: &str, marker: &str) {
    let expected = from_str::<Shape>(replacement).unwrap();
    let document = edit::<Spanned<Shape>>("# attached\n1\n", |mut root| {
        root.set(&expected)?;
        if !marker.is_empty() {
            root.set_collection_style(CollectionStyle::Block)?;
        }
        Ok(())
    });
    let syntax = document.syntax();
    assert_shape(document.value(), syntax, syntax.root(), &expected);
    assert_eq!(
        document.source(),
        format!(
            "# attached\n{marker}{}\n",
            replacement.replace("two:", "\"two\":")
        )
    );
}
