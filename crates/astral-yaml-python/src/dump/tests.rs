use super::Dumper;
use astral_yaml::ParseOptions;
use pyo3::prelude::*;
use std::ffi::CStr;
use test_case::test_case;

#[test_case(c"[0]", 1, 10, 100, "nesting limit"; "nesting depth")]
#[test_case(c"[0]", 10, 1, 100, "node limit"; "sequence nodes")]
#[test_case(c"{1: 2}", 10, 2, 100, "node limit"; "mapping nodes before snapshot")]
#[test_case(c"chr(0xd800) * 8", 10, 1, 4, "byte limit"; "before UTF-8 conversion")]
#[test_case(c"2**20000", 10, 1, 20, "byte limit"; "before decimal conversion")]
fn limits(expression: &CStr, depth: usize, nodes: usize, bytes: usize, message: &str) {
    Python::initialize();
    Python::attach(|py| {
        let options = ParseOptions {
            max_depth: depth,
            max_nodes: nodes,
            max_input_bytes: bytes,
        };
        let obj = py.eval(expression, None, None).unwrap();
        let error = Dumper::serialize(&obj, options).unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    });
}

#[test]
fn byte_limit_includes_inserted_anchors() {
    Python::initialize();
    Python::attach(|py| {
        let obj = py.eval(c"[[]] * 2", None, None).unwrap();
        let mut options = ParseOptions::default();
        let expected = Dumper::serialize(&obj, options).unwrap();

        options.max_input_bytes = expected.len();
        assert_eq!(Dumper::serialize(&obj, options).unwrap(), expected);

        options.max_input_bytes -= 1;
        let error = Dumper::serialize(&obj, options).unwrap_err();
        assert!(error.to_string().contains("byte limit"), "{error}");
    });
}
