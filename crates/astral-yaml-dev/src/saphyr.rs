//! Settings and workloads shared by the serde-saphyr comparison.

use serde::Deserialize;

/// Uses core boolean spellings, literal merge keys, and strict tag validation.
///
/// Other serde-saphyr defaults, including comment capture and resource limits,
/// remain enabled. These settings do not remove all scalar-resolution differences
/// and also reject local enum tags, so typed enum tests use the default options.
#[must_use]
pub fn options() -> serde_saphyr::Options {
    serde_saphyr::options! {
        strict_booleans: true,
        merge_keys: serde_saphyr::MergeKeyPolicy::AsOrdinary,
        reject_unsupported_tags: true,
    }
}

/// Parses and deserializes every document, including null documents.
///
/// # Errors
/// Returns a parsing or deserialization error from astral-yaml.
pub fn astral_documents<'a, T: Deserialize<'a>>(
    source: &'a str,
) -> Result<Vec<T>, astral_yaml_serde::Error> {
    let stream = astral_yaml::parse(source)?;

    (0..stream.documents().len())
        .map(|index| astral_yaml_serde::from_document(&stream, index))
        .collect()
}
