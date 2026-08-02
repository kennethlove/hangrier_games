//! Helper functions and types for the tributes module.
//!
//! Extracted from the monolithic `mod.rs` to keep the main module focused
//! on the `Tribute` struct and its core impl.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shared::afflictions::{Affliction, AfflictionKey};

/// Serialize `BTreeMap<AfflictionKey, Affliction>` as a `Vec<Affliction>`.
///
/// `BTreeMap` with tuple keys (`(AfflictionKind, Option<BodyPart>)`) cannot be
/// serialized as a JSON object because serde_json requires string map keys.
/// Instead, we serialize only the values (each `Affliction` already carries
/// `kind` and `body_part`), which also keeps the wire format more readable.
/// The map is reconstructed from values on deserialization via `key()`.
pub fn serialize_affliction_map<S>(
    map: &BTreeMap<AfflictionKey, Affliction>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec: Vec<&Affliction> = map.values().collect();
    vec.serialize(serializer)
}

/// Deserialize a `Vec<Affliction>` back into a `BTreeMap<AfflictionKey, Affliction>`.
pub fn deserialize_affliction_map<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<AfflictionKey, Affliction>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<Affliction> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().map(|a| (a.key(), a)).collect())
}
