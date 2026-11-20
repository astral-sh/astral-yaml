//! Serialize Python built-ins to a YAML document.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

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

#[derive(Clone, Copy)]
enum Position {
    Inline(usize),
    AfterKey(usize),
}

impl Position {
    fn depth(self) -> usize {
        match self {
            Self::Inline(depth) | Self::AfterKey(depth) => depth,
        }
    }

    fn indent(self) -> usize {
        (self.depth() - 1) * 2
    }

    fn start(self, output: &mut Output, block_indent: Option<usize>) -> PyResult<()> {
        if let Self::AfterKey(_) = self {
            separate(output, block_indent)?;
        }
        Ok(())
    }
}

fn separate(output: &mut Output, block_indent: Option<usize>) -> PyResult<()> {
    match block_indent {
        Some(indent) => newline(output, indent),
        None => output.push_str(" "),
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
        dumper.node(value, Position::Inline(1))?;
        dumper.output.push_str("\n")?;
        if dumper.anchors.is_empty() {
            return dumper.output.finish();
        }

        let mut anchored = Output::new(options.max_input_bytes, output_limit);
        let mut start = 0;
        for (site, label) in dumper.anchors {
            anchored.push_str(&dumper.output.as_ref()[start..site.offset])?;
            anchored.display(format_args!("&id{label}"))?;
            separate(&mut anchored, site.block_indent)?;
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

    fn node(&mut self, value: &Bound<'py, PyAny>, position: Position) -> PyResult<()> {
        self.enter(position.depth())?;
        if let Some(list) = as_builtin::<PyList>(value) {
            self.sequence(value, list.iter(), position)
        } else if let Some(tuple) = as_builtin::<PyTuple>(value) {
            self.sequence(value, tuple.iter_borrowed(), position)
        } else if let Some(dict) = as_builtin::<PyDict>(value) {
            self.mapping(dict, position)
        } else {
            position.start(&mut self.output, None)?;
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

    /// Prepare entries only on the first visit. Record the anchor site before
    /// writing any entry, so a child can refer back to this collection.
    fn collection<I: Iterator>(
        &mut self,
        value: &Bound<'py, PyAny>,
        position: Position,
        (opening, closing): (&str, &str),
        prepare: impl FnOnce(&Budget, &mut Output) -> PyResult<I>,
        mut write_entry: impl FnMut(&mut Self, I::Item) -> PyResult<()>,
    ) -> PyResult<()> {
        let slot = match self.seen.entry(value.as_ptr() as usize) {
            Entry::Vacant(slot) => slot,
            Entry::Occupied(slot) => {
                let (_, site) = slot.get();
                let next = self.anchors.len() + 1;
                let label = self.anchors.entry(*site).or_insert(next);
                position.start(&mut self.output, None)?;
                return self.output.display(format_args!("*id{label}"));
            }
        };

        let mut entries = prepare(&self.nodes, &mut self.output)?.peekable();
        let block = self.style == CollectionStyle::Block && entries.peek().is_some();
        let block_indent = block.then_some(position.indent());
        position.start(&mut self.output, block_indent)?;
        let site = AnchorSite {
            offset: self.output.as_ref().len(),
            block_indent,
        };
        slot.insert((value.clone(), site));

        if let Some(indent) = block_indent {
            for (index, entry) in entries.enumerate() {
                if index > 0 {
                    newline(&mut self.output, indent)?;
                }
                write_entry(self, entry)?;
            }
            Ok(())
        } else {
            self.output.push_str(opening)?;
            let mut first = true;
            for entry in entries {
                self.output.separator(&mut first)?;
                write_entry(self, entry)?;
            }
            self.output.push_str(closing)
        }
    }

    fn sequence(
        &mut self,
        value: &Bound<'py, PyAny>,
        items: impl Iterator<Item = impl AsRef<Py<PyAny>>>,
        position: Position,
    ) -> PyResult<()> {
        let child = Position::Inline(position.depth() + 1);
        self.collection(
            value,
            position,
            ("[", "]"),
            |_, _| Ok(items),
            |dumper, item| {
                if dumper.style == CollectionStyle::Block {
                    dumper.output.push_str("- ")?;
                }
                dumper.node(item.as_ref().bind(value.py()), child)
            },
        )
    }

    fn mapping(&mut self, dict: &Bound<'py, PyDict>, position: Position) -> PyResult<()> {
        let prepare = |nodes: &Budget, output: &mut Output| {
            // Take strong references under the dictionary lock before recursing.
            // Serializing a child can run Python finalizers that mutate the original dictionary.
            let entries = pyo3::sync::critical_section::with_critical_section(dict, || {
                if dict.len() > nodes.remaining() / 2 {
                    return Err(node_limit());
                }
                Ok(dict.iter().collect::<Vec<_>>())
            })?;
            // Allow for a short scalar key and value, their quotes, and the separators.
            output.reserve(entries.len().saturating_mul(16).min(4096));
            Ok(entries.into_iter())
        };

        let mut has_nan = false;
        self.collection(
            dict.as_any(),
            position,
            ("{", "}"),
            prepare,
            |dumper, (key, value)| {
                dumper.key(&key, position, &mut has_nan)?;
                dumper.node(&value, Position::AfterKey(position.depth() + 1))
            },
        )
    }

    fn key(
        &mut self,
        key: &Bound<'py, PyAny>,
        position: Position,
        has_nan: &mut bool,
    ) -> PyResult<()> {
        self.enter(position.depth() + 1)?;
        let start = self.output.as_ref().len();
        if !self.scalar(key)? {
            return Err(PyTypeError::new_err(
                "mapping keys must be built-in strings, numbers, booleans, or None",
            ));
        }
        if as_builtin::<PyFloat>(key).is_some_and(|n| n.value().is_nan())
            && std::mem::replace(has_nan, true)
        {
            return Err(YAMLError::new_err("duplicate mapping key .nan"));
        }

        if self.output.mark_explicit_key(start)? && self.style == CollectionStyle::Block {
            newline(&mut self.output, position.indent())?;
        }
        self.output.push_str(":")
    }
}

fn node_limit() -> PyErr {
    YAMLError::new_err("serialization node limit exceeded")
}

#[cfg(test)]
mod tests;
