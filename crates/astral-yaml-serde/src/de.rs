mod deserializer;
mod input;

use crate::Error;
use astral_yaml::Budget;

pub(crate) use deserializer::canonical_key;
#[cfg(feature = "editable")]
pub(crate) use deserializer::check_document_keys;
pub use deserializer::{
    DeserializationPolicy, from_document, from_document_with_policy, from_str,
    from_str_with_policy, from_stream,
};

#[inline]
pub(crate) fn charge_visit(depth: usize, budget: Option<&Budget>) -> Result<(), Error> {
    if depth >= 64 || budget.is_some_and(|budget| budget.try_charge(1).is_none()) {
        return Err(Error::new("deserialization resource limit exceeded"));
    }
    Ok(())
}
