mod deserializer;
mod input;

pub(crate) use deserializer::canonical_key;
#[cfg(feature = "editable")]
pub(crate) use deserializer::check_document_keys;
pub use deserializer::{
    DeserializationPolicy, from_document, from_document_with_policy, from_str,
    from_str_with_policy, from_stream,
};
