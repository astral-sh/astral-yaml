//! Value edits preserve YAML types: <https://yaml.org/spec/1.2.2/#103-core-schema>.

use test_case::test_case;

use super::support::edit;
use astral_yaml_dev::{edit_diff, formatdoc, indoc};
use astral_yaml_serde::{
    Editable,
    edit::{CollectionStyle, CommentPlacement, NodeKind, ScalarStyle},
};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use serde_json::{Value, json};

#[test_case(
    indoc! {r"
        !custom name: old
    "},
    "name";
    "plain tagged key"
)]
#[test_case(
    indoc! {r"
        !custom 'name': old
    "},
    "name";
    "quoted tagged key"
)]
#[test_case(
    indoc! {r"
        !custom true: old
    "},
    "true";
    "tagged boolean"
)]
#[test_case(
    indoc! {r"
        !custom 'true': old
    "},
    "true";
    "quoted tagged boolean"
)]
#[test_case(
    indoc! {r"
        !<tag:example.org,2026:key> name: old
    "},
    "name";
    "global tagged key"
)]
fn string_fields_do_not_match_application_tagged_keys(source: &str, name: &str) {
    // Node equality includes tags: https://yaml.org/spec/1.2.2/#3213-node-comparison
    let mut document = Editable::<
        std::collections::BTreeMap<astral_yaml_serde::Tagged<String>, String>,
    >::parse(source)
    .unwrap();
    document
        .edit(|editor| {
            assert!(editor.root_mut().get(name)?.is_none(), "{source:?}");
            assert!(editor.root_mut().field(name).is_err());
            assert!(!editor.root_mut().remove_field(name)?);
            Ok(())
        })
        .unwrap();
    assert_eq!(document.source(), source);
}

#[derive(Debug, Deserialize)]
struct Config {
    timeout: u8,
    servers: Vec<String>,
}

#[test_case(0)]
#[test_case(1)]
#[test_case(2)]
#[test_case(3)]
fn results_are_returned_only_after_callback_syntax_and_type_checks(operation: usize) {
    let source = indoc! {r"
        timeout: 30
        servers: [primary]
    "};
    let mut document = Editable::<Config>::parse(source).unwrap();
    let result = document.edit(|e| {
        match operation {
            0 => e.root_mut().field("timeout")?.set(60)?,
            1 => e.root_mut().field("timeout")?.set(256)?,
            2 => {
                e.root_mut().field("missing")?;
            }
            _ => e
                .root_mut()
                .field("servers")?
                .item(0)?
                .set_scalar_style(ScalarStyle::Literal)?,
        }
        Ok("committed")
    });
    if operation == 0 {
        assert_eq!(result.unwrap(), "committed");
        assert_eq!(document.value().timeout, 60);
    } else {
        assert!(result.is_err());
        assert_eq!(document.source(), source);
        assert_eq!(document.value().timeout, 30);
    }
}

#[test]
fn fluent_configuration_edits_keep_order_comments_and_unknown_fields() {
    let source = indoc! {r"
        timeout: 30 # seconds
        servers: [primary, retired]
        extra: 'retained'
    "};
    let mut config = Editable::<Config>::parse(source).unwrap();
    let result = config
        .edit(|e| {
            e.root_mut().field("timeout")?.set(60)?;
            e.root_mut().field("servers")?.item(1)?.set("backup")?;
            e.root_mut().field("servers")?.push("tertiary")?;
            e.root_mut().set_entry("enabled", true)?;
            Ok("committed")
        })
        .unwrap();
    assert_eq!(result, "committed");
    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup", "tertiary"]);
    insta::assert_snapshot!(edit_diff(source, config.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -timeout: 30 # seconds
    -servers: [primary, retired]
    +timeout: 60 # seconds
    +servers: [primary, backup, "tertiary"]
     extra: 'retained'
    +"enabled": true
    "#);
    let pointer = config.source().as_ptr();
    let yaml = config.into_yaml();
    assert_eq!(yaml.as_ptr(), pointer);
}

#[test]
fn rust_values_never_interpret_strings_as_yaml() {
    let source = indoc! {r"
        old
    "};
    let mut diffs = Vec::new();
    for (name, value) in [
        ("string", json!("false")),
        ("boolean", json!(false)),
        ("sequence", json!(["a", "b"])),
        ("mapping", json!({"a": 1})),
        ("null", Value::Null),
        ("YAML-looking string", json!("[a, b] # still a string")),
        ("empty string", json!("")),
        (
            "escaped string",
            json!(indoc! {r"
                it's a
                new line"}),
        ),
    ] {
        let document = edit::<Value>(source, |e| e.root_mut().set(&value));
        assert_eq!(document.value(), &value);
        diffs.push(formatdoc! {r"
            {name}
            {}", edit_diff(source, document.source())});
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    string
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"false"

    boolean
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +false

    sequence
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +["a", "b"]

    mapping
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +{"a": 1}

    null
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +null

    YAML-looking string
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +"[a, b] # still a string"

    empty string
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -old
    +""

    escaped string
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -old
    +it's a
    +
    +  new line
    "#);
}

#[test_case(
    "'it''s': old",
    "it's"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -'it''s': old
        \ No newline at end of file
        +'it''s': new
        \ No newline at end of file
    ");
    "escaped quote"
)]
#[test_case(
    "'a.b/[0]': old",
    "a.b/[0]"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -'a.b/[0]': old
        \ No newline at end of file
        +'a.b/[0]': new
        \ No newline at end of file
    ");
    "path punctuation"
)]
#[test_case(
    "é: old",
    "é"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -é: old
        \ No newline at end of file
        +é: new
        \ No newline at end of file
    ");
    "unicode"
)]
#[test_case(
    "!!str true: old",
    "true"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -!!str true: old
        \ No newline at end of file
        +!!str true: new
        \ No newline at end of file
    ");
    "core string tag"
)]
#[test_case(
    "! true: old",
    "true"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -! true: old
        \ No newline at end of file
        +! true: new
        \ No newline at end of file
    ");
    "non specific tag"
)]
#[test_case(
    "{true: ignored, 'true': old}",
    "true"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{true: ignored, 'true': old}
        \ No newline at end of file
        +{true: ignored, 'true': new}
        \ No newline at end of file
    ");
    "boolean spelling"
)]
#[test_case(
    "{1: ignored, '1': old}",
    "1"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{1: ignored, '1': old}
        \ No newline at end of file
        +{1: ignored, '1': new}
        \ No newline at end of file
    ");
    "integer spelling"
)]
#[test_case(
    "{!!int '1': ignored, '1': old}",
    "1"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{!!int '1': ignored, '1': old}
        \ No newline at end of file
        +{!!int '1': ignored, '1': new}
        \ No newline at end of file
    ");
    "explicit integer spelling"
)]
#[test_case(
    "{[complex, key]: ignored, name: old}",
    "name"
    => with |diff| insta::assert_snapshot!(diff, @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -{[complex, key]: ignored, name: old}
        \ No newline at end of file
        +{[complex, key]: ignored, name: new}
        \ No newline at end of file
    ");
    "complex sibling key"
)]
fn fields_use_literal_decoded_string_keys(source: &str, name: &str) -> String {
    let mut document = Editable::<IgnoredAny>::parse(source).unwrap();
    document
        .edit(|e| {
            let mut field = e.root_mut().field(name)?;
            assert!(
                matches!(field.node()?.kind(), NodeKind::Scalar { value, .. } if value == "old")
            );
            field.set("new")
        })
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    edit_diff(source, document.source())
}

#[test]
fn optional_lookup_and_removal_distinguish_absence() {
    let source = indoc! {r"
        first: 1 # removed
        second: 2 # retained
    "};
    let document = edit::<Value>(source, |e| {
        let mut root = e.root_mut();
        assert!(root.reborrow().get("absent")?.is_none());
        assert!(!root.remove_field("absent")?);
        assert!(root.remove_field("first")?);
        root.reborrow().get("second")?.unwrap().set(3)
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -first: 1 # removed
    -second: 2 # retained
    +second: 3 # retained
    ");
}

#[test]
fn insertion_returns_a_cursor_and_parents_can_be_reborrowed() {
    let source = indoc! {r"
        items: [old]
    "};
    let document = edit::<Value>(source, |e| {
        let mut items = e.root_mut().field("items")?;
        items
            .reborrow()
            .insert(0, "first")?
            .set_scalar_style(ScalarStyle::SingleQuoted)?;
        items.remove_item(1)?;
        items
            .reborrow()
            .push(json!({"value": "old"}))?
            .field("value")?
            .set("new")?;
        items.set_collection_style(CollectionStyle::Block)
    });
    assert_eq!(
        document.value(),
        &json!({"items": ["first", {"value": "new"}]})
    );
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,3 @@
    -items: [old]
    +items:
    +  - 'first'
    +  - {"value": "new"}
    "#);
}

#[test]
fn upserts_preserve_existing_positions_and_allow_further_edits() {
    let source = indoc! {r"
        first: 'old' # keep
        last: true
    "};
    let document = edit::<Value>(source, |e| {
        e.root_mut().set_entry("first", "it's new")?;
        e.root_mut().set_entry("added", ["initial"])?;
        e.root_mut()
            .set_entry("added", ["one"])?
            .push("two")?
            .append_comment(" added", CommentPlacement::Inline)?;
        Ok(())
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,4 @@
    -first: 'old' # keep
    +first: 'it''s new' # keep
     last: true
    +"added": ["one", "two" # added
    +  ]
    "#);
}

#[test]
fn styles_and_comments_can_be_changed_explicitly() {
    let source = indoc! {r"
        first: old # original
        last: retained
    "};
    let document = edit::<Value>(source, |e| {
        let mut first = e.root_mut().field("first")?;
        first.set("new")?;
        first.set_scalar_style(ScalarStyle::SingleQuoted)?;
        first.replace_comment(0, " updated")?;
        first.append_comment(" description", CommentPlacement::Leading)?;
        first.remove_comment(0)
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -first: old # original
    +# description
    +first: 'new'
     last: retained
    ");
}

#[test]
fn sequential_sets_refresh_the_candidate_and_validate_only_on_commit() {
    let source = indoc! {r"
        timeout: 30
        servers: [primary]
    "};
    let mut document = edit::<Config>(source, |e| {
        e.root_mut().field("timeout")?.set("temporarily invalid")?;
        e.root_mut().field("timeout")?.set(60)?;
        e.root_mut().field("servers")?.set(["old"])?;
        e.root_mut().field("servers")?.item(0)?.set("new")
    });
    assert_eq!(document.value().timeout, 60);
    assert_eq!(document.value().servers, ["new"]);
    let before = document.to_yaml();
    assert!(
        document
            .edit(|e| e.root_mut().field("timeout")?.set(256))
            .is_err()
    );
    assert_eq!(document.source(), before);
    assert_eq!(document.value().timeout, 60);
}

#[test]
fn aliases_are_not_followed_and_definitions_keep_their_anchor() {
    let source = indoc! {r"
        definition: &value 'old'
        copy: *value
    "};
    let document = edit::<Value>(source, |e| {
        assert!(e.root_mut().field("copy")?.field("nested").is_err());
        e.root_mut().field("definition")?.set("new")
    });
    assert_eq!(
        document.value(),
        &json!({"definition": "new", "copy": "new"})
    );
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -definition: &value 'old'
    +definition: &value 'new'
     copy: *value
    ");
}

#[test]
fn navigation_errors_identify_the_selected_path() {
    let mut errors = Vec::new();
    for operation in 0..4 {
        let mut document = Editable::<Value>::parse("items: [1]").unwrap();
        let error = document
            .edit(|e| {
                match operation {
                    0 => {
                        e.root_mut().field("missing")?;
                    }
                    1 => {
                        e.root_mut().field("items")?.item(3)?;
                    }
                    2 => {
                        e.root_mut().field("items")?.item(0)?.field("wrong")?;
                    }
                    _ => {
                        e.root_mut().item(0)?;
                    }
                }
                Ok(())
            })
            .unwrap_err();
        errors.push(error.to_string());
        assert_eq!(document.source(), "items: [1]");
    }
    insta::assert_snapshot!(errors.join("\n"), @r#"
    $["missing"]: missing mapping key
    $["items"][3]: item index out of bounds
    $["items"][0]: expected a mapping at bytes 0..10
    $: expected a sequence at bytes 0..10
    "#);
}

#[test_case(0)]
#[test_case(1)]
#[test_case(2)]
fn ambiguous_string_keys_are_rejected_after_raw_insertions(operation: usize) {
    let mut document = Editable::<Value>::parse("name: old").unwrap();
    let error = document
        .edit(|e| {
            let syntax = e.syntax_mut();
            syntax.insert_entry(&syntax.root(), 1, "'name'", "other")?;
            match operation {
                0 => {
                    e.root_mut().field("name")?;
                }
                1 => {
                    e.root_mut().set_entry("name", "new")?;
                }
                _ => {
                    e.root_mut().remove_field("name")?;
                }
            }
            Ok(())
        })
        .unwrap_err();
    assert!(error.message.contains("ambiguous mapping key"));
    assert_eq!(document.source(), "name: old");
}

struct FailsSerialization;

impl Serialize for FailsSerialization {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("serialization failed"))
    }
}

#[test]
fn caught_mutation_errors_still_roll_back_the_transaction() {
    let mut errors = Vec::new();
    for operation in 0..13 {
        let source = indoc! {r"
            items: [old]
        "};
        let mut document = Editable::<Value>::parse(source).unwrap();
        let error = document
            .edit(|e| {
                e.root_mut().field("items")?.item(0)?.set("changed")?;
                let result = match operation {
                    0 => e.root_mut().set(FailsSerialization),
                    1 => e.root_mut().push("wrong kind").map(|_| ()),
                    2 => e
                        .root_mut()
                        .field("items")?
                        .insert(3, "invalid index")
                        .map(|_| ()),
                    3 => e.root_mut().field("items")?.append_comment(
                        indoc! {r"
                            bad
                            comment"},
                        CommentPlacement::Inline,
                    ),
                    4 => e
                        .root_mut()
                        .field("items")?
                        .item(0)?
                        .set_scalar_style(ScalarStyle::Plain)
                        .and_then(|()| e.root_mut().field("items")?.item(0)?.remove_comment(0)),
                    5 => e.root_mut().remove_item(0),
                    6 => e.root_mut().replace_comment(0, " absent"),
                    7 => e.root_mut().set_scalar_style(ScalarStyle::Plain),
                    8 => e
                        .root_mut()
                        .field("items")?
                        .item(0)?
                        .set_collection_style(CollectionStyle::Block),
                    9 => e
                        .root_mut()
                        .field("items")?
                        .push(FailsSerialization)
                        .map(|_| ()),
                    10 => e
                        .root_mut()
                        .set_entry("missing", FailsSerialization)
                        .map(|_| ()),
                    11 => e
                        .root_mut()
                        .field("items")?
                        .remove_field("wrong kind")
                        .map(|_| ()),
                    _ => e
                        .root_mut()
                        .field("items")?
                        .item(0)?
                        .set_entry("wrong kind", true)
                        .map(|_| ()),
                };
                let error = result.unwrap_err();
                assert_eq!(e.root_mut().set(42).unwrap_err(), error);
                Ok(())
            })
            .unwrap_err();
        errors.push(error.message);
        assert_eq!(document.source(), source);
    }
    insta::assert_snapshot!(errors.join("\n"), @r#"
    $: serialization failed
    $: expected a sequence
    $["items"]: item index out of bounds
    $["items"]: comment text contains a line break
    $["items"][0]: comment index out of bounds
    $: expected a sequence
    $: comment index out of bounds
    $: expected a scalar
    $["items"][0]: expected a collection
    $["items"]: serialization failed
    $: serialization failed
    $["items"]: expected a mapping
    $["items"][0]: expected a mapping
    "#);
}
