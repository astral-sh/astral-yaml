use astral_yaml::Budget;
use astral_yaml_serde::{
    DeserializationPolicy, Tagged, Value, from_str, from_str_with_policy, to_string,
};
use serde::{Deserialize, Serialize};
use test_case::test_case;

fn roundtrip(source: &str) -> Value {
    let value = from_str::<Value>(source).unwrap();
    let serialized = to_string(&value).unwrap();
    assert_eq!(from_str::<Value>(&serialized).unwrap(), value);
    value
}

fn tagged(tag: &str, value: Value) -> Value {
    Value::Tagged(Box::new(Tagged {
        tag: tag.to_owned(),
        value,
    }))
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test_case("! 42", &Value::String("42".into()); "non specific tag")]
#[test_case("!!int '0x2a'", &Value::Unsigned(42); "explicit integer")]
#[test_case("!!str 0x2a", &Value::String("0x2a".into()); "explicit string")]
#[test_case("!!float 42", &Value::Float(42.0); "explicit float")]
#[test_case("-170141183460469231731687303715884105728", &Value::Integer(i128::MIN); "minimum signed integer")]
#[test_case("340282366920938463463374607431768211455", &Value::Unsigned(u128::MAX); "maximum unsigned integer")]
#[test_case(".inf", &Value::Float(f64::INFINITY); "positive infinity")]
#[test_case("-.inf", &Value::Float(f64::NEG_INFINITY); "negative infinity")]
#[test_case(".nan", &Value::Float(f64::NAN); "not a number")]
fn scalars_and_explicit_core_tags(source: &str, expected: &Value) {
    assert_eq!(&roundtrip(source), expected);
}

#[test_case("-170141183460469231731687303715884105729", "integer out of range"; "signed overflow")]
#[test_case("340282366920938463463374607431768211456", "integer out of range"; "unsigned overflow")]
#[test_case("!!int true", "scalar does not match its tag"; "invalid core scalar value")]
#[test_case("!!seq {}", "tag does not match requested type"; "invalid core collection")]
fn invalid_values(source: &str, message: &str) {
    assert_eq!(from_str::<Value>(source).unwrap_err().message, message);
}

// https://yaml.org/spec/1.2.2/#3211-nodes
#[test]
fn mappings_accept_collection_keys_and_keep_entry_order() {
    let value = roundtrip("{[0, !!str 4]: true, {right: 2, left: 1}: false}");
    insta::assert_snapshot!(to_string(&value).unwrap(), @r#"{? [0, "4"]: true, ? {? "right": 2, ? "left": 1}: false}"#);
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test_case("{{a: 1, b: 2}: .nan}", "{{b: 2, a: 1}: .nan}", true; "mapping order in keys")]
#[test_case("{a: 1}", "{a: 2}", false; "different value")]
#[test_case("{a: 1}", "{}", false; "different length")]
#[test_case("[1, 2]", "[2, 1]", false; "sequence order")]
#[test_case("1", "1.0", false; "different numeric type")]
#[test_case("-0", "0", true; "signed and unsigned zero")]
#[test_case("-1", "1", false; "signed and unsigned magnitude")]
#[test_case("-0.0", "0.0", true; "float zeros")]
#[test_case(".inf", ".nan", false; "infinity and NaN")]
#[test_case("!Left true", "!Right true", false; "different application tags")]
fn equality(left: &str, right: &str, expected: bool) {
    let left = from_str::<Value>(left).unwrap();
    let right = from_str::<Value>(right).unwrap();
    assert_eq!((left == right, right == left), (expected, expected));
}

#[test]
fn equality_handles_nan_payloads_and_repeated_entries() {
    assert_eq!(Value::Float(-f64::NAN), Value::Float(f64::NAN));

    let entry = (Value::Null, Value::Bool(true));
    assert_ne!(
        Value::Mapping(vec![entry.clone(), entry.clone()]),
        Value::Mapping(vec![entry, (Value::Null, Value::Bool(false))]),
    );
}

// https://yaml.org/spec/1.2.2/#691-node-tags
#[test_case("!Port 080", &tagged("!Port", Value::String("080".into())); "plain scalar")]
#[test_case("!Any", &tagged("!Any", Value::String(String::new())); "empty scalar")]
#[test_case("!Any \"a\\tb\"", &tagged("!Any", Value::String("a\tb".into())); "escaped scalar")]
#[test_case("!Port |\n  080\n", &tagged("!Port", Value::String("080\n".into())); "block scalar")]
fn application_tagged_scalars_preserve_decoded_text(source: &str, expected: &Value) {
    assert_eq!(&roundtrip(source), expected);
}

#[test]
fn application_tags_survive_nested_collections_and_expand_handles() {
    let source = "%TAG !app! tag:example.org,2026:\n--- !app!config {ports: !Ports [!Port 080]}";
    let value = roundtrip(source);
    insta::assert_snapshot!(to_string(&value).unwrap(), @r#"!<tag:example.org,2026:config> {? "ports": !Ports [!Port "080"]}"#);
}

#[test]
fn application_tags_and_text_distinguish_mapping_keys() {
    let value = roundtrip("{!Port 080: one, !Port 80: two, !Other 80: three}");
    insta::assert_snapshot!(to_string(&value).unwrap(), @r#"{? !Port "080": "one", ? !Port "80": "two", ? !Other "80": "three"}"#);
    let error = from_str::<Value>("{!Port 080: one, !Port '080': two}").unwrap_err();
    insta::assert_snapshot!(error, @"$.key_at(1): duplicate mapping key at bytes 17..28");
}

#[test_case(&tagged("tag:yaml.org,2002:str", Value::String("text".into())), "expected an application YAML tag"; "explicit core tag")]
#[test_case(&tagged("!", Value::String("text".into())), "expected an application YAML tag"; "non specific tag")]
#[test_case(&tagged("!Port", Value::Unsigned(80)), "a tagged YAML value requires a string, sequence, or mapping payload"; "integer payload")]
#[test_case(&tagged("!Outer", tagged("!Inner", Value::Sequence(vec![]))), "a tagged YAML value requires a string, sequence, or mapping payload"; "multiple tags on the same node")]
fn serializing_invalid_tags(value: &Value, message: &str) {
    assert_eq!(to_string(value).unwrap_err().message, message);
}

#[test]
fn typed_serde_fields_can_hold_dynamic_values() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Config {
        name: String,
        setting: Option<Value>,
        #[serde(default)]
        fallback: Value,
    }

    let value = from_str::<Config>("name: service\nsetting: !Port 080\n").unwrap();
    let encoded = to_string(&value).unwrap();
    insta::assert_snapshot!(&encoded, @r#"{? "name": "service", ? "setting": !Port "080", ? "fallback": null}"#);
    assert_eq!(from_str::<Config>(&encoded).unwrap(), value);
}

#[test]
fn ordinary_values_work_with_other_self_describing_deserializers() {
    let value = serde_json::from_value::<Value>(serde_json::json!({
        "extras": ["text", null, true, -42, 80, 1.5]
    }))
    .unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        serde_json::json!({ "extras": ["text", null, true, -42, 80, 1.5] })
    );
}

#[test]
fn unsupported_serde_types_report_that_a_yaml_value_is_expected() {
    let bytes = serde::de::value::BytesDeserializer::<astral_yaml_serde::Error>::new(b"bytes");
    insta::assert_snapshot!(Value::deserialize(bytes).unwrap_err(), @"invalid type: byte array, expected a YAML value");
}

#[test]
fn untagged_enums_can_buffer_core_values() {
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Config {
        Dynamic { data: Value },
    }

    let Config::Dynamic { data } = from_str("data: [1, !!str 2]").unwrap();
    assert_eq!(
        data,
        Value::Sequence(vec![Value::Unsigned(1), Value::String("2".into())])
    );

    let error = from_str::<Config>("data: [!Port 080]").unwrap_err();
    insta::assert_snapshot!(error, @r#"$["data"][0]: non-core YAML tags require Tagged<T> or with::yaml_tag at bytes 7..16"#);
}

// https://yaml.org/spec/1.2.2/#71-alias-nodes
#[test_case("[&port !Port 080, *port]"; "tagged scalar")]
#[test_case("[&ports !Ports [80], *ports]"; "tagged sequence")]
fn aliases_expand_application_tags(source: &str) {
    let value = roundtrip(source);
    let items = value.as_sequence().unwrap();
    assert!(matches!(&items[0], Value::Tagged(_)));
    assert_eq!(items[0], items[1]);
}

#[test]
fn alias_expansion_obeys_the_work_budget() {
    let budget = Budget::new(2);
    let policy = DeserializationPolicy {
        work_budget: Some(&budget),
    };
    let error = from_str_with_policy::<Value>("[&port !Port 080, *port]", policy).unwrap_err();
    insta::assert_snapshot!(error, @"$[1]: deserialization resource limit exceeded at bytes 18..23");
}

#[test]
fn cyclic_aliases_are_bounded() {
    let error = from_str::<Value>("&loop !Loop [*loop]").unwrap_err();
    assert_eq!(error.message, "deserialization resource limit exceeded");
}

#[test]
fn accessors_check_the_value_type_and_integer_range() {
    let mut null = Value::default();
    assert_eq!(null, Value::Null);
    assert_eq!(null.as_bool(), None);
    assert_eq!(null.as_str(), None);
    assert_eq!(null.as_i128(), None);
    assert_eq!(null.as_u128(), None);
    assert_eq!(null.as_f64(), None);
    assert_eq!(null.as_sequence(), None);
    assert_eq!(null.as_sequence_mut(), None);
    assert_eq!(null.as_mapping(), None);
    assert_eq!(null.as_mapping_mut(), None);

    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::String("value".into()).as_str(), Some("value"));
    assert_eq!(Value::Integer(-42).as_i128(), Some(-42));
    assert_eq!(Value::Integer(-42).as_u128(), None);
    assert_eq!(Value::Integer(42).as_u128(), Some(42));
    assert_eq!(Value::Unsigned(42).as_i128(), Some(42));
    assert_eq!(Value::Unsigned(u128::MAX).as_i128(), None);
    assert_eq!(Value::Unsigned(u128::MAX).as_u128(), Some(u128::MAX));
    assert_eq!(Value::Unsigned(42).as_f64(), None);
    assert_eq!(Value::Float(1.25).as_f64(), Some(1.25));

    let mut value = Value::Mapping(vec![(Value::Null, Value::Sequence(vec![]))]);
    let (_, list) = &mut value.as_mapping_mut().unwrap()[0];
    list.as_sequence_mut().unwrap().push(Value::Integer(42));
    let (_, list) = &value.as_mapping().unwrap()[0];
    assert_eq!(list.as_sequence(), Some([Value::Unsigned(42)].as_slice()));
}

#[cfg(feature = "editable")]
#[test]
fn editable_cursor_reads_and_writes_tagged_values() {
    let mut doc =
        astral_yaml_serde::Editable::<Value>::parse("ports: !Ports [80] # keep\n").unwrap();
    doc.edit_at(["ports"], |mut cursor| {
        let mut value = cursor.read::<Value>()?;
        let Value::Tagged(tagged) = &mut value else {
            panic!("expected a tagged value");
        };
        tagged
            .value
            .as_sequence_mut()
            .unwrap()
            .push(Value::Unsigned(443));
        cursor.set(&value)?;
        assert_eq!(cursor.read::<Value>()?, value);
        Ok(())
    })
    .unwrap();
    insta::assert_snapshot!(doc.source(), @"ports: !<!Ports> [80, 443] # keep");
}
