#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_serde::{Spanned, from_str, from_stream, to_string};
use libfuzzer_sys::fuzz_target;
use serde::{
    Deserialize, Serialize,
    de::{self, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};

#[derive(Deserialize, Serialize)]
struct Newtype(u64);

#[derive(Deserialize, Serialize)]
struct Pair(i16, String);

#[derive(Deserialize, Serialize)]
struct Unit;

// Exercise comment indexing for every parsed syntax shape, including cyclic
// aliases and tags that an ordinary application value would reject.
#[derive(Serialize)]
struct SyntaxOnly;

impl<'de> Deserialize<'de> for SyntaxOnly {
    fn deserialize<D: de::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Ok(Self)
    }
}

#[derive(Deserialize, Serialize)]
enum Kind<T> {
    Unit,
    Newtype(T),
    Tuple(u8, bool),
    Struct { value: i32 },
}

#[derive(Deserialize, Serialize)]
struct Configuration {
    #[serde(default)]
    name: String,
    #[serde(default)]
    count: u32,
}

struct Bytes(Vec<u8>);

impl Serialize for Bytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Bytes;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bytes")
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

        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

// Expand each type twice so borrowed wrappers can use independent input lifetimes.
macro_rules! check {
    ($ty:ty, $source:expr, $stream:expr) => {
        match from_stream::<Spanned<'_, $ty>>($stream) {
            Ok(value) => {
                let root = $stream.node($stream.documents()[0].root).unwrap();
                assert_eq!(value.span, root.span);
                assert!(value.span.slice($source).is_some());
                assert_eq!(value.comments.len(), $stream.comments().len());
                assert!(
                    value.comments.windows(2).all(|pair| {
                        pair[0].comment.span.start() < pair[1].comment.span.start()
                    })
                );

                for (attached, original) in value.comments.iter().zip($stream.comments()) {
                    let comment = &attached.comment;
                    assert_eq!(comment, original);
                    assert_eq!(
                        comment.span.slice($source).unwrap(),
                        format!("#{}", comment.text)
                    );
                    assert!(std::ptr::eq(
                        comment.text.as_ptr(),
                        $source[comment.span.start() + 1..].as_ptr()
                    ));
                }

                if let Ok(encoded) = to_string(&value) {
                    let decoded: Spanned<'_, $ty> = from_str(&encoded).unwrap();
                    assert!(decoded.comments.is_empty());

                    // Canonical output also compares NaN values and the sign of zero.
                    assert_eq!(to_string(&decoded).unwrap(), encoded);
                }
            }
            Err(error) => assert!(
                error.span.is_some_and(|span| span.slice($source).is_some()),
                "{error:?}"
            ),
        }
    };
}

fuzz_target!(|data: &[u8]| {
    let Some((&selector, input)) = data.split_first() else {
        return;
    };

    let Ok(source) = std::str::from_utf8(input) else {
        return;
    };

    let Ok(stream) = astral_yaml::parse(source) else {
        return;
    };

    match selector % 29 {
        0 => check!(i8, source, &stream),
        1 => check!(i16, source, &stream),
        2 => check!(i32, source, &stream),
        3 => check!(i64, source, &stream),
        4 => check!(i128, source, &stream),
        5 => check!(u8, source, &stream),
        6 => check!(u16, source, &stream),
        7 => check!(u32, source, &stream),
        8 => check!(u64, source, &stream),
        9 => check!(u128, source, &stream),
        10 => check!(f32, source, &stream),
        11 => check!(f64, source, &stream),
        12 => check!(bool, source, &stream),
        13 => check!(char, source, &stream),
        14 => check!(String, source, &stream),
        15 => check!(Option<Vec<u64>>, source, &stream),
        16 => check!((i8, String), source, &stream),
        17 => check!(Newtype, source, &stream),
        18 => check!(Pair, source, &stream),
        19 => check!(Unit, source, &stream),
        20 => check!(Kind<String>, source, &stream),
        21 => check!(BTreeMap<String, u8>, source, &stream),
        22 => check!(Bytes, source, &stream),
        23 => check!(Configuration, source, &stream),
        24 => check!(Vec<Spanned<'_, u64>>, source, &stream),
        25 => check!(BTreeMap<String, Spanned<'_, String>>, source, &stream),
        26 => check!(Spanned<'_, Option<String>>, source, &stream),
        27 => check!(Kind<Spanned<'_, String>>, source, &stream),
        _ => check!(SyntaxOnly, source, &stream),
    }
});
