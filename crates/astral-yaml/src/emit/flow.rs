use std::fmt::{self, Write};

/// A YAML flow collection.
#[derive(Clone, Copy, Debug)]
pub enum Collection {
    /// A sequence enclosed in square brackets.
    Sequence,
    /// A mapping enclosed in braces.
    Mapping,
}

/// Writes flow-style YAML into a [`String`] or a byte-limited [`LimitedWriter`].
///
/// The caller traverses the value, choosing which scalars and collections to emit.
/// Operations on a limited writer return its configured error when the encoded
/// YAML exceeds the byte budget.
///
/// ```
/// use astral_yaml::emit::{Collection, FlowWrite, LimitedWriter};
///
/// let mut yaml = LimitedWriter::new(32, || "document too large");
/// yaml.open(Collection::Mapping)?;
/// yaml.quote("name")?;
/// yaml.end_key()?;
/// yaml.quote("Ada")?;
/// yaml.close(Collection::Mapping)?;
/// assert_eq!(yaml.finish()?, r#"{"name": "Ada"}"#);
/// # Ok::<(), &'static str>(())
/// ```
pub trait FlowWrite<E = fmt::Error>: AsRef<str> + Write + Sized {
    /// Take the completed output.
    ///
    /// # Errors
    /// Returns an error if an earlier write exceeded the byte limit.
    fn finish(self) -> Result<String, E>;

    /// Reserve space for upcoming output within the byte budget.
    fn reserve(&mut self, bytes: usize);

    /// Check the budget before allocating or converting an input value.
    ///
    /// # Errors
    /// Returns an error if the requested bytes exceed the budget.
    #[inline]
    fn check(&mut self, _: usize) -> Result<(), E> {
        Ok(())
    }

    /// Translate a formatting result into the writer's error type.
    ///
    /// # Errors
    /// Returns the writer's error when formatting failed.
    fn write_result(&self, result: fmt::Result) -> Result<(), E>;

    /// Insert an explicit-key indicator before an already emitted key.
    ///
    /// # Panics
    /// Panics if `start` is outside the output or falls inside a UTF-8 character.
    ///
    /// # Errors
    /// Returns an error if the indicator exceeds the budget.
    fn insert_explicit_key(&mut self, start: usize) -> Result<(), E>;

    /// Append preformatted YAML.
    ///
    /// # Errors
    /// Returns an error if the text exceeds the budget.
    #[inline]
    fn push_str(&mut self, value: &str) -> Result<(), E> {
        let result = self.write_str(value);
        self.write_result(result)
    }

    /// Write a double-quoted scalar, including any necessary escapes.
    ///
    /// # Errors
    /// Returns an error if the scalar exceeds the budget.
    #[inline]
    fn quote(&mut self, value: &str) -> Result<(), E> {
        self.check(value.len().saturating_add(2))?;
        if value.len() >= 64 || self.as_ref().is_empty() {
            self.reserve(value.len() + 3);
        }
        let result = super::write_quoted(self, value);
        self.write_result(result)
    }

    /// Append the display representation of a number or other YAML token.
    ///
    /// # Errors
    /// Returns an error if formatting the token fails.
    #[inline]
    fn display(&mut self, value: impl fmt::Display) -> Result<(), E> {
        let result = write!(self, "{value}");
        self.write_result(result)
    }

    /// Format a value into a separate string, reserving room for surrounding YAML.
    ///
    /// # Errors
    /// Returns the limit error on overflow. Other formatter errors convert to `E`.
    fn collect(
        &mut self,
        value: &(impl fmt::Display + ?Sized),
        overhead: usize,
    ) -> Result<String, E>
    where
        E: From<fmt::Error>;

    /// Start a flow sequence or mapping.
    ///
    /// # Errors
    /// Returns an error if the delimiter exceeds the budget.
    #[inline]
    fn open(&mut self, kind: Collection) -> Result<(), E> {
        self.push_str(match kind {
            Collection::Sequence => "[",
            Collection::Mapping => "{",
        })
    }

    /// Write a comma before each entry after the first in a flow collection.
    ///
    /// # Errors
    /// Returns an error if the separator exceeds the budget.
    #[inline]
    fn separator(&mut self, first: &mut bool) -> Result<(), E> {
        if !std::mem::take(first) {
            self.push_str(", ")?;
        }
        Ok(())
    }

    /// Separate a mapping key from its value.
    ///
    /// # Errors
    /// Returns an error if the separator exceeds the budget.
    #[inline]
    fn end_key(&mut self) -> Result<(), E> {
        self.push_str(": ")
    }

    /// Finish a single-line key, inserting `? ` if the encoded key exceeds
    /// the [implicit key limit] of 1024 source characters.
    ///
    /// [implicit key limit]: https://yaml.org/spec/1.2.2/#flow-mapping-implicit-keys
    ///
    /// # Panics
    /// Panics if `start` is outside the output or falls inside a UTF-8 character.
    ///
    /// # Errors
    /// Returns an error if the indicators exceed the budget.
    #[inline]
    fn finish_implicit_key(&mut self, start: usize) -> Result<(), E> {
        let key = &self.as_ref()[start..];
        if key.len() > 1024 && key.chars().count() > 1024 {
            self.insert_explicit_key(start)?;
        }
        self.end_key()
    }

    /// Close a flow sequence or mapping.
    ///
    /// # Errors
    /// Returns an error if the delimiter exceeds the budget.
    #[inline]
    fn close(&mut self, kind: Collection) -> Result<(), E> {
        self.push_str(match kind {
            Collection::Sequence => "]",
            Collection::Mapping => "}",
        })
    }
}

impl<E: From<fmt::Error>> FlowWrite<E> for String {
    #[inline]
    fn finish(self) -> Result<String, E> {
        Ok(self)
    }

    #[inline]
    fn reserve(&mut self, bytes: usize) {
        self.reserve(bytes);
    }

    #[inline]
    fn write_result(&self, result: fmt::Result) -> Result<(), E> {
        result.map_err(E::from)
    }

    fn insert_explicit_key(&mut self, start: usize) -> Result<(), E> {
        self.insert_str(start, "? ");
        Ok(())
    }

    fn collect(&mut self, value: &(impl fmt::Display + ?Sized), _: usize) -> Result<String, E>
    where
        E: From<fmt::Error>,
    {
        let mut text = String::new();
        write!(text, "{value}")?;
        Ok(text)
    }
}

/// A string writer that enforces a byte limit, including across formatted writes.
#[derive(Debug)]
pub struct LimitedWriter<F> {
    text: String,
    max_bytes: usize,
    failed: bool,
    on_limit: F,
}

impl<F> LimitedWriter<F> {
    /// Create an empty writer that reports `on_limit()` on overflow.
    pub fn new(max_bytes: usize, on_limit: F) -> Self {
        Self {
            text: String::new(),
            max_bytes,
            failed: false,
            on_limit,
        }
    }

    /// The remaining output budget in UTF-8 bytes.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.max_bytes - self.text.len()
    }

    #[inline]
    fn check_bytes(&mut self, bytes: usize) -> fmt::Result {
        self.failed |= bytes > self.remaining();
        if self.failed { Err(fmt::Error) } else { Ok(()) }
    }
}

impl<F> AsRef<str> for LimitedWriter<F> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.text
    }
}

impl<F> Write for LimitedWriter<F> {
    #[inline]
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.check_bytes(value.len())?;
        self.text.push_str(value);
        Ok(())
    }
}

impl<E, F: Fn() -> E> FlowWrite<E> for LimitedWriter<F> {
    fn finish(mut self) -> Result<String, E> {
        self.check(0)?;
        Ok(self.text)
    }

    #[inline]
    fn reserve(&mut self, bytes: usize) {
        self.text.reserve(bytes.min(self.remaining()));
    }

    #[inline]
    fn check(&mut self, bytes: usize) -> Result<(), E> {
        let result = self.check_bytes(bytes);
        self.write_result(result)
    }

    #[inline]
    fn write_result(&self, result: fmt::Result) -> Result<(), E> {
        result.map_err(|_| (self.on_limit)())
    }

    fn insert_explicit_key(&mut self, start: usize) -> Result<(), E> {
        self.check(2)?;
        self.text.insert_str(start, "? ");
        Ok(())
    }

    fn collect(
        &mut self,
        value: &(impl fmt::Display + ?Sized),
        overhead: usize,
    ) -> Result<String, E>
    where
        E: From<fmt::Error>,
    {
        self.check(overhead)?;
        let mut formatted = LimitedWriter::new(self.remaining() - overhead, &self.on_limit);
        let result = write!(formatted, "{value}");
        if formatted.failed {
            self.failed = true;
            return Err((self.on_limit)());
        }
        result?;
        Ok(formatted.text)
    }
}

#[cfg(test)]
mod tests {
    use super::{FlowWrite, LimitedWriter};

    // https://yaml.org/spec/1.2.2/#flow-mapping-implicit-keys
    #[test_case::test_case(1024, 2, Ok(""); "implicit key at the character limit")]
    #[test_case::test_case(1025, 4, Ok("? "); "both indicators fit")]
    #[test_case::test_case(1025, 1, Err("limit"); "no room for the explicit key indicator")]
    fn key_indicators_count_toward_the_byte_limit(
        length: usize,
        spare: usize,
        expected: Result<&str, &str>,
    ) {
        let key = "k".repeat(length);
        let mut writer = LimitedWriter::new(key.len() + spare, || "limit");
        writer.push_str(&key).unwrap();

        assert_eq!(writer.finish_implicit_key(0), expected.map(|_| ()));
        assert_eq!(
            writer.finish(),
            expected.map(|prefix| format!("{prefix}{key}: "))
        );
    }
}
