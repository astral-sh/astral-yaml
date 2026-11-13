//! Value edits preserve YAML types: <https://yaml.org/spec/1.2.2/#103-core-schema>.

use test_case::test_case;

use super::support::edit;
use astral_yaml_dev::{edit_diff, indoc};
use astral_yaml_serde::edit::{
    CollectionStyle, CommentPlacement, Cursor, NodeKind, PathSegment, ScalarStyle, SequenceMut,
    edit_path,
};
use astral_yaml_serde::{Editable, EditableYaml, Error};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[test_case("!custom name: old\n", "name"; "plain tagged key")]
#[test_case("!custom 'name': old\n", "name"; "quoted tagged key")]
#[test_case("!custom true: old\n", "true"; "tagged boolean")]
#[test_case("!custom 'true': old\n", "true"; "quoted tagged boolean")]
#[test_case("!<tag:example.org,2026:key> name: old\n", "name"; "global tagged key")]
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

#[test_case(&json!("false"), r#""false""#; "string")]
#[test_case(&json!(false), "false"; "boolean")]
#[test_case(&json!(["a", "b"]), r#"["a", "b"]"#; "sequence")]
#[test_case(&json!({"a": 1}), r#"{"a": 1}"#; "mapping")]
#[test_case(&Value::Null, "null"; "null")]
#[test_case(&json!("[a, b] # still a string"), r#""[a, b] # still a string""#; "YAML-looking string")]
#[test_case(&json!(""), r#""""#; "empty string")]
#[test_case(&json!("it's a\nnew line"), "it's a\n\n  new line"; "multiline string")]
fn rust_values_never_interpret_strings_as_yaml(value: &Value, yaml: &str) {
    // Quoting can preserve a string whose plain form would resolve to a different core tag.
    // https://yaml.org/spec/1.2.2/#1032-tag-resolution
    let document = edit::<Value>("old\n", |mut root| root.set(value));
    assert_eq!(document.value(), value);
    assert_eq!(document.source(), format!("{yaml}\n"));
}

#[test_case("'it''s': old", "it's"; "escaped quote")]
#[test_case("'a.b/[0]': old", "a.b/[0]"; "path punctuation")]
#[test_case("\u{feff}é: old", "é"; "unicode")]
#[test_case("!!str true: old", "true"; "core string tag")]
#[test_case("! true: old", "true"; "non specific tag")]
#[test_case("{true: ignored, 'true': old}", "true"; "boolean spelling")]
#[test_case("{1: ignored, '1': old}", "1"; "integer spelling")]
#[test_case("{!!int '1': ignored, '1': old}", "1"; "explicit integer spelling")]
#[test_case("{anchor: &a 'it''s', ? *a : old}", "it's"; "alias of quoted string")]
#[test_case("{anchor: &a !!str true, ? *a : old}", "true"; "alias of tagged string")]
#[test_case("{anchor: &a true, ? *a : ignored, 'true': old}", "true"; "alias of boolean")]
#[test_case("{[complex, key]: ignored, name: old}", "name"; "complex sibling key")]
fn fields_use_literal_decoded_string_keys(source: &str, name: &str) {
    // Key equality uses the resolved tag and content, including when an alias supplies the key.
    // https://yaml.org/spec/1.2.2/#3213-node-comparison
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let document = edit::<IgnoredAny>(source, |root| {
        let mut field = root.field(name)?;
        assert!(matches!(field.node()?.kind(), NodeKind::Scalar { value, .. } if value == "old"));
        field.set("new")
    });
    assert_eq!(document.source(), source.replace("old", "new"));
}

// An alias refers to the anchored node, including when used as a key.
// https://yaml.org/spec/1.2.2/#71-alias-nodes
#[test_case(|document| document.set_at(["map", "foo"], 5); "path")]
#[test_case(|document| document.edit(|root| {
    root.field("map")?.into_mapping()?.set("foo", 5)?;
    Ok(())
}); "mapping set")]
fn updating_an_alias_key_preserves_its_spelling(
    update: fn(&mut EditableYaml) -> Result<(), Error>,
) {
    let mut document = EditableYaml::parse("anchor: &a foo\nmap: {? *a : 2}\n").unwrap();
    update(&mut document).unwrap();
    assert_eq!(document.source(), "anchor: &a foo\nmap: {? *a : 5}\n");
}

#[test]
fn named_removal_of_an_alias_key_keeps_its_target() {
    let document = edit::<IgnoredAny>("anchor: &a foo\nmap: {? *a : 2, keep: 3}\n", |root| {
        assert!(root.field("map")?.into_mapping()?.remove("foo")?);
        Ok(())
    });
    insta::assert_snapshot!(document.source(), @"
    anchor: &a foo
    map: {keep: 3}
    ");
}

#[test]
fn alias_key_lookup_uses_the_current_draft_target() {
    // A repeated anchor name binds an alias to the most recent preceding definition.
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let source = "first: &a foo\nmap: {? *a : 2}\nsecond: &a other\n";
    let document = edit::<IgnoredAny>(source, |mut root| {
        root.reborrow().field("first")?.set("renamed")?;
        let mut map = root.field("map")?.into_mapping()?;
        assert!(map.get("foo")?.is_none());
        assert!(map.get("other")?.is_none());
        map.field("renamed")?.set(5)
    });
    insta::assert_snapshot!(document.source(), @"
    first: &a renamed
    map: {? *a : 5}
    second: &a other
    ");
}

#[test]
fn named_edits_detect_ambiguity_after_an_alias_target_changes() {
    // An alias and a scalar that resolves to the same key collide in a mapping.
    // https://yaml.org/spec/1.2.2/#3213-node-comparison
    let source = "anchor: &a foo\nmap: {? *a : 2, other: 3}\n";
    let mut document = EditableYaml::parse(source).unwrap();
    let error = document
        .edit(|mut root| {
            root.reborrow().field("anchor")?.set("other")?;
            root.field("map")?.into_mapping()?.set("other", 5)?;
            Ok(())
        })
        .unwrap_err();
    insta::assert_snapshot!(error, @r#"$["map"]["other"]: ambiguous mapping key at bytes 0..40"#);
    assert_eq!(document.source(), source);
}

#[test]
fn a_retired_alias_key_target_does_not_block_other_named_edits() {
    let document = edit::<IgnoredAny>("anchor: &a foo\nmap: {? *a : 2, keep: 3}\n", |root| {
        let mut root = root.into_mapping()?;
        assert!(root.remove("anchor")?);
        let mut map = root.field("map")?.into_mapping()?;
        assert!(map.get("foo")?.is_none());
        map.set("keep", 5)?;
        map.remove_at(0)
    });
    insta::assert_snapshot!(document.source(), @"map: {keep: 5}");
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

#[test_case(
    &["missing".into()]
    => with |error| insta::assert_snapshot!(error, @r#"
      |
    1 | items: [1]
      | ^^^^^^^^^^ $["missing"]: missing mapping key
    "#);
    "missing field"
)]
#[test_case(
    &["items".into(), 3.into()]
    => with |error| insta::assert_snapshot!(error, @r#"
      |
    1 | items: [1]
      | ^^^^^^^^^^ $["items"][3]: item index out of bounds
    "#);
    "missing item"
)]
#[test_case(
    &["items".into(), 0.into(), "wrong".into()]
    => with |error| insta::assert_snapshot!(error, @r#"
      |
    1 | items: [1]
      | ^^^^^^^^^^ $["items"][0]["wrong"]: expected a mapping
    "#);
    "field of scalar"
)]
#[test_case(
    &[0.into()]
    => with |error| insta::assert_snapshot!(error, @"
      |
    1 | items: [1]
      | ^^^^^^^^^^ $[0]: expected a sequence
    ");
    "item of mapping"
)]
fn navigation_errors_identify_the_selected_path(path: &[PathSegment<'_>]) -> String {
    let source = "items: [1]";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let error = document.edit_at(path, |_| Ok(())).unwrap_err();
    assert_eq!(document.source(), source);
    astral_yaml_dev::diagnostic_snapshot(source, error)
}

#[test_case(|root| root.field("name").map(|_| ()); "field lookup")]
#[test_case(|root| root.into_mapping()?.set("name", "new").map(|_| ()); "upsert")]
#[test_case(|root| root.into_mapping()?.remove("name").map(|_| ()); "remove")]
#[test_case(|root| root.into_mapping()?.entry("name").map(|_| ()); "entry")]
fn ambiguous_string_keys_are_rejected_after_renaming_a_key(
    operation: fn(Cursor<'_, '_>) -> Result<(), Error>,
) {
    // A mapping requires unique keys.
    // https://yaml.org/spec/1.2.2/#3211-nodes
    let mut document = Editable::<Value>::parse("name: old").unwrap();
    let error = document
        .edit(|mut root| {
            let mut fields = root.reborrow().into_mapping()?;
            fields.set("other", "other")?;
            fields.key_at(1)?.set("name")?;
            operation(root)
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

fn items<'a, 'doc>(root: Cursor<'a, 'doc>) -> Result<SequenceMut<'a, 'doc>, Error> {
    root.field("items")?.into_sequence()
}

const ITEMS: &str = r#"$["items"]"#;
const ITEM: &str = r#"$["items"][0]"#;
const EMPTY: &str = r#"$["empty"]"#;
const MISSING: &str = r#"$["missing"]"#;

#[test_case(|mut root| root.set(FailsSerialization),
    "$", "serialization failed"; "set value")]
#[test_case(|root| items(root)?.insert(3, "new").map(|_| ()),
    ITEMS, "item index out of bounds"; "insert out of bounds")]
#[test_case(|root| items(root)?.remove(4),
    ITEMS, "item index out of bounds"; "remove out of bounds")]
#[test_case(|root| root.into_mapping()?.remove_at(2),
    "$", "entry index out of bounds"; "remove entry out of bounds")]
#[test_case(|root| items(root)?.append_comment("bad\ncomment", CommentPlacement::Inline),
    ITEMS, "comment text contains a line break"; "append comment")]
#[test_case(|root| items(root)?.insert_comment(1, " new", CommentPlacement::Leading),
    ITEMS, "comment index out of bounds"; "insert comment out of bounds")]
#[test_case(|root| {
    let mut sequence = items(root)?;
    let mut item = sequence.item(0)?;
    item.set_scalar_style(ScalarStyle::Plain)?;
    item.remove_comment(0)
}, ITEM, "comment index out of bounds"; "remove absent comment")]
#[test_case(|mut root| root.replace_comment(0, " absent"),
    "$", "comment index out of bounds"; "replace absent comment")]
#[test_case(|mut root| root.set_scalar_style(ScalarStyle::Plain),
    "$", "expected a scalar"; "style on a mapping")]
#[test_case(|root| items(root)?.item(0)?.set_collection_style(CollectionStyle::Block),
    ITEM, "expected a collection"; "style on a scalar")]
#[test_case(|root| items(root)?.push(FailsSerialization).map(|_| ()),
    ITEMS, "serialization failed"; "serialize appended value")]
#[test_case(|root| root.field("empty")?.into_sequence()?.insert(0, FailsSerialization).map(|_| ()),
    EMPTY, "serialization failed"; "serialize inserted value")]
#[test_case(|root| root.into_mapping()?.set("missing", FailsSerialization).map(|_| ()),
    MISSING, "serialization failed"; "serialize mapping value")]
#[test_case(|root| root.into_mapping()?.entry("missing")?.or_insert(FailsSerialization).map(|_| ()),
    MISSING, "serialization failed"; "serialize vacant entry")]
fn caught_mutation_errors_still_roll_back_the_transaction(
    operation: fn(Cursor<'_, '_>) -> Result<(), Error>,
    path: &str,
    message: &str,
) {
    let source = "items: [old]\nempty: []\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let mut caught = None;
    let result = document
        .edit(|mut root| {
            root.reborrow().field("items")?.item(0)?.set("changed")?;
            let error = operation(root.reborrow()).unwrap_err();
            assert_eq!(error.path.as_deref(), Some(path));
            assert_eq!(error.message, message);
            assert_eq!(root.set(42).unwrap_err(), error);
            let changed = root.field("items")?.item(0)?;
            assert_eq!(changed.read::<&str>()?, "changed");
            caught = Some(error);
            Ok(())
        })
        .unwrap_err();
    assert_eq!(Some(result), caught);
    assert_eq!(document.source(), source);
    assert_eq!(document.value()["items"][0], "old");
}

#[test]
fn stored_mapping_views_support_sibling_edits_and_nested_defaults() {
    let source = "services: {db: {healthcheck: {retries: 3, interval: '30s'}}}\n";
    let document = edit::<Value>(source, |root| {
        let mut health = root.field("services")?.field("db")?.field("healthcheck")?;
        let missing = health.reborrow().field("missing").unwrap_err();
        let mut health = health.into_mapping()?;
        assert_eq!(health.len(), 2);
        assert!(!health.is_empty());
        assert_eq!(health.field("missing").unwrap_err(), missing);
        health.field("retries")?.set(5)?;
        health.field("interval")?.set("5s")?;
        assert!(health.entry("retries")?.is_occupied());
        health.entry("retries")?.or_insert(FailsSerialization)?;
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
            .push(json!({"value": "old"}))?
            .field("value")?
            .set("new")?;
        items.insert(1, true)?;
        items.get(0)?.unwrap().set("changed")?;
        items.item(1)?.set(false)?;
        assert_eq!(items.len(), 3);
        items.remove(0)?;
        assert_eq!(items.len(), 2);
        items.into_cursor().item(1)?.field("value")?.set("last")
    });
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    @@ -1 +1 @@
    -items: []
    +items: [false, {"value": "last"}]
    "#);
}

#[test]
fn indexed_mapping_selection_supports_arbitrary_keys() {
    // Mapping keys can be scalars, sequences, or mappings.
    // https://yaml.org/spec/1.2.2/#10111-generic-mapping
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
    @@ -1 +1,2 @@
    -{[complex, key]: old, true: other}
    +{? [changed, key]
    +  : new, false: other}
    ");
}

#[test]
fn removing_non_string_keys_by_index_preserves_other_entries() {
    // The integer key and the quoted string key have different tags.
    // https://yaml.org/spec/1.2.2/#3213-node-comparison
    let source = "1: old\n? [a, b]\n: complex\n'1': keep # existing\n";
    let document = edit::<IgnoredAny>(source, |root| {
        let mut mapping = root.into_mapping()?;
        mapping.remove_at(0)?;
        mapping.remove_at(0)?;
        assert_eq!(mapping.len(), 1);
        Ok(())
    });
    insta::assert_snapshot!(document.source(), @"'1': keep # existing");
}

#[test]
fn setting_a_value_preserves_its_anchor_style_and_comments() {
    // The alias continues to refer to the anchored value after the edit.
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let source = "value: &old 'old' # keep\ncopy: *old\n";
    let document = edit::<Value>(source, |root| root.field("value")?.set("new"));
    insta::assert_snapshot!(edit_diff(source, document.source()), @r"
    @@ -1,2 +1,2 @@
    -value: &old 'old' # keep
    +value: &old 'new' # keep
     copy: *old
    ");
    assert_eq!(document.value()["copy"], "new");
}

#[test_case("- first\n\n# gap\n\n- last\n"; "sequence")]
#[test_case("a: first\n\n# gap\n\nb: last\n"; "mapping")]
fn replacing_a_collection_with_a_scalar_keeps_its_detached_comment(source: &str) {
    use astral_yaml_serde::edit::CommentPosition;

    let document = edit::<Value>(source, |mut root| {
        let node = root.node()?;
        let following = match node.kind() {
            NodeKind::Sequence { items, .. } => items[1],
            NodeKind::Mapping { entries, .. } => entries[1].0,
            _ => unreachable!(),
        };
        assert_eq!(
            node.comments()[0].position(),
            CommentPosition::BeforeChild(following)
        );
        root.set(false)?;
        assert_eq!(
            root.node()?.comments()[0].position(),
            CommentPosition::After
        );
        Ok(())
    });
    assert_eq!(document.source(), "false\n\n# gap\n");
}

#[test]
fn replacing_a_sequence_preserves_header_and_detached_comments() {
    let source = indoc! {r"
        &items # header
        - first

        # gap

        - last
    "};
    let document = edit::<Value>(source, |mut root| root.set([1, 2]));
    insta::assert_snapshot!(document.source(), @"
    &items # header
    - 1

    # gap

    - 2
    ");
}

#[test]
fn inserting_a_leading_comment_preserves_the_inline_comment() {
    let document = edit::<String>("old # existing\n", |mut root| {
        root.insert_comment(0, " explanation", CommentPlacement::Leading)
    });
    insta::assert_snapshot!(document.source(), @"
    # explanation
    old # existing
    ");
}

#[test_case("--- # start\na: b\n"; "block mapping")]
#[test_case("--- # start\n- item\n... # end\n"; "block sequence")]
#[test_case("--- # start\n...\n"; "empty document body")]
fn inserting_an_inline_comment_before_the_start_marker_comment_rolls_back(source: &str) {
    let mut document = EditableYaml::parse(source).unwrap();
    let error = document
        .edit(|mut root| root.insert_comment(0, " new", CommentPlacement::Inline))
        .unwrap_err();
    assert_eq!(
        error.message,
        "emission changed comment ownership or placement"
    );
    assert_eq!(document.source(), source);
}

#[test]
fn inserting_an_inline_comment_between_prefix_comments_preserves_their_order() {
    // A standalone document suffix can carry a comment before the next document.
    // https://yaml.org/spec/1.2.2/#rule-l-document-suffix
    // https://yaml.org/spec/1.2.2/#92-streams
    let source = "# prefix\n\n--- # start\na: b\n";
    let document = edit::<Value>(source, |mut root| {
        root.insert_comment(1, " inserted", CommentPlacement::Inline)
    });
    insta::assert_snapshot!(document.source(), @"
    # prefix

    ... # inserted
    --- # start
    a: b
    ");
}

#[test]
fn inserting_a_detached_comment_reuses_the_following_child_boundary() {
    let source = "a: first\n\n# gap\n\nb: last\n";
    let document = edit::<Value>(source, |root| {
        root.into_mapping()?
            .insert_comment(0, " added", CommentPlacement::Detached)
    });
    insta::assert_snapshot!(document.source(), @"
    a: first

    # added


    # gap

    b: last
    ");
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
fn an_empty_edit_path_selects_the_root_and_a_missing_path_skips_the_callback() {
    let mut document = EditableYaml::parse("services.db: [[80]] # public\n").unwrap();
    document
        .edit_at(edit_path![], |root| {
            root.at(edit_path!["services.db", 0, 0])?.set(8080)
        })
        .unwrap();
    let source = document.to_yaml();
    let error = document
        .edit_at(edit_path!["services.db", 1], |_| -> Result<(), _> {
            panic!("a failed selection must not call the edit callback")
        })
        .unwrap_err();
    insta::assert_snapshot!(astral_yaml_dev::diagnostic_snapshot(&source, error), @r#"
      |
    1 | services.db: [[8080]] # public
      | ^^^^^^^^^^^^^^^^^^^^^ $["services.db"][1]: item index out of bounds
    "#);
    assert_eq!(document.source(), source);
}

#[test]
fn collection_defaults_reuse_existing_children_and_leave_scalars_alone() {
    let mut document = Editable::<Value>::parse("enabled: true # retained\n").unwrap();
    document
        .edit(|mut root| {
            assert!(root.reborrow().into_sequence().is_err());
            let mut settings = root.into_mapping()?;
            assert!(settings.ensure_sequence("enabled").is_err());
            assert!(settings.ensure_mapping("enabled").is_err());

            settings.ensure_mapping("services")?.set("web", 80)?;
            let mut services = settings.ensure_mapping("services")?;
            services.ensure_sequence("ports")?.push(443)?;
            services.ensure_sequence("ports")?.push(8443)?;
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
