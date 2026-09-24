#![no_main]
#![forbid(unsafe_code)]

use astral_yaml::Source;
use astral_yaml_dev::validate_spans;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match Source::with_limit(data, 65_536) {
        Ok(source) => match source.parse() {
            Ok(stream) => {
                validate_spans(&stream).unwrap();

                for node in stream.nodes() {
                    assert!(
                        source
                            .original_span(node.span)
                            .unwrap()
                            .slice_bytes(data)
                            .is_some()
                    );
                }
            }
            Err(error) => {
                assert!(
                    source
                        .original_span(error.span)
                        .unwrap()
                        .slice_bytes(data)
                        .is_some()
                );
            }
        },
        Err(error) => assert!(error.span.slice_bytes(data).is_some()),
    }
});
