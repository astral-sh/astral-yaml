use astral_yaml::emit::FlowWrite;
use std::fmt;

mod keys;
mod serializer;
mod tag;

#[cfg(feature = "editable")]
pub(crate) use astral_yaml::emit::{non_printable, requires_double_quotes};
pub use serializer::to_string;
#[cfg(feature = "editable")]
pub(crate) use serializer::{quote_with_limit, to_string_with_limit};

#[inline]
pub(crate) fn write_quoted(output: &mut String, value: &str) {
    FlowWrite::<fmt::Error>::quote(output, value).expect("String write");
}

#[cfg(feature = "editable")]
pub(crate) const INSERTED_BYTE_LIMIT: &str = "inserted YAML exceeds input byte limit";
