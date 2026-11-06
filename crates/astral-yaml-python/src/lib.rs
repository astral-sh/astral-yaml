//! Python bindings for YAML 1.2 serialization and deserialization.

use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;

mod dump;
mod load;

create_exception!(
    astral_yaml,
    YAMLError,
    PyValueError,
    "An error reading or writing a YAML document."
);

// Python strings can contain lone surrogates. YAML scalars require Unicode
// scalar values, so report an encoding failure as a YAML error at this boundary.
fn unicode_text<'a>(text: &'a Bound<'_, PyString>) -> PyResult<&'a str> {
    text.to_str()
        .map_err(|err| conversion_error(text.py(), err))
}

fn conversion_error(py: Python<'_>, err: PyErr) -> PyErr {
    if err.is_instance_of::<PyValueError>(py) {
        YAMLError::new_err(err.to_string())
    } else {
        err
    }
}

/// Read and write YAML 1.2 documents using Python values.
#[pymodule(name = "astral_yaml")]
mod bindings {
    #[pymodule_export]
    use super::YAMLError;
    #[pymodule_export]
    use super::dump::{dump, dumps};
    #[pymodule_export]
    use super::load::{load, loads};
}

#[cfg(test)]
mod tests {
    use super::{bindings, conversion_error};
    use pyo3::exceptions::PyMemoryError;
    use pyo3::prelude::*;

    #[test]
    fn conversion_preserves_memory_errors() {
        Python::initialize();
        Python::attach(|py| {
            let err = PyMemoryError::new_err("out of memory");
            assert!(conversion_error(py, err).is_instance_of::<PyMemoryError>(py));
        });
    }

    #[test]
    #[ignore = "cargo xtask cov runs the public API tests with pytest installed"]
    fn python_api() -> PyResult<()> {
        Python::initialize();
        Python::attach(|py| {
            let module = pyo3::wrap_pymodule!(bindings)(py);
            py.import("sys")?
                .getattr("modules")?
                .set_item("astral_yaml", module)?;

            let tests = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");
            let result = py
                .import("pytest")?
                .call_method1("main", (["-q", tests],))?;
            assert_eq!(result.extract::<i32>()?, 0, "pytest failed");
            Ok(())
        })
    }
}
