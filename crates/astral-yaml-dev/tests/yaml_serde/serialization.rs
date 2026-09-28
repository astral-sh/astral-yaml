//! Read emitter output with a separately implemented YAML parser.

use std::{collections::BTreeMap, fmt::Debug};

use serde::{Serialize, de::DeserializeOwned};

fn roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let yaml = astral_yaml_serde::to_string(value).unwrap();
    assert_eq!(
        &yaml_serde::from_str::<T>(&yaml).unwrap(),
        value,
        "{yaml:?}"
    );
    assert_eq!(
        &astral_yaml_serde::from_str::<T>(&yaml).unwrap(),
        value,
        "{yaml:?}"
    );
}

// https://yaml.org/spec/1.2.2/#51-character-set
// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test]
fn every_unicode_scalar_survives_serialization() {
    let mut chunk = String::new();

    for codepoint in 0..=0x0010_ffff {
        if let Some(character) = char::from_u32(codepoint) {
            chunk.push(character);
        }

        if codepoint % 1024 == 1023 {
            roundtrip(&chunk);
            chunk.clear();
        }
    }

    assert!(chunk.is_empty());
}

// https://yaml.org/spec/1.2.2/#10214-floating-point
#[test]
fn floating_point_exponents_and_mantissas_roundtrip() {
    for sign in [0, 1_u64 << 63] {
        for exponent in 0..=0x7ff {
            for mantissa in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1] {
                let value = f64::from_bits(sign | (exponent << 52) | mantissa);
                let yaml = astral_yaml_serde::to_string(&value).unwrap();

                for decoded in [
                    yaml_serde::from_str::<f64>(&yaml).unwrap(),
                    astral_yaml_serde::from_str::<f64>(&yaml).unwrap(),
                ] {
                    if value.is_nan() {
                        assert!(decoded.is_nan());
                    } else {
                        assert_eq!(decoded.to_bits(), value.to_bits(), "{yaml:?}");
                    }
                }
            }
        }
    }

    for sign in [0, 1_u32 << 31] {
        for exponent in 0..=0xff {
            for mantissa in [0, 1, (1 << 22) - 1, 1 << 22, (1 << 23) - 1] {
                let value = f32::from_bits(sign | (exponent << 23) | mantissa);
                let yaml = astral_yaml_serde::to_string(&value).unwrap();

                for decoded in [
                    yaml_serde::from_str::<f32>(&yaml).unwrap(),
                    astral_yaml_serde::from_str::<f32>(&yaml).unwrap(),
                ] {
                    if value.is_nan() {
                        assert!(decoded.is_nan());
                    } else {
                        assert_eq!(decoded.to_bits(), value.to_bits(), "{yaml:?}");
                    }
                }
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn complex_and_long_keys_remain_valid_yaml() {
    roundtrip(&BTreeMap::from([
        (
            vec!["one".to_owned(), "two".to_owned()],
            vec![Some(1), None],
        ),
        (vec![], vec![]),
    ]));
    roundtrip(&BTreeMap::from([
        (BTreeMap::from([("nested".to_owned(), 1)]), true),
        (BTreeMap::new(), false),
    ]));
    roundtrip(&BTreeMap::from([
        ("é".repeat(1025), "multibyte".to_owned()),
        ("a".repeat(4096), "long".to_owned()),
    ]));
    roundtrip(&[i128::MIN, i128::MAX, 0]);
    roundtrip(&[u128::MIN, u128::MAX]);
}
