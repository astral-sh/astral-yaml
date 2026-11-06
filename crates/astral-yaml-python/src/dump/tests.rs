use super::{Dumper, Output, dump};
use astral_yaml::ParseOptions;
use pyo3::exceptions::{PyOSError, PyTypeError, PyZeroDivisionError};
use pyo3::prelude::*;
use std::ffi::{CStr, CString};
use test_case::test_case;

fn serialize(expression: &str, options: ParseOptions) -> PyResult<String> {
    Python::initialize();
    Python::attach(|py| {
        let expression = CString::new(expression).unwrap();
        Dumper::serialize(&py.eval(&expression, None, None)?, options)
    })
}

// https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
#[test]
fn anchors_mark_only_referenced_containers() {
    let expression = "(lambda a, b: [a, b, b, a, [], a])([1], {'key': True})";
    let output = serialize(expression, ParseOptions::default()).unwrap();
    insta::assert_snapshot!(output, @r#"[&id2 [1], &id1 {"key": true}, *id1, *id2, [], *id2]"#);
}

// https://yaml.org/spec/1.2.2/#733-plain-style
#[test_case("None", "null\n"; "null")]
#[test_case("False", "false\n"; "false")]
#[test_case("2**100", "1267650600228229401496703205376\n"; "arbitrary integer")]
#[test_case("(-0.0, 3.0, 2e100)", "[-0.0, 3.0, 2e+100]\n"; "round-trip floats")]
#[test_case("[float('nan'), float('inf'), -float('inf')]", "[.nan, .inf, -.inf]\n"; "non-finite floats")]
#[test_case("('null', True, [None], {})", "[\"null\", true, [null], {}]\n"; "built-in containers")]
#[test_case("{float('nan'): 0}", "{.nan: 0}\n"; "single NaN key")]
#[test_case("(lambda d: (d.update(self=d), d)[1])({})", "&id1 {\"self\": *id1}\n"; "recursive dict")]
fn values(expression: &str, expected: &str) {
    assert_eq!(
        serialize(expression, ParseOptions::default()).unwrap(),
        expected
    );
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test_case(1022, false; "implicit Unicode key including quotes")]
#[test_case(1023, true; "explicit Unicode key including quotes")]
fn long_keys(characters: usize, explicit: bool) {
    let key = "é".repeat(characters);
    let source = serialize(&format!("{{'{key}': 1}}"), ParseOptions::default()).unwrap();
    assert!(astral_yaml::parse(&source).is_ok());
    assert_eq!(source.starts_with("{? "), explicit);
}

#[test_case("object()", "cannot serialize"; "arbitrary object")]
#[test_case("type('Subclass', (str,), {})('x')", "cannot serialize"; "subclass")]
#[test_case("{(): 1}", "mapping keys"; "collection key")]
#[test_case("{float('nan'): 1, float('nan'): 2}", "duplicate mapping key"; "duplicate NaN keys")]
fn unsupported(expression: &str, message: &str) {
    let error = serialize(expression, ParseOptions::default()).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
}

#[test_case("['x']", 1, 10, 100, "nesting"; "node depth")]
#[test_case("[None]", 10, 1, 100, "node limit"; "nodes")]
#[test_case("{1: 2}", 10, 2, 100, "node limit"; "dictionary nodes")]
#[test_case("0", 10, 1, 1, "byte limit"; "trailing newline")]
#[test_case("'string'", 10, 1, 6, "byte limit"; "quoted string")]
#[test_case("chr(0xd800) * 8", 10, 1, 4, "byte limit"; "oversized string before encoding")]
#[test_case("'éé'", 10, 1, 5, "byte limit"; "UTF-8 bytes after encoding")]
#[test_case("2**20000", 10, 1, 20, "byte limit"; "large integer before decimal conversion")]
#[test_case("12345", 10, 1, 4, "byte limit"; "small integer digits")]
#[test_case("'\\x00'", 10, 1, 3, "byte limit"; "string escapes")]
#[test_case("(lambda a: [a, a])([])", 10, 10, 8, "byte limit"; "alias")]
#[test_case("(lambda a: [a, a])([])", 10, 10, 13, "byte limit"; "anchor and following text")]
fn limits(expression: &str, depth: usize, nodes: usize, bytes: usize, message: &str) {
    let options = ParseOptions {
        max_depth: depth,
        max_nodes: nodes,
        max_input_bytes: bytes,
    };
    let error = serialize(expression, options).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
}

#[test]
fn unpaired_surrogate_raises_yaml_error() {
    let error = serialize("chr(0xd800)", ParseOptions::default()).unwrap_err();
    Python::attach(|py| assert!(error.is_instance_of::<crate::YAMLError>(py)));
}

#[test_case(c"10**100", 101; "within the minimum digit limit")]
#[test_case(c"10**5000", 5001; "beyond the default digit limit")]
fn python_decimal_digit_limit(expression: &CStr, digits: usize) {
    Python::initialize();
    Python::attach(|py| {
        let number = py.eval(expression, None, None).unwrap();
        let sys = py.import("sys").unwrap();
        let limit: usize = sys
            .call_method0("get_int_max_str_digits")
            .unwrap()
            .extract()
            .unwrap();
        let result = Dumper::serialize(&number, ParseOptions::default());
        assert_eq!(result.is_err(), limit != 0 && digits > limit);
        if let Err(error) = result {
            assert!(error.is_instance_of::<crate::YAMLError>(py));
        }
    });
}

#[test]
fn anchors_and_aliases_count_toward_the_byte_limit() {
    let expression = "[alias for item in ([], [], [], [], [], []) for alias in (item, item)]";
    let expected = serialize(expression, ParseOptions::default()).unwrap();
    for limit in 0..=expected.len() {
        let options = ParseOptions {
            max_input_bytes: limit,
            ..ParseOptions::default()
        };
        let result = serialize(expression, options);
        if limit == expected.len() {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().to_string().contains("byte limit"));
        }
    }
}

#[test]
fn explicit_key_indicator_uses_output_budget() {
    let mut output = Output::new(1025);
    output.push(&"x".repeat(1025)).unwrap();
    Python::initialize();
    assert!(
        output
            .finish_key(0)
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );
}

#[test]
fn dump_requires_a_writer_and_propagates_lookup_errors() {
    Python::initialize();
    Python::attach(|py| {
        let none = py.None().into_bound(py);
        let missing = dump(&none, &none).unwrap_err();
        assert!(missing.is_instance_of::<PyTypeError>(py));

        let broken = py
            .eval(
                c"type('Stream', (), {'write': property(lambda self: 1 / 0)})()",
                None,
                None,
            )
            .unwrap();
        let error = dump(&none, &broken).unwrap_err();
        assert!(error.is_instance_of::<PyZeroDivisionError>(py));
    });
}

#[test]
fn dump_reports_short_text_writes() {
    Python::initialize();
    Python::attach(|py| {
        let writer = py
            .eval(
                c"type('ShortWriter', (), {'write': lambda self, text: len(text) - 1})()",
                None,
                None,
            )
            .unwrap();
        let text = py.eval(c"'αβγ'", None, None).unwrap();
        let error = dump(&text, &writer).unwrap_err();
        assert!(error.is_instance_of::<PyOSError>(py));
        assert!(error.to_string().contains("short write"));
    });
}
