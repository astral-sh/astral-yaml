mod deserializer;
mod input;

pub(crate) use deserializer::canonical_key;
#[cfg(feature = "editable")]
pub(crate) use deserializer::{check_document_keys, from_edit_value};
pub use deserializer::{from_document, from_str, from_stream};
