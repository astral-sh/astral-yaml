use super::Dumper;
use astral_yaml::{CollectionStyle, ParseOptions};
use pyo3::prelude::*;
use std::ffi::CStr;
use test_case::{test_case, test_matrix};

#[test_case(i64::MIN)]
#[test_case(i64::MAX)]
#[test_case(0)]
fn integer_spelling(value: i64) {
    let mut output = super::Output::new(20, super::output_limit);
    super::write_decimal(&mut output, value, 0).unwrap();
    assert_eq!(output.as_ref(), value.to_string());
}

#[test_case(0.0)]
#[test_case(0.0001)]
#[test_case(0.000_101)]
#[test_case(0.01)]
#[test_case(0.123)]
#[test_case(1.0)]
#[test_case(10.000_01)]
#[test_case(999_999_999.999_999)]
#[test_case(1e9)]
fn float_spelling(boundary: f64) {
    for value in [boundary.next_down(), boundary, boundary.next_up()] {
        for number in [value, -value] {
            let mut output = super::Output::new(64, super::output_limit);
            super::write_float(&mut output, number).unwrap();
            assert_eq!(output.as_ref(), format!("{number:?}"));
        }
    }
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
    [c"[[]] * 2", c"[[0]] * 2", c"{'ports': [80, 443]}", c"{chr(233): 0}", c"[[[[[[[[[[1, 2]]]]]]]]]]"]
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
