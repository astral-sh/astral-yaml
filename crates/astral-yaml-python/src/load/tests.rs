use super::{Input, Loader, read_document};
use pyo3::prelude::*;
use test_case::test_case;

#[test]
fn bound_input_before_converting_unicode_or_copying_bytearrays() {
    Python::initialize();
    Python::attach(|py| {
        let surrogate = py.eval(c"'x\\ud800'", None, None).unwrap();
        let error = Input::extract(&surrogate, 1).err().unwrap();
        assert_eq!(error.value(py).to_string(), "input byte limit exceeded");

        let bytes = pyo3::types::PyByteArray::new(py, b"12");
        assert!(Input::extract(bytes.as_any(), 1).is_err());
    });
}

#[test_case(1, false; "UTF-8 exceeds the limit")]
#[test_case(2, true; "exact UTF-8 limit")]
fn text_reader_byte_limit(limit: usize, accepted: bool) {
    Python::initialize();
    Python::attach(|py| {
        let reader = py
            .import("io")
            .unwrap()
            .call_method1("StringIO", ("é",))
            .unwrap();
        assert_eq!(read_document(&reader, limit).is_ok(), accepted);
    });
}

#[test_case(5, false; "one byte short")]
#[test_case(6, true; "exact collision budget")]
fn numeric_key_work_budget(budget: usize, accepted: bool) {
    Python::initialize();
    Python::attach(|py| {
        // Python's -1 and -2 hash to the same value. The alias key charges
        // for comparing its two digits against both keys in that bucket.
        let stream = astral_yaml::parse("[&n -1, {-2: a, *n : b}]").unwrap();
        let mut loader = Loader {
            py,
            stream: &stream,
            encoded: None,
            anchors: std::collections::HashMap::new(),
            key_bytes: budget,
        };
        assert_eq!(loader.value(stream.documents()[0].root).is_ok(), accepted);
    });
}
