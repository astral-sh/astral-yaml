use astral_yaml_dev::saphyr::options;
use serde_json::{Value, json};

#[test]
fn both_serializers_roundtrip_generated_values_through_both_readers() {
    let strings = [
        "",
        "null",
        "~",
        "true",
        "on",
        "42",
        "0x10",
        "1e3",
        "- item",
        "# comment",
        "a: b",
        " leading",
        "trailing ",
        "a\nb\n",
        "\n",
        "\n\n",
        "\r\n",
        "quotes: \"'\\",
        "\0\u{85}\u{2028}\u{fffe}\u{ffff}",
        "é🌍中",
    ];

    for number in 0_u16..1024 {
        let index = usize::from(number);
        let text = strings[index % strings.len()];
        let value = json!({
            "name": text,
            "nested": [null, number % 2 == 0, number, -i64::from(number), f64::from(number) / 8.0],
            "mapping": {text: [text, {"inner": strings[(index / strings.len()) % strings.len()]}]},
        });

        for yaml in [
            astral_yaml_serde::to_string(&value).unwrap(),
            serde_saphyr::to_string(&value).unwrap(),
        ] {
            assert_eq!(
                astral_yaml_serde::from_str::<Value>(&yaml).unwrap(),
                value,
                "{yaml:?}"
            );
            assert_eq!(
                serde_saphyr::from_str_with_options::<Value>(&yaml, options()).unwrap(),
                value,
                "{yaml:?}"
            );
        }
    }
}

#[test]
fn both_serializers_preserve_typed_numbers() {
    for value in [i128::MIN, i128::MAX, -1, 0, 1] {
        for yaml in [
            astral_yaml_serde::to_string(&value).unwrap(),
            serde_saphyr::to_string(&value).unwrap(),
        ] {
            assert_eq!(astral_yaml_serde::from_str::<i128>(&yaml).unwrap(), value);
            assert_eq!(serde_saphyr::from_str::<i128>(&yaml).unwrap(), value);
        }
    }

    for value in [u128::MAX, u128::from(u64::MAX) + 1, 0, 1] {
        for yaml in [
            astral_yaml_serde::to_string(&value).unwrap(),
            serde_saphyr::to_string(&value).unwrap(),
        ] {
            assert_eq!(astral_yaml_serde::from_str::<u128>(&yaml).unwrap(), value);
            assert_eq!(serde_saphyr::from_str::<u128>(&yaml).unwrap(), value);
        }
    }

    for value in [
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ] {
        for yaml in [
            astral_yaml_serde::to_string(&value).unwrap(),
            serde_saphyr::to_string(&value).unwrap(),
        ] {
            let ours = astral_yaml_serde::from_str::<f64>(&yaml).unwrap();
            let theirs = serde_saphyr::from_str::<f64>(&yaml).unwrap();
            assert!(ours.to_bits() == value.to_bits() || (ours.is_nan() && value.is_nan()));
            assert!(theirs.to_bits() == value.to_bits() || (theirs.is_nan() && value.is_nan()));
        }
    }
}
