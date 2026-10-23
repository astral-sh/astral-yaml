use crate::Span;
use std::fmt;

/// An error and its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The range in the input to the failing operation. Decoding uses the original bytes.
    pub span: Span,
    /// The diagnostic text without its source range.
    pub message: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at bytes {}..{}",
            self.message,
            self.span.start(),
            self.span.end()
        )
    }
}

impl std::error::Error for Error {}
