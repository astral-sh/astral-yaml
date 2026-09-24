use astral_yaml_serde::{Error, from_str, to_string};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::{self, SerializeMap},
};
use std::{collections::BTreeMap, fmt};

#[test]
fn every_integer_width_checks_its_boundaries() {
    macro_rules! check {
        ($($ty:ty),*) => {
            $(
                for value in [<$ty>::MIN, 0, <$ty>::MAX] {
                    let yaml = to_string(&value).unwrap();
                    assert_eq!(from_str::<$ty>(&yaml).unwrap(), value);
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

    for source in [
        "text",
        "[]",
        "!!float 0xff",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    ] {
        let error = from_str::<f64>(source).unwrap_err();
        assert!(error.span.is_some(), "{source}");
    }

    for source in [
        "340282366920938463463374607431768211456",
        "-170141183460469231731687303715884105729",
    ] {
        assert!(from_str::<de::IgnoredAny>(source).is_err());
        assert!(from_str::<de::IgnoredAny>(&format!("{{{source}: null}}")).is_err());
    }

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
    for source in ["abc", "'abc'", "\"a\\u0062c\"", "[97, 98, 99]"] {
        assert_eq!(from_str::<Bytes>(source).unwrap(), Bytes(b"abc".to_vec()));
    }

    let source = "'abc'";
    let borrowed: &[u8] = from_str(source).unwrap();
    assert_eq!(borrowed, b"abc");
    assert!(std::ptr::eq(borrowed.as_ptr(), source[1..].as_ptr()));
    assert!(from_str::<&[u8]>("\"a\\nb\"").is_err());

    for bytes in [vec![], vec![0, 1, 128, 255]] {
        let value = Bytes(bytes);
        assert_eq!(
            from_str::<Bytes>(&to_string(&value).unwrap()).unwrap(),
            value
        );
    }

    for source in ["[256]", "{}"] {
        assert!(from_str::<Bytes>(source).is_err());
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Known {
    known: u8,
}

#[test]
fn ignored_fields_still_validate_their_values() {
    let value = from_str::<Known>("known: 1\nextra: [null, {a: true}]").unwrap();
    assert_eq!(value, Known { known: 1 });

    for source in [
        "known: 1\nextra: {a: 1, a: 2}",
        "known: 1\nextra: !!int invalid",
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
    assert_eq!(from_str::<Variant>("{Unit: null}").unwrap(), Variant::Unit);
    assert_eq!(from_str::<Variant>("! Unit").unwrap(), Variant::Unit);
    assert_eq!(
        from_str::<Vec<Variant>>("[&a !Unit, *a]").unwrap(),
        [Variant::Unit, Variant::Unit]
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

#[test]
fn map_visitors_must_follow_the_protocol() {
    from_str::<ProtocolMap>("{normal: 1}").unwrap();

    for (source, expected) in [
        ("{}", "mapping value requested before key"),
        ("{key: 1}", "mapping value was not consumed"),
        ("{partial: 1}", "mapping was not fully consumed"),
        ("[]", "expected a mapping"),
        ("{a: 1, a: 2}", "duplicate mapping key"),
    ] {
        let error = from_str::<ProtocolMap>(source).unwrap_err();
        assert_eq!(error.message, expected);
        assert!(error.span.is_some());
    }
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
