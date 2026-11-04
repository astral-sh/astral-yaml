use super::quoted;
use crate::Error;
use std::fmt::{self, Write};

#[cfg(feature = "editable")]
use astral_yaml::Span;

/// The serializer writes directly to a String unless an edit supplies a byte limit.
pub(super) trait Output {
    fn as_str(&self) -> &str;
    fn finish(self) -> Result<String, Error>;
    fn check(&mut self, bytes: usize) -> Result<(), Error>;
    fn reserve(&mut self, bytes: usize);
    fn push(&mut self, ch: char) -> Result<(), Error>;
    fn push_str(&mut self, value: &str) -> Result<(), Error>;
    fn number(&mut self, value: impl fmt::Display) -> Result<(), Error>;
    fn quote(&mut self, value: &str) -> Result<(), Error>;
    fn collect(
        &mut self,
        value: &(impl fmt::Display + ?Sized),
        overhead: usize,
    ) -> Result<String, Error>;
}

impl Output for String {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }

    #[inline]
    fn finish(self) -> Result<String, Error> {
        Ok(self)
    }

    #[inline]
    fn check(&mut self, _: usize) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn reserve(&mut self, bytes: usize) {
        self.reserve(bytes);
    }

    #[inline]
    fn push(&mut self, ch: char) -> Result<(), Error> {
        self.push(ch);
        Ok(())
    }

    #[inline]
    fn push_str(&mut self, value: &str) -> Result<(), Error> {
        self.push_str(value);
        Ok(())
    }

    #[inline]
    fn number(&mut self, value: impl fmt::Display) -> Result<(), Error> {
        write!(self, "{value}").expect("String write");
        Ok(())
    }

    #[inline]
    fn quote(&mut self, value: &str) -> Result<(), Error> {
        quoted::write_quoted(self, value);
        Ok(())
    }

    #[inline]
    fn collect(&mut self, value: &(impl fmt::Display + ?Sized), _: usize) -> Result<String, Error> {
        Ok(value.to_string())
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

    fn fits(&mut self, bytes: usize) -> bool {
        self.failed |= bytes > self.remaining();
        !self.failed
    }

    fn error(&self) -> Error {
        Error::at(INSERTED_BYTE_LIMIT, self.span)
    }

    fn finish_collected(&mut self, formatted: Self, result: fmt::Result) -> Result<String, Error> {
        if formatted.failed {
            self.failed = true;
            return Err(self.error());
        }
        result.map_err(|_| Error::new("display formatting failed"))?;
        Ok(formatted.text)
    }
}

#[cfg(feature = "editable")]
impl Output for LimitedOutput {
    fn as_str(&self) -> &str {
        &self.text
    }

    fn finish(mut self) -> Result<String, Error> {
        self.check(0)?;
        Ok(self.text)
    }

    fn check(&mut self, bytes: usize) -> Result<(), Error> {
        if self.fits(bytes) {
            Ok(())
        } else {
            Err(self.error())
        }
    }

    fn reserve(&mut self, bytes: usize) {
        self.text.reserve(bytes.min(self.remaining()));
    }

    fn push(&mut self, ch: char) -> Result<(), Error> {
        self.push_str(ch.encode_utf8(&mut [0; 4]))
    }

    fn push_str(&mut self, value: &str) -> Result<(), Error> {
        self.write_str(value).map_err(|_| self.error())
    }

    fn number(&mut self, value: impl fmt::Display) -> Result<(), Error> {
        write!(self, "{value}").map_err(|_| self.error())
    }

    fn quote(&mut self, value: &str) -> Result<(), Error> {
        self.check(value.len().saturating_add(2))?;
        quoted::write_quoted_to(self, value).map_err(|_| self.error())
    }

    fn collect(
        &mut self,
        value: &(impl fmt::Display + ?Sized),
        overhead: usize,
    ) -> Result<String, Error> {
        self.check(overhead)?;
        let mut formatted = Self::new(self.remaining() - overhead, self.span);
        let result = write!(formatted, "{value}");
        self.finish_collected(formatted, result)
    }
}

#[cfg(feature = "editable")]
impl Write for LimitedOutput {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if !self.fits(value.len()) {
            return Err(fmt::Error);
        }

        self.text.push_str(value);
        Ok(())
    }
}
