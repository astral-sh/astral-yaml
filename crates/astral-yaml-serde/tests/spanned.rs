mod support;

use astral_yaml::{Source, Span};
use astral_yaml_serde::{Spanned, from_document, from_str, from_stream, to_string};
use serde::{Deserialize, Serialize, de};
use std::collections::{BTreeMap, HashMap};
use support::{YamlTag, from_tag};

#[test]
fn scalars_retain_their_lexical_spans() {
    for (source, value, lexeme) in [
        ("  hello # comment\n", "hello", "hello"),
        ("'héllo'", "héllo", "'héllo'"),
        ("\"a\\nb\"", "a\nb", "\"a\\nb\""),
        ("&name 42", "42", "&name 42"),
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
        let spanned: Spanned<'_, String> = from_str(source).unwrap();
        assert_eq!(*spanned, value);
        assert_eq!(spanned.span.slice(source), Some(lexeme));
    }

    let number: Spanned<'_, u16> = from_str("0xff").unwrap();
    assert_eq!(*number, 255);
    assert_eq!(number.span, Span::new(0, 4).unwrap());

    let mut text: Spanned<'_, String> = from_str("hello").unwrap();
    text.push_str(" world");
    assert_eq!(text.span, Span::new(0, 5).unwrap());
    assert_eq!(text.into_inner(), "hello world");
}

#[derive(Debug, Deserialize, Serialize)]
struct Config<'a> {
    #[serde(borrow)]
    name: Spanned<'a, &'a str>,
    #[serde(borrow)]
    counts: Spanned<'a, Vec<Spanned<'a, u8>>>,
    enabled: bool,
    #[serde(borrow)]
    optional: Option<Spanned<'a, String>>,
}

#[test]
fn wrappers_compose_with_structs_collections_and_borrowing() {
    let source = "{name: 'é🌍', counts: [1, 2], enabled: true}";
    let config: Spanned<'_, Config<'_>> = from_str(source).unwrap();
    assert_eq!(config.span.slice(source), Some(source));
    assert_eq!(*config.name, "é🌍");
    assert_eq!(config.name.span.slice(source), Some("'é🌍'"));
    assert!(std::ptr::eq(config.name.as_ptr(), source[8..].as_ptr()));
    assert_eq!(config.counts.span.slice(source), Some("[1, 2]"));
    assert_eq!(
        config
            .counts
            .iter()
            .map(|value| **value)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(config.counts[1].span.slice(source), Some("2"));
    assert!(config.enabled);
    assert!(config.optional.is_none());

    let source = "{é: [3, 4]}";
    let mapping: HashMap<Spanned<'_, String>, Spanned<'_, Vec<Spanned<'_, u8>>>> =
        from_str(source).unwrap();
    let (key, values) = mapping.iter().next().unwrap();
    assert_eq!(key.value, "é");
    assert_eq!(key.span, Span::new(1, 3).unwrap());
    assert_eq!(values.span.slice(source), Some("[3, 4]"));
    assert_eq!(values[0].span.slice(source), Some("3"));

    let bytes: Spanned<'_, &[u8]> = from_str("'abc'").unwrap();
    assert_eq!(*bytes, b"abc");
    assert_eq!(bytes.span, Span::new(0, 5).unwrap());
}

#[test]
fn options_and_nested_wrappers_keep_the_same_node_span() {
    for source in ["null", "~", "---\n"] {
        let value: Spanned<'_, Option<u8>> = from_str(source).unwrap();
        assert!(value.is_none());
        assert_eq!(
            value.span.slice(source),
            Some(source.trim_start_matches("---\n"))
        );
        assert!(
            from_str::<Option<Spanned<'_, u8>>>(source)
                .unwrap()
                .is_none()
        );
    }

    let value: Option<Spanned<'_, u8>> = from_str("42").unwrap();
    assert_eq!(value.unwrap().value, 42);

    let value: Spanned<'_, Spanned<'_, Option<u8>>> = from_str("42").unwrap();
    assert_eq!(value.span, value.value.span);
    assert_eq!(**value, Some(42));

    let source = "{empty: }";
    let mapping: BTreeMap<String, Spanned<'_, Option<u8>>> = from_str(source).unwrap();
    assert!(mapping["empty"].is_none());
    assert!(mapping["empty"].span.is_empty());
    assert_eq!(mapping["empty"].span.slice(source), Some(""));
}

#[test]
fn aliases_use_the_occurrence_span_and_children_use_definition_spans() {
    let source = "[&number 42, *number]";
    let values: Vec<Spanned<'_, u8>> = from_str(source).unwrap();
    assert_eq!(values[0].value, values[1].value);
    assert_eq!(values[0].span.slice(source), Some("&number 42"));
    assert_eq!(values[1].span.slice(source), Some("*number"));

    let source = "[&items [1, 2], *items]";
    let values: Vec<Spanned<'_, Vec<Spanned<'_, u8>>>> = from_str(source).unwrap();
    assert_eq!(values[0].value, values[1].value);
    assert_eq!(values[1].span.slice(source), Some("*items"));
    assert_eq!(values[1][0].span.slice(source), Some("1"));
}

#[derive(Debug, Deserialize, PartialEq)]
struct Newtype<'a>(#[serde(borrow)] Option<Spanned<'a, u8>>);

#[derive(Debug, Deserialize, PartialEq)]
enum Kind<'a> {
    Unit,
    Newtype(#[serde(borrow)] Spanned<'a, Newtype<'a>>),
    Tuple(#[serde(borrow)] Spanned<'a, u8>, bool),
    Struct {
        #[serde(borrow)]
        value: Spanned<'a, u8>,
    },
    Map(#[serde(borrow)] Spanned<'a, BTreeMap<String, Spanned<'a, u8>>>),
}

#[test]
fn enum_tags_are_consumed_once_through_transparent_wrappers() {
    for source in ["Unit", "!Unit", "{Unit: null}"] {
        let value: Spanned<'_, Kind> = from_tag(source).unwrap();
        assert_eq!(*value, Kind::Unit);
        assert_eq!(value.span.slice(source), Some(source));
    }

    for source in ["!Newtype 42", "{Newtype: 42}"] {
        let value: Spanned<'_, Kind> = from_tag(source).unwrap();
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
        let value: Spanned<'_, Kind> = from_tag(source).unwrap();
        let number = match value.value {
            Kind::Tuple(number, false) | Kind::Struct { value: number } => number,
            Kind::Map(mut values) => values.remove("value").unwrap(),
            _ => panic!("unexpected enum value"),
        };

        assert_eq!(*number, 42);
        assert_eq!(number.span.slice(source), Some("42"));
    }

    let source = "[&value !Newtype 42, *value]";
    let values: Vec<Spanned<'_, Kind>> = from_str::<Vec<YamlTag<Spanned<'_, Kind>>>>(source)
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
            comments: Vec::new(),
        }
    );
}

#[test]
fn document_offsets_and_encoded_sources_use_the_core_span_convention() {
    let source = "---\nfirst\n...\n---\n'é🌍'\n";
    let stream = astral_yaml::parse(source).unwrap();
    let value: Spanned<'_, &str> = from_document(&stream, 1).unwrap();
    assert_eq!(*value, "é🌍");
    assert_eq!(value.span.start(), source.find('\'').unwrap());
    assert_eq!(value.span.slice(source), Some("'é🌍'"));

    let bytes: Vec<u8> = "\u{feff}name: 'é🌍'"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let source = Source::from_bytes(&bytes).unwrap();
    let stream = source.parse().unwrap();
    let values: BTreeMap<String, Spanned<'_, &str>> = from_stream(&stream).unwrap();
    let span = values["name"].span;
    assert_eq!(span.slice(source.text()), Some("'é🌍'"));
    let original = source.original_span(span).unwrap();
    let expected: Vec<u8> = "'é🌍'".encode_utf16().flat_map(u16::to_le_bytes).collect();
    assert_eq!(original.slice_bytes(&bytes), Some(expected.as_slice()));
}

#[test]
fn serialization_emits_only_the_value() {
    let source = "{name: 'é🌍', counts: [1, 2], enabled: true}";
    let config: Spanned<'_, Config<'_>> = from_str(source).unwrap();
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
    struct Recursive<'a>(
        #[allow(dead_code)]
        #[serde(borrow)]
        Spanned<'a, Box<Recursive<'a>>>,
    );

    for source in ["256", "!!bool yes", "[]"] {
        let plain = from_str::<u8>(source).unwrap_err();
        let wrapped = from_str::<Spanned<'_, u8>>(source).unwrap_err();
        assert_eq!(plain, wrapped);
    }

    let error = from_str::<BTreeMap<String, Spanned<'_, u8>>>("count: 256").unwrap_err();
    assert_eq!(error.span.unwrap().slice("count: 256"), Some("256"));

    for source in ["{a: 1, a: 2}", "&a [*a]", "!!str []"] {
        let error = from_str::<Spanned<'_, de::IgnoredAny>>(source).unwrap_err();
        assert!(error.span.unwrap().slice(source).is_some());
    }

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
    enum Buffered<'a> {
        Number(
            #[allow(dead_code)]
            #[serde(borrow)]
            Spanned<'a, u8>,
        ),
    }

    #[derive(Debug, Deserialize)]
    struct Flattened<T> {
        #[serde(flatten)]
        fields: BTreeMap<String, T>,
    }

    let value: Spanned<'_, Untagged> = from_str("42").unwrap();
    assert_eq!(*value, Untagged::Number(42));
    assert_eq!(value.span, Span::new(0, 2).unwrap());

    assert!(from_str::<Buffered>("42").is_err());

    let source = "{count: 42}";
    let flattened: Spanned<'_, Flattened<u8>> = from_str(source).unwrap();
    assert_eq!(flattened.fields["count"], 42);
    assert_eq!(flattened.span.slice(source), Some(source));
    assert!(from_str::<Flattened<Spanned<'_, u8>>>(source).is_err());

    let error = Spanned::<u8>::deserialize(de::value::U8Deserializer::<de::value::Error>::new(42))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source-spanned value from astral-yaml-serde")
    );
    assert!(from_str::<Spanned<'_, u8>>("{start: 0, end: 1, value: 42}").is_err());
}

#[test]
fn ignored_tagged_payloads_preserve_alias_spans() {
    #[derive(Debug, Deserialize)]
    enum Ignored<'a> {
        Value(#[serde(borrow)] Spanned<'a, de::IgnoredAny>),
    }

    let source = "[&a !Value 42, *a]";
    let values: Vec<YamlTag<Ignored>> = from_str(source).unwrap();
    let Ignored::Value(value) = &values[1].0;
    assert_eq!(value.span.slice(source), Some("*a"));
}

mod protocol {
    use serde::de::{self, Deserialize, SeqAccess, Visitor};
    use std::fmt;

    enum Field {
        Number,
        Comments,
    }

    impl<'de> de::DeserializeSeed<'de> for Field {
        type Value = usize;

        fn deserialize<D: de::Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            match self {
                Self::Number => usize::deserialize(deserializer),
                Self::Comments => {
                    de::IgnoredAny::deserialize(deserializer)?;
                    Ok(0)
                }
            }
        }
    }

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
                    assert_eq!(seq.size_hint(), Some(4));
                    let start: usize = seq.next_element_seed(Field::Number)?.unwrap();
                    assert_eq!(seq.size_hint(), Some(3));
                    let end: usize = seq.next_element_seed(Field::Number)?.unwrap();
                    assert_eq!(seq.size_hint(), Some(2));

                    if end - start > 1 {
                        let _: usize = seq.next_element_seed(Field::Number)?.unwrap();
                        assert_eq!(seq.size_hint(), Some(1));
                        let _ = seq.next_element_seed(Field::Comments)?.unwrap();
                        assert_eq!(seq.size_hint(), Some(0));
                        assert!(seq.next_element_seed(Field::Number)?.is_none());
                        assert!(seq.next_element_seed(Field::Number)?.is_none());
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

        let error = astral_yaml_serde::from_str::<SpanProtocol>("1").unwrap_err();
        assert_eq!(error.message, "spanned value was not fully consumed");
        assert_eq!(error.span.unwrap().slice("1"), Some("1"));

        let error = astral_yaml_serde::from_str::<SpanProtocol>("bad").unwrap_err();
        assert_eq!(error.message, "expected an integer");
        assert_eq!(error.span.unwrap().slice("bad"), Some("bad"));

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

        for (values, expected) in [
            (vec![], "invalid length 0"),
            (vec![N(0)], "invalid length 1"),
            (vec![N(0), N(1)], "invalid length 2"),
            (vec![N(0), N(1), N(42)], "invalid length 3"),
            (
                vec![N(0), N(1), N(42), S(vec![]), N(99)],
                "invalid length 5",
            ),
            (vec![N(2), N(1), N(42)], "span starts after its end"),
        ] {
            let error = Spanned::<usize>::deserialize(S(values)).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }

        for placement in 0..4 {
            let envelope = S(vec![
                N(0),
                N(2),
                N(42),
                S(vec![S(vec![N(3), N(8), Text("note"), N(placement)])]),
            ]);
            let value = Spanned::<usize>::deserialize(envelope).unwrap();
            assert_eq!(value.value, 42);
            assert_eq!(value.span, astral_yaml::Span::new(0, 2).unwrap());
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
            let error =
                Spanned::<usize>::deserialize(S(vec![N(0), N(2), N(42), S(vec![S(record)])]))
                    .unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }
}
