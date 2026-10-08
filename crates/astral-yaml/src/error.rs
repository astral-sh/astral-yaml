use crate::Span;
use std::fmt;

/// An error and its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub span: Span,
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
