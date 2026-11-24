//! Collection style across edits.
//! <https://yaml.org/spec/1.2.2/#74-flow-collection-styles>
//! <https://yaml.org/spec/1.2.2/#82-block-collection-styles>

use super::support::edit;
use astral_yaml_dev::indoc;
use astral_yaml_serde::EditableYaml;
use astral_yaml_serde::edit::CollectionStyle;
use serde_json::{Value, json};

#[test]
fn replacement_preserves_the_outer_style_only_for_the_same_kind() {
    let source = indoc! {r"
        flow: [1]
        block:
          - [1]
        changed: {old: 1}
    "};
    let document = edit::<Value>(source, |root| {
        let mut fields = root.into_mapping()?;
        fields.set("flow", [[2]])?;
        fields.set("block", [[3]])?;
        fields.set("changed", [[4]])?;
        Ok(())
    });
    insta::assert_snapshot!(document.source(), @"
    flow: [[2]]
    block:
      - - 3
    changed:
      - - 4
    ");
}

#[test]
fn empty_collections_follow_the_parent() {
    let source = "ports: [] # listen\nlimits: {}\nflags: {}\n";
    let document = edit::<Value>(source, |root| {
        let mut fields = root.into_mapping()?;
        fields.ensure_sequence("ports")?.push(80)?;
        fields.set("limits", json!({"max": [2]}))?;
        fields.ensure_mapping("flags")?.set("enabled", true)?;
        Ok(())
    });
    insta::assert_snapshot!(document.source(), @"
    ports: # listen
      - 80
    limits:
      max:
        - 2
    flags:
      enabled: true
    ");
}

#[test]
fn growing_a_collection_in_flow_context_preserves_surrounding_spacing() {
    let document = edit::<Value>("{ports: [],  other:   1}\n", |root| {
        root.field("ports")?.into_sequence()?.push([80, 443])?;
        Ok(())
    });
    insta::assert_snapshot!(document.source(), @"{ports: [[80, 443]],  other:   1}");
}

#[test]
fn nested_insertions_follow_the_final_parent_style() {
    let document = edit::<Value>("{old: [1]}\n", |root| {
        let mut fields = root.into_mapping()?;
        fields.set("before", [[2]])?;
        fields.set_collection_style(CollectionStyle::Block)?;
        fields.set("after", [[2]])?;
        fields
            .set("compact", [[3]])?
            .set_collection_style(CollectionStyle::Flow)
    });
    insta::assert_snapshot!(document.source(), @"
    old: [1]
    before:
      - - 2
    after:
      - - 2
    compact: [[3]]
    ");
}

#[test]
fn explicit_style_on_an_empty_collection_lasts_until_commit() {
    let mut doc = EditableYaml::parse("first: []\nsecond: []\n").unwrap();
    doc.edit(|mut root| {
        let mut first = root.reborrow().field("first")?.into_sequence()?;
        first.set_collection_style(CollectionStyle::Flow)?;
        first.push(1)?;
        root.field("second")?
            .set_collection_style(CollectionStyle::Flow)
    })
    .unwrap();
    doc.set_at(["second"], [2]).unwrap();
    insta::assert_snapshot!(doc.source(), @"
    first: [1]
    second:
      - 2
    ");
}

#[test]
fn canceled_growth_preserves_empty_delimiters_verbatim() {
    let source = "a: [  ]\nb: { }\n";
    let document = edit::<Value>(source, |root| {
        let mut items = root.field("a")?.into_sequence()?;
        items.push(1)?;
        items.remove(0)
    });
    assert_eq!(document.source(), source);
}

#[test]
fn same_kind_replacement_keeps_an_explicit_style_request() {
    let source = "{ports: []}\n";
    let mut doc = EditableYaml::parse(source).unwrap();
    let error = doc
        .edit(|root| {
            let mut ports = root.field("ports")?;
            ports.set_collection_style(CollectionStyle::Block)?;
            ports.set([80])
        })
        .unwrap_err();
    assert!(error.message.contains("requested style"));
    assert_eq!(doc.source(), source);
}
