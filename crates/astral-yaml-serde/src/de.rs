mod deserializer;
mod input;

pub(crate) use deserializer::canonical_key;
#[cfg(feature = "editable")]
pub(crate) use deserializer::check_document_keys;
pub use deserializer::{
    DeserializationPolicy, from_document, from_document_with_policy, from_str,
    from_str_with_policy, from_stream,
};

pub(crate) const DEFAULT_WORK_LIMIT: usize = 1_000_000;

#[inline]
pub(crate) fn charge_visit(depth: usize, budget: &astral_yaml::Budget) -> Result<(), crate::Error> {
    if depth >= 64 || budget.try_charge(1).is_none() {
        return Err(crate::Error::new("deserialization resource limit exceeded"));
    }
    Ok(())
}
