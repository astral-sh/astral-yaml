use astral_yaml_serde::{from_str, from_stream, to_string};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, fmt::Write};

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
        from_str::<Kind>("!Newtype abc").unwrap(),
        Kind::Newtype("abc".into())
    );
    assert_eq!(
        from_str::<Kind>("!Tuple [7, false]").unwrap(),
        Kind::Tuple(7, false)
    );
    assert_eq!(
        from_str::<Kind>("!Struct {value: -2}").unwrap(),
        Kind::Struct { value: -2 }
    );
    assert_eq!(from_str::<Kind>("!Unit").unwrap(), Kind::Unit);
}

#[test]
fn core_resolution_and_numeric_limits() {
    for source in [
        "yes",
        "on",
        "NO",
        "Off",
        "0b11",
        "1_000",
        "2001-12-15",
        "1:20",
    ] {
        assert_eq!(from_str::<String>(source).unwrap(), source);
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
    assert!(from_str::<u8>("256").is_err());
    assert!(from_str::<u8>("-1").is_err());
    assert!(from_str::<i8>("128").is_err());
    assert!(from_str::<i32>("'42'").is_err());
    assert_eq!(from_str::<i32>("!!int '42'").unwrap(), 42);
    assert_eq!(from_str::<String>("!!str 42").unwrap(), "42");
    assert!(from_str::<bool>("!!bool yes").is_err());
    assert!(from_str::<String>("!unknown value").is_err());
    roundtrip(&i128::MIN);
    roundtrip(&u128::MAX);
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

#[test]
fn located_failures_and_document_count() {
    for source in ["", "---\n---\n", "[", "256"] {
        let error = from_str::<u8>(source).unwrap_err();
        assert!(error.span.is_some());
        assert!(!error.to_string().is_empty());
        assert_eq!(error.clone(), error);
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
    assert!(from_str::<Anything>("&a [*a]").is_err());
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
}
