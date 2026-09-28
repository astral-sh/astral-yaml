mod support;

use astral_yaml_serde::{Spanned, Tagged, from_str, to_string, with::yaml_tag};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use std::{collections::BTreeMap, fmt::Write};
use support::{YamlTag, from_tag};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Message {
    Ready,
    Text(String),
    Pair(u8, bool),
    Config { enabled: bool },
}

#[test]
fn enums_use_strings_and_maps_by_default() {
    let mut output = String::new();

    for (name, value) in [
        ("unit", Message::Ready),
        ("newtype", Message::Text("hello".into())),
        ("tuple", Message::Pair(1, true)),
        ("struct", Message::Config { enabled: true }),
    ] {
        let yaml = to_string(&value).unwrap();
        assert_eq!(from_str::<Message>(&yaml).unwrap(), value);
        write!(output, "{name}: {yaml}").unwrap();
    }

    insta::assert_snapshot!(output, @r#"
    unit: "Ready"
    newtype: {? "Text": "hello"}
    tuple: {"Pair": [1, true]}
    struct: {"Config": {? "enabled": true}}
    "#);
}

#[test]
fn non_core_tags_require_opt_in_including_ignored_values_and_keys() {
    #[derive(Debug, Deserialize)]
    struct Empty {}

    for source in [
        "!Text hello",
        "!Ready",
        "!Pair [1, true]",
        "!Config {enabled: true}",
        "!str hello",
        "!!binary aGVsbG8=",
        "!!timestamp 2026-09-28",
        "!!set {x: null}",
        "!<tag:example.org,2026:text> hello",
        "%TAG !! tag:example.org,2026:\n--- !!str hello",
        "[!Text hello]",
        "{unknown: !Text hello}",
        "{!Text key: value}",
        "[&x !Text hello, *x]",
    ] {
        let error = from_str::<IgnoredAny>(source).unwrap_err();
        assert_eq!(
            error.message, "non-core YAML tags require Tagged<T> or with::yaml_tag",
            "{source}"
        );
        assert!(error.span.unwrap().slice(source).is_some());
        assert!(from_str::<serde_json::Value>(source).is_err());
    }

    assert!(from_str::<Empty>("{ignored: !Count 42}").is_err());
    assert!(from_str::<Option<u32>>("!Count 42").is_err());
    assert!(from_str::<Spanned<'_, String>>("!Text hello").is_err());
    assert!(from_str::<Vec<u8>>("!Items [1]").is_err());
    assert!(from_str::<BTreeMap<String, u8>>("!Config {x: 1}").is_err());
    assert!(from_str::<Message>("!Text hello").is_err());
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn core_tags_work_without_opt_in() {
    #[derive(Debug, Deserialize)]
    struct Empty {}

    for (source, expected) in [
        ("!!str null", serde_json::json!("null")),
        ("!!int '42'", serde_json::json!(42)),
        ("!!float '1.5'", serde_json::json!(1.5)),
        ("!!bool 'true'", serde_json::json!(true)),
        ("!!null null", serde_json::Value::Null),
        ("!!seq [!!int '1']", serde_json::json!([1])),
        (
            "!!map {!!str key: !!int '1'}",
            serde_json::json!({"key": 1}),
        ),
        ("! null", serde_json::json!("null")),
        ("! [1]", serde_json::json!([1])),
        ("! {key: 1}", serde_json::json!({"key": 1})),
        ("!<tag:yaml.org,2002:str> null", serde_json::json!("null")),
        (
            "%TAG !c! tag:yaml.org,2002:\n--- !c!int '42'",
            serde_json::json!(42),
        ),
        ("[&x !!int '42', *x]", serde_json::json!([42, 42])),
    ] {
        assert_eq!(from_str::<serde_json::Value>(source).unwrap(), expected);
        from_str::<IgnoredAny>(source).unwrap();
    }

    from_str::<Empty>("{ignored: !!int 42}").unwrap();
    assert_eq!(from_str::<Option<u32>>("!!int '42'").unwrap(), Some(42));
    assert_eq!(from_str::<Vec<u8>>("!!seq [1]").unwrap(), [1]);
    assert_eq!(
        from_str::<BTreeMap<String, u8>>("!!map {x: 1}").unwrap(),
        BTreeMap::from([("x".into(), 1)])
    );

    let source = "!!str hello # greeting";
    let text = from_str::<Spanned<'_, &str>>(source).unwrap();
    assert_eq!(text.value, "hello");
    assert!(std::ptr::eq(text.value.as_ptr(), source[6..].as_ptr()));
    assert_eq!(text.span.slice(source), Some("!!str hello"));
    assert_eq!(text.comments[0].comment.text, " greeting");

    for source in ["!!null", "!!null null", "!!null '~'"] {
        assert_eq!(from_str::<Option<String>>(source).unwrap(), None);
        assert_eq!(from_tag::<Option<Message>>(source).unwrap(), None);
        assert_eq!(from_str::<Option<Tagged<()>>>(source).unwrap(), None);
        let value = from_str::<Tagged<Option<String>>>(source).unwrap();
        assert_eq!(value.tag, "tag:yaml.org,2002:null");
        assert_eq!(value.value, None);
    }

    assert_eq!(
        from_str::<Option<String>>("!!str null").unwrap(),
        Some("null".into())
    );
    assert_eq!(
        from_str::<Vec<Option<String>>>("[&n !!null null, *n]").unwrap(),
        [None, None]
    );
}

// https://yaml.org/spec/1.2.2/#333-recognized-and-valid-tags
#[test]
fn core_tags_still_validate_their_payloads() {
    for source in [
        "!!bool yes",
        "!!int text",
        "!!float text",
        "!!null text",
        "!!str []",
        "!!seq {}",
        "!!map []",
        "!!seq text",
        "{ignored: !!int text}",
    ] {
        assert!(from_str::<IgnoredAny>(source).is_err(), "{source}");
        assert!(from_str::<serde_json::Value>(source).is_err(), "{source}");
    }

    assert_eq!(
        from_str::<bool>("!!bool yes").unwrap_err().message,
        "scalar does not match its tag"
    );
    assert!(from_str::<u32>("!!str 42").is_err());
    assert!(from_str::<Option<u32>>("!!int null").is_err());
    assert!(from_str::<Vec<u32>>("!!map [1]").is_err());
}

#[test]
fn strings_accept_implicit_types_and_explicit_string_tags() {
    for (source, expected) in [
        ("42", "42"),
        ("'42'", "42"),
        ("true", "true"),
        ("1.5", "1.5"),
        ("null", "null"),
        ("!!str false", "false"),
        ("! null", "null"),
        ("!<tag:yaml.org,2002:str> 42", "42"),
        ("%TAG !c! tag:yaml.org,2002:\n--- !c!str 42", "42"),
    ] {
        assert_eq!(from_str::<String>(source).unwrap(), expected, "{source}");
        assert_eq!(from_str::<&str>(source).unwrap(), expected, "{source}");
    }

    for source in ["!Text 42", "!!str 42", "! 42"] {
        assert_eq!(from_str::<Tagged<String>>(source).unwrap().value, "42");
        assert_eq!(from_str::<Tagged<&str>>(source).unwrap().value, "42");
    }
}

// Explicit tags determine the YAML type even on quoted scalars.
// https://yaml.org/spec/1.2.2/#691-node-tags
#[test]
fn strings_reject_explicit_non_string_core_tags() {
    let mut output = String::new();

    for source in [
        "!!int 42",
        "!!int '42'",
        "!!bool true",
        "!!float 1.5",
        "!!null null",
        "!<tag:yaml.org,2002:int> 42",
        "%TAG !c! tag:yaml.org,2002:\n--- !c!int 42",
        "!!int nope",
        "!!map 'string'",
    ] {
        let error = from_str::<String>(source).unwrap_err();
        assert_eq!(from_str::<&str>(source).unwrap_err(), error);
        assert_eq!(from_str::<Tagged<String>>(source).unwrap_err(), error);
        assert_eq!(from_str::<Tagged<&str>>(source).unwrap_err(), error);
        assert_eq!(from_str::<Spanned<'_, String>>(source).unwrap_err(), error);
        writeln!(output, "{source:?}: {error}").unwrap();
    }

    insta::assert_snapshot!(output, @r#"
    "!!int 42": tag does not match string type at bytes 6..8
    "!!int '42'": tag does not match string type at bytes 6..10
    "!!bool true": tag does not match string type at bytes 7..11
    "!!float 1.5": tag does not match string type at bytes 8..11
    "!!null null": tag does not match string type at bytes 7..11
    "!<tag:yaml.org,2002:int> 42": tag does not match string type at bytes 25..27
    "%TAG !c! tag:yaml.org,2002:\n--- !c!int 42": tag does not match string type at bytes 39..41
    "!!int nope": scalar does not match its tag at bytes 6..10
    "!!map 'string'": unsupported scalar tag at bytes 0..14
    "#);
}

#[test]
fn string_tag_checks_apply_to_aliases_keys_and_wrapped_values() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Text(String);

    for (scalar, accepted) in [("42", true), ("!!str 42", true), ("!!int 42", false)] {
        let source = format!("[&value {scalar}, *value]");
        let result = from_str::<(IgnoredAny, String)>(&source).map(|(_, value)| value);
        let borrowed = from_str::<(IgnoredAny, &str)>(&source).map(|(_, value)| value);

        if accepted {
            assert_eq!(result.unwrap(), "42");
            assert_eq!(borrowed.unwrap(), "42");
            assert_eq!(
                from_str::<Option<String>>(scalar).unwrap(),
                Some("42".into())
            );
            assert_eq!(from_str::<Text>(scalar).unwrap(), Text("42".into()));
            assert_eq!(
                from_str::<BTreeMap<String, u8>>(&format!("{{{scalar}: 1}}")).unwrap(),
                BTreeMap::from([("42".into(), 1)])
            );
        } else {
            let error = result.unwrap_err();
            assert_eq!(borrowed.unwrap_err(), error);
            assert_eq!(error.span.unwrap().slice(&source), Some("42"));
            assert!(from_str::<Option<String>>(scalar).is_err());
            assert!(from_str::<Text>(scalar).is_err());
            assert!(from_str::<BTreeMap<String, u8>>(&format!("{{{scalar}: 1}}")).is_err());
        }
    }
}

#[test]
fn adapter_roundtrips_every_variant_shape() {
    for (yaml, expected) in [
        ("!Ready", Message::Ready),
        ("!Text \"hello\"", Message::Text("hello".into())),
        ("!Pair [1, true]", Message::Pair(1, true)),
        (
            "!Config {? \"enabled\": true}",
            Message::Config { enabled: true },
        ),
    ] {
        let value = from_tag::<Message>(yaml).unwrap();
        assert_eq!(value, expected);
        assert_eq!(to_string(&YamlTag(value)).unwrap(), format!("{yaml}\n"));
        assert_eq!(
            from_tag::<Message>(&format!("&a {yaml}")).unwrap(),
            expected
        );
    }

    assert_eq!(from_tag::<Message>("Ready").unwrap(), Message::Ready);
    assert_eq!(from_str::<Message>("!!str Ready").unwrap(), Message::Ready);
    assert_eq!(from_tag::<Message>("!!str Ready").unwrap(), Message::Ready);
    assert_eq!(from_tag::<Message>("! Ready").unwrap(), Message::Ready);
    assert_eq!(
        from_tag::<Message>("!!map {Text: !!str hello}").unwrap(),
        Message::Text("hello".into())
    );
    assert_eq!(from_tag::<Message>("!Ready null").unwrap(), Message::Ready);
    assert_eq!(
        from_tag::<Message>("{Text: hello}").unwrap(),
        Message::Text("hello".into())
    );
    for yaml in [
        "!Ready value",
        "!Ready []",
        "!Ready ''",
        "!Unknown x",
        "!<tag:example.org,2026:Ready>",
    ] {
        assert!(from_tag::<Message>(yaml).is_err(), "{yaml}");
    }
}

#[test]
fn mixed_enums_buffer_tags_only_when_opted_in() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    enum Mixed {
        Ready,
        Text(String),
        Pair(u8, bool),
        Config {
            enabled: bool,
        },
        #[serde(untagged)]
        Number(u32),
    }

    for (source, expected) in [
        ("!Ready", Mixed::Ready),
        ("!Ready null", Mixed::Ready),
        ("!Text hello", Mixed::Text("hello".into())),
        ("!Pair [1, true]", Mixed::Pair(1, true)),
        ("!Config {enabled: true}", Mixed::Config { enabled: true }),
        (
            "!Config {enabled: !!bool true}",
            Mixed::Config { enabled: true },
        ),
        ("42", Mixed::Number(42)),
    ] {
        let value = from_tag::<Mixed>(source).unwrap();
        assert_eq!(value, expected);
        let encoded = to_string(&YamlTag(value)).unwrap();
        assert_eq!(from_tag::<Mixed>(&encoded).unwrap(), expected);
        assert_eq!(from_str::<Mixed>(source).is_err(), source.starts_with('!'));
    }

    let values = from_str::<Vec<YamlTag<Mixed>>>("[&a !Text hello, 42, *a]").unwrap();
    assert_eq!(values[0], values[2]);
    for source in [
        "!Ready value",
        "!Ready ''",
        "!Unknown hello",
        "!Pair [1]",
        "!Text []",
        "!Config {enabled: !Bool true}",
    ] {
        assert!(from_tag::<Mixed>(source).is_err(), "{source}");
    }
}

#[test]
fn adapter_scope_survives_transparent_wrappers_but_stops_at_children() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    enum Outer {
        Inner(Message),
        Explicit(YamlTag<Message>),
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Newtype(Message);

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Config {
        #[serde(with = "astral_yaml_serde::with::yaml_tag")]
        tagged: Option<Newtype>,
        ordinary: Message,
    }

    let source = "tagged: !Text hello\nordinary: {Text: world}";
    let value = from_str::<Config>(source).unwrap();
    let encoded = to_string(&value).unwrap();
    assert!(encoded.contains("!Text"));
    assert!(encoded.contains("{? \"Text\": \"world\"}"));
    assert_eq!(from_str::<Config>(&encoded).unwrap(), value);
    assert!(from_str::<Config>("tagged: !Ready\nordinary: !Ready").is_err());
    let value = from_str::<Config>("tagged: null\nordinary: Ready").unwrap();
    assert_eq!(
        from_str::<Config>(&to_string(&value).unwrap()).unwrap(),
        value
    );

    for source in ["!Inner {Text: hello}", "!Explicit !Text hello"] {
        // The second spelling puts two tags on the same node, which YAML forbids.
        assert_eq!(
            from_tag::<Outer>(source).is_ok(),
            source.starts_with("!Inner")
        );
    }
    let value = from_tag::<Outer>("!Inner {Text: hello}").unwrap();
    assert_eq!(
        from_tag::<Outer>(&to_string(&YamlTag(value)).unwrap()).unwrap(),
        Outer::Inner(Message::Text("hello".into()))
    );
    assert!(to_string(&YamlTag(Outer::Explicit(YamlTag(Message::Ready)))).is_err());
    assert!(from_tag::<Vec<Message>>("[!Ready]").is_err());
    assert!(from_tag::<Message>("!Config {enabled: !Bool true}").is_err());
    assert_eq!(
        from_tag::<Message>("!Config {enabled: !!bool true}").unwrap(),
        Message::Config { enabled: true }
    );
}

#[test]
fn explicit_wrappers_preserve_tags_and_metadata() {
    #[derive(Debug, Deserialize)]
    enum Borrowed<'a> {
        Text(#[serde(borrow)] &'a str),
    }

    let source = "# greeting\n!Text hello # inline\n";
    let value = from_tag::<Spanned<'_, Message>>(source).unwrap();
    assert_eq!(value.value, Message::Text("hello".into()));
    assert_eq!(value.span.slice(source), Some("!Text hello"));
    assert_eq!(value.comments.len(), 2);

    let Borrowed::Text(text) = from_tag::<Borrowed<'_>>(source).unwrap();
    assert!(std::ptr::eq(text.as_ptr(), source[17..].as_ptr()));
    assert_eq!(
        from_str::<Tagged<String>>("!Text hello").unwrap().tag,
        "!Text"
    );
    assert_eq!(
        from_str::<Option<Tagged<u32>>>("!!int '42'")
            .unwrap()
            .unwrap()
            .value,
        42
    );
    assert!(
        to_string(&Tagged {
            tag: "!Outer".into(),
            value: YamlTag(Message::Ready)
        })
        .is_err()
    );
}

#[test]
fn renamed_variants_validate_tag_names() {
    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    enum Renamed {
        #[serde(rename = "some-name", alias = "old-name")]
        Name,
        #[serde(rename = "a,b")]
        Punctuation,
        #[serde(rename = "not a URI")]
        Invalid,
        #[serde(rename = "")]
        Empty,
    }

    assert_eq!(from_tag::<Renamed>("!old-name").unwrap(), Renamed::Name);
    assert_eq!(to_string(&YamlTag(Renamed::Name)).unwrap(), "!some-name\n");
    assert_eq!(
        to_string(&YamlTag(Renamed::Punctuation)).unwrap(),
        "!<!a,b>\n"
    );
    assert_eq!(
        from_tag::<Renamed>("!<!a,b>").unwrap(),
        Renamed::Punctuation
    );
    assert!(to_string(&YamlTag(Renamed::Invalid)).is_err());
    assert!(to_string(&YamlTag(Renamed::Empty)).is_err());

    let error = yaml_tag::deserialize::<Message, _>(serde::de::value::U8Deserializer::<
        serde::de::value::Error,
    >::new(1))
    .unwrap_err();
    assert!(error.to_string().contains("an enum with YAML tags enabled"));
}

#[derive(Debug)]
struct BufferedMap;

impl<'de> Deserialize<'de> for BufferedMap {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;

        impl<'de> serde::de::Visitor<'de> for MapVisitor {
            type Value = BufferedMap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a buffered map")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<BufferedMap, A::Error> {
                assert_eq!(map.size_hint(), Some(1));
                let key = map.next_key::<String>()?.unwrap();
                if key == "partial" {
                    return Ok(BufferedMap);
                }

                if key == "key" {
                    map.next_key::<String>()?;
                }

                map.next_value::<u8>()?;
                assert_eq!(map.size_hint(), Some(0));
                if key == "value" {
                    map.next_value::<u8>()?;
                }

                assert!(map.next_key::<String>()?.is_none());
                Ok(BufferedMap)
            }
        }

        deserializer.deserialize_any(MapVisitor)
    }
}

#[test]
fn tag_buffering_enforces_map_consumption() {
    from_tag::<BufferedMap>("!normal 1").unwrap();
    for (source, expected) in [
        ("!partial 1", "tagged mapping was not fully consumed"),
        ("!key 1", "mapping key has no value"),
        ("!value 1", "mapping value has no key"),
    ] {
        let error = from_tag::<BufferedMap>(source).unwrap_err();
        assert_eq!(error.message, expected);
        assert_eq!(error.span.unwrap().slice(source), Some(source));
    }
}

#[test]
fn recursive_opt_in_remains_bounded() {
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    enum Recursive {
        Children(Vec<YamlTag<Recursive>>),
    }

    let error = from_tag::<Recursive>("&self !Children [*self]").unwrap_err();
    assert!(error.message.contains("resource limit"));
}
