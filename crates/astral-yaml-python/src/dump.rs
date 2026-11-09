//! Serialize Python built-ins to a YAML document.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use astral_yaml::ParseOptions;
use astral_yaml::emit::{FlowWrite, LimitedWriter, non_finite_float};
use pyo3::exceptions::{PyOSError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};

use crate::YAMLError;

/// Serialize a Python value as a YAML 1.2 document.
#[pyfunction]
pub(super) fn dumps(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    Dumper::serialize(obj, ParseOptions::default())
}

/// Serialize a Python value and write the document to a text stream.
#[pyfunction]
pub(super) fn dump(obj: &Bound<'_, PyAny>, fp: &Bound<'_, PyAny>) -> PyResult<()> {
    let write = fp
        .getattr_opt(pyo3::intern!(fp.py(), "write"))?
        .ok_or_else(|| PyTypeError::new_err("expected a text stream with a write method"))?;
    let text = PyString::new(obj.py(), &dumps(obj)?);
    let written = write.call1((&text,))?;
    if let Ok(count) = written.cast::<PyInt>()
        && count.lt(text.code_point_len()?)?
    {
        return Err(PyOSError::new_err(
            "short write while writing YAML document",
        ));
    }
    Ok(())
}

// Avoid building a PyO3 cast error while trying the other builtin types.
fn as_builtin<'a, 'py, T: pyo3::PyTypeInfo>(
    value: &'a Bound<'py, PyAny>,
) -> Option<&'a Bound<'py, T>> {
    if value.is_exact_instance_of::<T>() {
        value.cast_exact().ok()
    } else {
        None
    }
}

fn output_limit() -> PyErr {
    YAMLError::new_err("serialized YAML exceeds the output byte limit")
}

struct Dumper<'py> {
    output: LimitedWriter<fn() -> PyErr>,
    remaining_nodes: usize,
    max_depth: usize,
    // Hold a reference to every visited container so its identity stays valid.
    seen: BTreeMap<usize, (Bound<'py, PyAny>, usize)>,
    // First occurrences need an anchor only when we encounter an alias later.
    // Source order lets us insert all anchors with one linear copy.
    anchors: BTreeMap<usize, usize>,
}

impl<'py> Dumper<'py> {
    fn serialize(value: &Bound<'py, PyAny>, options: ParseOptions) -> PyResult<String> {
        let mut dumper = Self {
            output: LimitedWriter::new(options.max_input_bytes, output_limit),
            remaining_nodes: options.max_nodes,
            max_depth: options.max_depth,
            seen: BTreeMap::new(),
            anchors: BTreeMap::new(),
        };
        dumper.node(value, 1)?;
        dumper.output.push_str("\n")?;
        if dumper.anchors.is_empty() {
            return dumper.output.finish();
        }

        let mut anchored = LimitedWriter::new(options.max_input_bytes, output_limit);
        let mut start = 0;
        for (offset, label) in dumper.anchors {
            anchored.push_str(&dumper.output.as_ref()[start..offset])?;
            anchored.display(format_args!("&id{label} "))?;
            start = offset;
        }
        anchored.push_str(&dumper.output.as_ref()[start..])?;
        anchored.finish()
    }

    fn enter(&mut self, depth: usize) -> PyResult<()> {
        if depth > self.max_depth {
            return Err(YAMLError::new_err("serialization nesting limit exceeded"));
        }
        self.remaining_nodes = self.remaining_nodes.checked_sub(1).ok_or_else(node_limit)?;
        Ok(())
    }

    fn node(&mut self, value: &Bound<'py, PyAny>, depth: usize) -> PyResult<()> {
        self.enter(depth)?;
        if self.scalar(value)? {
            return Ok(());
        }

        if let Some(list) = as_builtin::<PyList>(value) {
            self.sequence(value, list.iter(), depth)
        } else if let Some(tuple) = as_builtin::<PyTuple>(value) {
            self.sequence(value, tuple.iter_borrowed(), depth)
        } else if let Some(dict) = as_builtin::<PyDict>(value) {
            self.mapping(dict, depth)
        } else {
            Err(PyTypeError::new_err(format!(
                "cannot serialize an object of type '{}'",
                value.get_type().name()?
            )))
        }
    }

    #[inline]
    fn scalar(&mut self, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Some(text) = as_builtin::<PyString>(value) {
            // CPython may allocate a UTF-8 copy on the first conversion.
            self.output
                .check(text.code_point_len()?.saturating_add(2))?;
            let text = crate::unicode_text(text)?;
            self.output.quote(text)?;
        } else if value.is_none() {
            self.output.push_str("null")?;
        } else if let Some(boolean) = as_builtin::<PyBool>(value) {
            self.output
                .push_str(if boolean.is_true() { "true" } else { "false" })?;
        } else if let Some(integer) = as_builtin::<PyInt>(value) {
            self.integer(integer)?;
        } else if let Some(float) = as_builtin::<PyFloat>(value) {
            let number = float.value();
            if let Some(spelling) = non_finite_float(number) {
                self.output.push_str(spelling)?;
            } else {
                // Debug formatting retains the fractional part of integral floats.
                self.output.display(format_args!("{number:?}"))?;
            }
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    fn integer(&mut self, integer: &Bound<'_, PyInt>) -> PyResult<()> {
        if let Ok(small) = integer.extract::<i64>() {
            return self.output.display(small);
        }

        // Each decimal digit carries fewer than four bits. Bound the size of
        // the temporary decimal string even when Python's digit limit is disabled.
        let bits: usize = integer.call_method0("bit_length")?.extract()?;
        if bits > self.output.remaining().saturating_mul(4) {
            return Err(output_limit());
        }
        let decimal = integer
            .str()
            .map_err(|err| crate::conversion_error(integer.py(), err))?;
        self.output.push_str(decimal.to_str()?)
    }

    fn alias(&mut self, value: &Bound<'py, PyAny>) -> PyResult<bool> {
        match self.seen.entry(value.as_ptr() as usize) {
            Entry::Vacant(entry) => {
                entry.insert((value.clone(), self.output.as_ref().len()));
                Ok(false)
            }
            Entry::Occupied(entry) => {
                let (_, offset) = entry.get();
                let next = self.anchors.len() + 1;
                let label = self.anchors.entry(*offset).or_insert(next);
                self.output.display(format_args!("*id{label}"))?;
                Ok(true)
            }
        }
    }

    fn sequence(
        &mut self,
        value: &Bound<'py, PyAny>,
        items: impl Iterator<Item = impl AsRef<Py<PyAny>>>,
        depth: usize,
    ) -> PyResult<()> {
        if self.alias(value)? {
            return Ok(());
        }

        self.output.push_str("[")?;
        let mut first = true;
        for item in items {
            self.output.separator(&mut first)?;
            self.node(item.as_ref().bind(value.py()), depth + 1)?;
        }
        self.output.push_str("]")
    }

    fn mapping(&mut self, dict: &Bound<'py, PyDict>, depth: usize) -> PyResult<()> {
        if self.alias(dict.as_any())? {
            return Ok(());
        }

        // Take strong references under the dictionary lock before recursing.
        // Serializing a child can run Python finalizers that mutate the original dictionary.
        let entries = pyo3::sync::critical_section::with_critical_section(dict, || {
            if dict.len() > self.remaining_nodes / 2 {
                return Err(node_limit());
            }
            Ok(dict.iter().collect::<Vec<_>>())
        })?;
        // Allow for a short scalar key and value, their quotes, and the separators.
        self.output
            .reserve(entries.len().saturating_mul(16).min(4096));
        self.output.push_str("{")?;
        let mut first = true;
        let mut has_nan = false;
        for (key, value) in entries {
            self.output.separator(&mut first)?;
            self.enter(depth + 1)?;
            let start = self.output.as_ref().len();
            if !self.scalar(&key)? {
                return Err(PyTypeError::new_err(
                    "mapping keys must be built-in strings, numbers, booleans, or None",
                ));
            }
            if as_builtin::<PyFloat>(&key).is_some_and(|n| n.value().is_nan())
                && std::mem::replace(&mut has_nan, true)
            {
                return Err(YAMLError::new_err("duplicate mapping key .nan"));
            }
            self.output.finish_implicit_key(start)?;
            self.node(&value, depth + 1)?;
        }
        self.output.push_str("}")
    }
}

fn node_limit() -> PyErr {
    YAMLError::new_err("serialization node limit exceeded")
}

#[cfg(test)]
mod tests;
