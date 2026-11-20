use super::Dumper;
use astral_yaml::{CollectionStyle, ParseOptions};
use pyo3::prelude::*;
use std::ffi::CStr;
use test_case::{test_case, test_matrix};

#[test_case(i64::MIN)]
#[test_case(i64::MAX)]
#[test_case(0)]
fn integer_spelling(value: i64) {
    Python::initialize();
    Python::attach(|py| {
        let obj = value.into_pyobject(py).unwrap();
        let text = Dumper::serialize(&obj, ParseOptions::default(), CollectionStyle::Flow).unwrap();
        assert_eq!(text, format!("{value}\n"));
    });
}

#[test]
fn decimal_float_spellings_match_debug() {
    Python::initialize();
    Python::attach(|py| {
        // Exercise the fixed-point range boundaries, fractional widths,
        // leading and trailing zeros, and the adjacent binary floats.
        for magnitude in [
            0.0_f64,
            0.0001,
            0.000_101,
            0.01,
            0.123,
            1.0,
            10.000_01,
            999_999_999.999_999,
            1e9,
        ] {
            for number in [magnitude.next_down(), magnitude, magnitude.next_up()] {
                for value in [number, -number] {
                    let obj = value.into_pyobject(py).unwrap();
                    let text =
                        Dumper::serialize(&obj, ParseOptions::default(), CollectionStyle::Flow)
                            .unwrap();
                    assert_eq!(text, format!("{value:?}\n"));
                }
            }
        }
    });
}

#[test_matrix(
    [CollectionStyle::Block, CollectionStyle::Flow],
    [
        (c"[0]", 1, 10, 100, "nesting limit"),
        (c"{1: 2}", 1, 10, 100, "nesting limit"),
        (c"[0]", 10, 1, 100, "node limit"),
        (c"{1: 2}", 10, 2, 100, "node limit"),
        (c"chr(0xd800) * 8", 10, 1, 4, "byte limit"),
        (c"2**20000", 10, 1, 20, "byte limit"),
    ]
)]
fn limits(
    style: CollectionStyle,
    (expression, depth, nodes, bytes, message): (&CStr, usize, usize, usize, &str),
) {
    Python::initialize();
    Python::attach(|py| {
        let options = ParseOptions {
            max_depth: depth,
            max_nodes: nodes,
            max_input_bytes: bytes,
        };
        let obj = py.eval(expression, None, None).unwrap();
        let error = Dumper::serialize(&obj, options, style).unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    });
}

#[test_matrix(
    [CollectionStyle::Block, CollectionStyle::Flow],
    [c"[[]] * 2", c"[[0]] * 2", c"{'ports': [80, 443]}", c"[[[[[[[[[[1, 2]]]]]]]]]]"]
)]
fn byte_limit_counts_complete_output(style: CollectionStyle, expression: &CStr) {
    Python::initialize();
    Python::attach(|py| {
        let obj = py.eval(expression, None, None).unwrap();
        let mut options = ParseOptions::default();
        let expected = Dumper::serialize(&obj, options, style).unwrap();

        options.max_input_bytes = expected.len();
        assert_eq!(Dumper::serialize(&obj, options, style).unwrap(), expected);

        options.max_input_bytes -= 1;
        let error = Dumper::serialize(&obj, options, style).unwrap_err();
        assert!(error.to_string().contains("byte limit"), "{error}");
    });
}
