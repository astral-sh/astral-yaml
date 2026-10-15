use astral_yaml::Span;
use std::fmt;

/// A YAML error, optionally associated with a path and source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub message: String,
    /// The position being edited or deserialized, such as `$["ports"][1]`.
    /// A path through an alias describes its use, while the span can refer to its target.
    pub path: Option<String>,
    pub span: Option<Span>,
}

impl Error {
    pub(crate) fn located(mut self, span: impl Into<Option<Span>>) -> Self {
        self.span = self.span.or(span.into());
        self
    }

    pub(crate) fn at(message: impl Into<String>, span: impl Into<Option<Span>>) -> Self {
        Self {
            message: message.into(),
            path: None,
            span: span.into(),
        }
    }

    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            span: None,
        }
    }

    // Deserialization discovers the path from leaf to root as it returns the error.
    pub(crate) fn in_path(mut self, segment: fmt::Arguments<'_>) -> Self {
        let suffix = self
            .path
            .as_deref()
            .and_then(|path| path.strip_prefix('$'))
            .unwrap_or("");
        self.path = Some(format!("${segment}{suffix}"));
        self
    }
}

impl From<astral_yaml::Error> for Error {
    fn from(error: astral_yaml::Error) -> Self {
        Self::at(error.message, error.span)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{path}: ")?;
        }
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
