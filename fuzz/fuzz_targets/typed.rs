#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_serde::{Spanned, from_str, from_stream, to_string};
use libfuzzer_sys::fuzz_target;
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeOwned, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};

#[derive(Deserialize, Serialize)]
struct Newtype(u64);

#[derive(Deserialize, Serialize)]
struct Pair(i16, String);

#[derive(Deserialize, Serialize)]
struct Unit;

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

fn check<T: DeserializeOwned + Serialize>(source: &str, stream: &astral_yaml::Stream<'_>) {
    match from_stream::<Spanned<T>>(stream) {
        Ok(value) => {
            let root = stream.node(stream.documents()[0].root).unwrap();
            assert_eq!(value.span, root.span);
            assert!(value.span.slice(source).is_some());

            if let Ok(encoded) = to_string(&value) {
                let decoded: Spanned<T> = from_str(&encoded).unwrap();

                // Canonical output also compares NaN values and the sign of zero.
                assert_eq!(to_string(&decoded).unwrap(), encoded);
            }
        }
        Err(error) => assert!(
            error.span.is_some_and(|span| span.slice(source).is_some()),
            "{error:?}"
        ),
    }
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

    match selector % 28 {
        0 => check::<i8>(source, &stream),
        1 => check::<i16>(source, &stream),
        2 => check::<i32>(source, &stream),
        3 => check::<i64>(source, &stream),
        4 => check::<i128>(source, &stream),
        5 => check::<u8>(source, &stream),
        6 => check::<u16>(source, &stream),
        7 => check::<u32>(source, &stream),
        8 => check::<u64>(source, &stream),
        9 => check::<u128>(source, &stream),
        10 => check::<f32>(source, &stream),
        11 => check::<f64>(source, &stream),
        12 => check::<bool>(source, &stream),
        13 => check::<char>(source, &stream),
        14 => check::<String>(source, &stream),
        15 => check::<Option<Vec<u64>>>(source, &stream),
        16 => check::<(i8, String)>(source, &stream),
        17 => check::<Newtype>(source, &stream),
        18 => check::<Pair>(source, &stream),
        19 => check::<Unit>(source, &stream),
        20 => check::<Kind<String>>(source, &stream),
        21 => check::<BTreeMap<String, u8>>(source, &stream),
        22 => check::<Bytes>(source, &stream),
        23 => check::<Configuration>(source, &stream),
        24 => check::<Vec<Spanned<u64>>>(source, &stream),
        25 => check::<BTreeMap<String, Spanned<String>>>(source, &stream),
        26 => check::<Spanned<Option<String>>>(source, &stream),
        _ => check::<Kind<Spanned<String>>>(source, &stream),
    }
});
