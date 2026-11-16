use astral_yaml::Budget;
use astral_yaml_serde::{
    DeserializationPolicy, Tagged, Value, from_str, from_str_with_policy, to_string,
};
use serde::Deserialize;
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
#[test]
fn tagged_values_keep_the_expanded_name_and_scalar_text() {
    let source = "%TAG !app! tag:example.org,2026:\n--- !app!config [&ports !Ports [&port !Port 080, *port], *ports]";
    let value = roundtrip(source);
    insta::assert_snapshot!(to_string(&value).unwrap(), @r#"!<tag:example.org,2026:config> [!Ports [!Port "080", !Port "080"], !Ports [!Port "080", !Port "080"]]"#);
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn core_values_roundtrip_in_mapping_keys_and_values() {
    let source = "{!Port 080: one, !Port 80: two, [0, ! 4]: !!int '0x2a', {right: 2, left: 1}: [-.inf, .inf, .nan]}";
    let value = roundtrip(source);
    insta::assert_snapshot!(to_string(&value).unwrap(), @r#"{? !Port "080": "one", ? !Port "80": "two", ? [0, "4"]: 42, ? {? "right": 2, ? "left": 1}: [-.inf, .inf, .nan]}"#);
}

#[test_case(&tagged("tag:yaml.org,2002:str", Value::String("text".into())), "expected an application YAML tag"; "explicit core tag")]
#[test_case(&tagged("!", Value::String("text".into())), "expected an application YAML tag"; "non specific tag")]
#[test_case(&tagged("!Port", Value::Unsigned(80)), "a tagged YAML value requires a string, sequence, or mapping payload"; "integer payload")]
#[test_case(&tagged("!Outer", tagged("!Inner", Value::Sequence(vec![]))), "a tagged YAML value requires a string, sequence, or mapping payload"; "multiple tags on the same node")]
fn serializing_invalid_tags(value: &Value, message: &str) {
    assert_eq!(to_string(value).unwrap_err().message, message);
}

#[test]
fn tagged_value_in_a_serde_option() {
    let value = from_str::<Option<Value>>("!Port 080").unwrap();
    assert_eq!(value, Some(tagged("!Port", Value::String("080".into()))));
}

#[test]
fn ordinary_values_work_with_other_self_describing_deserializers() {
    let json = serde_json::json!({ "extras": ["text", null, true, -42, 80, 1.5] });
    let value = serde_json::from_value::<Value>(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(value).unwrap(), json);
}

#[test]
fn unsupported_serde_types_report_that_a_yaml_value_is_expected() {
    let bytes = serde::de::value::BytesDeserializer::<astral_yaml_serde::Error>::new(b"bytes");
    insta::assert_snapshot!(Value::deserialize(bytes).unwrap_err(), @"invalid type: byte array, expected a YAML value");
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
