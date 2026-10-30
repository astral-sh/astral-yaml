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
//! Unit variants serialize as empty tagged nodes. Untagged variants keep their
//! usual representation. Deserialization also accepts ordinary enum names and
//! singleton mappings. Core tags such as `!!str` keep their usual meaning, and
//! other global tags are rejected. Use [`crate::Tagged`] to capture a tag as data.
//!
//! The adapter works through options and newtype wrappers. Nested fields and
//! collection elements need their own opt-in. Mixed tagged/untagged enums buffer
//! local tags as singleton mappings, which means [`crate::Spanned`] and
//! [`crate::Tagged`] cannot recover metadata inside that buffer.
//!
//! YAML tag handling requires this crate's serializer and deserializer.

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

pub(crate) const NAME: &str = "$astral_yaml::with::yaml_tag";

/// Serialize the annotated value, using YAML tags for enum variants.
///
/// # Errors
/// Propagates errors from the serializer.
pub fn serialize<T: Serialize + ?Sized, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_newtype_struct(NAME, value)
}

/// Deserialize the annotated value, accepting local YAML tags as enum names.
///
/// # Errors
/// Propagates errors from the deserializer.
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
