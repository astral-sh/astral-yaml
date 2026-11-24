//! Source and comment preservation when editing block collections.
//! <https://yaml.org/spec/1.2.2/#66-comments>

use super::support::edit;
use astral_yaml_serde::EditableYaml;
use astral_yaml_serde::edit::{CollectionStyle, CommentPlacement};
use serde_json::Value;
use test_case::test_case;

#[test]
fn expanding_a_value_keeps_the_rest_of_the_mapping_verbatim() {
    let source = "ports: []\n\nkeep:  !!str true\n";
    let mut doc = EditableYaml::parse_with_options(
        source,
        astral_yaml::ParseOptions {
            max_input_bytes: 40,
            ..Default::default()
        },
    )
    .unwrap();
    doc.edit(|root| {
        let mut ports = root.field("ports")?;
        ports.set([80])?;
        ports.set_collection_style(CollectionStyle::Block)
    })
    .unwrap();
    insta::assert_snapshot!(doc.source(), @"
    ports:
      - 80

    keep:  !!str true
    ");
}

#[test]
fn an_opening_comment_moves_to_the_retained_field_header() {
    let doc = edit::<Value>("ports:\n  [ # listen\n  ]\nother:   [1,  2]\n", |root| {
        let mut ports = root.field("ports")?.into_sequence()?;
        ports.push(80)?;
        ports.set_collection_style(CollectionStyle::Block)
    });
    insta::assert_snapshot!(doc.source(), @"
    ports: # listen
      - 80
    other:   [1,  2]
    ");
}

#[test]
fn an_indentless_sequence_moves_inward_to_keep_a_nested_suffix() {
    let doc = edit::<Value>("ports:\n- [\n  # keep\n  ]\nother:   1\n", |root| {
        let mut ports = root.field("ports")?.item(0)?.into_sequence()?;
        ports.push(80)?;
        ports.set_collection_style(CollectionStyle::Block)
    });
    assert_eq!(
        doc.source(),
        "ports:\n  - - 80\n    \n    # keep\nother:   1\n"
    );
}

#[test]
fn reindenting_a_sequence_keeps_the_enclosing_suffix_outside() {
    let doc = edit::<Value>("ports:\n- []\n  # tail\n", |root| {
        let mut item = root.field("ports")?.item(0)?;
        item.set([80])?;
        item.set_collection_style(CollectionStyle::Block)?;
        item.append_comment(" listen", CommentPlacement::Trailing)
    });
    insta::assert_snapshot!(doc.source(), @"
    ports:
      - - 80
        # listen
    # tail
    ");
}

#[test]
fn a_root_delimiter_separates_detached_and_trailing_comments() {
    let doc = edit::<Value>("- 1\n\n# middle\n\n- 2\n# tail\n", |mut root| root.set([1]));
    assert_eq!(doc.source(), "[1,\n  \n  # middle\n  ]\n# tail\n");
}

#[test]
fn a_marker_comment_and_a_leading_comment_can_precede_a_block_root() {
    let mut doc = EditableYaml::parse("--- # header\n...\n").unwrap();
    doc.edit(|mut root| {
        root.set([1])?;
        root.set_collection_style(CollectionStyle::Block)?;
        root.append_comment(" leading", CommentPlacement::Leading)
    })
    .unwrap();
    insta::assert_snapshot!(doc.source(), @"
    --- # header
    # leading
      - 1
    ...
    ");
}

#[test]
fn a_delimiter_separates_a_property_header_from_the_first_field_prefix() {
    let doc = edit::<Value>("v: &a\n  k: [1]\nr: *a", |root| {
        let mut field = root.field("v")?.field("k")?;
        field.append_comment(" before", CommentPlacement::Leading)?;
        field.append_comment(" header", CommentPlacement::Inline)?;
        field.append_comment(" body", CommentPlacement::Leading)
    });
    insta::assert_snapshot!(doc.source(), @"
    v: &a {
      # before
      k: # header
      # body
      [1]}
    r: *a
    ");
}

#[test]
fn a_flow_delimiter_after_a_property_clears_the_enclosing_block() {
    let doc = edit::<Value>("v: &a\n  - 1\nr: *a", |root| {
        let mut value = root.field("v")?;
        value.append_comment(" open", CommentPlacement::Inline)?;
        value.append_comment(" tail", CommentPlacement::Trailing)?;
        value.append_comment(" close", CommentPlacement::Inline)
    });
    insta::assert_snapshot!(doc.source(), @"
    v: &a # open
      [1,
      # tail
      ] # close
    r: *a
    ");
}

#[test_case("\t{}\n"; "tab before flow token")]
#[test_case(":\n"; "omitted key at start of document")]
fn a_leading_comment_stays_before_the_generated_start_marker(source: &str) {
    let mut doc = EditableYaml::parse(source).unwrap();
    doc.edit(|mut root| {
        root.set([1])?;
        root.set_collection_style(CollectionStyle::Block)?;
        root.append_comment(" leading", CommentPlacement::Leading)
    })
    .unwrap();
    assert_eq!(doc.source(), "# leading\n---\n- 1\n");
}

#[test]
fn expanding_below_a_retained_header_preserves_its_indentation() {
    let doc = edit::<Value>("ports: # listen\n    {}\nother:   1\n", |root| {
        let mut ports = root.field("ports")?;
        ports.set([80])?;
        ports.set_collection_style(CollectionStyle::Block)
    });
    insta::assert_snapshot!(doc.source(), @"
    ports: # listen
        - 80
    other:   1
    ");
}

#[test]
fn punctuation_in_a_comment_cannot_supply_a_field_indicator() {
    let doc = edit::<Value>("ports: # listen:\n  &ports [] # default\n", |root| {
        let mut ports = root.field("ports")?;
        ports.set([80])?;
        ports.set_collection_style(CollectionStyle::Block)
    });
    insta::assert_snapshot!(doc.source(), @"
    ports: # listen:
      &ports # default
      - 80
    ");
}

#[test]
fn changing_a_field_prefix_updates_the_header_when_the_value_expands() {
    let doc = edit::<Value>("# old\nports: []\n", |root| {
        let mut ports = root.field("ports")?;
        ports.replace_comment(0, " listen")?;
        ports.set([80])?;
        ports.set_collection_style(CollectionStyle::Block)
    });
    insta::assert_snapshot!(doc.source(), @"
    # listen
    ports:
      - 80
    ");
}
