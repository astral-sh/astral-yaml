//! Value edits preserve YAML types: <https://yaml.org/spec/1.2.2/#103-core-schema>.

use astral_yaml_dev::edit_diff;
use astral_yaml_serde::{
    Editable,
    edit::{CollectionStyle, CommentPlacement, NodeKind, ScalarStyle},
};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct Config {
    timeout: u8,
    servers: Vec<String>,
}

#[test]
fn results_are_returned_only_after_callback_syntax_and_type_checks() {
    let source = "timeout: 30\nservers: [primary]\n";
    for operation in 0..4 {
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
}

#[test]
fn fluent_configuration_edits_keep_order_comments_and_unknown_fields() {
    let source = "timeout: 30 # seconds\nservers: [primary, retired]\nextra: 'retained'\n";
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
    insta::assert_snapshot!(edit_diff(source, config.source()));
    let pointer = config.source().as_ptr();
    let yaml = config.into_yaml();
    assert_eq!(yaml.as_ptr(), pointer);
}

#[test]
fn rust_values_never_interpret_strings_as_yaml() {
    let mut diffs = Vec::new();
    for (name, value) in [
        ("string", json!("false")),
        ("boolean", json!(false)),
        ("sequence", json!(["a", "b"])),
        ("mapping", json!({"a": 1})),
        ("null", Value::Null),
        ("YAML-looking string", json!("[a, b] # still a string")),
        ("empty string", json!("")),
        ("escaped string", json!("it's a\nnew line")),
    ] {
        let mut document = Editable::<Value>::parse("old\n").unwrap();
        document.edit(|e| e.root_mut().set(&value)).unwrap();
        assert_eq!(document.value(), &value);
        diffs.push(format!("{name}\n{}", edit_diff("old\n", document.source())));
    }
    insta::assert_snapshot!(diffs.join("\n"));
}

#[test]
fn fields_use_literal_decoded_string_keys() {
    for (source, name) in [
        ("'it''s': old", "it's"),
        ("'a.b/[0]': old", "a.b/[0]"),
        ("é: old", "é"),
        ("!!str true: old", "true"),
        ("! true: old", "true"),
        ("{true: ignored, 'true': old}", "true"),
        ("{1: ignored, '1': old}", "1"),
        ("{!!int '1': ignored, '1': old}", "1"),
        ("{[complex, key]: ignored, name: old}", "name"),
    ] {
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
        assert!(document.source().contains("new"));
    }
}

#[test]
fn optional_lookup_and_removal_distinguish_absence() {
    let source = "first: 1 # removed\nsecond: 2 # retained\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|e| {
            let mut root = e.root_mut();
            assert!(root.reborrow().get("absent")?.is_none());
            assert!(!root.remove_field("absent")?);
            assert!(root.remove_field("first")?);
            root.reborrow().get("second")?.unwrap().set(3)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()));
}

#[test]
fn insertion_returns_a_cursor_and_parents_can_be_reborrowed() {
    let source = "items: [old]\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|e| {
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
        })
        .unwrap();
    assert_eq!(
        document.value(),
        &json!({"items": ["first", {"value": "new"}]})
    );
    insta::assert_snapshot!(edit_diff(source, document.source()));
}

#[test]
fn upserts_preserve_existing_positions_and_allow_further_edits() {
    let source = "first: 'old' # keep\nlast: true\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|e| {
            e.root_mut().set_entry("first", "it's new")?;
            e.root_mut().set_entry("added", ["initial"])?;
            e.root_mut()
                .set_entry("added", ["one"])?
                .push("two")?
                .append_comment(" added", CommentPlacement::Inline)?;
            Ok(())
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()));
}

#[test]
fn styles_and_comments_can_be_changed_explicitly() {
    let source = "first: old # original\nlast: retained\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|e| {
            let mut first = e.root_mut().field("first")?;
            first.set("new")?;
            first.set_scalar_style(ScalarStyle::SingleQuoted)?;
            first.replace_comment(0, " updated")?;
            first.append_comment(" description", CommentPlacement::Leading)?;
            first.remove_comment(0)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()));
}

#[test]
fn sequential_sets_refresh_the_candidate_and_validate_only_on_commit() {
    let source = "timeout: 30\nservers: [primary]\n";
    let mut document = Editable::<Config>::parse(source).unwrap();
    document
        .edit(|e| {
            e.root_mut().field("timeout")?.set("temporarily invalid")?;
            e.root_mut().field("timeout")?.set(60)?;
            e.root_mut().field("servers")?.set(["old"])?;
            e.root_mut().field("servers")?.item(0)?.set("new")
        })
        .unwrap();
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
    let source = "definition: &value 'old'\ncopy: *value\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|e| {
            assert!(e.root_mut().field("copy")?.field("nested").is_err());
            e.root_mut().field("definition")?.set("new")
        })
        .unwrap();
    assert_eq!(
        document.value(),
        &json!({"definition": "new", "copy": "new"})
    );
    insta::assert_snapshot!(edit_diff(source, document.source()));
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
    insta::assert_snapshot!(errors.join("\n"));
}

#[test]
fn ambiguous_string_keys_are_rejected_after_raw_insertions() {
    for operation in 0..3 {
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
        let source = "items: [old]\n";
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
                    3 => e
                        .root_mut()
                        .field("items")?
                        .append_comment("bad\ncomment", CommentPlacement::Inline),
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
    insta::assert_snapshot!(errors.join("\n"));
}
