//! Serialize Python built-ins to a YAML document.

use std::collections::BTreeMap;

use astral_yaml::emit::{FlowWrite, LimitedWriter, non_finite_float};
use astral_yaml::{Budget, CollectionStyle, ParseOptions};
use pyo3::exceptions::{PyOSError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};

use crate::YAMLError;

/// Serialize a Python value as a YAML 1.2 document.
///
/// Use style="flow" for compact collections with brackets and braces.
#[pyfunction(signature = (obj, *, style = "block"))]
pub(super) fn dumps(obj: &Bound<'_, PyAny>, style: &str) -> PyResult<String> {
    Dumper::serialize(obj, ParseOptions::default(), parse_style(style)?)
}

/// Serialize a Python value and write the document to a text stream.
///
/// Use style="flow" for compact collections with brackets and braces.
#[pyfunction(signature = (obj, fp, *, style = "block"))]
pub(super) fn dump(obj: &Bound<'_, PyAny>, fp: &Bound<'_, PyAny>, style: &str) -> PyResult<()> {
    let style = parse_style(style)?;
    let write = fp
        .getattr_opt(pyo3::intern!(fp.py(), "write"))?
        .ok_or_else(|| PyTypeError::new_err("expected a text stream with a write method"))?;
    let text = Dumper::serialize(obj, ParseOptions::default(), style)?;
    let text = PyString::new(obj.py(), &text);
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

fn parse_style(style: &str) -> PyResult<CollectionStyle> {
    match style {
        "block" => Ok(CollectionStyle::Block),
        "flow" => Ok(CollectionStyle::Flow),
        _ => Err(PyValueError::new_err("style must be 'block' or 'flow'")),
    }
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

type Output = LimitedWriter<fn() -> PyErr>;

/// Where a node starts relative to its enclosing collection's indicator.
#[derive(Clone, Copy)]
enum Position {
    Inline,
    SequenceItem(usize),
    MappingValue(usize),
}

impl Position {
    fn start_inline(self, output: &mut Output) -> PyResult<()> {
        match self {
            Self::Inline => Ok(()),
            Self::SequenceItem(_) | Self::MappingValue(_) => output.push_str(" "),
        }
    }

    fn start_block(self, output: &mut Output) -> PyResult<usize> {
        match self {
            Self::Inline => Ok(0),
            Self::SequenceItem(indent) => {
                self.start_inline(output)?;
                Ok(indent)
            }
            Self::MappingValue(indent) => {
                newline(output, indent)?;
                Ok(indent)
            }
        }
    }
}

fn newline(output: &mut Output, indent: usize) -> PyResult<()> {
    output.display(format_args!("\n{:indent$}", ""))
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct AnchorSite {
    offset: usize,
    block_indent: Option<usize>,
}

struct Dumper<'py> {
    output: Output,
    nodes: Budget,
    max_depth: usize,
    style: CollectionStyle,
    // Hold a reference to every visited container so its identity stays valid.
    seen: BTreeMap<usize, (Bound<'py, PyAny>, AnchorSite)>,
    // First occurrences need an anchor only when we encounter an alias later.
    // Source order lets us insert all anchors with one linear copy.
    anchors: BTreeMap<AnchorSite, usize>,
}

impl<'py> Dumper<'py> {
    fn serialize(
        value: &Bound<'py, PyAny>,
        options: ParseOptions,
        style: CollectionStyle,
    ) -> PyResult<String> {
        let mut dumper = Self {
            output: LimitedWriter::new(options.max_input_bytes, output_limit),
            nodes: Budget::new(options.max_nodes),
            max_depth: options.max_depth,
            style,
            seen: BTreeMap::new(),
            anchors: BTreeMap::new(),
        };
        dumper.node(value, 1, Position::Inline)?;
        dumper.output.push_str("\n")?;
        if dumper.anchors.is_empty() {
            return dumper.output.finish();
        }

        let mut anchored = Output::new(options.max_input_bytes, output_limit);
        let mut start = 0;
        for (site, label) in dumper.anchors {
            anchored.push_str(&dumper.output.as_ref()[start..site.offset])?;
            anchored.display(format_args!("&id{label}"))?;
            if let Some(indent) = site.block_indent {
                newline(&mut anchored, indent)?;
            } else {
                anchored.push_str(" ")?;
            }
            start = site.offset;
        }
        anchored.push_str(&dumper.output.as_ref()[start..])?;
        anchored.finish()
    }

    fn enter(&self, depth: usize) -> PyResult<()> {
        if depth > self.max_depth {
            return Err(YAMLError::new_err("serialization nesting limit exceeded"));
        }
        self.nodes.try_charge(1).ok_or_else(node_limit)
    }

    fn node(
        &mut self,
        value: &Bound<'py, PyAny>,
        depth: usize,
        position: Position,
    ) -> PyResult<()> {
        self.enter(depth)?;
        if let Some(list) = as_builtin::<PyList>(value) {
            self.sequence(value, list.iter(), depth, position)
        } else if let Some(tuple) = as_builtin::<PyTuple>(value) {
            self.sequence(value, tuple.iter_borrowed(), depth, position)
        } else if let Some(dict) = as_builtin::<PyDict>(value) {
            self.mapping(dict, depth, position)
        } else {
            position.start_inline(&mut self.output)?;
            if self.scalar(value)? {
                return Ok(());
            }
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

    fn alias(&mut self, value: &Bound<'py, PyAny>, position: Position) -> PyResult<bool> {
        let Some((_, site)) = self.seen.get(&(value.as_ptr() as usize)) else {
            return Ok(false);
        };
        let next = self.anchors.len() + 1;
        let label = self.anchors.entry(*site).or_insert(next);
        position.start_inline(&mut self.output)?;
        self.output.display(format_args!("*id{label}"))?;
        Ok(true)
    }

    fn start_collection(
        &mut self,
        value: &Bound<'py, PyAny>,
        position: Position,
        block: bool,
        opening: &str,
    ) -> PyResult<Option<usize>> {
        let block_indent = if block {
            Some(position.start_block(&mut self.output)?)
        } else {
            position.start_inline(&mut self.output)?;
            None
        };
        self.seen.insert(
            value.as_ptr() as usize,
            (
                value.clone(),
                AnchorSite {
                    offset: self.output.as_ref().len(),
                    block_indent,
                },
            ),
        );
        if block_indent.is_none() {
            self.output.push_str(opening)?;
        }
        Ok(block_indent)
    }

    fn entry_separator(&mut self, first: &mut bool, block_indent: Option<usize>) -> PyResult<()> {
        if let Some(indent) = block_indent {
            if std::mem::take(first) {
                Ok(())
            } else {
                newline(&mut self.output, indent)
            }
        } else {
            self.output.separator(first)
        }
    }

    fn sequence(
        &mut self,
        value: &Bound<'py, PyAny>,
        items: impl Iterator<Item = impl AsRef<Py<PyAny>>>,
        depth: usize,
        position: Position,
    ) -> PyResult<()> {
        if self.alias(value, position)? {
            return Ok(());
        }

        let mut items = items.peekable();
        let block = self.style == CollectionStyle::Block && items.peek().is_some();
        let block_indent = self.start_collection(value, position, block, "[")?;
        let mut first = true;
        for item in items {
            self.entry_separator(&mut first, block_indent)?;
            let position = if let Some(indent) = block_indent {
                self.output.push_str("-")?;
                Position::SequenceItem(indent + 2)
            } else {
                Position::Inline
            };
            self.node(item.as_ref().bind(value.py()), depth + 1, position)?;
        }
        if block_indent.is_none() {
            self.output.push_str("]")?;
        }
        Ok(())
    }

    fn mapping(
        &mut self,
        dict: &Bound<'py, PyDict>,
        depth: usize,
        position: Position,
    ) -> PyResult<()> {
        if self.alias(dict.as_any(), position)? {
            return Ok(());
        }

        // Take strong references under the dictionary lock before recursing.
        // Serializing a child can run Python finalizers that mutate the original dictionary.
        let entries = pyo3::sync::critical_section::with_critical_section(dict, || {
            if dict.len() > self.nodes.remaining() / 2 {
                return Err(node_limit());
            }
            Ok(dict.iter().collect::<Vec<_>>())
        })?;
        // Allow for a short scalar key and value, their quotes, and the separators.
        self.output
            .reserve(entries.len().saturating_mul(16).min(4096));
        let block = self.style == CollectionStyle::Block && !entries.is_empty();
        let block_indent = self.start_collection(dict.as_any(), position, block, "{")?;
        let mut first = true;
        let mut has_nan = false;
        for (key, value) in entries {
            self.entry_separator(&mut first, block_indent)?;
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
            let position = if let Some(indent) = block_indent {
                let key = &self.output.as_ref()[start..];
                // https://yaml.org/spec/1.2.2/#822-block-mappings
                if key.len() > 1024 && key.chars().count() > 1024 {
                    self.output.insert_explicit_key(start)?;
                    newline(&mut self.output, indent)?;
                }
                self.output.push_str(":")?;
                Position::MappingValue(indent + 2)
            } else {
                self.output.finish_implicit_key(start)?;
                Position::Inline
            };
            self.node(&value, depth + 1, position)?;
        }
        if block_indent.is_none() {
            self.output.push_str("}")?;
        }
        Ok(())
    }
}

fn node_limit() -> PyErr {
    YAMLError::new_err("serialization node limit exceeded")
}

#[cfg(test)]
mod tests;
