use test_case::test_case;

mod support;

use astral_yaml_dev::diagnostic_snapshot;
use astral_yaml_serde::{Error, Spanned, Tagged, Value, from_str, to_string};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::{self, SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU8;
use support::YamlTag;

// A YAML integer can exceed the range of the requested native type.
// https://yaml.org/spec/1.2.2/#10213-integer
#[test]
fn every_integer_width_checks_its_boundaries() {
    macro_rules! check {
        ($($ty:ty),*) => {
            $(
                for value in [<$ty>::MIN, 0, <$ty>::MAX] {
                    let yaml = to_string(&value).unwrap();
                    assert_eq!(from_str::<$ty>(&yaml).unwrap(), value);
                    let tree = from_str::<Value>(&yaml).unwrap();
                    assert_eq!(<$ty>::deserialize(&tree).unwrap(), value);
                }

                assert!(from_str::<$ty>(&format!("{}0", <$ty>::MAX)).is_err());
                assert!(from_str::<$ty>("true").is_err());
            )*
        };
    }

    check!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
}

#[test]
fn numeric_and_type_errors_are_located() {
    from_str::<()>("!!null null").unwrap();
    assert_eq!(from_str::<f64>("0o10").unwrap().to_bits(), 8.0f64.to_bits());

    // The core schema recognizes hexadecimal integers and decimal floats.
    // https://yaml.org/spec/1.2.2/#1032-tag-resolution
    let overflowing_integer = format!("0x{}", "f".repeat(257));
    let errors = ["text", "[]", "!!float 0xff", &overflowing_integer].map(|source| {
        let error = from_str::<f64>(source).unwrap_err();
        diagnostic_snapshot(source, error)
    });
    insta::assert_snapshot!(errors.join("\n\n"), @"
      |
    1 | text
      | ^^^^ $: expected a number

      |
    1 | []
      | ^^ $: expected a scalar

      |
    1 | !!float 0xff
      |         ^^^^ $: invalid float

      |
    1 | 0xffffffffffffffffffffffffffffffffffff...ffffffffffffffffffffffffffffffffffffff
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^...^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ $: float out of range
    ");

    let source = "{340282366920938463463374607431768211456: null}";
    let error = from_str::<de::IgnoredAny>(source).unwrap_err();
    assert_eq!(error.message, "integer key out of range");

    assert!(from_str::<Vec<u8>>("1").is_err());
    assert!(from_str::<BTreeMap<String, u8>>("1").is_err());

    for value in [0.0f32, -0.0, 42.0, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            from_str::<f32>(&to_string(&value).unwrap())
                .unwrap()
                .to_bits(),
            value.to_bits()
        );
    }

    assert!(
        from_str::<f32>(&to_string(&f32::NAN).unwrap())
            .unwrap()
            .is_nan()
    );
}

#[derive(Debug, PartialEq)]
struct Bytes(Vec<u8>);

impl Serialize for Bytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bytes or a sequence of bytes")
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Bytes, E> {
        Ok(Bytes(bytes.to_vec()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Bytes, A::Error> {
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }

        Ok(Bytes(bytes))
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

#[test]
fn bytes_borrow_or_decode_as_needed() {
    for source in [
        "abc",
        "'abc'",
        "\"a\\u0062c\"",
        "[97, 98, 99]",
        "!!str abc",
        "!!seq [97, 98, 99]",
        "! abc",
    ] {
        let bytes = from_str::<Bytes>(source).unwrap();
        assert_eq!(bytes, Bytes(b"abc".to_vec()));
        let tree = from_str::<Value>(source).unwrap();
        assert_eq!(Bytes::deserialize(&tree).unwrap(), bytes);
    }

    assert_eq!(
        from_str::<Tagged<Bytes>>("!Bytes 7").unwrap().value,
        Bytes(b"7".to_vec())
    );

    let source = "'abc'";
    let borrowed: &[u8] = from_str(source).unwrap();
    assert_eq!(borrowed, b"abc");
    assert!(std::ptr::eq(borrowed.as_ptr(), source[1..].as_ptr()));
    assert!(from_str::<&[u8]>("\"a\\nb\"").is_err());

    let value: Value = from_str(source).unwrap();
    assert_eq!(<&[u8]>::deserialize(&value).unwrap(), b"abc");

    for bytes in [vec![], vec![0, 1, 128, 255]] {
        let value = Bytes(bytes);
        assert_eq!(
            from_str::<Bytes>(&to_string(&value).unwrap()).unwrap(),
            value
        );
    }

    for source in ["[256]", "{}", "!!int 7"] {
        assert!(from_str::<Bytes>(source).is_err());
    }
}

#[derive(Debug)]
struct Keys<K>(Vec<K>);

impl<'de, K: Deserialize<'de>> Deserialize<'de> for Keys<K> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor<K>(PhantomData<K>);

        impl<'de, K: Deserialize<'de>> Visitor<'de> for KeyVisitor<K> {
            type Value = Keys<K>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a mapping")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut keys = Vec::new();
                while let Some((key, de::IgnoredAny)) = map.next_entry()? {
                    keys.push(key);
                }
                Ok(Keys(keys))
            }
        }

        deserializer.deserialize_map(KeyVisitor(PhantomData))
    }
}

#[test]
fn overlapping_character_and_byte_keys() {
    let source = r#"{1: a, "\u0031": b}"#;
    let error = from_str::<Keys<char>>(source).unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
    let error = from_str::<Keys<Bytes>>(source).unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
}

#[test_case("{1: a, 1.0: b}"; "integer to float")]
#[test_case("{9007199254740992: a, 9007199254740993: b}"; "integer rounding")]
#[test_case("{0: a, -0.0: b}"; "signed zero")]
fn overlapping_f64_keys(source: &str) {
    let error = from_str::<Keys<f64>>(source).unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
}

#[test]
fn floating_point_keys_compare_at_the_requested_precision() {
    let source = "{16777216: a, 16777217: b}";
    let error = from_str::<Keys<f32>>(source).unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");

    let Keys(keys) = from_str::<Keys<f64>>(source).unwrap();
    assert_eq!(keys, [16_777_216.0, 16_777_217.0]);
}

#[test]
fn overlapping_scalar_enum_keys() {
    #[derive(Debug, Deserialize)]
    enum Label {
        #[serde(rename = "1")]
        One,
        Ready,
    }

    let error = from_str::<Keys<Label>>("{1: a, '1': b}").unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
    let error = from_str::<Keys<YamlTag<Label>>>("{Ready: a, ? !Ready null : b}").unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
}

#[test]
fn tagged_keys_keep_their_tag_identity() {
    let Keys(keys) = from_str::<Keys<Tagged<String>>>("{!First 1: a, !Second 1: b}").unwrap();
    assert_eq!(
        keys.iter().map(|key| key.tag.as_str()).collect::<Vec<_>>(),
        ["!First", "!Second"]
    );
}

#[test]
fn spanned_keys_keep_their_span_identity() {
    let keys = from_str::<HashMap<Spanned<String>, u8>>("{1: 1, '1': 2}").unwrap();
    assert_eq!(keys.len(), 2);
}

#[test]
fn compound_keys_use_yaml_identity() {
    let source = "{? [1]: a, ? ['1']: b}";
    let Keys(keys) = from_str::<Keys<(String,)>>(source).unwrap();
    assert_eq!(keys, [("1".into(),), ("1".into(),)]);
}

#[test]
fn a_visitor_cannot_skip_an_overlapping_key() {
    #[derive(Debug)]
    struct RecoveringMap;

    impl<'de> Deserialize<'de> for RecoveringMap {
        fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct RecoveringVisitor;

            impl<'de> Visitor<'de> for RecoveringVisitor {
                type Value = RecoveringMap;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a mapping")
                }

                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    while !matches!(map.next_key::<String>(), Ok(None)) {
                        map.next_value::<de::IgnoredAny>()?;
                    }
                    Ok(RecoveringMap)
                }
            }

            deserializer.deserialize_map(RecoveringVisitor)
        }
    }

    let error = from_str::<RecoveringMap>("{1: a, '1': b}").unwrap_err();
    assert_eq!(error.message, "mapping keys overlap after conversion");
}

#[derive(Debug, Deserialize, PartialEq)]
struct Known {
    known: u8,
}

// The YAML numeric tags cover values outside the native destination ranges.
// https://yaml.org/spec/1.2.2/#10213-integer
// https://yaml.org/spec/1.2.2/#10214-floating-point
#[test_case("340282366920938463463374607431768211456"; "unsigned integer")]
#[test_case("-170141183460469231731687303715884105729"; "signed integer")]
#[test_case("1e309"; "float")]
#[test_case("!!int '0x100000000000000000000000000000000'"; "tagged radix integer")]
#[test_case("[&n !!float '-1e309', *n]"; "alias to tagged float")]
fn ignored_fields_accept_numbers_outside_native_ranges(extra: &str) {
    let source = format!("known: 1\nextra: {extra}\n");
    assert_eq!(from_str::<Known>(&source).unwrap(), Known { known: 1 });

    #[cfg(feature = "editable")]
    {
        let document = astral_yaml_serde::EditableYaml::parse(&source).unwrap();
        assert_eq!(document.source(), source);
    }
}

#[test]
fn ignored_fields_still_validate_their_values() {
    let value = from_str::<Known>("known: 1\nextra: [null, {a: true}]").unwrap();
    assert_eq!(value, Known { known: 1 });
    let tree = from_str::<Value>("known: 1\nextra: !Note text").unwrap();
    assert_eq!(Known::deserialize(&tree).unwrap(), value);

    // Mapping keys must be unique and explicit core tags must match their content.
    // The cyclic sequence exercises the Serde traversal limit.
    // https://yaml.org/spec/1.2.2/#3213-node-comparison
    // https://yaml.org/spec/1.2.2/#333-recognized-and-valid-tags
    for source in [
        "known: 1\nextra: {a: 1, a: 2}",
        "known: 1\nextra: !!int invalid",
        "known: 1\nextra: !!float 0xff",
        "known: 1\nextra: !!str []",
        "known: 1\nextra: &a [*a]",
    ] {
        assert!(from_str::<Known>(source).is_err(), "{source}");
    }
}

#[derive(Debug, Deserialize, PartialEq)]
enum Variant {
    Unit,
    Tuple(u8, u8),
    Struct { field: u8 },
}

#[test]
fn enum_payloads_and_aliases() {
    assert_eq!(
        support::from_tag::<Variant>("{Unit: null}").unwrap(),
        Variant::Unit
    );
    assert_eq!(from_str::<Variant>("! Unit").unwrap(), Variant::Unit);
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    assert_eq!(
        from_str::<Vec<YamlTag<Variant>>>("[&a !Unit, *a]").unwrap(),
        [YamlTag(Variant::Unit), YamlTag(Variant::Unit)]
    );

    for source in [
        "Tuple",
        "Struct",
        "{Unit: 1}",
        "!Unit []",
        "{[a]: b}",
        "!!bool Unit",
        "!!str {Unit: null}",
    ] {
        assert!(from_str::<Variant>(source).is_err(), "{source}");
    }
}

#[test_case("[1, 0, 3]", from_str::<Vec<NonZeroU8>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @"
  |
1 | [1, 0, 3]
  |     ^ $[1]: invalid value: integer `0`, expected a nonzero u8
"); "sequence item")]
#[test_case("{1: a, 0: b}", from_str::<BTreeMap<NonZeroU8, String>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @"
  |
1 | {1: a, 0: b}
  |        ^ $.key_at(1): invalid value: integer `0`, expected a nonzero u8
"); "mapping key")]
#[test_case("{limit: [1, 0]}", from_str::<BTreeMap<String, Vec<NonZeroU8>>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @r#"
  |
1 | {limit: [1, 0]}
  |             ^ $["limit"][1]: invalid value: integer `0`, expected a nonzero u8
"#); "nested value")]
#[test_case("{limit: 0}", from_str::<BTreeMap<String, NonZeroU8>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @r#"
  |
1 | {limit: 0}
  |         ^ $["limit"]: invalid value: integer `0`, expected a nonzero u8
"#); "mapping value")]
#[test_case("{Ok: 0}", from_str::<Result<NonZeroU8, String>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @r#"
  |
1 | {Ok: 0}
  |      ^ $["Ok"]: invalid value: integer `0`, expected a nonzero u8
"#); "variant payload")]
#[test_case("{Other: 1}", from_str::<Result<NonZeroU8, String>> => with |diagnostic| insta::assert_snapshot!(diagnostic, @r#"
  |
1 | {Other: 1}
  |  ^^^^^ $["Other"]: unknown variant `Other`, expected `Ok` or `Err`
"#); "unknown variant")]
#[test_case("[&n 0, *n]", from_str::<(u8, NonZeroU8)> => with |diagnostic| insta::assert_snapshot!(diagnostic, @"
  |
1 | [&n 0, *n]
  |        ^^ $[1]: invalid value: integer `0`, expected a nonzero u8
"); "alias use")]
fn visitor_errors_point_to_the_rejected_value<'de, T: fmt::Debug>(
    source: &'de str,
    parse: fn(&'de str) -> Result<T, Error>,
) -> String {
    diagnostic_snapshot(source, parse(source).unwrap_err())
}

// Custom visitors exercise consumption and access-order contracts.
#[derive(Debug)]
struct ProtocolMap;

impl<'de> Deserialize<'de> for ProtocolMap {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = ProtocolMap;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a mapping")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                if map.size_hint() == Some(0) {
                    map.next_value::<u8>()?;
                }

                while let Some(key) = map.next_key::<String>()? {
                    if key == "partial" {
                        return Ok(ProtocolMap);
                    }

                    if key == "key" {
                        map.next_key::<String>()?;
                    }

                    map.next_value::<u8>()?;
                }

                Ok(ProtocolMap)
            }
        }

        deserializer.deserialize_map(MapVisitor)
    }
}

#[test_case("{normal: 1}" => Ok(()); "consumed")]
#[test_case(
    "{}"
    => with |result| insta::assert_snapshot!(Result::unwrap_err(result), @"
      |
    1 | {}
      | ^^ $: mapping value requested before key
    ");
    "value before key"
)]
#[test_case(
    "{key: 1}"
    => with |result| insta::assert_snapshot!(Result::unwrap_err(result), @"
      |
    1 | {key: 1}
      | ^^^^^^^^ $: mapping value was not consumed
    ");
    "unconsumed value"
)]
#[test_case(
    "{partial: 1}"
    => with |result| insta::assert_snapshot!(Result::unwrap_err(result), @"
      |
    1 | {partial: 1}
      | ^^^^^^^^^^^^ $: mapping was not fully consumed
    ");
    "unconsumed mapping"
)]
#[test_case(
    "[]"
    => with |result| insta::assert_snapshot!(Result::unwrap_err(result), @"
      |
    1 | []
      | ^^ $: expected a mapping
    ");
    "wrong type"
)]
// https://yaml.org/spec/1.2.2/#3213-node-comparison
#[test_case(
    "{a: 1, a: 2}"
    => with |result| insta::assert_snapshot!(Result::unwrap_err(result), @"
      |
    1 | {a: 1, a: 2}
      |        ^ $.key_at(1): duplicate mapping key
    ");
    "duplicate key"
)]
fn map_visitors_must_follow_the_protocol(source: &str) -> Result<(), String> {
    from_str::<ProtocolMap>(source)
        .map(|_| ())
        .map_err(|error| diagnostic_snapshot(source, error))
}

#[derive(Debug)]
struct ProtocolSeq;

impl<'de> Deserialize<'de> for ProtocolSeq {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SeqVisitor;

        impl<'de> Visitor<'de> for SeqVisitor {
            type Value = ProtocolSeq;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while let Some(byte) = seq.next_element::<u8>()? {
                    if byte == 255 {
                        break;
                    }
                }

                Ok(ProtocolSeq)
            }
        }

        deserializer.deserialize_seq(SeqVisitor)
    }
}

#[test]
fn sequence_visitors_must_consume_all_elements() {
    from_str::<ProtocolSeq>("[1, 2, 3]").unwrap();
    assert_eq!(
        from_str::<ProtocolSeq>("[255, 0]").unwrap_err().message,
        "sequence was not fully consumed"
    );
    assert_eq!(
        from_str::<ProtocolSeq>("{}").unwrap_err().message,
        "expected a sequence"
    );
}

struct BadSerializer(u8);

impl Serialize for BadSerializer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0 == 3 {
            return Err(ser::Error::custom("custom failure"));
        }

        let mut map = serializer.serialize_map(Some(1))?;
        if self.0 == 4 {
            map.serialize_entry("a", &1)?;
        } else if self.0 == 0 {
            map.serialize_value(&1)?;
        } else {
            map.serialize_key("a")?;
            if self.0 == 1 {
                map.serialize_key("b")?;
            }
        }

        map.end()
    }
}

#[test]
fn map_serializers_must_follow_the_protocol() {
    assert_eq!(to_string(&BadSerializer(4)).unwrap(), "{? \"a\": 1}\n");

    for (mode, expected) in [
        (0, "mapping value has no key"),
        (1, "mapping key has no value"),
        (2, "mapping key has no value"),
        (3, "custom failure"),
    ] {
        let error = to_string(&BadSerializer(mode)).unwrap_err();
        assert_eq!(error.to_string(), expected);
        assert!(error.span.is_none());
        let _: &dyn std::error::Error = &error;
    }

    let error: Error = de::Error::custom("custom deserialization failure");
    assert_eq!(error.message, "custom deserialization failure");
}

#[derive(Debug)]
struct Recursive;

impl<'de> Deserialize<'de> for Recursive {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NewtypeVisitor;

        impl<'de> Visitor<'de> for NewtypeVisitor {
            type Value = Recursive;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a recursive newtype")
            }

            fn visit_newtype_struct<D: de::Deserializer<'de>>(
                self,
                de: D,
            ) -> Result<Self::Value, D::Error> {
                Recursive::deserialize(de)
            }
        }

        deserializer.deserialize_newtype_struct("Recursive", NewtypeVisitor)
    }
}

#[test]
fn recursive_newtype_visitors_are_bounded() {
    assert_eq!(
        from_str::<Recursive>("null").unwrap_err().message,
        "deserialization resource limit exceeded"
    );
}

enum FailedField {
    Element,
    Key,
    Value,
}

impl Serialize for FailedField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let invalid = [Tagged {
            tag: "tag:yaml.org,2002:bool".into(),
            value: 3,
        }];
        match self {
            Self::Element => {
                let mut seq = serializer.serialize_seq(None)?;
                if seq.serialize_element(&invalid).is_err() {
                    let _ = seq.serialize_element(&0);
                }
                seq.end()
            }
            Self::Key => {
                let mut map = serializer.serialize_map(None)?;
                if map.serialize_key(&invalid).is_err() {
                    let _ = map.serialize_entry("fallback", &0);
                }
                map.end()
            }
            Self::Value => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("key")?;
                if map.serialize_value(&invalid).is_err() {
                    let _ = map.serialize_value(&0);
                }
                map.end()
            }
        }
    }
}

#[test_case(&FailedField::Element; "sequence element")]
#[test_case(&FailedField::Key; "mapping key")]
#[test_case(&FailedField::Value; "mapping value")]
fn handled_field_errors_cannot_publish_partial_yaml(field: &FailedField) {
    assert_eq!(
        to_string(field).unwrap_err().message,
        "collection serialization already failed"
    );
}
