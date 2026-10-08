use astral_yaml::Span;
use std::fmt;

/// A YAML error, optionally associated with a source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub message: String,
    pub span: Option<Span>,
}

impl Error {
    pub(crate) fn located(mut self, span: Span) -> Self {
        self.span.get_or_insert(span);
        self
    }

    pub(crate) fn at(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }
}

impl From<astral_yaml::Error> for Error {
    fn from(error: astral_yaml::Error) -> Self {
        Self::at(error.message, error.span)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;
        if let Some(span) = self.span {
            write!(f, " at bytes {}..{}", span.start(), span.end())?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::new(message.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::new(message.to_string())
    }
}
