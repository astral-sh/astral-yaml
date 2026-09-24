#![no_main]
#![forbid(unsafe_code)]

use astral_yaml_serde::{from_str, to_string};
use libfuzzer_sys::fuzz_target;
use yaml_serde::Value;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let quoted = to_string(source).unwrap();
    assert_eq!(from_str::<String>(&quoted).unwrap(), source);
    match from_str::<Value>(source) {
        Ok(value) => {
            if let Ok(encoded) = to_string(&value) {
                assert_eq!(from_str::<Value>(&encoded).unwrap(), value);
            }
        }
        Err(error) => {
            assert!(
                error.span.is_some_and(|span| span.slice(source).is_some()),
                "{error:?}"
            );
        }
    }
});
