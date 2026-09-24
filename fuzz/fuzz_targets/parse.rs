#![no_main]
#![forbid(unsafe_code)]

use astral_yaml::{ParseOptions, parse_with_options};
use astral_yaml_dev::validate_spans;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let options = ParseOptions {
        max_depth: 64,
        max_nodes: 16_384,
        max_input_bytes: 65_536,
    };

    match parse_with_options(source, options) {
        Ok(stream) => validate_spans(&stream).unwrap(),
        Err(error) => assert!(
            error.span.slice(source).is_some(),
            "invalid error span: {error:?}"
        ),
    }
});
