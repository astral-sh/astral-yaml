mod keys;
mod output;
mod serializer;
mod tag;

#[cfg(feature = "editable")]
pub(crate) use astral_yaml::emit::{non_printable, requires_double_quotes};
#[cfg(feature = "editable")]
pub(crate) use output::INSERTED_BYTE_LIMIT;
pub use serializer::to_string;
#[cfg(feature = "editable")]
pub(crate) use serializer::{quote_with_limit, to_string_with_limit};

#[inline]
pub(crate) fn write_quoted(output: &mut String, value: &str) {
    if value.len() >= 64 || output.is_empty() {
        output.reserve(value.len() + 3);
    }
    astral_yaml::emit::write_quoted(output, value).expect("String write");
}
