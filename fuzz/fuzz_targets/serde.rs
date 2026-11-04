#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_dev::Value;
use astral_yaml_serde::{Tagged, from_str, to_string};
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

mod roundtrip;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Message {
    Ready,
    Text(String),
    Items(Vec<String>),
    #[serde(untagged)]
    Number(i64),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct YamlTag(#[serde(with = "astral_yaml_serde::with::yaml_tag")] Message);

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let quoted = to_string(source).unwrap();
    if quoted.len() <= astral_yaml::ParseOptions::default().max_input_bytes {
        assert_eq!(from_str::<String>(&quoted).unwrap(), source);
    }

    match from_str::<Value>(source) {
        Ok(value) => {
            if let Ok(encoded) = to_string(&value)
                && let Some(decoded) = roundtrip::read::<Value>(&encoded)
            {
                assert_eq!(decoded, value);
            }
        }
        Err(error) => {
            assert!(
                error.span.is_some_and(|span| span.slice(source).is_some()),
                "{error:?}"
            );
        }
    }

    if let Ok(value) = from_str::<Tagged<Value>>(source)
        && let Ok(encoded) = to_string(&value)
        && let Some(decoded) = roundtrip::read::<Tagged<Value>>(&encoded)
    {
        assert_eq!(decoded, value);
    }

    let text = YamlTag(Message::Text(source.to_owned()));
    let encoded = to_string(&text).unwrap();
    if let Some(decoded) = roundtrip::read::<YamlTag>(&encoded) {
        assert_eq!(decoded, text);
    }

    if let Ok(value) = from_str::<YamlTag>(source)
        && let Ok(encoded) = to_string(&value)
        && let Some(decoded) = roundtrip::read::<YamlTag>(&encoded)
    {
        assert_eq!(decoded, value);
    }
});
