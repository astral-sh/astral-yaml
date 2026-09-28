mod support;

use astral_yaml_serde::{Spanned, Tagged, from_str};
use serde::{Deserialize, de::IgnoredAny};
use std::{collections::BTreeMap, fmt::Write};
use support::from_tag;

#[derive(Deserialize)]
struct Newtype(String);

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
        "!!seq [7]",
        "!!map {7: null}",
    ];
    let mut output = format!("{:12}", "destination");

    for tag in ["null", "bool", "int", "float", "str", "seq", "map"] {
        write!(output, " {tag:>5}").unwrap();
    }

    output.push('\n');

    macro_rules! row {
        ($label:literal, $ty:ty) => {
            write!(output, "{:<12}", $label).unwrap();

            for source in sources {
                let accepted = from_str::<$ty>(source).is_ok();
                write!(output, " {:>5}", if accepted { "ok" } else { "-" }).unwrap();
            }

            output.push('\n');
        };
    }

    row!("bool", bool);
    row!("i64", i64);
    row!("u64", u64);
    row!("f32", f32);
    row!("f64", f64);
    row!("()", ());
    row!("String", String);
    row!("&str", &str);
    row!("char", char);
    row!("&[u8]", &[u8]);
    row!("Vec<u8>", Vec<u8>);
    row!("map", BTreeMap<String, Option<u8>>);
    row!("enum", Choice);

    insta::assert_snapshot!(output, @"
    destination   null  bool   int float   str   seq   map
    bool             -    ok     -     -     -     -     -
    i64              -     -    ok     -     -     -     -
    u64              -     -    ok     -     -     -     -
    f32              -     -    ok    ok     -     -     -
    f64              -     -    ok    ok     -     -     -
    ()              ok     -     -     -     -     -     -
    String           -     -     -     -    ok     -     -
    &str             -     -     -     -    ok     -     -
    char             -     -     -     -    ok     -     -
    &[u8]            -     -     -     -    ok     -     -
    Vec<u8>          -     -     -     -     -    ok     -
    map              -     -     -     -     -     -    ok
    enum             -     -     -     -    ok     -    ok
    ");
}

#[test]
fn wrappers_and_aliases_preserve_tag_checks() {
    for source in ["!!str 7", "!!int 7"] {
        let expected = from_str::<String>(source);
        assert_eq!(
            from_str::<Tagged<String>>(source).map(|v| v.value),
            expected
        );
        assert_eq!(
            from_str::<Spanned<'_, String>>(source).map(|v| v.value),
            expected
        );
        assert_eq!(from_str::<Newtype>(source).map(|v| v.0), expected);
        assert_eq!(
            from_str::<Option<String>>(source),
            expected.clone().map(Some)
        );

        let alias = format!("[&value {source}, *value]");
        let result = from_str::<(IgnoredAny, &str)>(&alias);
        assert_eq!(result.is_ok(), expected.is_ok());

        if let Err(error) = result {
            assert_eq!(error.span.unwrap().slice(&alias), Some("&value !!int 7"));
        }
    }
}

// Explicit tags retain their meaning across quoting and alternate tag spellings.
// https://yaml.org/spec/1.2.2/#691-node-tags
#[test]
fn tag_mismatch_errors_retain_the_node_span() {
    let mut output = String::new();

    for source in [
        "!!int '42'",
        "!<tag:yaml.org,2002:int> 42",
        "%TAG !c! tag:yaml.org,2002:\n--- !c!int 42",
    ] {
        let error = from_str::<String>(source).unwrap_err();
        assert_eq!(from_str::<&str>(source).unwrap_err(), error);
        writeln!(output, "{source:?}: {error}").unwrap();
    }

    insta::assert_snapshot!(output, @r#"
    "!!int '42'": tag does not match requested type at bytes 0..10
    "!<tag:yaml.org,2002:int> 42": tag does not match requested type at bytes 0..27
    "%TAG !c! tag:yaml.org,2002:\n--- !c!int 42": tag does not match requested type at bytes 32..41
    "#);
}

#[test]
fn enum_names_and_field_names_preserve_tag_types() {
    for (scalar, accepted) in [("7", true), ("!!str 7", true), ("!!int 7", false)] {
        assert_eq!(from_tag::<Choice>(scalar).is_ok(), accepted);

        let mapping = format!("{{{scalar}: null}}");
        assert_eq!(from_str::<Choice>(&mapping).is_ok(), accepted);
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
    for source in ["7", "'7'", "!!str 7", "! 7"] {
        assert_eq!(from_str::<char>(source).unwrap(), '7');
        assert_eq!(from_str::<&[u8]>(source).unwrap(), b"7");
    }

    assert_eq!(from_str::<Tagged<char>>("!Character 7").unwrap().value, '7');
    assert_eq!(
        from_str::<Tagged<Choice>>("!Choice 7").unwrap().value,
        Choice::Seven
    );
    assert_eq!(from_tag::<Choice>("!7").unwrap(), Choice::Seven);

    for source in ["!!int '7'", "!!int 0x7"] {
        assert_eq!(from_str::<f32>(source).unwrap().to_bits(), 7_f32.to_bits());
        assert_eq!(from_str::<f64>(source).unwrap().to_bits(), 7_f64.to_bits());
    }
}
