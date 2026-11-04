mod support;

use astral_yaml::Source;
use astral_yaml_dev::{diagnostic_snapshot, span_snapshot};
use astral_yaml_serde::{Spanned, from_document, from_str, from_stream, to_string};
use serde::{Deserialize, Serialize, de};
use std::collections::{BTreeMap, HashMap};
use support::{YamlTag, from_tag};

#[test]
fn scalars_retain_their_lexical_spans() {
    let mut snapshots = Vec::new();
    for (source, value, lexeme) in [
        ("  hello # comment\n", "hello", "hello"),
        ("'héllo'", "héllo", "'héllo'"),
        ("'it''s'", "it's", "'it''s'"),
        ("''", "", "''"),
        ("''''", "'", "''''"),
        ("\"a\\nb\"", "a\nb", "\"a\\nb\""),
        ("!!str &name 42", "42", "!!str &name 42"),
        (
            "|-\n  hello\n  world\n",
            "hello\nworld",
            "|-\n  hello\n  world\n",
        ),
        (
            ">-\n  hello\n  world\n",
            "hello world",
            ">-\n  hello\n  world\n",
        ),
    ] {
        let spanned: Spanned<String> = from_str(source).unwrap();
        assert_eq!(*spanned, value);
        assert_eq!(spanned.span.slice(source), Some(lexeme));
        snapshots.push(span_snapshot(
            source,
            [(format!("value: {value:?}"), spanned.span)],
        ));
    }
    insta::assert_snapshot!(snapshots.join("\n\n"), @r#"
      |
    1 |   hello # comment
      |   ^^^^^ value: "hello"

      |
    1 | 'héllo'
      | ^^^^^^^ value: "héllo"

      |
    1 | 'it''s'
      | ^^^^^^^ value: "it's"

      |
    1 | ''
      | ^^ value: ""

      |
    1 | ''''
      | ^^^^ value: "'"

      |
    1 | "a\nb"
      | ^^^^^^ value: "a\nb"

      |
    1 | !!str &name 42
      | ^^^^^^^^^^^^^^ value: "42"

      |
    1 | / |-
    2 | |   hello
    3 | |   world
      | |________^ value: "hello\nworld"

      |
    1 | / >-
    2 | |   hello
    3 | |   world
      | |________^ value: "hello world"
    "#);
}

#[test]
fn numeric_values_retain_their_lexical_spans() {
    let source = "0xff";
    let number: Spanned<u16> = from_str(source).unwrap();
    assert_eq!(*number, 255);
    insta::assert_snapshot!(span_snapshot(source, [("value: 255", number.span)]), @"
      |
    1 | 0xff
      | ^^^^ value: 255
    ");
}

#[test]
fn mutating_the_value_retains_its_source_span() {
    let mut text: Spanned<String> = from_str("hello").unwrap();
    let original_span = text.span;
    text.push_str(" world");
    assert_eq!(text.span, original_span);
    assert_eq!(text.into_inner(), "hello world");
}

#[test]
fn decoded_strings_require_owned_storage_and_report_lexical_spans() {
    let mut snapshots = Vec::new();
    for source in ["'it''s'", "'hello\n  world'", "\"a\\nb\""] {
        let error = from_str::<Spanned<&str>>(source).unwrap_err();
        snapshots.push(diagnostic_snapshot(source, error));
    }
    insta::assert_snapshot!(snapshots.join("\n\n"), @r#"
      |
    1 | 'it''s'
      | ^^^^^^^ $: invalid type: string "it's", expected a borrowed string

      |
    1 | / 'hello
    2 | |   world'
      | |________^ $: invalid type: string "hello world", expected a borrowed string

      |
    1 | "a\nb"
      | ^^^^^^ $: invalid type: string "a\nb", expected a borrowed string
    "#);
}

#[derive(Debug, Deserialize, Serialize)]
struct Config<'a> {
    #[serde(borrow)]
    name: Spanned<&'a str>,

    counts: Spanned<Vec<Spanned<u8>>>,
    enabled: bool,

    optional: Option<Spanned<String>>,
}

#[test]
fn wrappers_compose_with_structs_collections_and_borrowing() {
    let source = "{name: 'é🌍', counts: [1, 2], enabled: true}";
    let config: Spanned<Config<'_>> = from_str(source).unwrap();
    assert_eq!(*config.name, "é🌍");
    assert!(std::ptr::eq(config.name.as_ptr(), source[8..].as_ptr()));
    assert_eq!(
        config
            .counts
            .iter()
            .map(|value| **value)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(config.enabled);
    assert!(config.optional.is_none());
    let snapshots = [
        span_snapshot(source, [("config", config.span)]),
        span_snapshot(
            source,
            [("name", config.name.span), ("counts", config.counts.span)],
        ),
        span_snapshot(
            source,
            [
                ("counts[0]", config.counts[0].span),
                ("counts[1]", config.counts[1].span),
            ],
        ),
    ];
    insta::assert_snapshot!(snapshots.join("\n\n"), @"
      |
    1 | {name: 'é🌍', counts: [1, 2], enabled: true}
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ config

      |
    1 | {name: 'é🌍', counts: [1, 2], enabled: true}
      |        ^^^^^ name     ^^^^^^ counts

      |
    1 | {name: 'é🌍', counts: [1, 2], enabled: true}
      |                        ^  ^ counts[1]
      |                        |
      |                        counts[0]
    ");
}

#[test]
fn mapping_keys_and_sequence_elements_retain_their_spans() {
    let source = "{é: [3, 4]}";
    let mapping: HashMap<Spanned<String>, Spanned<Vec<Spanned<u8>>>> = from_str(source).unwrap();
    let (key, values) = mapping.iter().next().unwrap();
    assert_eq!(key.value, "é");
    let snapshots = [
        span_snapshot(source, [("key", key.span), ("values", values.span)]),
        span_snapshot(
            source,
            [("values[0]", values[0].span), ("values[1]", values[1].span)],
        ),
    ];
    insta::assert_snapshot!(snapshots.join("\n\n"), @"
      |
    1 | {é: [3, 4]}
      |  ^  ^^^^^^ values
      |  |
      |  key

      |
    1 | {é: [3, 4]}
      |      ^  ^ values[1]
      |      |
      |      values[0]
    ");

    let source = "'abc'";
    let bytes: Spanned<&[u8]> = from_str(source).unwrap();
    assert_eq!(*bytes, b"abc");
    insta::assert_snapshot!(span_snapshot(source, [("bytes", bytes.span)]), @"
      |
    1 | 'abc'
      | ^^^^^ bytes
    ");
}

#[test]
fn options_and_nested_wrappers_keep_the_same_node_span() {
    let mut snapshots = Vec::new();
    for source in ["null", "~", "---\n"] {
        let value: Spanned<Option<u8>> = from_str(source).unwrap();
        assert!(value.is_none());
        assert_eq!(
            value.span.slice(source),
            Some(source.trim_start_matches("---\n"))
        );
        if value.span.is_empty() {
            assert_eq!(value.span.start(), "---".len());
        }
        snapshots.push(span_snapshot(source, [("None", value.span)]));
        assert!(from_str::<Option<Spanned<u8>>>(source).unwrap().is_none());
    }
    insta::assert_snapshot!(snapshots.join("\n\n"), @"
      |
    1 | null
      | ^^^^ None

      |
    1 | ~
      | ^ None

      |
    1 | ---
      |    ^ None
    ");

    let value: Option<Spanned<u8>> = from_str("42").unwrap();
    assert_eq!(value.unwrap().value, 42);

    let value: Spanned<Spanned<Option<u8>>> = from_str("42").unwrap();
    assert_eq!(value.span, value.value.span);
    assert_eq!(**value, Some(42));
    insta::assert_snapshot!(span_snapshot("42", [("nested wrappers: Some(42)", value.span)]), @"
      |
    1 | 42
      | ^^ nested wrappers: Some(42)
    ");

    let source = "{empty: }";
    let mapping: BTreeMap<String, Spanned<Option<u8>>> = from_str(source).unwrap();
    assert!(mapping["empty"].is_none());
    assert!(mapping["empty"].span.is_empty());
    insta::assert_snapshot!(span_snapshot(source, [("empty: None", mapping["empty"].span)]), @"
      |
    1 | {empty: }
      |        ^ empty: None
    ");
}

#[test]
fn aliases_use_the_occurrence_span_and_children_use_definition_spans() {
    let source = "[&number 42, *number]";
    let values: Vec<Spanned<u8>> = from_str(source).unwrap();
    assert_eq!(values[0].value, values[1].value);
    insta::assert_snapshot!(span_snapshot(source, [
        ("definition", values[0].span),
        ("alias", values[1].span),
    ]), @"
      |
    1 | [&number 42, *number]
      |  ^^^^^^^^^^  ^^^^^^^ alias
      |  |
      |  definition
    ");

    let source = "[&items [1, 2], *items]";
    let values: Vec<Spanned<Vec<Spanned<u8>>>> = from_str(source).unwrap();
    assert_eq!(values[0].value, values[1].value);
    let snapshots = [
        span_snapshot(
            source,
            [("definition", values[0].span), ("alias", values[1].span)],
        ),
        span_snapshot(
            source,
            [
                ("alias[0]", values[1][0].span),
                ("alias[1]", values[1][1].span),
            ],
        ),
    ];
    insta::assert_snapshot!(snapshots.join("\n\n"), @"
      |
    1 | [&items [1, 2], *items]
      |  ^^^^^^^^^^^^^  ^^^^^^ alias
      |  |
      |  definition

      |
    1 | [&items [1, 2], *items]
      |          ^  ^ alias[1]
      |          |
      |          alias[0]
    ");
}

#[derive(Debug, Deserialize, PartialEq)]
struct Newtype(Option<Spanned<u8>>);

#[derive(Debug, Deserialize, PartialEq)]
enum Kind {
    Unit,
    Newtype(Spanned<Newtype>),
    Tuple(Spanned<u8>, bool),
    Struct { value: Spanned<u8> },
    Map(Spanned<BTreeMap<String, Spanned<u8>>>),
}

#[test]
fn enum_tags_are_consumed_once_through_transparent_wrappers() {
    for source in ["Unit", "!Unit", "{Unit: null}"] {
        let value: Spanned<Kind> = from_tag(source).unwrap();
        assert_eq!(*value, Kind::Unit);
        assert_eq!(value.span.slice(source), Some(source));
    }

    for source in ["!Newtype 42", "{Newtype: 42}"] {
        let value: Spanned<Kind> = from_tag(source).unwrap();
        let Kind::Newtype(payload) = value.value else {
            panic!("expected a newtype variant");
        };

        let number = payload.value.0.unwrap();
        assert_eq!(number.value, 42);
        assert_eq!(number.span, payload.span);
        assert_eq!(
            payload.span.slice(source),
            Some(if source.starts_with('!') {
                source
            } else {
                "42"
            })
        );
    }

    for source in [
        "!Tuple [42, false]",
        "!Struct {value: 42}",
        "!Map {value: 42}",
    ] {
        let value: Spanned<Kind> = from_tag(source).unwrap();
        let number = match value.value {
            Kind::Tuple(number, false) | Kind::Struct { value: number } => number,
            Kind::Map(mut values) => values.remove("value").unwrap(),
            _ => panic!("unexpected enum value"),
        };

        assert_eq!(*number, 42);
        assert_eq!(number.span.slice(source), Some("42"));
    }

    let source = "[&value !Newtype 42, *value]";
    let values: Vec<Spanned<Kind>> = from_str::<Vec<YamlTag<Spanned<Kind>>>>(source)
        .unwrap()
        .into_iter()
        .map(|value| value.0)
        .collect();
    assert_eq!(values[1].span.slice(source), Some("*value"));
    let Kind::Newtype(payload) = &values[1].value else {
        panic!("expected a newtype variant");
    };
    assert_eq!(payload.span, values[1].span);
    assert_eq!(
        *payload.value.0.as_ref().unwrap(),
        Spanned {
            value: 42,
            span: values[1].span,
        }
    );
}

#[test]
fn document_offsets_and_encoded_sources_use_the_core_span_convention() {
    let source = "---\nfirst\n...\n---\n'é🌍'\n";
    let stream = astral_yaml::parse(source).unwrap();
    let value: Spanned<&str> = from_document(&stream, 1).unwrap();
    assert_eq!(*value, "é🌍");
    assert_eq!(value.span.start(), source.find('\'').unwrap());
    assert_eq!(value.span.slice(source), Some("'é🌍'"));
    insta::assert_snapshot!(span_snapshot(source, [("document 2", value.span)]), @"
      |
    1 | ---
    2 | first
    3 | ...
    4 | ---
    5 | 'é🌍'
      | ^^^^^ document 2
    ");

    let bytes: Vec<u8> = "\u{feff}name: 'é🌍'"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let source = Source::from_bytes(&bytes).unwrap();
    let stream = source.parse().unwrap();
    let values: BTreeMap<String, Spanned<&str>> = from_stream(&stream).unwrap();
    let span = values["name"].span;
    assert_eq!(span.slice(source.text()), Some("'é🌍'"));
    let original = source.original_span(span).unwrap();
    let expected: Vec<u8> = "'é🌍'".encode_utf16().flat_map(u16::to_le_bytes).collect();
    assert_eq!(original.slice_bytes(&bytes), Some(expected.as_slice()));
    insta::assert_snapshot!(span_snapshot(source.text(), [("name (decoded UTF-16)", span)]), @"
      |
    1 | ﻿name: 'é🌍'
      |       ^^^^^ name (decoded UTF-16)
    ");
}

#[test]
fn serialization_emits_only_the_value() {
    let source = "{name: 'é🌍', counts: [1, 2], enabled: true}";
    let config: Spanned<Config<'_>> = from_str(source).unwrap();
    assert_eq!(
        to_string(&config).unwrap(),
        to_string(&config.value).unwrap()
    );

    let encoded = to_string(&config).unwrap();
    let decoded: Config<'_> = from_str(&encoded).unwrap();
    assert_eq!(decoded.name.value, config.name.value);
    assert_eq!(decoded.name.span.slice(&encoded), Some("\"é🌍\""));
    assert_eq!(to_string(&decoded).unwrap(), encoded);
}

#[test]
fn wrapping_preserves_validation_and_located_errors() {
    #[derive(Debug, Deserialize)]
    struct Recursive(#[allow(dead_code)] Spanned<Box<Recursive>>);

    for source in ["256", "!!bool yes", "[]"] {
        let plain = from_str::<u8>(source).unwrap_err();
        let wrapped = from_str::<Spanned<u8>>(source).unwrap_err();
        assert_eq!(plain, wrapped);
    }

    let source = "count: 256";
    let error = from_str::<BTreeMap<String, Spanned<u8>>>(source).unwrap_err();
    insta::assert_snapshot!(diagnostic_snapshot(source, error), @r#"
      |
    1 | count: 256
      |        ^^^ $["count"]: integer out of range
    "#);

    let errors = ["{a: 1, a: 2}", "&a [*a]", "!!str []"].map(|source| {
        let error = from_str::<Spanned<de::IgnoredAny>>(source).unwrap_err();
        span_snapshot(source, [(error.message, error.span.unwrap())])
    });
    insta::assert_snapshot!(errors.join("\n\n"), @"
      |
    1 | {a: 1, a: 2}
      |        ^ duplicate mapping key

      |
    1 | &a [*a]
      |     ^^ deserialization resource limit exceeded

      |
    1 | !!str []
      | ^^^^^^^^ tag does not match requested type
    ");

    let error = from_str::<Recursive>("null").unwrap_err();
    assert_eq!(error.message, "deserialization resource limit exceeded");
}

#[test]
fn buffered_serde_values_require_wrapping_outside_the_buffer() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum Untagged {
        Number(u8),
        Text(String),
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Buffered {
        Number(#[allow(dead_code)] Spanned<u8>),
    }

    #[derive(Debug, Deserialize)]
    struct Flattened<T> {
        #[serde(flatten)]
        fields: BTreeMap<String, T>,
    }

    let value: Spanned<Untagged> = from_str("42").unwrap();
    assert_eq!(*value, Untagged::Number(42));
    insta::assert_snapshot!(span_snapshot("42", [("Untagged::Number(42)", value.span)]), @"
      |
    1 | 42
      | ^^ Untagged::Number(42)
    ");

    assert!(from_str::<Buffered>("42").is_err());

    let source = "{count: 42}";
    let flattened: Spanned<Flattened<u8>> = from_str(source).unwrap();
    assert_eq!(flattened.fields["count"], 42);
    assert_eq!(flattened.span.slice(source), Some(source));
    assert!(from_str::<Flattened<Spanned<u8>>>(source).is_err());

    let error = Spanned::<u8>::deserialize(de::value::U8Deserializer::<de::value::Error>::new(42))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source-spanned value from astral-yaml-serde")
    );
    assert!(from_str::<Spanned<u8>>("{start: 0, end: 1, value: 42}").is_err());
}

#[test]
fn ignored_tagged_payloads_preserve_alias_spans() {
    #[derive(Debug, Deserialize)]
    enum Ignored {
        Value(Spanned<de::IgnoredAny>),
    }

    let source = "[&a !Value 42, *a, *a]";
    let values: Vec<YamlTag<Ignored>> = from_str(source).unwrap();
    let payloads = values.iter().enumerate().map(|(index, value)| {
        let Ignored::Value(payload) = &value.0;
        (format!("payload[{index}]"), payload.span)
    });
    insta::assert_snapshot!(span_snapshot(source, payloads), @"
      |
    1 | [&a !Value 42, *a, *a]
      |  ^^^^^^^^^^^^  ^^  ^^ payload[2]
      |  |             |
      |  |             payload[1]
      |  payload[0]
    ");
}

mod protocol {
    use serde::de::{self, Deserialize, SeqAccess, Visitor};
    use std::fmt;

    #[derive(Debug)]
    struct SpanProtocol;

    impl<'de> Deserialize<'de> for SpanProtocol {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct SpanVisitor;

            impl<'de> Visitor<'de> for SpanVisitor {
                type Value = SpanProtocol;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a spanned value")
                }

                fn visit_newtype_struct<D: de::Deserializer<'de>>(
                    self,
                    deserializer: D,
                ) -> Result<Self::Value, D::Error> {
                    deserializer.deserialize_newtype_struct("$astral_yaml::Spanned", self)
                }

                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                    assert_eq!(seq.size_hint(), Some(3));
                    let start: usize = seq.next_element::<usize>()?.unwrap();
                    assert_eq!(seq.size_hint(), Some(2));
                    let end: usize = seq.next_element::<usize>()?.unwrap();
                    assert_eq!(seq.size_hint(), Some(1));

                    if end - start > 1 {
                        let _: usize = seq.next_element::<usize>()?.unwrap();
                        assert_eq!(seq.size_hint(), Some(0));
                        assert!(seq.next_element::<usize>()?.is_none());
                        assert!(seq.next_element::<usize>()?.is_none());
                    }

                    Ok(SpanProtocol)
                }
            }

            deserializer.deserialize_newtype_struct("SpanProtocol", SpanVisitor)
        }
    }

    #[test]
    fn spanned_visitors_must_consume_the_value() {
        astral_yaml_serde::from_str::<SpanProtocol>("42 # comment").unwrap();

        let snapshots = ["1", "bad"].map(|source| {
            let error = astral_yaml_serde::from_str::<SpanProtocol>(source).unwrap_err();
            super::diagnostic_snapshot(source, error)
        });
        insta::assert_snapshot!(snapshots.join("\n\n"), @"
          |
        1 | 1
          | ^ $: metadata value was not fully consumed

          |
        1 | bad
          | ^^^ $: expected an integer
        ");

        let error = SpanProtocol::deserialize(
            de::value::U8Deserializer::<astral_yaml_serde::Error>::new(1),
        )
        .unwrap_err();
        assert!(error.message.contains("a spanned value"));
    }

    #[derive(Debug)]
    enum Wire {
        Number(u64),
        Text(&'static str),
        Owned(String),
        Sequence(Vec<Self>),
    }

    impl de::IntoDeserializer<'_, de::value::Error> for Wire {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self {
            self
        }
    }

    impl<'de> de::Deserializer<'de> for Wire {
        type Error = de::value::Error;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            match self {
                Self::Number(value) => visitor.visit_u64(value),
                Self::Text(value) => visitor.visit_borrowed_str(value),
                Self::Owned(value) => visitor.visit_string(value),
                Self::Sequence(values) => {
                    visitor.visit_seq(de::value::SeqDeserializer::new(values.into_iter()))
                }
            }
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    #[test]
    fn span_envelopes_require_ordered_offsets_and_exactly_one_value() {
        use Wire::{Number as N, Sequence as S, Text};
        use astral_yaml_serde::Spanned;

        let value = Spanned::<usize>::deserialize(S(vec![N(0), N(2), N(42)])).unwrap();
        assert_eq!(value.value, 42);
        assert_eq!(value.span, astral_yaml::Span::new(0, 2).unwrap());

        for (values, expected) in [
            (vec![Text("bad")], "invalid type"),
            (vec![N(0), Text("bad")], "invalid type"),
            (vec![], "invalid length 0"),
            (vec![N(0)], "invalid length 1"),
            (vec![N(0), N(1)], "invalid length 2"),
            (vec![N(0), N(1), N(42), N(99)], "invalid length 4"),
            (vec![N(2), N(1), N(42)], "span starts after its end"),
        ] {
            let error = Spanned::<usize>::deserialize(S(values)).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }

        for (values, expected) in [
            (vec![], "invalid length 0"),
            (vec![N(42)], "invalid length 1"),
            (vec![N(42), S(vec![]), N(99)], "invalid length 3"),
        ] {
            let error = astral_yaml_serde::Commented::<usize>::deserialize(S(values)).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }

        for placement in 0..4 {
            let envelope = S(vec![
                N(42),
                S(vec![S(vec![N(3), N(8), Text("note"), N(placement)])]),
            ]);
            let value = astral_yaml_serde::Commented::<usize>::deserialize(envelope).unwrap();
            assert_eq!(value.value, 42);
            assert_eq!(value.comments[0].comment.text, "note");
        }

        for (record, expected) in [
            (
                vec![N(8), N(3), Text("note"), N(0)],
                "comment span starts after its end",
            ),
            (
                vec![N(3), N(8), Text("note"), N(4)],
                "invalid comment placement",
            ),
            (
                vec![N(3), N(8), Wire::Owned("note".into()), N(0)],
                "borrowed string",
            ),
        ] {
            let error = astral_yaml_serde::Commented::<usize>::deserialize(S(vec![
                N(42),
                S(vec![S(record)]),
            ]))
            .unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }
}
