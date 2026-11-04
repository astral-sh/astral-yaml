use super::quoted;
use crate::Error;
use std::fmt::{self, Write};

#[cfg(feature = "editable")]
use astral_yaml::Span;

/// The serializer writes directly to a String unless an edit supplies a byte limit.
pub(super) trait Output: AsRef<str> + Write {
    fn finish(self) -> Result<String, Error>;
    fn reserve(&mut self, bytes: usize);
    fn quote(&mut self, value: &str) -> Result<(), Error>;
    fn write_result(&self, result: fmt::Result) -> Result<(), Error>;

    #[inline]
    fn check(&mut self, _: usize) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn push(&mut self, ch: char) -> Result<(), Error> {
        let result = self.write_char(ch);
        self.write_result(result)
    }

    #[inline]
    fn push_str(&mut self, value: &str) -> Result<(), Error> {
        let result = self.write_str(value);
        self.write_result(result)
    }

    #[inline]
    fn number(&mut self, value: impl fmt::Display) -> Result<(), Error> {
        let result = write!(self, "{value}");
        self.write_result(result)
    }

    #[inline]
    fn collect(&mut self, value: &(impl fmt::Display + ?Sized), _: usize) -> Result<String, Error> {
        Ok(value.to_string())
    }
}

impl Output for String {
    #[inline]
    fn finish(self) -> Result<String, Error> {
        Ok(self)
    }

    #[inline]
    fn reserve(&mut self, bytes: usize) {
        self.reserve(bytes);
    }

    #[inline]
    fn quote(&mut self, value: &str) -> Result<(), Error> {
        quoted::write_quoted(self, value);
        Ok(())
    }

    #[inline]
    fn write_result(&self, result: fmt::Result) -> Result<(), Error> {
        result.expect("String write");
        Ok(())
    }
}

#[cfg(feature = "editable")]
pub(crate) const INSERTED_BYTE_LIMIT: &str = "inserted YAML exceeds input byte limit";

#[cfg(feature = "editable")]
pub(super) struct LimitedOutput {
    text: String,
    max_bytes: usize,
    span: Span,
    failed: bool,
}

#[cfg(feature = "editable")]
impl LimitedOutput {
    pub(super) fn new(max_bytes: usize, span: Span) -> Self {
        Self {
            text: String::new(),
            max_bytes,
            span,
            failed: false,
        }
    }

    fn remaining(&self) -> usize {
        self.max_bytes - self.text.len()
    }

    fn check_bytes(&mut self, bytes: usize) -> fmt::Result {
        self.failed |= bytes > self.remaining();
        if self.failed { Err(fmt::Error) } else { Ok(()) }
    }

    fn error(&self) -> Error {
        Error::at(INSERTED_BYTE_LIMIT, self.span)
    }
}

#[cfg(feature = "editable")]
impl AsRef<str> for LimitedOutput {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

#[cfg(feature = "editable")]
impl Output for LimitedOutput {
    fn finish(mut self) -> Result<String, Error> {
        self.check(0)?;
        Ok(self.text)
    }

    fn check(&mut self, bytes: usize) -> Result<(), Error> {
        let result = self.check_bytes(bytes);
        self.write_result(result)
    }

    fn reserve(&mut self, bytes: usize) {
        self.text.reserve(bytes.min(self.remaining()));
    }

    fn write_result(&self, result: fmt::Result) -> Result<(), Error> {
        result.map_err(|_| self.error())
    }

    fn quote(&mut self, value: &str) -> Result<(), Error> {
        self.check(value.len().saturating_add(2))?;
        let result = quoted::write_quoted_to(self, value);
        self.write_result(result)
    }

    fn collect(
        &mut self,
        value: &(impl fmt::Display + ?Sized),
        overhead: usize,
    ) -> Result<String, Error> {
        self.check(overhead)?;
        let mut formatted = Self::new(self.remaining() - overhead, self.span);
        let result = write!(formatted, "{value}");
        if formatted.failed {
            self.failed = true;
            return Err(self.error());
        }
        result.map_err(|_| Error::new("display formatting failed"))?;
        Ok(formatted.text)
    }
}

#[cfg(feature = "editable")]
impl Write for LimitedOutput {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.check_bytes(value.len())?;
        self.text.push_str(value);
        Ok(())
    }
}
