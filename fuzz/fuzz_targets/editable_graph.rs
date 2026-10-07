#![no_main]

//! Valid structural edits must agree with an independent recursive alias graph.

libfuzzer_sys::fuzz_target!(|data: &[u8]| astral_yaml_dev::check_recursive_edits(data));
