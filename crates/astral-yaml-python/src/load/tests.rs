use super::{Input, Loader, read_document};
use astral_yaml::Budget;
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

// Python hashes -1 and -2 identically, including when the key is an alias.
#[test_case("[&n -1, {-2: a, *n : b}]", 6; "equal hashes with an alias")]
// These small integers have distinct hashes with the same low bits.
#[test_case("{1: a, 9: b, 17: c}", 7; "distinct hashes")]
#[test_case("{a: a, 1: b, z: c}", 3; "mixed key types")]
#[test_case("{a: a, b: b, null: c}", 0; "map without numeric keys")]
#[test_case("[{1: a}, {2: b}]", 2; "shared across mappings")]
fn numeric_key_work_budget(source: &str, minimum_budget: usize) {
    Python::initialize();
    Python::attach(|py| {
        let stream = astral_yaml::parse(source).unwrap();
        let load = |budget| {
            Loader {
                py,
                stream: &stream,
                encoded: None,
                anchors: std::collections::HashMap::new(),
                key_work: Budget::new(budget),
            }
            .value(stream.documents()[0].root)
        };

        load(minimum_budget).unwrap();
        if minimum_budget > 0 {
            assert!(load(minimum_budget - 1).is_err());
        }
    });
}
