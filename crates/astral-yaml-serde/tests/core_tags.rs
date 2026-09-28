mod support;

use astral_yaml_dev::Value;
use astral_yaml_serde::{Spanned, Tagged, from_str};
use serde::{
    Deserialize, Deserializer,
    de::{self, IgnoredAny, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, fmt::Write};
use support::from_tag;

#[derive(Deserialize)]
struct Newtype<T>(T);

#[derive(Deserialize)]
struct Unit;

#[derive(Deserialize)]
struct Tuple(u8, u8);

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "7")]
    _value: (),
}

#[derive(Debug, Deserialize, PartialEq)]
enum Choice {
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "true")]
    True,
    #[serde(rename = "~")]
    Null,
}

struct ByteBuf(Vec<u8>);

impl<'de> Deserialize<'de> for ByteBuf {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = ByteBuf;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("bytes or a sequence of bytes")
            }

            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<ByteBuf, E> {
                Ok(ByteBuf(value.to_vec()))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<ByteBuf, A::Error> {
                let mut bytes = Vec::new();

                while let Some(byte) = sequence.next_element()? {
                    bytes.push(byte);
                }

                Ok(ByteBuf(bytes))
            }
        }

        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

macro_rules! destination_rows {
    ($output:ident, $sources:ident; $($label:literal => $ty:ty),+ $(,)?) => {
        $(
            write!($output, "{:<18}", $label).unwrap();

            for source in $sources {
                let accepted = from_str::<$ty>(source).is_ok();
                assert_eq!(
                    from_str::<Tagged<$ty>>(source).is_ok(),
                    accepted,
                    "{source}"
                );
                assert_eq!(
                    from_str::<Spanned<'_, $ty>>(source).is_ok(),
                    accepted,
                    "{source}"
                );
                assert_eq!(
                    from_str::<Newtype<$ty>>(source).is_ok(),
                    accepted,
                    "{source}"
                );
                assert_eq!(
                    from_str::<Option<$ty>>(source).is_ok(),
                    accepted || source == "!!null ~",
                    "{source}"
                );

                let alias = format!("[&value {source}, *value]");
                assert_eq!(
                    from_str::<(IgnoredAny, $ty)>(&alias).is_ok(),
                    accepted,
                    "{alias}"
                );
                write!($output, " {:>5}", if accepted { "ok" } else { "-" }).unwrap();
            }

            $output.push('\n');
        )+
    };
}

// Core tags determine the YAML type; the Rust destination must accept that type.
// https://yaml.org/spec/1.2.2/#1031-tags
#[test]
fn core_tags_and_destination_types() {
    let sources = [
        "!!null ~",
        "!!bool true",
        "!!int 7",
        "!!float 7",
        "!!str 7",
        "!!seq [7, 7]",
        "!!map {7: null}",
    ];
    let mut output = format!("{:18}", "destination");

    for tag in ["null", "bool", "int", "float", "str", "seq", "map"] {
        write!(output, " {tag:>5}").unwrap();
    }

    output.push('\n');

    destination_rows! {
        output, sources;
        "bool" => bool,
        "i8" => i8,
        "i16" => i16,
        "i32" => i32,
        "i64" => i64,
        "i128" => i128,
        "u8" => u8,
        "u16" => u16,
        "u32" => u32,
        "u64" => u64,
        "u128" => u128,
        "f32" => f32,
        "f64" => f64,
        "()" => (),
        "unit struct" => Unit,
        "String" => String,
        "&str" => &str,
        "char" => char,
        "&[u8]" => &[u8],
        "byte buffer" => ByteBuf,
        "Vec<u8>" => Vec<u8>,
        "tuple" => (u8, u8),
        "tuple struct" => Tuple,
        "map" => BTreeMap<String, Option<u8>>,
        "struct" => Record,
        "enum" => Choice,
        "Value" => Value,
        "IgnoredAny" => IgnoredAny,
    }

    insta::assert_snapshot!(output, @"
    destination         null  bool   int float   str   seq   map
    bool                   -    ok     -     -     -     -     -
    i8                     -     -    ok     -     -     -     -
    i16                    -     -    ok     -     -     -     -
    i32                    -     -    ok     -     -     -     -
    i64                    -     -    ok     -     -     -     -
    i128                   -     -    ok     -     -     -     -
    u8                     -     -    ok     -     -     -     -
    u16                    -     -    ok     -     -     -     -
    u32                    -     -    ok     -     -     -     -
    u64                    -     -    ok     -     -     -     -
    u128                   -     -    ok     -     -     -     -
    f32                    -     -    ok    ok     -     -     -
    f64                    -     -    ok    ok     -     -     -
    ()                    ok     -     -     -     -     -     -
    unit struct           ok     -     -     -     -     -     -
    String                 -     -     -     -    ok     -     -
    &str                   -     -     -     -    ok     -     -
    char                   -     -     -     -    ok     -     -
    &[u8]                  -     -     -     -    ok     -     -
    byte buffer            -     -     -     -    ok    ok     -
    Vec<u8>                -     -     -     -     -    ok     -
    tuple                  -     -     -     -     -    ok     -
    tuple struct           -     -     -     -     -    ok     -
    map                    -     -     -     -     -     -    ok
    struct                 -     -     -     -     -     -    ok
    enum                   -     -     -     -    ok     -    ok
    Value                 ok    ok    ok    ok    ok    ok    ok
    IgnoredAny            ok    ok    ok    ok    ok    ok    ok
    ");
}

// https://yaml.org/spec/1.2.2/#333-recognized-and-valid-tags
#[test]
fn tags_validate_scalar_contents_and_node_kinds() {
    let mut output = format!("{:8}", "tag");

    for kind in ["null", "bool", "int", "float", "text", "seq", "map"] {
        write!(output, " {kind:>5}").unwrap();
    }

    output.push('\n');

    for tag in ["null", "bool", "int", "float", "str", "seq", "map"] {
        write!(output, "{tag:8}").unwrap();

        for value in ["~", "true", "7", "1.5", "text", "[]", "{}"] {
            let source = format!("!!{tag} {value}");
            let accepted = from_str::<Value>(&source).is_ok();
            assert_eq!(
                from_str::<IgnoredAny>(&source).is_ok(),
                accepted,
                "{source}"
            );
            write!(output, " {:>5}", if accepted { "ok" } else { "-" }).unwrap();
        }

        output.push('\n');
    }

    insta::assert_snapshot!(output, @"
    tag       null  bool   int float  text   seq   map
    null        ok     -     -     -     -     -     -
    bool         -    ok     -     -     -     -     -
    int          -     -    ok     -     -     -     -
    float        -     -    ok    ok     -     -     -
    str         ok    ok    ok    ok    ok     -     -
    seq          -     -     -     -     -    ok     -
    map          -     -     -     -     -     -    ok
    ");
}

#[test]
fn enum_names_and_field_names_preserve_tag_types() {
    for (scalar, expected) in [
        ("7", Some(Choice::Seven)),
        ("!!str 7", Some(Choice::Seven)),
        ("! 7", Some(Choice::Seven)),
        ("!!int 7", None),
        ("!!float 7", None),
        ("!!bool true", None),
        ("!!null ~", None),
    ] {
        let accepted = expected.is_some();
        assert_eq!(from_str::<Choice>(scalar).ok(), expected);
        assert_eq!(from_tag::<Choice>(scalar).ok(), expected);
        assert_eq!(
            from_str::<Tagged<Choice>>(scalar).is_ok(),
            accepted && scalar != "7"
        );

        let mapping = format!("{{{scalar}: null}}");
        assert_eq!(from_str::<Choice>(&mapping).ok(), expected);
        assert_eq!(from_str::<Record>(&mapping).is_ok(), accepted);
        assert_eq!(
            from_str::<BTreeMap<String, Option<u8>>>(&mapping).is_ok(),
            accepted
        );

        let alias_key = format!("[&key {scalar}, {{*key : null}}]");
        assert_eq!(
            from_str::<(IgnoredAny, Choice)>(&alias_key).is_ok(),
            accepted
        );
    }
}

#[test]
fn implicit_text_and_application_tags_remain_usable() {
    let Tuple(first, second) = from_str("!!seq [7, 7]").unwrap();
    assert_eq!((first, second), (7, 7));

    for source in ["7", "'7'", "!!str 7", "! 7"] {
        assert_eq!(from_str::<char>(source).unwrap(), '7');
        assert_eq!(from_str::<&[u8]>(source).unwrap(), b"7");
        assert_eq!(from_str::<ByteBuf>(source).unwrap().0, b"7");
    }

    assert_eq!(from_str::<Tagged<char>>("!Character 7").unwrap().value, '7');
    assert_eq!(
        from_str::<Tagged<ByteBuf>>("!Bytes 7").unwrap().value.0,
        b"7"
    );
    assert_eq!(
        from_str::<Tagged<Choice>>("!Choice 7").unwrap().value,
        Choice::Seven
    );
    assert_eq!(from_tag::<Choice>("!7").unwrap(), Choice::Seven);

    for source in ["7", "!!int 7", "!!int '7'", "!!int 0x7", "!!float 7"] {
        assert_eq!(from_str::<f32>(source).unwrap().to_bits(), 7_f32.to_bits());
        assert_eq!(from_str::<f64>(source).unwrap().to_bits(), 7_f64.to_bits());
    }
}
