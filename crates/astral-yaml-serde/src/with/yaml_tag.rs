//! Serialize and deserialize enum variants using explicit local YAML tags.
//!
//! Apply this adapter to the field containing the enum:
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! enum Message {
//!     Text(String),
//!     #[serde(untagged)]
//!     Number(u32),
//! }
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "astral_yaml_serde::with::yaml_tag")]
//!     message: Message,
//! }
//!
//! let config: Config = astral_yaml_serde::from_str("message: !Text hello")?;
//! assert_eq!(config.message, Message::Text("hello".into()));
//! let yaml = astral_yaml_serde::to_string(&config)?;
//! assert_eq!(astral_yaml_serde::from_str::<Config>(&yaml)?, config);
//! # Ok::<(), astral_yaml_serde::Error>(())
//! ```
//!
//! Unit variants use an empty tagged node. Untagged variants retain their usual
//! representation. Deserialization also accepts ordinary enum names and
//! singleton mappings. Core tags such as `!!str` retain their usual meaning;
//! other global tags are rejected. Use [`crate::Tagged`] to capture tags as data.
//!
//! The adapter applies to this value through options and newtype wrappers, not
//! to collection elements or fields inside its payload. Mixed tagged/untagged
//! enums work by buffering local tags as singleton mappings. Other field
//! adapters that require direct access to this crate's deserializer, including
//! [`crate::Spanned`] and [`crate::Tagged`], cannot recover metadata from that
//! buffer. YAML tag handling requires this crate's serializer and deserializer.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use std::{fmt, marker::PhantomData};

pub(crate) const NAME: &str = "$astral_yaml::with::yaml_tag";

/// Serializes the annotated value with YAML tags for enum variants.
///
/// # Errors
/// Returns the serializer's error for invalid tags, payloads, or resource limits.
pub fn serialize<T: Serialize + ?Sized, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_newtype_struct(NAME, value)
}

/// Deserializes the annotated value, accepting local YAML tags as enum names.
///
/// # Errors
/// Returns the deserializer's error for invalid tags, payloads, or resource limits.
pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    deserializer.deserialize_newtype_struct(NAME, EnumVisitor(PhantomData))
}

struct EnumVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for EnumVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an enum with YAML tags enabled")
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
        T::deserialize(deserializer)
    }
}
