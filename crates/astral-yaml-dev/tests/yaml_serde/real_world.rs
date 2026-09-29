use astral_yaml_dev::{
    Value,
    benchmarks::{astral, reference},
    fixtures::{manifest, scalar_differences},
};
use serde::Deserialize;

fn leading_zero_integer(actual: &Value, reference: &Value) -> bool {
    // Core-schema decimal integers can have leading zeroes (§10.3.2).
    // yaml_serde instead leaves these spellings as strings.
    matches!((actual, reference), (Value::Unsigned(number), Value::String(text))
        if text.len() > 1
            && text.starts_with('0')
            && text.bytes().all(|byte| byte.is_ascii_digit())
            && text.parse::<u64>() == Ok(*number))
}

#[test]
fn real_world_values_and_serialization_match_reference() {
    for fixture in manifest() {
        let source = fixture.source();

        if !fixture.valid_syntax() {
            // LibYAML accepts these under-indented continuations.
            assert!(astral_yaml::parse(&source).is_err());
            assert!(!reference(&source).is_empty());
            continue;
        }

        if let Some(expected) = fixture.serde_error {
            let error = astral_yaml_serde::from_str::<Value>(&source).unwrap_err();
            assert_eq!(error.message, expected, "{}", fixture.file);

            let values: Result<Vec<_>, _> = yaml_serde::Deserializer::from_str(&source)
                .map(Value::deserialize)
                .collect();
            if expected == "duplicate mapping key" {
                assert!(values.is_ok(), "{}", fixture.file);
            } else {
                assert!(
                    values
                        .unwrap_err()
                        .to_string()
                        .contains("invalid type: enum"),
                    "{}",
                    fixture.file
                );
            }

            continue;
        }

        let values = astral(&source);
        let references = reference(&source);
        if fixture.leading_zero_integers == 0 {
            assert_eq!(values, references, "{}", fixture.file);
        } else {
            assert_eq!(values.len(), references.len());
            let differences: usize = values
                .iter()
                .zip(&references)
                .map(|(value, reference)| {
                    scalar_differences(value, reference, leading_zero_integer)
                })
                .sum();
            assert_eq!(
                differences, fixture.leading_zero_integers,
                "{}",
                fixture.file
            );
        }

        for value in values {
            let yaml = astral_yaml_serde::to_string(&value).unwrap();
            assert_eq!(reference(&yaml), vec![value], "{}", fixture.file);
        }
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn flow_key_and_value_separation_matches_reference() {
    for source in ["{a: [b]}", "{'a':[b]}", "{[]:value}"] {
        assert_eq!(astral(source), reference(source), "{source}");
    }
}
