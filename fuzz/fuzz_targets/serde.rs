#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_dev::{Value, assert_eq_with_nan};
use astral_yaml_serde::{Tagged, from_str, to_string};
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

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
    assert_eq!(from_str::<String>(&quoted).unwrap(), source);
    assert_eq!(yaml_serde::from_str::<String>(&quoted).unwrap(), source);

    match from_str::<Value>(source) {
        Ok(value) => {
            if let Ok(encoded) = to_string(&value) {
                assert_eq_with_nan(&from_str::<Value>(&encoded).unwrap(), &value);
                assert_eq_with_nan(&yaml_serde::from_str::<Value>(&encoded).unwrap(), &value);
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
    {
        let decoded = from_str::<Tagged<Value>>(&encoded).unwrap();
        assert_eq!(decoded.tag, value.tag);
        assert_eq_with_nan(&decoded.value, &value.value);
    }

    let text = YamlTag(Message::Text(source.to_owned()));
    let encoded = to_string(&text).unwrap();
    assert_eq!(from_str::<YamlTag>(&encoded).unwrap(), text);

    if let Ok(value) = from_str::<YamlTag>(source)
        && let Ok(encoded) = to_string(&value)
    {
        assert_eq!(from_str::<YamlTag>(&encoded).unwrap(), value);
    }
});
