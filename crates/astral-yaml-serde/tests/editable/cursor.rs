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
        .edit(|mut root| {
            assert!(root.reborrow().get(name)?.is_none(), "{source:?}");
            assert!(root.reborrow().field(name).is_err());
            assert!(!root.into_mapping()?.remove(name)?);
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
    let result = document.edit(|root| {
        match operation {
            0 => root.field("timeout")?.set(60)?,
            1 => root.field("timeout")?.set(256)?,
            2 => {
                root.field("missing")?;
            }
            _ => root
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
        .edit(|root| {
            let mut settings = root.into_mapping()?;
            settings.field("timeout")?.set(60)?;
            let mut servers = settings.field("servers")?.into_sequence()?;
            servers.item(1)?.set("backup")?;
            servers.push("tertiary")?;
            settings.set("enabled", true)?;
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
        let document = edit::<Value>(source, |mut root| root.set(&value));
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
        .edit(|root| {
            let mut field = root.field(name)?;
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
    let document = edit::<Value>(source, |root| {
        let mut root = root.into_mapping()?;
        assert!(root.get("absent")?.is_none());
        assert!(!root.remove("absent")?);
        assert!(root.remove("first")?);
        root.get("second")?.unwrap().set(3)
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
fn insertion_returns_a_cursor_and_parents_remain_available() {
    let source = indoc! {r"
        items: [old]
    "};
    let document = edit::<Value>(source, |root| {
        let mut items = root.field("items")?.into_sequence()?;
        items
            .insert(0, "first")?
            .set_scalar_style(ScalarStyle::SingleQuoted)?;
        items.remove(1)?;
        items
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
    let document = edit::<Value>(source, |root| {
        let mut settings = root.into_mapping()?;
        settings.set("first", "it's new")?;
        settings.set("added", ["initial"])?;
        settings
            .set("added", ["one"])?
            .into_sequence()?
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
    let document = edit::<Value>(source, |root| {
        let mut first = root.field("first")?;
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
    let mut document = edit::<Config>(source, |root| {
        let mut settings = root.into_mapping()?;
        let mut timeout = settings.field("timeout")?;
        timeout.set("temporarily invalid")?;
        timeout.set(60)?;
        let mut servers = settings.field("servers")?;
        servers.set(["old"])?;
        servers.item(0)?.set("new")
    });
    assert_eq!(document.value().timeout, 60);
    assert_eq!(document.value().servers, ["new"]);
    let before = document.to_yaml();
    assert!(
        document
            .edit(|root| root.field("timeout")?.set(256))
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
    let document = edit::<Value>(source, |mut root| {
        assert!(root.reborrow().field("copy")?.field("nested").is_err());
        root.field("definition")?.set("new")
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
            .edit(|root| {
                match operation {
                    0 => {
                        root.field("missing")?;
                    }
                    1 => {
                        root.field("items")?.item(3)?;
                    }
                    2 => {
                        root.field("items")?.item(0)?.field("wrong")?;
                    }
                    _ => {
                        root.item(0)?;
                    }
                }
                Ok(())
            })
            .unwrap_err();
        errors.push(error.to_string());
        assert_eq!(document.source(), "items: [1]");
    }
    insta::assert_snapshot!(errors.join("\n"), @r#"
    $["missing"]: missing mapping key at bytes 0..10
    $["items"][3]: item index out of bounds at bytes 0..10
    $["items"][0]["wrong"]: expected a mapping at bytes 0..10
    $[0]: expected a sequence at bytes 0..10
    "#);
}

#[test_case(0)]
#[test_case(1)]
#[test_case(2)]
#[test_case(3)]
fn ambiguous_string_keys_are_rejected_after_raw_insertions(operation: usize) {
    let mut document = Editable::<Value>::parse("name: old").unwrap();
    let error = document
        .edit(|mut root| {
            let mut syntax = root.reborrow().into_syntax().into_mapping()?;
            syntax.set_yaml("other", "other")?;
            syntax.key_at(1)?.replace_yaml("'name'")?;
            match operation {
                0 => {
                    root.field("name")?;
                }
                1 => {
                    root.into_mapping()?.set("name", "new")?;
                }
                2 => {
                    root.into_mapping()?.remove("name")?;
                }
                _ => {
                    root.into_mapping()?.entry("name")?;
                }
            }
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.path.as_deref(), Some(r#"$["name"]"#));
    assert_eq!(error.message, "ambiguous mapping key");
    assert_eq!(document.source(), "name: old");
}

struct FailsSerialization;

impl Serialize for FailsSerialization {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("serialization failed"))
    }
}

#[test_case(0, "$: serialization failed")]
#[test_case(1, "$: unterminated flow sequence")]
#[test_case(2, "$[\"items\"]: item index out of bounds")]
#[test_case(3, "$[\"items\"]: comment text contains a line break")]
#[test_case(4, "$[\"items\"][0]: comment index out of bounds")]
#[test_case(5, "$[\"items\"]: item index out of bounds")]
#[test_case(6, "$: comment index out of bounds")]
#[test_case(7, "$: expected a scalar")]
#[test_case(8, "$[\"items\"][0]: expected a collection")]
#[test_case(9, "$[\"items\"]: serialization failed")]
#[test_case(10, "$[\"missing\"]: serialization failed")]
#[test_case(11, "$[\"missing\"]: serialization failed")]
#[test_case(12, "$[\"missing\"]: unterminated flow sequence")]
fn caught_mutation_errors_still_roll_back_the_transaction(operation: usize, expected: &str) {
    let source = indoc! {r"
        items: [old]
    "};
    let mut document = Editable::<Value>::parse(source).unwrap();
    let error = document
        .edit(|mut root| {
            root.reborrow().field("items")?.item(0)?.set("changed")?;
            let result = match operation {
                0 => root.set(FailsSerialization),
                1 => root.set_yaml("["),
                2 => root
                    .reborrow()
                    .field("items")?
                    .into_sequence()?
                    .insert(3, "invalid index")
                    .map(|_| ()),
                3 => root.reborrow().field("items")?.append_comment(
                    indoc! {r"
                        bad
                        comment"},
                    CommentPlacement::Inline,
                ),
                4 => root
                    .reborrow()
                    .field("items")?
                    .item(0)?
                    .set_scalar_style(ScalarStyle::Plain)
                    .and_then(|()| root.reborrow().field("items")?.item(0)?.remove_comment(0)),
                5 => root.reborrow().field("items")?.into_sequence()?.remove(4),
                6 => root.replace_comment(0, " absent"),
                7 => root.set_scalar_style(ScalarStyle::Plain),
                8 => root
                    .reborrow()
                    .field("items")?
                    .item(0)?
                    .set_collection_style(CollectionStyle::Block),
                9 => root
                    .reborrow()
                    .field("items")?
                    .into_sequence()?
                    .push(FailsSerialization)
                    .map(|_| ()),
                10 => root
                    .reborrow()
                    .into_mapping()?
                    .set("missing", FailsSerialization)
                    .map(|_| ()),
                11 => root
                    .reborrow()
                    .into_mapping()?
                    .entry("missing")?
                    .or_insert(FailsSerialization)
                    .map(|_| ()),
                _ => root
                    .reborrow()
                    .into_mapping()?
                    .set_yaml("missing", "[")
                    .map(|_| ()),
            };
            let error = result.unwrap_err();
            assert_eq!(root.set(42).unwrap_err(), error);
            Ok(())
        })
        .unwrap_err();
    let (path, message) = expected.split_once(": ").unwrap();
    assert_eq!(error.path.as_deref(), Some(path));
    assert_eq!(error.message, message);
    assert_eq!(document.source(), source);
}

#[test]
fn stored_mapping_views_support_sibling_edits_and_nested_defaults() {
    let source = "services: {db: {healthcheck: {retries: 3, interval: '30s'}}}\n";
    let document = edit::<Value>(source, |root| {
        let mut health = root
            .field("services")?
            .field("db")?
            .field("healthcheck")?
            .into_mapping()?;
        assert_eq!(health.len(), 2);
        assert!(!health.is_empty());
        assert!(health.field("missing").is_err());
        health.field("retries")?.set(5)?;
        health.field("interval")?.set("5s")?;
        assert!(health.entry("retries")?.is_occupied());
        health.entry("retries")?.or_insert(FailsSerialization)?;
        health.entry("interval")?.or_insert_yaml("[invalid")?;
        assert!(!health.entry("timeout")?.is_occupied());
        health.entry("timeout")?.or_insert("10s")?;
        health.entry("timeout")?.or_insert("unused")?;
        let mut options = health.entry("options")?.or_mapping()?;
        assert!(options.is_empty());
        options.entry("enabled")?.or_insert(true)?;
        options.into_cursor().field("enabled")?.set(false)?;
        assert_eq!(health.len(), 4);
        Ok(())
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -services: {db: {healthcheck: {retries: 3, interval: '30s'}}}
    +services: {db: {healthcheck: {retries: 5, interval: '5s', "timeout": "10s", "options": {"enabled": false}}}}
    "#);
}

#[test]
fn sequence_views_track_lengths_and_return_borrowed_children() {
    let source = "items: []\n";
    let document = edit::<Value>(source, |root| {
        let mut items = root.field("items")?.into_sequence()?;
        assert!(items.is_empty());
        assert!(items.get(0)?.is_none());
        assert!(items.item(0).is_err());
        items.push("first")?;
        items
            .push_yaml("{value: old}")?
            .field("value")?
            .set("new")?;
        items.insert_yaml(1, "true")?;
        items.get(0)?.unwrap().set("changed")?;
        items.item(1)?.set(false)?;
        assert_eq!(items.len(), 3);
        items.remove(0)?;
        assert_eq!(items.len(), 2);
        items.into_cursor().item(1)?.field("value")?.set("last")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -items: []
    +items: [false, {value: last}]
    "#);
}

#[test]
fn indexed_mapping_selection_supports_arbitrary_keys() {
    let source = "{[complex, key]: old, true: other}\n";
    let document = edit::<IgnoredAny>(source, |root| {
        let mut mapping = root.into_mapping()?;
        assert!(mapping.key_at(2).is_err());
        assert!(mapping.value_at(2).is_err());
        mapping.key_at(0)?.item(0)?.set("changed")?;
        mapping.value_at(0)?.set("new")?;
        mapping.key_at(1)?.set(false)?;
        Ok(())
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -{[complex, key]: old, true: other}
    +{? [changed, key]
    +  : new, false: other}
    ");
}

#[test_case(false)]
#[test_case(true)]
fn raw_selected_replacement_controls_style_and_anchor(replace: bool) {
    // Anchors identify serialization occurrences: https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
    let source = "value: &old 'old' # keep\n";
    let document = edit::<Value>(source, |root| {
        let mut value = root.field("value")?;
        if replace {
            value.replace_yaml("&new \"new\"")
        } else {
            value.set_yaml("&new \"new\"")
        }
    });
    if replace {
        insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -value: &old 'old' # keep
        +value: &new "new" # keep
        "#);
    } else {
        insta::assert_snapshot!(edit_diff(source, document.source()), @r"
        --- before.yaml
        +++ after.yaml
        @@ -1 +1 @@
        -value: &old 'old' # keep
        +value: &old 'new' # keep
        ");
    }
}

#[test]
fn collection_views_expose_node_comments_and_style() {
    let source = "mapping: {value: old} # map\nsequence: [old] # seq\n";
    let document = edit::<Value>(source, |mut root| {
        let mut mapping = root.reborrow().field("mapping")?.into_mapping()?;
        assert!(matches!(mapping.node()?.kind(), NodeKind::Mapping { .. }));
        mapping.replace_comment(0, " replaced")?;
        mapping.remove_comment(0)?;
        mapping.append_comment(" mapping", CommentPlacement::Leading)?;
        mapping.set_collection_style(CollectionStyle::Block)?;
        let mut sequence = root.field("sequence")?.into_sequence()?;
        assert!(matches!(sequence.node()?.kind(), NodeKind::Sequence { .. }));
        sequence.replace_comment(0, " replaced")?;
        sequence.remove_comment(0)?;
        sequence.append_comment(" sequence", CommentPlacement::Leading)?;
        sequence.set_collection_style(CollectionStyle::Block)
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,6 @@
    -mapping: {value: old} # map
    -sequence: [old] # seq
    +# mapping
    +mapping:
    +  value: old
    +# sequence
    +sequence:
    +  - old
    ");
}

#[test]
fn wrong_kind_conversions_are_recoverable_navigation_errors() {
    let source = "value: old\n";
    let document = edit::<Value>(source, |mut root| {
        assert!(root.reborrow().into_sequence().is_err());
        assert!(root.reborrow().field("value")?.into_mapping().is_err());
        assert!(root.reborrow().field("value")?.into_sequence().is_err());
        root.field("value")?.set("new")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -value: old
    +value: new
    ");
}

#[test_case(0; "replace YAML")]
#[test_case(1; "insert serialization")]
#[test_case(2; "append YAML")]
#[test_case(3; "default YAML")]
fn caught_fragment_and_serialization_failures_prevent_commit(operation: usize) {
    let source = "items: []\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let error = document
        .edit(|mut root| {
            let result = match operation {
                0 => root.replace_yaml("["),
                1 => root
                    .reborrow()
                    .field("items")?
                    .into_sequence()?
                    .insert(0, FailsSerialization)
                    .map(|_| ()),
                2 => root
                    .reborrow()
                    .field("items")?
                    .into_sequence()?
                    .push_yaml("[")
                    .map(|_| ()),
                _ => root
                    .reborrow()
                    .into_mapping()?
                    .entry("missing")?
                    .or_insert_yaml("[")
                    .map(|_| ()),
            };
            let error = result.unwrap_err();
            assert_eq!(root.set(42).unwrap_err(), error);
            Ok(())
        })
        .unwrap_err();
    assert!(error.path.is_some());
    assert_eq!(document.source(), source);
}

#[test]
fn a_failed_yaml_setter_keeps_the_first_error_and_rolls_back_prior_edits() {
    let source = "value: old\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let error = document
        .edit(|root| {
            let mut value = root.field("value")?;
            value.set_yaml("changed")?;
            let first_error = value.set_yaml("[").unwrap_err();
            assert_eq!(value.set_yaml("later").unwrap_err(), first_error);
            assert!(
                matches!(value.node()?.kind(), NodeKind::Scalar { value, .. } if value == "changed")
            );
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.path.as_deref(), Some(r#"$["value"]"#));
    assert_eq!(error.message, "unterminated flow sequence");
    assert_eq!(document.source(), source);
    assert_eq!(document.value(), &json!({"value": "old"}));
}

#[test]
fn an_empty_edit_path_selects_the_root_and_a_missing_path_skips_the_callback() {
    use astral_yaml_serde::edit::PathSegment::{Index, Key};

    let mut document = Editable::<Value>::parse("services.db: [[80]] # public\n").unwrap();
    document
        .edit_at(std::iter::empty::<&str>(), |root| {
            root.at([Key("services.db"), Index(0), Index(0)])?.set(8080)
        })
        .unwrap();
    let source = document.to_yaml();
    let error = document
        .edit_at([Key("services.db"), Index(1)], |_| -> Result<(), _> {
            panic!("a failed selection must not call the edit callback")
        })
        .unwrap_err();
    insta::assert_snapshot!(error, @r#"$["services.db"][1]: item index out of bounds at bytes 0..21"#);
    assert_eq!(document.source(), source);
}

#[test]
fn collection_defaults_reuse_existing_children_and_leave_scalars_alone() {
    let mut document = Editable::<Value>::parse("enabled: true # retained\n").unwrap();
    document
        .edit(|root| {
            let mut settings = root.into_mapping()?;
            assert!(settings.entry("enabled")?.or_sequence().is_err());
            assert!(settings.entry("enabled")?.or_mapping().is_err());

            settings.entry("services")?.or_mapping()?.set("web", 80)?;
            let mut services = settings.entry("services")?.or_mapping()?;
            services.entry("ports")?.or_sequence()?.push(443)?;
            services.entry("ports")?.or_sequence()?.push(8443)?;
            Ok(())
        })
        .unwrap();
    insta::assert_snapshot!(document.source(), @r#"
    enabled: true # retained
    "services": {"web": 80, "ports": [443, 8443]}
    "#);
}

#[test]
fn optional_lookup_distinguishes_an_absent_field_from_the_wrong_parent_kind() {
    let mut document = Editable::<Value>::parse("port: 80").unwrap();
    document
        .edit(|mut root| {
            assert!(root.reborrow().get("missing")?.is_none());
            let mut port = root.get("port")?.expect("port is present");
            let error = port.reborrow().get("nested").unwrap_err();
            assert_eq!(error.path.as_deref(), Some(r#"$["port"]["nested"]"#));
            port.set(8080)
        })
        .unwrap();
    assert_eq!(document.source(), "port: 8080");
}

#[test]
fn a_commit_error_at_a_complex_key_identifies_the_mapping_entry() {
    use std::collections::BTreeMap;

    let source = "{? [1, 2] : 80}";
    let mut document = Editable::<BTreeMap<Vec<u8>, u16>>::parse(source).unwrap();
    let error = document
        .edit(|root| root.into_mapping()?.value_at(0)?.set("http"))
        .unwrap_err();
    assert_eq!(error.path.as_deref(), Some("$.value_at(0)"));
    assert_eq!(document.source(), source);
}

#[test]
fn handling_a_core_cursor_mutation_error_still_rolls_back_the_typed_edit() {
    let source = "port: 80 # retained\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let result = document.edit(|root| {
        let mut fields = root.into_mapping()?;
        let mut port = fields.field("port")?.into_syntax();
        port.set_yaml("81")?;
        assert!(port.replace_yaml("[").is_err());
        assert_eq!(fields.len(), 1);
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(document.source(), source);
    assert_eq!(document.value()["port"], 80);
}
