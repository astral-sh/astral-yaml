use astral_yaml::Encoding;
use test_case::test_case;

mod support;

use astral_yaml_serde::{from_str, from_stream, to_string};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, fmt::Write};
use support::from_tag;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Configuration<'a> {
    name: &'a str,
    enabled: bool,
    count: u64,
    rate: f64,
    args: Vec<String>,
    optional: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Kind {
    Unit,
    Newtype(String),
    Tuple(u8, bool),
    Struct { value: i32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Wrapper(u32);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Pair(i16, String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

fn roundtrip<T>(value: &T)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + fmt::Debug,
{
    let yaml = to_string(value).unwrap();
    assert_eq!(&from_str::<T>(&yaml).unwrap(), value, "{yaml:?}");
}

#[test_case("42", "42")]
#[test_case("true", "true")]
#[test_case("1.5", "1.5")]
#[test_case("null", "null")]
#[test_case("!!str '42'", "42")]
#[test_case("!!str false", "false")]
#[test_case("! null", "null")]
#[test_case("!<tag:yaml.org,2002:str> 42", "42")]
#[test_case("%TAG !c! tag:yaml.org,2002:\n--- !c!str 42", "42")]
fn untagged_scalars_deserialize_as_text(source: &str, expected: &str) {
    assert_eq!(from_str::<String>(source).unwrap(), expected);
    assert_eq!(from_str::<&str>(source).unwrap(), expected);
}

#[test_case("[]")]
#[test_case("{}")]
#[test_case("!!int '42'")]
#[test_case("!!int nope")]
#[test_case("!!bool 2")]
#[test_case("!!seq scalar")]
#[test_case("!Local x")]
fn non_string_tags_reject_text_destinations(source: &str) {
    assert!(from_str::<String>(source).is_err(), "{source}");
    assert!(from_str::<&str>(source).is_err(), "{source}");
}

#[test]
fn derived_types() {
    let source = "name: astral\nenabled: TRUE\ncount: 0xff\nrate: 1.5\nargs: [one, 'two', \"three\\nfour\"]\noptional: null\n";
    let config: Configuration<'_> = from_str(source).unwrap();
    assert_eq!(
        config,
        Configuration {
            name: "astral",
            enabled: true,
            count: 255,
            rate: 1.5,
            args: vec!["one".into(), "two".into(), "three\nfour".into()],
            optional: None
        }
    );
    assert!(std::ptr::eq(config.name.as_ptr(), source[6..].as_ptr()));

    let yaml = to_string(&config).unwrap();
    assert_eq!(from_str::<Configuration<'_>>(&yaml).unwrap(), config);

    for value in [
        Kind::Unit,
        Kind::Newtype("data".into()),
        Kind::Tuple(7, false),
        Kind::Struct { value: -2 },
    ] {
        roundtrip(&value);
    }

    roundtrip(&Wrapper(42));
    roundtrip(&Pair(-1, "x".into()));
    roundtrip(&Unit);
    roundtrip(&(1, "x".to_owned()));
    roundtrip(&Some("null".to_owned()));
    roundtrip(&None::<String>);
    assert_eq!(
        from_tag::<Kind>("!Newtype abc").unwrap(),
        Kind::Newtype("abc".into())
    );
    assert_eq!(
        from_tag::<Kind>("!Tuple [7, false]").unwrap(),
        Kind::Tuple(7, false)
    );
    assert_eq!(
        from_tag::<Kind>("!Struct {value: -2}").unwrap(),
        Kind::Struct { value: -2 }
    );
    assert_eq!(from_tag::<Kind>("!Unit").unwrap(), Kind::Unit);
    assert_eq!(from_str::<Kind>("! Unit").unwrap(), Kind::Unit);
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn core_resolution_and_numeric_limits() {
    for source in [
        "yes",
        "on",
        "no",
        "off",
        "NO",
        "Off",
        "0b11",
        "+0xB",
        "0XB",
        "1_000",
        "2001-12-15",
        "1:20",
    ] {
        assert_eq!(from_str::<String>(source).unwrap(), source);
        assert_eq!(from_str::<serde_json::Value>(source).unwrap(), source);
        assert!(from_str::<bool>(source).is_err());
    }

    for source in ["true", "True", "TRUE"] {
        assert!(from_str::<bool>(source).unwrap());
    }

    for source in ["false", "False", "FALSE"] {
        assert!(!from_str::<bool>(source).unwrap());
    }

    assert_eq!(from_str::<i16>("012").unwrap(), 12);
    assert_eq!(from_str::<u16>("0o77").unwrap(), 63);
    assert_eq!(
        from_str::<f64>("0xff").unwrap().to_bits(),
        255.0f64.to_bits()
    );
    assert_eq!(
        from_str::<f64>("-.INF").unwrap().to_bits(),
        f64::NEG_INFINITY.to_bits()
    );
    assert!(from_str::<f64>(".NaN").unwrap().is_nan());

    for source in [".inf", "-.inf", ".nan"] {
        assert!(from_str::<astral_yaml_dev::Value>(source).is_ok());
    }

    assert!(from_str::<u8>("256").is_err());
    assert!(from_str::<u8>("-1").is_err());
    assert!(from_str::<i8>("128").is_err());
    assert!(from_str::<i32>("'42'").is_err());

    assert_eq!(from_str::<i32>("!!int '42'").unwrap(), 42);
    assert_eq!(from_str::<String>("!!str 42").unwrap(), "42");
    assert!(from_str::<bool>("!!bool yes").is_err());
    assert!(from_str::<String>("!unknown value").is_err());

    roundtrip(&i128::MIN);
    roundtrip(&i128::MAX);
    roundtrip(&u128::MAX);
    roundtrip(&(u128::from(u64::MAX) + 1));
    roundtrip(&f32::MIN_POSITIVE);
    roundtrip(&f64::MAX);
    roundtrip(&f64::INFINITY);
    roundtrip(&f64::NEG_INFINITY);
    assert!(
        from_str::<f64>(&to_string(&f64::NAN).unwrap())
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        from_str::<f64>(&to_string(&-0.0).unwrap())
            .unwrap()
            .to_bits(),
        (-0.0f64).to_bits()
    );

    for value in [f64::MIN, f64::MIN_POSITIVE, f64::from_bits(1)] {
        let yaml = to_string(&value).unwrap();
        assert_eq!(from_str::<f64>(&yaml).unwrap().to_bits(), value.to_bits());
    }

    let source = "defaults: &defaults {answer: 42}\ncopy: {<<: *defaults}\n";
    let value = from_str::<serde_json::Value>(source).unwrap();
    assert_eq!(value["copy"], serde_json::json!({"<<": {"answer": 42}}));
}

// https://yaml.org/spec/1.2.2/#10213-integer
#[test]
fn unsigned_integers_accept_negative_zero() {
    macro_rules! check {
        ($($ty:ty),* $(,)?) => {
            $(
                for source in ["-0", "-00", "!!int '-0'"] {
                    assert_eq!(from_str::<$ty>(source).unwrap(), 0, "{source}");
                }

                assert!(from_str::<$ty>("-1").is_err());
            )*
        };
    }

    check!(u8, u16, u32, u64, u128);
}

// https://yaml.org/spec/1.2.2/#10213-integer
#[test]
fn wide_radix_integers_round_to_f64() {
    let two_to_128 = 2_f64.powi(128);
    let halfway = format!("0x1{}8{}", "0".repeat(13), "0".repeat(18));
    let above_halfway = format!("0x1{}8{}1", "0".repeat(13), "0".repeat(17));
    let max_f64 = format!("0x{}8{}", "f".repeat(13), "0".repeat(242));
    let below_overflow = format!("0x{}b{}", "f".repeat(13), "f".repeat(242));
    let overflow = format!("0x{}c{}", "f".repeat(13), "0".repeat(242));

    for (source, expected) in [
        (format!("0x1{}", "0".repeat(32)), two_to_128),
        (format!("0o1{}", "0".repeat(43)), 2_f64.powi(129)),
        (format!("!!int {halfway}"), two_to_128),
        (above_halfway, f64::from_bits(two_to_128.to_bits() + 1)),
        (max_f64, f64::MAX),
        (below_overflow, f64::MAX),
        (format!("0x{}1", "0".repeat(300)), 1.0),
    ] {
        assert_eq!(
            from_str::<f64>(&source).unwrap().to_bits(),
            expected.to_bits(),
            "{source}"
        );
    }

    assert_eq!(
        from_str::<f64>(&overflow).unwrap_err().message,
        "float out of range"
    );
    assert!(from_str::<f32>(&format!("0x1{}", "0".repeat(32))).is_err());
}

// https://yaml.org/spec/1.2.2/#10214-floating-point
#[test]
fn finite_float_overflow_is_an_error() {
    for source in ["1e999", "-1e999", "!!float '1e999'"] {
        let error = from_str::<f64>(source).unwrap_err();
        assert_eq!(error.message, "float out of range");
        assert!(error.span.unwrap().slice(source).is_some());
    }

    for source in [
        "3.5e38",
        "-3.5e38",
        "340282366920938463463374607431768211455",
        "0xffffffffffffffffffffffffffffffff",
    ] {
        assert!(from_str::<f64>(source).unwrap().is_finite());
        assert_eq!(
            from_str::<f32>(source).unwrap_err().message,
            "float out of range"
        );
    }

    for source in ["3.4028235e38", "1.0000000596046448", "1e-999", "-0.0"] {
        assert_eq!(
            from_str::<f32>(source).unwrap().to_bits(),
            source.parse::<f32>().unwrap().to_bits()
        );
    }

    assert_eq!(
        from_str::<f32>("0xff").unwrap().to_bits(),
        255_f32.to_bits()
    );

    // Just above a rounding midpoint: converting through f64 loses the +1.
    let integer = (1_u128 << 64) + (1_u128 << 40) + 1;
    let expected = integer.to_string().parse::<f32>().unwrap();
    for source in [format!("0x{integer:x}"), format!("0o{integer:o}")] {
        assert_eq!(
            from_str::<f32>(&source).unwrap().to_bits(),
            expected.to_bits()
        );
    }

    assert!(from_str::<f32>(".nan").unwrap().is_nan());
    assert_eq!(
        from_str::<f32>(".inf").unwrap().to_bits(),
        f32::INFINITY.to_bits()
    );
    assert_eq!(
        from_str::<f32>("-.inf").unwrap().to_bits(),
        f32::NEG_INFINITY.to_bits()
    );
    assert!(from_str::<f32>("'1.0'").is_err());
}

#[test]
fn optional_tagged_enums() {
    for (source, expected) in [
        ("!Unit", Kind::Unit),
        ("!Newtype text", Kind::Newtype("text".to_owned())),
        ("!Tuple [1, true]", Kind::Tuple(1, true)),
        ("!Struct {value: 2}", Kind::Struct { value: 2 }),
        ("&mode !Unit", Kind::Unit),
    ] {
        assert_eq!(from_tag::<Option<Kind>>(source).unwrap(), Some(expected));
    }

    assert_eq!(from_str::<Option<Kind>>("null").unwrap(), None);
    assert!(from_str::<Option<String>>("!unknown value").is_err());
    assert!(from_str::<Option<u32>>("!!int null").is_err());
    assert!(from_str::<Option<Kind>>("!Unit value").is_err());
}

#[test]
fn all_string_characters_and_complex_keys() {
    let mut text: String = (0..=255).filter_map(char::from_u32).collect();
    text.push_str("\u{2028}\u{2029}\u{feff}\u{fffe}\u{ffff}🌍");
    roundtrip(&text);
    roundtrip(&'🌍');
    assert!(from_str::<char>("ab").is_err());

    roundtrip(&BTreeMap::from([
        (vec![1, 2], "first".to_owned()),
        (vec![], "second".to_owned()),
    ]));

    let map: BTreeMap<String, Vec<String>> = from_str("a: &a [one, two]\nb: *a\n").unwrap();
    assert_eq!(map["a"], map["b"]);
}

// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test]
fn serialization_preserves_text_and_escape_spelling() {
    let mut output = String::new();

    for (name, value) in [
        ("empty", ""),
        ("ASCII", "plain ASCII"),
        ("Unicode", "é — 中 🌍\u{a0}"),
        ("escapes", "é \"quoted\" 🌍\\\n"),
        ("controls", "\u{7f}\u{85}\u{2028}\0"),
    ] {
        write!(output, "{name}: {}", to_string(value).unwrap()).unwrap();
    }

    write!(output, "floats: {}", to_string(&[1.0, -0.0, 1.25]).unwrap()).unwrap();
    insta::assert_snapshot!(output, @r#"
    empty: ""
    ASCII: "plain ASCII"
    Unicode: "é — 中 🌍 "
    escapes: "é \"quoted\" 🌍\\\n"
    controls: "\u007F\u0085\u2028\u0000"
    floats: [1.0, -0.0, 1.25]
    "#);
}

#[test]
fn generated_nested_values_roundtrip() {
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
        let value = serde_json::json!({
            "name": text,
            "nested": [null, number % 2 == 0, number, -i64::from(number), f64::from(number) / 8.0],
            "mapping": {text: [text, {"inner": strings[(index / strings.len()) % strings.len()]}]},
        });

        roundtrip(&value);
    }
}

#[test]
fn punctuation_heavy_strings_roundtrip_as_keys_and_values() {
    use astral_yaml_dev::Value;

    let fragments = [
        "", "-", "?", ":", ",", "[", "]", "{", "}", "#", "&", "*", "!", "|", ">", "'", "\"", "%",
        "@", "`", "\\", "---", "...", "null", "true", "0x1f", "0o17", "1e3", ".nan", "é", "🌍",
        "\0", "\t", "\n",
    ];

    for fragment in fragments {
        for adjacent in ["", "a", " ", ":", "#", "\n", "1"] {
            for text in [
                format!("{fragment}{adjacent}"),
                format!("{adjacent}{fragment}"),
            ] {
                let text = Value::String(text);
                let value = Value::Sequence(vec![
                    text.clone(),
                    Value::Mapping(vec![(text.clone(), text)]),
                ]);

                roundtrip(&value);
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test_case(0)]
#[test_case(1)]
#[test_case(2)]
#[test_case(3)]
#[test_case(4)]
#[test_case(5)]
#[test_case(6)]
#[test_case(7)]
#[test_case(8)]
#[test_case(9)]
#[test_case(15)]
#[test_case(16)]
#[test_case(31)]
#[test_case(32)]
#[test_case(63)]
#[test_case(64)]
fn serialization_escapes_at_different_offsets(padding: usize) {
    let characters = (0..=255).filter_map(char::from_u32).chain([
        '\u{2028}', '\u{2029}', '\u{feff}', '\u{fffe}', '\u{ffff}', '中', '🌍',
    ]);
    for character in characters {
        let text = format!("{}{character}tail", "a".repeat(padding));
        roundtrip(&text);
    }
}

#[test]
fn located_failures_and_document_count() {
    for source in ["", "---\n---\n", "[", "256"] {
        let error = from_str::<u8>(source).unwrap_err();
        assert!(error.span.is_some());
    }

    for source in ["[]", "[1]", "[1,2,3]"] {
        assert!(from_str::<(u8, u8)>(source).is_err());
    }

    assert!(from_str::<Unit>("x").is_err());
    assert!(from_str::<Kind>("[1]").is_err());
    assert!(from_str::<Kind>("Newtype").is_err());
    assert!(from_str::<Kind>("!Unit value").is_err());

    let stream = astral_yaml::parse("42").unwrap();
    assert_eq!(from_stream::<u64>(&stream).unwrap(), 42);
}

// https://yaml.org/spec/1.2.2/#81-block-scalar-styles
// https://yaml.org/spec/1.2.2/#912-document-markers
#[test]
fn block_scalars_and_nulls_across_documents() {
    for style in ["|", ">"] {
        for (chomp, expected) in [("-", "text"), ("", "text\n"), ("+", "text\n\n")] {
            for boundary in ["--- kept\n", "...\n--- kept\n", "---\tkept\n"] {
                let source = format!("--- {style}{chomp}\ntext\n\n{boundary}");
                let stream = astral_yaml::parse(&source).unwrap();
                let documents: Vec<String> = (0..stream.documents().len())
                    .map(|index| astral_yaml_serde::from_document(&stream, index).unwrap())
                    .collect();
                assert_eq!(documents, [expected, "kept"], "{source}");
                assert!(from_str::<String>(&source).is_err());
            }
        }
    }

    let stream = astral_yaml::parse("--- null\n--- ''\n--- ~\n--- kept\n").unwrap();
    let documents: Vec<serde_json::Value> = (0..stream.documents().len())
        .map(|index| astral_yaml_serde::from_document(&stream, index).unwrap())
        .collect();
    assert_eq!(
        serde_json::Value::Array(documents),
        serde_json::json!([null, "", null, "kept"])
    );
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_case(Encoding::Utf8)]
#[test_case(Encoding::Utf16Be)]
#[test_case(Encoding::Utf16Le)]
#[test_case(Encoding::Utf32Be)]
#[test_case(Encoding::Utf32Le)]
fn deserialize_encoded_streams(encoding: Encoding) {
    use astral_yaml::Source;

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
        let value = from_stream::<serde_json::Value>(&stream).unwrap();
        assert_eq!(value, serde_json::json!({"message": "é 🌍"}));
    }
}

#[derive(Debug)]
struct Anything;

impl<'de> Deserialize<'de> for Anything {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AnyVisitor;

        impl<'de> Visitor<'de> for AnyVisitor {
            type Value = Anything;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any YAML value")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_i128<E: de::Error>(self, _: i128) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_u128<E: de::Error>(self, _: u128) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_str<E: de::Error>(self, _: &str) -> Result<Anything, E> {
                Ok(Anything)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Anything, A::Error> {
                while seq.next_element::<Anything>()?.is_some() {}

                Ok(Anything)
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Anything, A::Error> {
                while map.next_entry::<Anything, Anything>()?.is_some() {}

                Ok(Anything)
            }
        }

        deserializer.deserialize_any(AnyVisitor)
    }
}

#[test]
fn arbitrary_values_and_alias_limits() {
    from_str::<Anything>("[null, true, -2, 3, 4.5, text, {a: b}, -170141183460469231731687303715884105728, 340282366920938463463374607431768211455]").unwrap();
    assert!(from_str::<Anything>("!Values [!Text hello, !Count 42]").is_err());
    from_tag::<Anything>("!Values [hello, 42]").unwrap();
    assert!(from_str::<Anything>("!Config {key: 1, key: 2}").is_err());
    assert!(from_str::<Anything>("&a [*a]").is_err());

    for source in ["&self [*self]", "&self {next: *self}"] {
        assert!(astral_yaml::parse(source).is_ok());
        assert!(from_str::<de::IgnoredAny>(source).is_err());
    }

    let mut bomb = "a: &a [0,0]\n".to_owned();

    for i in b'b'..=b'z' {
        let name = char::from(i);
        let previous = char::from(i - 1);
        writeln!(bomb, "{name}: &{name} [*{previous}, *{previous}]").unwrap();
    }

    assert!(from_str::<Anything>(&bomb).is_err());
}

#[test]
fn serialization_recursion_limit() {
    use astral_yaml_dev::Value;

    struct Recursive;

    impl Serialize for Recursive {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_some(self)
        }
    }

    assert!(
        to_string(&Recursive)
            .unwrap_err()
            .message
            .contains("nesting")
    );

    let mut nested = Value::Null;
    for _ in 0..63 {
        nested = Value::Sequence(vec![nested]);
    }

    to_string(&nested).unwrap();
    nested = Value::Sequence(vec![nested]);
    assert_eq!(
        to_string(&nested).unwrap_err().message,
        "serialization nesting limit exceeded"
    );
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test_case("''", "''", true)]
#[test_case("ab", "ab", true)]
#[test_case("abc", "abc", true)]
#[test_case("abcd", "abcd", true)]
#[test_case("abcde0", "abcde1", false)]
#[test_case("text", "text", true)]
#[test_case("42", "0x2a", true)]
#[test_case("-0.0", "0.0", true)]
#[test_case(".nan", ".nan", true)]
#[test_case("[1, true]", "[1, true]", true)]
#[test_case("{a: 1, b: 2}", "{b: 2, a: 1}", true)]
#[test_case("1", "1.0", false)]
#[test_case("'null'", "null", false)]
fn serialization_rejects_duplicate_mapping_keys(left: &str, right: &str, duplicate: bool) {
    use astral_yaml_dev::Value;

    let key = |source| astral_yaml_serde::from_str::<Value>(source).unwrap();
    for preceding in [0, 30, 31] {
        let mut entries: Vec<_> = (0..preceding)
            .map(|index| (Value::String(format!("prefix-{index}")), Value::Null))
            .collect();
        entries.extend([
            (key(left), Value::Integer(1)),
            (key(right), Value::Integer(2)),
        ]);
        let value = Value::Mapping(entries);
        let result = astral_yaml_serde::to_string(&value);
        if duplicate {
            assert_eq!(
                result.unwrap_err().message,
                "duplicate mapping key",
                "{left}: {right}"
            );
        } else {
            let source = result.unwrap();
            assert_eq!(
                astral_yaml_serde::from_str::<Value>(&source).unwrap(),
                value
            );
        }
    }
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test_case(1)]
#[test_case(31)]
#[test_case(32)]
fn serialized_mapping_keys_are_scoped_to_each_mapping(width: usize) {
    use astral_yaml_dev::Value;

    // Equal-length keys sharing a prefix must still compare their full text.
    let mut entries: Vec<_> = (0..width)
        .map(|index| (Value::String(format!("shared-{index:03}")), Value::Null))
        .collect();
    entries.push((Value::Unsigned(1), Value::Null));
    let child = Value::Mapping(entries.clone());
    entries.push((
        Value::String("children".into()),
        Value::Sequence(vec![child.clone(), child]),
    ));
    let parent = Value::Mapping(entries);

    // Reuse key storage after both nested and sibling mappings close.
    roundtrip(&Value::Sequence(vec![parent.clone(), parent]));
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test]
fn mapping_key_uniqueness_uses_core_types() {
    for source in [
        "{a: 1, a: 2}",
        "{1: a, 01: b}",
        "{0xB: a, 11: b}",
        "{0o13: a, 11: b}",
        "{0: a, -0: b}",
        "{[0xB, -0]: a, [11, +0]: b}",
        "{{0xB: 0o13}: a, {11: 11}: b}",
        "{&key 0xB: a, *key : b}",
        "{true: a, TRUE: b}",
        "{null: a, ~: b}",
        "{1.0: a, 1e0: b}",
        "{.nan: a, .NaN: b}",
        "{0.0: a, -0.0: b}",
        "{[a, b]: 1, [a, b]: 2}",
        "{{a: 1, b: 2}: x, {b: 2, a: 1}: y}",
        "{&a key: 1, *a : 2}",
        "{{a: 1, a: 2}: x}",
        "{first: 0, 1: a, 01: b}",
        "{first: 0, [a]: 1, [a]: 2}",
        "{!!str true: a, 'true': b}",
    ] {
        let error = from_str::<Anything>(source).unwrap_err();
        assert_eq!(error.message, "duplicate mapping key", "{source}: {error}");
        assert!(error.span.is_some());
        assert!(
            from_str::<serde::de::IgnoredAny>(source).is_err(),
            "{source}"
        );
    }

    from_str::<Anything>("{1: a, '1': b, true: c, 'true': d}").unwrap();

    let source = "{9007199254740992: first, 9007199254740993: second}";
    assert_eq!(
        from_str::<BTreeMap<u64, String>>(source).unwrap(),
        BTreeMap::from([
            (9_007_199_254_740_992, "first".into()),
            (9_007_199_254_740_993, "second".into()),
        ])
    );

    assert_eq!(
        from_str::<BTreeMap<BTreeMap<String, i32>, i32>>("{{'null': 1}: 2}").unwrap(),
        BTreeMap::from([(BTreeMap::from([("null".into(), 1)]), 2)])
    );
    assert_eq!(
        from_str::<BTreeMap<Option<String>, String>>(": value\n").unwrap(),
        BTreeMap::from([(None, "value".into())])
    );
}

// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test_case(1)]
#[test_case(7)]
#[test_case(8)]
#[test_case(9)]
#[test_case(64)]
fn mapping_key_uniqueness_across_mapping_sizes(count: usize) {
    let mut source = String::new();

    for key in 0..count {
        writeln!(source, "{key}: value").unwrap();
    }

    let mapping = from_str::<BTreeMap<u64, String>>(&source).unwrap();
    assert_eq!(mapping.len(), count);

    source.push_str("0x00: duplicate\n");
    let error = from_str::<BTreeMap<u64, String>>(&source).unwrap_err();
    assert_eq!(error.message, "duplicate mapping key");
    assert_eq!(error.span.unwrap().slice(&source), Some("0x00"));

    let mut source = String::new();

    for key in 0..count {
        writeln!(source, "key{key}: value").unwrap();
    }

    let mapping = from_str::<BTreeMap<String, String>>(&source).unwrap();
    assert_eq!(mapping.len(), count);

    source.push_str("key0: duplicate\n");
    let error = from_str::<BTreeMap<String, String>>(&source).unwrap_err();
    assert_eq!(error.message, "duplicate mapping key");
    assert_eq!(error.span.unwrap().slice(&source), Some("key0"));
}

// https://yaml.org/spec/1.2.2/#333-recognized-and-valid-tags
#[test]
fn collection_tags_must_match_the_requested_type() {
    for source in ["!!str []", "!!int {}", "!!map []", "!!seq {}"] {
        assert!(from_str::<Anything>(source).is_err(), "{source}");
    }

    from_str::<Anything>("!!map {a: !!seq [1]}").unwrap();
}
