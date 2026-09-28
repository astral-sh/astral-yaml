use std::{collections::BTreeMap, fmt::Debug, fs, path::Path};

use astral_yaml_dev::saphyr::{astral_documents, options};
use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Value, json};

fn agree<'de, T: Deserialize<'de> + PartialEq + Debug>(source: &'de str, expected: &T) {
    let ours = astral_yaml_serde::from_str::<T>(source);
    let theirs = serde_saphyr::from_str_with_options::<T>(source, options());

    assert_eq!(
        ours.as_ref().ok(),
        Some(expected),
        "astral: {source:?}: {ours:?}"
    );
    assert_eq!(
        theirs.as_ref().ok(),
        Some(expected),
        "saphyr: {source:?}: {theirs:?}"
    );
}

#[test]
fn null_like_strings() {
    for (source, expected) in [
        ("!!str null", "null"),
        ("!!str Null", "Null"),
        ("!!str NULL", "NULL"),
        ("!!str ~", "~"),
        ("!!str", ""),
        ("! null", "null"),
        ("! ~", "~"),
        ("!", ""),
        ("!<tag:yaml.org,2002:str> null", "null"),
        ("'null'", "null"),
        ("\"~\"", "~"),
        ("''", ""),
        ("\"\"", ""),
        ("|\n", ""),
        ("|-\n", ""),
        ("|+\n", ""),
        (">\n", ""),
        (">-\n", ""),
        (">+\n", ""),
        ("|-\n  null\n", "null"),
        (">-\n  null\n", "null"),
    ] {
        agree(source, &Some(expected.to_owned()));
        agree(source, &Value::String(expected.to_owned()));
    }

    for source in [
        "null",
        "Null",
        "NULL",
        "~",
        "---\n",
        "!!null",
        "!!null null",
        "!!null ~",
    ] {
        agree::<Option<String>>(source, &None);
        agree(source, &Value::Null);
    }
}

#[test]
fn scalar_tags_and_aliases() {
    for (source, expected) in [
        ("!!int '42'", json!(42)),
        ("!!int -42", json!(-42)),
        ("!!int 0x2a", json!(42)),
        ("!!int 0o52", json!(42)),
        ("!!int |-\n  42", json!(42)),
        ("!!float '42'", json!(42.0)),
        ("!!float >-\n  4.25", json!(4.25)),
        ("!!bool 'false'", json!(false)),
        ("!!bool |-\n  true", json!(true)),
        ("%TAG !n! tag:yaml.org,2002:\n--- !n!int '42'", json!(42)),
        ("[&n !!int '42', *n]", json!([42, 42])),
        ("{a: &n ! null, b: *n}", json!({"a":"null", "b":"null"})),
    ] {
        agree(source, &expected);
    }

    for tag in ["!!int", "!!float", "!!bool"] {
        for value in ["", "null", "~", "wrong"] {
            let source = format!("{tag} {value}");
            assert!(
                astral_yaml_serde::from_str::<Option<Value>>(&source).is_err(),
                "{source}"
            );
            assert!(
                serde_saphyr::from_str_with_options::<Option<Value>>(&source, options()).is_err(),
                "{source}"
            );
        }
    }

    for source in ["!!float -0", "!!float '-0'"] {
        let ours = astral_yaml_serde::from_str::<f64>(source).unwrap();
        let theirs = serde_saphyr::from_str::<f64>(source).unwrap();
        assert_eq!(ours.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(theirs.to_bits(), ours.to_bits());
    }
}

#[test]
fn root_block_scalars_and_document_boundaries() {
    for style in ["|", ">"] {
        for (chomp, expected) in [("-", "text"), ("", "text\n"), ("+", "text\n\n")] {
            for boundary in ["--- kept\n", "...\n--- kept\n", "---\tkept\n"] {
                let source = format!("--- {style}{chomp}\ntext\n\n{boundary}");
                let expected = vec![expected.to_owned(), "kept".to_owned()];
                assert_eq!(
                    astral_documents::<String>(&source).unwrap(),
                    expected,
                    "{source}"
                );
                assert_eq!(
                    serde_saphyr::from_multiple::<String>(&source).unwrap(),
                    expected,
                    "{source}"
                );
                assert!(astral_yaml_serde::from_str::<String>(&source).is_err());
                assert!(serde_saphyr::from_str::<String>(&source).is_err());
            }
        }
    }
}

#[test]
fn json_surrogate_pairs() {
    for scalar in [0x10000_u32, 0x1d11e, 0x1f30d, 0x0010_ffff] {
        let high = 0xd800 + ((scalar - 0x10000) >> 10);
        let low = 0xdc00 + ((scalar - 0x10000) & 0x3ff);
        let source = format!(r#""\u{high:04X}\u{low:04X}""#);
        let expected = char::from_u32(scalar).unwrap().to_string();
        assert_eq!(serde_json::from_str::<String>(&source).unwrap(), expected);
        agree(&source, &expected);
    }

    for source in [
        r#""\uD800""#,
        r#""\uDC00""#,
        r#""\uDC00\uD800""#,
        r#""\uD800\uD800""#,
    ] {
        assert!(astral_yaml_serde::from_str::<String>(source).is_err());
        assert!(serde_saphyr::from_str::<String>(source).is_err());
    }
}

#[test]
fn complex_keys_and_duplicate_identity() {
    for source in [
        "{0xB: a, 11: b}",
        "{0o13: a, 11: b}",
        "{-0: a, +0: b}",
        "{[0xB, -0]: a, [11, +0]: b}",
        "{{0xB: 0o13}: a, {11: 11}: b}",
        "{&key 0xB: a, *key : b}",
        "{a: 1, a: 2}",
    ] {
        assert!(
            astral_yaml_serde::from_str::<IgnoredAny>(source).is_err(),
            "{source}"
        );
        assert!(
            serde_saphyr::from_str_with_options::<IgnoredAny>(source, options()).is_err(),
            "{source}"
        );
    }

    // Different spellings of the same null key must not bypass uniqueness.
    let source = "{null: a, ~: b}";
    assert!(astral_yaml_serde::from_str::<IgnoredAny>(source).is_err());
    assert!(serde_saphyr::from_str_with_options::<IgnoredAny>(source, options()).is_ok());

    agree(
        "{9007199254740992: first, 9007199254740993: second}",
        &BTreeMap::from([
            (9_007_199_254_740_992_u64, "first".to_owned()),
            (9_007_199_254_740_993, "second".to_owned()),
        ]),
    );
    agree(
        "{[1, 2]: first, []: second}",
        &BTreeMap::from([
            (vec![1_i32, 2], "first".to_owned()),
            (vec![], "second".to_owned()),
        ]),
    );
    agree(
        "{{'null': 1}: 2}",
        &BTreeMap::from([(BTreeMap::from([("null".to_owned(), 1_i32)]), 2_i32)]),
    );
    agree(
        ": value\n",
        &BTreeMap::from([(None::<String>, "value".to_owned())]),
    );
}

#[test]
fn real_world_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/real-world");
    let mut accepted = 0;

    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "yaml") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        if path.file_name().unwrap() == "sentry-compose.yaml" {
            assert!(astral_yaml::parse(&source).is_err());
            assert!(
                serde_saphyr::granit_parser::Parser::new_from_str(&source)
                    .collect::<Result<Vec<_>, _>>()
                    .is_ok()
            );
            let result =
                serde_saphyr::from_multiple_with_options::<yaml_serde::Value>(&source, options());
            assert!(matches!(
                result.unwrap_err().without_snippet(),
                serde_saphyr::Error::Budget { .. }
            ));
            continue;
        }

        let ours = astral_documents::<yaml_serde::Value>(&source).unwrap();
        let theirs =
            serde_saphyr::from_multiple_with_options::<yaml_serde::Value>(&source, options())
                .unwrap();
        assert_eq!(ours, theirs, "{}", path.display());
        accepted += 1;
    }

    assert_eq!(accepted, 11);
}

#[test]
fn numeric_range_and_non_finite_floats() {
    for source in ["1e999", "-1e999", "!!float '1e999'"] {
        assert!(
            astral_yaml_serde::from_str::<f64>(source).is_err(),
            "{source}"
        );
        assert!(serde_saphyr::from_str::<f64>(source).is_err(), "{source}");
    }

    for source in [
        "3.5e38",
        "-3.5e38",
        "340282366920938463463374607431768211455",
    ] {
        assert!(
            astral_yaml_serde::from_str::<f32>(source).is_err(),
            "{source}"
        );
        assert!(serde_saphyr::from_str::<f32>(source).is_err(), "{source}");
    }

    for source in [".inf", "-.inf", ".nan"] {
        let ours = astral_yaml_serde::from_str::<f64>(source).unwrap();
        let theirs = serde_saphyr::from_str::<f64>(source).unwrap();
        assert!(ours.to_bits() == theirs.to_bits() || (ours.is_nan() && theirs.is_nan()));
        assert!(astral_yaml_serde::from_str::<yaml_serde::Value>(source).is_ok());
        assert!(serde_saphyr::from_str::<yaml_serde::Value>(source).is_err());
    }
}

#[test]
fn schema_and_api_differences() {
    for (source, expected) in [("yes", true), ("on", true), ("no", false), ("off", false)] {
        assert!(astral_yaml_serde::from_str::<bool>(source).is_err());
        assert_eq!(serde_saphyr::from_str::<bool>(source).unwrap(), expected);
        assert!(serde_saphyr::from_str_with_options::<bool>(source, options()).is_err());
        agree(source, &Value::String(source.to_owned()));
    }

    for (source, expected) in [("0b11", 3), ("+0xB", 11), ("0XB", 11), ("1_000", 1000)] {
        assert_eq!(
            astral_yaml_serde::from_str::<Value>(source).unwrap(),
            json!(source)
        );
        assert_eq!(
            serde_saphyr::from_str_with_options::<Value>(source, options()).unwrap(),
            json!(expected)
        );
    }

    let source = "defaults: &defaults {answer: 42}\ncopy: {<<: *defaults}\n";
    let ours = astral_yaml_serde::from_str::<Value>(source).unwrap();
    assert_eq!(ours["copy"], json!({"<<": {"answer": 42}}));
    assert_eq!(
        serde_saphyr::from_str::<Value>(source).unwrap()["copy"],
        json!({"answer": 42})
    );
    agree(source, &ours);

    let source = "!application value";
    assert!(astral_yaml_serde::from_str::<String>(source).is_err());
    assert_eq!(serde_saphyr::from_str::<String>(source).unwrap(), "value");
    assert!(serde_saphyr::from_str_with_options::<String>(source, options()).is_err());

    let source = "--- null\n--- ''\n--- ~\n--- kept\n";
    assert_eq!(
        astral_documents::<Value>(source).unwrap(),
        vec![Value::Null, json!(""), Value::Null, json!("kept")]
    );
    assert_eq!(
        serde_saphyr::from_multiple::<Value>(source).unwrap(),
        vec![json!(""), json!("kept")]
    );
}

#[test]
fn recursive_aliases_are_bounded_even_when_ignored() {
    for source in ["&self [*self]", "&self {next: *self}"] {
        assert!(astral_yaml::parse(source).is_ok());
        assert!(astral_yaml_serde::from_str::<IgnoredAny>(source).is_err());
        assert!(serde_saphyr::from_str::<IgnoredAny>(source).is_err());
    }
}

#[derive(Debug, Deserialize, PartialEq)]
enum Mode {
    Ready,
    Text(String),
    Pair(u8, bool),
    Config { enabled: bool },
}

#[test]
fn typed_enums_and_optional_tags() {
    for (source, expected) in [
        ("!Ready", Mode::Ready),
        ("!Text value", Mode::Text("value".to_owned())),
        ("!Pair [3, true]", Mode::Pair(3, true)),
        ("!Config {enabled: true}", Mode::Config { enabled: true }),
        ("{Text: value}", Mode::Text("value".to_owned())),
    ] {
        assert_eq!(
            astral_yaml_serde::from_str::<Mode>(source).unwrap(),
            expected
        );
        // serde-saphyr's reject_unsupported_tags mode also rejects enum tags.
        assert_eq!(serde_saphyr::from_str::<Mode>(source).unwrap(), expected);
        assert_eq!(
            astral_yaml_serde::from_str::<Option<Mode>>(source).unwrap(),
            Some(expected)
        );
        assert!(serde_saphyr::from_str::<Option<Mode>>(source).is_ok());
    }
}

#[test]
fn local_tags_in_serde_content_buffers() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Mixed {
        Text(String),
        #[serde(untagged)]
        Number(u32),
    }

    // Serde buffers this enum through deserialize_any. Its Content visitor
    // rejects enum input; serde-saphyr adapts local tags into maps instead.
    let source = "[!Text value, 42]";
    assert_eq!(
        astral_yaml_serde::from_str::<Vec<Mixed>>(source)
            .unwrap_err()
            .message,
        "untagged and internally tagged enums do not support enum input"
    );
    assert_eq!(
        serde_saphyr::from_str::<Vec<Mixed>>(source).unwrap(),
        vec![Mixed::Text("value".to_owned()), Mixed::Number(42)]
    );
    agree(
        "[{Text: value}, 42]",
        &vec![Mixed::Text("value".to_owned()), Mixed::Number(42)],
    );
}

#[test]
fn non_utf8_input_encodings() {
    use astral_yaml::{Encoding, Source};

    let mut differences = Vec::new();

    for encoding in [
        Encoding::Utf8,
        Encoding::Utf16Be,
        Encoding::Utf16Le,
        Encoding::Utf32Be,
        Encoding::Utf32Le,
    ] {
        for bom in [false, true] {
            let text = format!("{}message: 'é 🌍'\n", if bom { "\u{feff}" } else { "" });
            let bytes: Vec<u8> = match encoding {
                Encoding::Utf8 => text.as_bytes().to_vec(),
                Encoding::Utf16Be => text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
                Encoding::Utf16Le => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
                Encoding::Utf32Be => text
                    .chars()
                    .flat_map(|c| u32::from(c).to_be_bytes())
                    .collect(),
                Encoding::Utf32Le => text
                    .chars()
                    .flat_map(|c| u32::from(c).to_le_bytes())
                    .collect(),
            };
            let source = Source::from_bytes(&bytes).unwrap();
            let stream = source.parse().unwrap();
            let expected = json!({"message": "é 🌍"});
            assert_eq!(
                astral_yaml_serde::from_stream::<Value>(&stream).unwrap(),
                expected
            );

            match serde_saphyr::from_reader::<_, Value>(bytes.as_slice()) {
                Ok(value) => assert_eq!(value, expected),
                Err(_) => differences.push((encoding, bom)),
            }
        }
    }

    assert_eq!(
        differences,
        [
            (Encoding::Utf16Be, false),
            (Encoding::Utf16Le, false),
            (Encoding::Utf32Be, false),
            (Encoding::Utf32Be, true),
            (Encoding::Utf32Le, false),
            (Encoding::Utf32Le, true),
        ]
    );
}
