//! Serialize Python built-ins to a YAML document.

use std::collections::BTreeMap;

use astral_yaml::emit::{FlowWrite, LimitedWriter, non_finite_float, plain_block_string_key};
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

fn newline(output: &mut Output, mut indent: usize) -> PyResult<()> {
    const SPACES: &str = "                ";
    output.push_str("\n")?;
    while indent > SPACES.len() {
        output.push_str(SPACES)?;
        indent -= SPACES.len();
    }
    output.push_str(&SPACES[..indent])
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct AnchorSite {
    offset: usize,
    block_indent: Option<usize>,
}

// Hold a reference to each container so its identity stays valid.
type Visit<'py> = (Bound<'py, PyAny>, AnchorSite);

#[derive(Default)]
struct Visits<'py> {
    // Keep short documents allocation-free for identity tracking.
    inline: [Option<Visit<'py>>; 2],
    overflow: BTreeMap<usize, Option<Visit<'py>>>,
}

impl<'py> Visits<'py> {
    #[inline(never)]
    fn slot(&mut self, value: &Bound<'py, PyAny>) -> &mut Option<Visit<'py>> {
        for slot in &mut self.inline {
            if slot.as_ref().is_none_or(|(seen, _)| seen.is(value)) {
                return slot;
            }
        }
        self.overflow.entry(value.as_ptr() as usize).or_default()
    }
}

struct Dumper<'py> {
    output: Output,
    nodes: Budget,
    max_depth: usize,
    style: CollectionStyle,
    seen: Visits<'py>,
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
            seen: Visits::default(),
            anchors: BTreeMap::new(),
        };
        dumper.node(value, Position::Inline(1))?;
        dumper.output.push_str("\n")?;
        if !dumper.anchors.is_empty() {
            let mut anchored = Output::new(options.max_input_bytes, output_limit);
            let mut start = 0;
            for (site, label) in dumper.anchors {
                anchored.push_str(&dumper.output.as_ref()[start..site.offset])?;
                anchored.display(format_args!("&id{label}"))?;
                separate(&mut anchored, site.block_indent)?;
                start = site.offset;
            }
            anchored.push_str(&dumper.output.as_ref()[start..])?;
            dumper.output = anchored;
        }
        // A plain key can start the stream with a non-ASCII character.
        // Check after inserting anchors, which can supply an ASCII prefix.
        if !dumper.output.as_ref().as_bytes()[0].is_ascii() {
            dumper.nodes.try_charge(1).ok_or_else(node_limit)?;
            dumper.output.check('\u{feff}'.len_utf8())?;
            let mut text = dumper.output.finish()?;
            text.insert(0, '\u{feff}');
            return Ok(text);
        }
        dumper.output.finish()
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
            if self.scalar(value, false)? {
                return Ok(());
            }
            Err(PyTypeError::new_err(format!(
                "cannot serialize an object of type '{}'",
                value.get_type().name()?
            )))
        }
    }

    #[inline(never)]
    fn scalar(&mut self, value: &Bound<'_, PyAny>, block_key: bool) -> PyResult<bool> {
        if let Some(text) = as_builtin::<PyString>(value) {
            // CPython may allocate a UTF-8 copy on the first conversion.
            self.output
                .check(text.code_point_len()?.saturating_add(2))?;
            let text = crate::unicode_text(text)?;
            if block_key && plain_block_string_key(text) {
                self.output.push_str(text)?;
            } else {
                self.output.quote(text)?;
            }
        } else if value.is_none() {
            self.output.push_str("null")?;
        } else if let Some(boolean) = as_builtin::<PyBool>(value) {
            self.output
                .push_str(if boolean.is_true() { "true" } else { "false" })?;
        } else if let Some(integer) = as_builtin::<PyInt>(value) {
            self.integer(integer)?;
        } else if let Some(float) = as_builtin::<PyFloat>(value) {
            write_float(&mut self.output, float.value())?;
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    fn integer(&mut self, integer: &Bound<'_, PyInt>) -> PyResult<()> {
        if let Ok(small) = integer.extract::<i64>() {
            return write_decimal(&mut self.output, small, 0);
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
        let slot = self.seen.slot(value);
        if let Some((_, site)) = slot {
            let site = *site;
            return self.alias(site, position);
        }

        let mut entries = prepare(&self.nodes, &mut self.output)?;
        let first = entries.next();
        let block = self.style == CollectionStyle::Block && first.is_some();
        let block_indent = block.then_some(position.indent());
        position.start(&mut self.output, block_indent)?;
        let site = AnchorSite {
            offset: self.output.as_ref().len(),
            block_indent,
        };
        *slot = Some((value.clone(), site));

        if !block {
            self.output.push_str(opening)?;
        }
        if let Some(first) = first {
            write_entry(self, first)?;
            for entry in entries {
                match block_indent {
                    Some(indent) => newline(&mut self.output, indent)?,
                    None => self.output.push_str(", ")?,
                }
                write_entry(self, entry)?;
            }
        }
        if !block {
            self.output.push_str(closing)?;
        }
        Ok(())
    }

    #[cold]
    fn alias(&mut self, site: AnchorSite, position: Position) -> PyResult<()> {
        let next = self.anchors.len() + 1;
        let label = self.anchors.entry(site).or_insert(next);
        position.start(&mut self.output, None)?;
        self.output.display(format_args!("*id{label}"))
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
            // Allow for the key, value, and their surrounding YAML syntax.
            output.reserve(entries.len().saturating_mul(24).min(4096));
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
        if !self.scalar(key, self.style == CollectionStyle::Block)? {
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

fn write_float(output: &mut Output, number: f64) -> PyResult<()> {
    if let Some(spelling) = non_finite_float(number) {
        return output.push_str(spelling);
    }

    // In this range Debug uses fixed notation and each float's rounding
    // interval contains at most one millionth. If that millionth rounds
    // back to the input, its shortest spelling ends at the last nonzero
    // fractional digit (or .0 for an integer).
    let scaled = (number * 1e6).round();
    if !(1e-4..1e9).contains(&number.abs()) || (scaled / 1e6).to_bits() != number.to_bits() {
        return output.display(format_args!("{number:?}"));
    }
    #[allow(clippy::cast_possible_truncation)] // Rounded and bounded by 1e15.
    let mut value = scaled as i64;
    let mut places = 6;
    while places > 1 && value % 10 == 0 {
        value /= 10;
        places -= 1;
    }
    write_decimal(output, value, places)
}

/// Write a signed coefficient with `places` fractional digits.
fn write_decimal(output: &mut Output, value: i64, places: usize) -> PyResult<()> {
    let mut digits = value.unsigned_abs();
    let mut buffer = [0; 21]; // 19 digits, a sign, and a decimal point.
    let mut start = buffer.len();
    let mut prepend = |byte| {
        start -= 1;
        buffer[start] = byte;
    };

    for _ in 0..places {
        prepend(b'0' + (digits % 10) as u8);
        digits /= 10;
    }
    if places > 0 {
        prepend(b'.');
    }

    loop {
        prepend(b'0' + (digits % 10) as u8);
        digits /= 10;
        if digits == 0 {
            break;
        }
    }
    if value < 0 {
        prepend(b'-');
    }
    output.push_str(std::str::from_utf8(&buffer[start..]).expect("ASCII decimal"))
}

fn node_limit() -> PyErr {
    YAMLError::new_err("serialization node limit exceeded")
}

#[cfg(test)]
mod tests;
