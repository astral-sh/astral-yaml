use test_case::test_case;

use std::collections::BTreeMap;
use std::fmt::Debug;

use astral_yaml_serde::{from_str, to_string};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let yaml = to_string(value).unwrap();
    assert_eq!(&from_str::<T>(&yaml).unwrap(), value, "{yaml:?}");
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

    assert_eq!(chunk, "");
}

// YAML leaves precision to the implementation. Both native widths preserve the
// bits of non-NaN values, including the sign of zero.
// https://yaml.org/spec/1.2.2/#10214-floating-point
#[test_case(0; "positive")]
#[test_case(1 << 63; "negative")]
fn f64_exponents_and_mantissas_roundtrip(sign: u64) {
    for exponent in 0..=0x7ff {
        for mantissa in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1] {
            let value = f64::from_bits(sign | (exponent << 52) | mantissa);
            let yaml = to_string(&value).unwrap();
            let decoded = from_str::<f64>(&yaml).unwrap();

            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded.to_bits(), value.to_bits(), "{yaml:?}");
            }
        }
    }
}

#[test_case(0; "positive")]
#[test_case(1 << 31; "negative")]
fn f32_exponents_and_mantissas_roundtrip(sign: u32) {
    for exponent in 0..=0xff {
        for mantissa in [0, 1, (1 << 22) - 1, 1 << 22, (1 << 23) - 1] {
            let value = f32::from_bits(sign | (exponent << 23) | mantissa);
            let yaml = to_string(&value).unwrap();
            let decoded = from_str::<f32>(&yaml).unwrap();

            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded.to_bits(), value.to_bits(), "{yaml:?}");
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn complex_and_long_keys_roundtrip() {
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
