use astral_yaml_serde::{Error, from_str};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>", serialize = "T: Serialize"))]
pub(crate) struct YamlTag<T>(#[serde(with = "astral_yaml_serde::with::yaml_tag")] pub(crate) T);

pub(crate) fn from_tag<'de, T: Deserialize<'de>>(source: &'de str) -> Result<T, Error> {
    from_str::<YamlTag<T>>(source).map(|value| value.0)
}
