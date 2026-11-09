//! Construct Python values directly from the parser's node arena.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use astral_yaml::schema::{self, CoreType, ScalarType};
use astral_yaml::{
    Entry, NodeId, NodeKind, ParseOptions, Scalar, ScalarStyle, Source, Span, Stream,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString};

use crate::{YAMLError, unicode_text};

/// Deserialize a YAML document from text or encoded bytes.
#[pyfunction]
pub(super) fn loads<'py>(data: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let limit = ParseOptions::default().max_input_bytes;
    match Input::extract(data, limit)? {
        Input::Text(text) => document(data.py(), text, None),
        Input::Bytes(bytes) => decode(data.py(), &bytes),
    }
}

/// Deserialize a YAML document from an open text or binary file.
#[pyfunction]
pub(super) fn load<'py>(fp: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    read_document(fp, ParseOptions::default().max_input_bytes)
}

enum Input<'a> {
    Text(&'a str),
    Bytes(Cow<'a, [u8]>),
}

impl<'a> Input<'a> {
    fn extract(obj: &'a Bound<'_, PyAny>, limit: usize) -> PyResult<Self> {
        let input = if let Ok(text) = obj.cast::<PyString>() {
            // Encoding a Python string can allocate a UTF-8 cache.
            check_size(text.code_point_len()?, limit)?;
            Self::Text(unicode_text(text)?)
        } else if let Ok(bytes) = obj.cast::<PyBytes>() {
            Self::Bytes(Cow::Borrowed(bytes.as_bytes()))
        } else if let Ok(array) = obj.cast::<PyByteArray>() {
            let bytes = pyo3::sync::critical_section::with_critical_section(array, || {
                check_size(array.len(), limit)?;
                Ok::<_, PyErr>(array.to_vec())
            })?;
            return Ok(Self::Bytes(Cow::Owned(bytes)));
        } else {
            return Err(PyTypeError::new_err("expected str, bytes, or bytearray"));
        };

        check_size(input.as_bytes().len(), limit)?;
        Ok(input)
    }

    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }
}

fn check_size(size: usize, limit: usize) -> PyResult<()> {
    if size > limit {
        return Err(YAMLError::new_err("input byte limit exceeded"));
    }
    Ok(())
}

fn read_document<'py>(fp: &Bound<'py, PyAny>, limit: usize) -> PyResult<Bound<'py, PyAny>> {
    let read = fp
        .getattr_opt(pyo3::intern!(fp.py(), "read"))?
        .ok_or_else(|| {
            PyTypeError::new_err("expected a text or binary file with a read() method")
        })?;
    let mut buffer = Vec::new();
    let mut text = None;

    loop {
        // Text streams count characters and may return more UTF-8 bytes than requested.
        // At the limit, read one more unit to distinguish EOF from oversized input.
        let remaining = limit - buffer.len();
        let chunk = read.call1(((remaining + 1).min(64 * 1024),))?;
        let input = Input::extract(&chunk, remaining)?;
        let is_text = matches!(input, Input::Text(_));
        if *text.get_or_insert(is_text) != is_text {
            return Err(PyTypeError::new_err("read() mixed text and binary data"));
        }

        if input.as_bytes().is_empty() {
            return if is_text {
                let source = std::str::from_utf8(&buffer).expect("buffer contains only UTF-8 text");
                document(fp.py(), source, None)
            } else {
                decode(fp.py(), &buffer)
            };
        }
        buffer.extend_from_slice(input.as_bytes());
    }
}

fn decode<'py>(py: Python<'py>, bytes: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    let source = Source::from_bytes(bytes).map_err(|err| YAMLError::new_err(err.to_string()))?;
    document(py, source.text(), Some(&source))
}

fn located(message: impl fmt::Display, span: Span, source: Option<&Source<'_>>) -> PyErr {
    let span = source
        .and_then(|source| source.original_span(span))
        .unwrap_or(span);
    YAMLError::new_err(format!(
        "{message} at bytes {}..{}",
        span.start(),
        span.end()
    ))
}

fn document<'py>(
    py: Python<'py>,
    text: &str,
    encoded: Option<&Source<'_>>,
) -> PyResult<Bound<'py, PyAny>> {
    let stream = astral_yaml::parse(text).map_err(|err| located(err.message, err.span, encoded))?;
    match stream.documents() {
        [] => Ok(py.None().into_bound(py)),
        [doc] => Loader {
            py,
            stream: &stream,
            encoded,
            anchors: HashMap::new(),
            key_bytes: ParseOptions::default().max_input_bytes,
        }
        .value(doc.root),
        [_, second, ..] => Err(located("expected one YAML document", second.span, encoded)),
    }
}

struct Loader<'py, 'a> {
    py: Python<'py>,
    stream: &'a Stream<'a>,
    encoded: Option<&'a Source<'a>>,
    anchors: HashMap<NodeId, Bound<'py, PyAny>>,
    key_bytes: usize,
}

impl<'py> Loader<'py, '_> {
    fn error(&self, message: impl fmt::Display, span: Span) -> PyErr {
        located(message, span, self.encoded)
    }

    fn value(&mut self, id: NodeId) -> PyResult<Bound<'py, PyAny>> {
        let node = &self.stream.nodes()[id.index()];
        if let NodeKind::Alias { target, .. } = &node.kind {
            // YAML aliases refer to preceding anchors. Collections enter this cache
            // before their children, including aliases back to the collection itself.
            return Ok(self.anchors[target].clone());
        }

        let tag = match node.tag.as_ref().map(|tag| tag.value.as_ref()) {
            Some("!") if matches!(node.kind, NodeKind::Scalar(_)) => {
                Some(CoreType::Scalar(ScalarType::String))
            }
            None | Some("!") => None,
            Some(tag) => Some(CoreType::from_tag(tag).ok_or_else(|| {
                self.error(format_args!("unsupported YAML tag {tag}"), node.span)
            })?),
        };

        match (&node.kind, tag) {
            (NodeKind::Scalar(scalar), None | Some(CoreType::Scalar(_))) => {
                let kind = if let Some(CoreType::Scalar(kind)) = tag {
                    if !CoreType::Scalar(kind).matches_scalar(&scalar.value) {
                        return Err(self.error("scalar does not match its tag", scalar.span));
                    }
                    kind
                } else if scalar.style == ScalarStyle::Plain {
                    schema::classify(&scalar.value)
                } else {
                    ScalarType::String
                };

                let object = self.scalar(scalar, kind)?;
                self.remember(id, &object);
                Ok(object)
            }
            (NodeKind::Sequence { items, .. }, None | Some(CoreType::Sequence)) => {
                let list = PyList::empty(self.py);
                self.remember(id, list.as_any());
                for &item in items {
                    list.append(self.value(item)?)?;
                }
                Ok(list.into_any())
            }
            (NodeKind::Mapping { entries, .. }, None | Some(CoreType::Mapping)) => {
                self.mapping(id, entries)
            }
            _ => Err(self.error("tag does not match node kind", node.span)),
        }
    }

    fn mapping(&mut self, id: NodeId, entries: &[Entry]) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(self.py);
        self.remember(id, dict.as_any());
        let mut seen_nan = false;
        let mut hashes = None;

        for entry in entries {
            let key_node = &self.stream.nodes()[entry.key.index()];
            let span = key_node.span;
            let target = match &key_node.kind {
                NodeKind::Alias { target, .. } => &self.stream.nodes()[target.index()],
                _ => key_node,
            };
            let NodeKind::Scalar(scalar) = &target.kind else {
                return Err(self.error("collection keys cannot be used in a Python dict", span));
            };

            let key = self.value(entry.key)?;
            // Scalar keys other than strings and None are numbers (including bool).
            let duplicate_nan = if key.is_exact_instance_of::<PyString>() || key.is_none() {
                false
            } else {
                self.charge_numeric_key(
                    &key,
                    scalar.value.len(),
                    span,
                    hashes.get_or_insert_with(HashMap::new),
                )?;
                // Python considers distinct NaN objects unequal. YAML has one NaN key.
                key.cast::<PyFloat>().is_ok_and(|n| n.value().is_nan())
                    && std::mem::replace(&mut seen_nan, true)
            };
            if duplicate_nan || dict.contains(&key)? {
                return Err(self.error("duplicate mapping key", span));
            }
            dict.set_item(key, self.value(entry.value)?)?;
        }
        Ok(dict.into_any())
    }

    // Python's numeric hashes are deterministic. Constructing a dict from many
    // different integers with the same hash compares each key with all preceding
    // keys in that bucket. Integer hashing and comparison can also scan its digits.
    fn charge_numeric_key(
        &mut self,
        key: &Bound<'py, PyAny>,
        digits: usize,
        span: Span,
        hashes: &mut HashMap<isize, usize>,
    ) -> PyResult<()> {
        let count = hashes.entry(key.hash()?).or_default();
        *count += 1;
        let bytes = digits.saturating_mul(*count);
        self.key_bytes = self
            .key_bytes
            .checked_sub(bytes)
            .ok_or_else(|| self.error("numeric mapping key work limit exceeded", span))?;
        Ok(())
    }

    fn remember(&mut self, id: NodeId, value: &Bound<'py, PyAny>) {
        if self.stream.nodes()[id.index()].anchor.is_some() {
            self.anchors.insert(id, value.clone());
        }
    }

    fn scalar(&self, scalar: &Scalar<'_>, kind: ScalarType) -> PyResult<Bound<'py, PyAny>> {
        let text = scalar.value.as_ref();
        Ok(match kind {
            ScalarType::Null => self.py.None().into_bound(self.py),
            ScalarType::Bool => PyBool::new(self.py, text.eq_ignore_ascii_case("true"))
                .to_owned()
                .into_any(),
            ScalarType::String => PyString::new(self.py, text).into_any(),
            ScalarType::Int => {
                // Base zero would reject YAML decimal integers with leading zeros.
                let (digits, base) = schema::radix_digits(text).unwrap_or((text, 10));
                // Short integers can be converted without constructing a Python string.
                // Longer decimals go through int() to preserve Python's digit limit.
                if digits.len() <= 19
                    && let Ok(value) = i64::from_str_radix(digits, base)
                {
                    return Ok(value.into_pyobject(self.py)?.into_any());
                }
                self.py
                    .get_type::<PyInt>()
                    .call1((digits, base))
                    .map_err(|err| self.error(err, scalar.span))?
            }
            ScalarType::Float => {
                let value = if let Some(number) = schema::special_float(text) {
                    number
                } else {
                    let value: f64 = text.parse().expect("classified decimal float");
                    if !value.is_finite() {
                        return Err(self.error("float out of range", scalar.span));
                    }
                    value
                };
                PyFloat::new(self.py, value).into_any()
            }
        })
    }
}

#[cfg(test)]
mod tests;
