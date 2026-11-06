//! Serialize Python built-ins to a YAML document.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};

use astral_yaml::{ParseOptions, emit};
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
        .getattr_opt("write")?
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

struct Output {
    text: String,
    max_bytes: usize,
}

impl Output {
    fn new(max_bytes: usize) -> Self {
        Self {
            text: String::new(),
            max_bytes,
        }
    }

    fn check(&self, additional: usize) -> PyResult<()> {
        if additional > self.remaining() {
            return Err(output_limit());
        }
        Ok(())
    }

    fn remaining(&self) -> usize {
        self.max_bytes - self.text.len()
    }

    fn push(&mut self, value: &str) -> PyResult<()> {
        self.check(value.len())?;
        self.text.push_str(value);
        Ok(())
    }

    // An implicit key can contain at most 1024 source characters, including quotes.
    fn finish_key(&mut self, start: usize) -> PyResult<()> {
        let key = &self.text[start..];
        if key.len() > 1024 && key.chars().count() > 1024 {
            self.check(2)?;
            self.text.insert_str(start, "? ");
        }
        self.push(": ")
    }
}

impl Write for Output {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if value.len() > self.remaining() {
            return Err(fmt::Error);
        }
        self.text.push_str(value);
        Ok(())
    }
}

fn output_limit() -> PyErr {
    YAMLError::new_err("serialized YAML exceeds the output byte limit")
}

struct Dumper<'py> {
    output: Output,
    remaining_nodes: usize,
    max_depth: usize,
    // Hold a reference to every visited container so its identity stays valid.
    seen: HashMap<usize, (Bound<'py, PyAny>, usize)>,
    // First occurrences need an anchor only when we encounter an alias later.
    // Source order lets us insert all anchors with one linear copy.
    anchors: BTreeMap<usize, usize>,
}

impl<'py> Dumper<'py> {
    fn serialize(value: &Bound<'py, PyAny>, options: ParseOptions) -> PyResult<String> {
        let mut dumper = Self {
            output: Output::new(options.max_input_bytes),
            remaining_nodes: options.max_nodes,
            max_depth: options.max_depth,
            seen: HashMap::new(),
            anchors: BTreeMap::new(),
        };
        dumper.node(value, 1)?;
        dumper.output.push("\n")?;
        if dumper.anchors.is_empty() {
            return Ok(dumper.output.text);
        }

        let mut anchored = Output::new(options.max_input_bytes);
        let mut start = 0;
        for (offset, label) in dumper.anchors {
            anchored.push(&dumper.output.text[start..offset])?;
            write!(anchored, "&id{label} ").map_err(|_| output_limit())?;
            start = offset;
        }
        anchored.push(&dumper.output.text[start..])?;
        Ok(anchored.text)
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

        if let Ok(list) = value.cast_exact::<PyList>() {
            self.sequence(value, list.iter(), depth)
        } else if let Ok(tuple) = value.cast_exact::<PyTuple>() {
            self.sequence(value, tuple.iter(), depth)
        } else if let Ok(dict) = value.cast_exact::<PyDict>() {
            self.mapping(dict, depth)
        } else {
            Err(PyTypeError::new_err(format!(
                "cannot serialize an object of type '{}'",
                value.get_type().name()?
            )))
        }
    }

    fn scalar(&mut self, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        if value.is_none() {
            self.output.push("null")?;
        } else if let Ok(text) = value.cast_exact::<PyString>() {
            // CPython may allocate a UTF-8 copy on the first conversion.
            self.output
                .check(text.code_point_len()?.saturating_add(2))?;
            let text = crate::unicode_text(text)?;
            self.output.check(text.len().saturating_add(2))?;
            emit::write_quoted(&mut self.output, text).map_err(|_| output_limit())?;
        } else if let Ok(boolean) = value.cast_exact::<PyBool>() {
            self.output
                .push(if boolean.is_true() { "true" } else { "false" })?;
        } else if let Ok(integer) = value.cast_exact::<PyInt>() {
            self.integer(integer)?;
        } else if let Ok(float) = value.cast_exact::<PyFloat>() {
            let number = float.value();
            if number.is_nan() {
                self.output.push(".nan")?;
            } else if number == f64::INFINITY {
                self.output.push(".inf")?;
            } else if number == f64::NEG_INFINITY {
                self.output.push("-.inf")?;
            } else {
                // Python's float repr preserves the type and round-trips through f64.
                self.output.push(float.repr()?.to_str()?)?;
            }
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    fn integer(&mut self, integer: &Bound<'_, PyInt>) -> PyResult<()> {
        if let Ok(small) = integer.extract::<i64>() {
            return write!(self.output, "{small}").map_err(|_| output_limit());
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
        self.output.push(decimal.to_str()?)
    }

    fn alias(&mut self, value: &Bound<'py, PyAny>) -> PyResult<bool> {
        match self.seen.entry(value.as_ptr() as usize) {
            Entry::Vacant(entry) => {
                entry.insert((value.clone(), self.output.text.len()));
                Ok(false)
            }
            Entry::Occupied(entry) => {
                let (_, offset) = entry.get();
                let next = self.anchors.len() + 1;
                let label = self.anchors.entry(*offset).or_insert(next);
                write!(self.output, "*id{label}").map_err(|_| output_limit())?;
                Ok(true)
            }
        }
    }

    fn sequence(
        &mut self,
        value: &Bound<'py, PyAny>,
        items: impl Iterator<Item = Bound<'py, PyAny>>,
        depth: usize,
    ) -> PyResult<()> {
        if self.alias(value)? {
            return Ok(());
        }

        self.output.push("[")?;
        for (index, item) in items.enumerate() {
            if index > 0 {
                self.output.push(", ")?;
            }
            self.node(&item, depth + 1)?;
        }
        self.output.push("]")
    }

    fn mapping(&mut self, dict: &Bound<'py, PyDict>, depth: usize) -> PyResult<()> {
        if self.alias(dict.as_any())? {
            return Ok(());
        }

        if dict.len() > self.remaining_nodes / 2 {
            return Err(node_limit());
        }
        self.output.push("{")?;
        let mut has_nan = false;
        // Snapshot the entries before recursing. Python can run finalizers during
        // allocations, and PyO3's dictionary iterator panics if its keys change.
        for (index, (key, value)) in dict.copy()?.iter().enumerate() {
            if index > 0 {
                self.output.push(", ")?;
            }
            self.enter(depth + 1)?;
            let start = self.output.text.len();
            if !self.scalar(&key)? {
                return Err(PyTypeError::new_err(
                    "mapping keys must be built-in strings, numbers, booleans, or None",
                ));
            }
            if &self.output.text[start..] == ".nan" && std::mem::replace(&mut has_nan, true) {
                return Err(YAMLError::new_err("duplicate mapping key .nan"));
            }
            self.output.finish_key(start)?;
            self.node(&value, depth + 1)?;
        }
        self.output.push("}")
    }
}

fn node_limit() -> PyErr {
    YAMLError::new_err("serialization node limit exceeded")
}

#[cfg(test)]
mod tests;
