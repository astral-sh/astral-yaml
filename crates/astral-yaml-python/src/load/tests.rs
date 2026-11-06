use super::{Input, document, loads, read_document};
use crate::YAMLError;
use pyo3::exceptions::{PyOSError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDict, PyString};
use test_case::test_case;

fn python<T>(test: impl FnOnce(Python<'_>) -> T) -> T {
    Python::initialize();
    Python::attach(test)
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn core_schema() {
    python(|py| {
        let source = "
defaults: [~, true, FALSE, on, 2001-02-03, +012, 0xFF, 0o20]
numbers: [0.25, -.INF, .Inf, .NAN, 999999999999999999999999999999999999999999999]
tagged: !!seq [! 1, !!str false, !!null '~', !!float '01', !!int '0xF', !!bool TRUE]
collections: ! [!!map {null: 'false'}]
block: |
  12
";
        let value = document(py, source, None).unwrap();
        let formatted = py
            .import("pprint")
            .unwrap()
            .call_method1("pformat", (value,))
            .unwrap();
        insta::assert_snapshot!(formatted.extract::<String>().unwrap(), @r"
        {'block': '12\n',
         'collections': [{None: 'false'}],
         'defaults': [None, True, False, 'on', '2001-02-03', 12, 255, 16],
         'numbers': [0.25,
                     -inf,
                     inf,
                     nan,
                     999999999999999999999999999999999999999999999],
         'tagged': ['1', 'false', None, 1.0, 15, True]}
        ");
    });
}

#[test_case("!custom 1", "unsupported YAML tag !custom")]
#[test_case("!!int [1]", "tag does not match node kind")]
#[test_case("!!seq x", "tag does not match node kind")]
#[test_case("!!bool yes", "scalar does not match its tag")]
#[test_case("!!float 0o7", "scalar does not match its tag")]
#[test_case("--- first\n--- second", "expected one YAML document")]
#[test_case("1e10000", "float out of range")]
#[test_case("{.nan: 0, .NaN: 1}", "duplicate mapping key")]
#[test_case("{42: 0, 042: 1}", "duplicate mapping key")]
#[test_case("{false: 0, -0.0: 1}", "duplicate mapping key")]
#[test_case("{[1]: 0}", "collection keys cannot be used in a Python dict")]
#[test_case("{{1: 2}: 0}", "collection keys cannot be used in a Python dict")]
fn unrepresentable_document(source: &str, expected: &str) {
    python(|py| {
        let err = document(py, source, None).unwrap_err();
        assert!(err.is_instance_of::<YAMLError>(py));
        assert!(err.value(py).to_string().starts_with(expected), "{err}");
    });
}

// https://yaml.org/spec/1.2.2/#71-alias-nodes
#[test]
fn rebinding_and_recursive_aliases() {
    python(|py| {
        let source = "[&k word, {*k : &k [*k]}, *k]";
        let value = document(py, source, None).unwrap();
        let dict = value.get_item(1).unwrap();
        let recursive = dict.get_item("word").unwrap();
        assert!(recursive.is(recursive.get_item(0).unwrap()));
        assert!(recursive.is(value.get_item(2).unwrap()));
    });
}

#[test]
fn python_decimal_int_limit() {
    python(|py| {
        let max = py
            .import("sys")
            .unwrap()
            .call_method0("get_int_max_str_digits")
            .unwrap()
            .extract::<usize>()
            .unwrap();
        let text = "1".repeat(max.min(8192) + 1);
        let limited = max != 0 && text.len() > max;
        let error = document(py, &text, None).err();
        assert_eq!(
            error.is_some_and(|err| err.is_instance_of::<YAMLError>(py)),
            limited
        );
    });
}

#[test]
fn byte_input_error_spans_use_the_original_encoding() {
    python(|py| {
        let source = "!!bool x"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let bytes = PyBytes::new(py, &source);
        let err = loads(bytes.as_any()).unwrap_err();
        insta::assert_snapshot!(err.value(py).to_string(), @"scalar does not match its tag at bytes 14..16");

        let invalid = PyBytes::new(py, &[0xff]);
        let err = loads(invalid.as_any()).unwrap_err();
        assert!(err.is_instance_of::<YAMLError>(py));

        let syntax = loads(PyString::new(py, "[").as_any()).unwrap_err();
        assert!(syntax.value(py).to_string().contains("at bytes"));
    });
}

#[test]
fn input_types_and_limits() {
    python(|py| {
        let py_data = PyByteArray::new(py, b"17");
        assert_eq!(
            loads(py_data.as_any()).unwrap().extract::<u8>().unwrap(),
            17
        );
        assert!(Input::extract(py_data.as_any(), 1).is_err());
        let err = super::load(PyDict::new(py).as_any()).unwrap_err();
        assert!(err.is_instance_of::<PyTypeError>(py));
        let err = loads(PyDict::new(py).as_any()).unwrap_err();
        assert!(err.is_instance_of::<PyTypeError>(py));
    });
}

#[test_case(c"['1', '7', '']", 2, Ok(17); "short text reads")]
#[test_case(c"[b'1', bytearray(b'7'), b'']", 2, Ok(17); "short binary reads")]
#[test_case(c"['abc']", 2, Err("input byte limit exceeded"); "oversize reader response")]
#[test_case(c"['17', '1']", 2, Err("input byte limit exceeded"); "one more byte than limit")]
#[test_case(c"['éé']", 3, Err("input byte limit exceeded"); "text limits count bytes")]
#[test_case(c"['1', b'']", 2, Err("read() mixed text and binary data"); "cannot change type on EOF")]
fn bounded_reads(chunks: &std::ffi::CStr, limit: usize, expected: Result<u8, &str>) {
    python(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("chunks", py.eval(chunks, None, None).unwrap())
            .unwrap();
        let reader = py
            .eval(
                c"type('Reader', (), {'read': lambda self, n: chunks.pop(0)})()",
                Some(&globals),
                None,
            )
            .unwrap();
        let result = read_document(&reader, limit);
        match expected {
            Ok(n) => assert_eq!(result.unwrap().extract::<u8>().unwrap(), n),
            Err(msg) => assert_eq!(result.unwrap_err().value(py).to_string(), msg),
        }
    });
}

#[test]
fn read_exceptions_propagate() {
    python(|py| {
        let locals = PyDict::new(py);
        py.run(
            c"class Reader:\n    def read(self, size):\n        raise OSError('cannot read')",
            Some(&locals),
            None,
        )
        .unwrap();
        let reader = locals.get_item("Reader").unwrap().unwrap().call0().unwrap();
        let error = read_document(&reader, 4).unwrap_err();
        assert!(error.is_instance_of::<PyOSError>(py));
    });
}

#[test]
fn numeric_key_collisions_charge_for_prior_keys_with_the_same_hash() {
    python(|py| {
        let modulus = py
            .import("sys")
            .unwrap()
            .getattr("hash_info")
            .unwrap()
            .getattr("modulus")
            .unwrap()
            .extract::<u64>()
            .unwrap();
        // Keep one key behind an alias so the charge includes the target's digits.
        let text = format!("[&large {modulus}, {{0: a, *large : b}}]");
        let stream = astral_yaml::parse(&text).unwrap();
        let budget = 1 + modulus.to_string().len() * 2;
        let root = stream.documents()[0].root;
        let mut loader = super::Loader {
            py,
            stream: &stream,
            encoded: None,
            anchors: std::collections::HashMap::new(),
            key_bytes: budget,
        };
        loader.value(root).unwrap();
        assert_eq!(loader.key_bytes, 0);
        loader.key_bytes = budget - 1;
        let error = loader.value(root).unwrap_err();
        assert!(
            error
                .value(py)
                .to_string()
                .contains("numeric mapping key work limit exceeded")
        );
    });
}

#[test]
fn text_size_is_checked_before_utf8_conversion() {
    python(|py| {
        let surrogate = py.eval(c"'x\\ud800'", None, None).unwrap();
        let error = Input::extract(&surrogate, 1).err().unwrap();
        assert_eq!(error.value(py).to_string(), "input byte limit exceeded");
        let error = loads(&surrogate).unwrap_err();
        assert!(error.is_instance_of::<YAMLError>(py));
    });
}
