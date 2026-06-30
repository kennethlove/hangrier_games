//! Helper functions and types for the tributes module.
//!
//! Extracted from the monolithic `mod.rs` to keep the main module focused
//! on the `Tribute` struct and its core impl.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
use uuid::Uuid;

use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, AfflictionSource, BodyPart, Severity,
};

/// Serialize `Vec<Uuid>` as `Vec<String>` for SurrealDB compatibility.
/// The Surreal Rust SDK's bespoke serializer wires `uuid::Uuid` as raw bytes,
/// which Surreal then renders as base64 and rejects against `array<uuid>`
/// constraints. Storing as strings on the wire (and as `array<string>` in
/// the schema) follows the same convention as `message.event_id`.
pub fn serialize_uuids_as_strings<S>(uuids: &[Uuid], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(uuids.len()))?;
    for u in uuids {
        seq.serialize_element(&u.to_string())?;
    }
    seq.end()
}

/// Deserialize `Vec<Uuid>` from either a sequence of strings (the wire format
/// we write) or a sequence of native uuid values (test fixtures, JSON read
/// back through serde's standard Uuid impl).
pub fn deserialize_uuids_lenient<'de, D>(deserializer: D) -> Result<Vec<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUuid {
        S(String),
        U(Uuid),
    }

    let raw: Vec<StringOrUuid> = Vec::deserialize(deserializer)?;
    raw.into_iter()
        .map(|item| match item {
            StringOrUuid::S(s) => Uuid::parse_str(&s).map_err(serde::de::Error::custom),
            StringOrUuid::U(u) => Ok(u),
        })
        .collect()
}

/// Serialize a single `Uuid` as a string for the same reasons as
/// `serialize_uuids_as_strings`.
pub fn serialize_uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

/// Deserialize a single `Uuid` from either a string (our wire format) or the
/// SDK's native uuid bytes representation.
pub fn deserialize_uuid_lenient<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUuid {
        S(String),
        U(Uuid),
    }

    match StringOrUuid::deserialize(deserializer)? {
        StringOrUuid::S(s) => Uuid::parse_str(&s).map_err(serde::de::Error::custom),
        StringOrUuid::U(u) => Ok(u),
    }
}

/// Sanity level at which a tribute may attempt suicide.
pub(crate) const SANITY_BREAK_LEVEL: u32 = 9;

/// A draft affliction ready for acquisition resolution.
#[derive(Clone)]
pub struct AfflictionDraft {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub source: AfflictionSource,
}

/// Calculates the stamina cost for a tribute action based on:
/// - Base action cost
/// - Terrain movement multiplier
/// - Terrain affinity modifier (0.8 if tribute has affinity, 1.0 otherwise)
/// - Desperation multiplier based on health (1.0 + 0.5 * (1.0 - health%))
pub fn calculate_stamina_cost(
    action: &Action,
    terrain: &crate::terrain::TerrainType,
    tribute: &Tribute,
) -> u32 {
    // Base costs for each action type
    let base_cost: f32 = match action {
        Action::Move(_) => 20.0,
        Action::Hide => 15.0,
        Action::TakeItem => 10.0,
        Action::Attack => 25.0,
        Action::Rest | Action::None => 0.0,
        Action::UseItem(_) => 10.0,
        // Proposing an alliance is a low-cost social action.
        Action::ProposeAlliance => 5.0,
        // Survival actions: foraging/seeking shelter cost some stamina;
        // eating and drinking are essentially free overhead.
        Action::SeekShelter => 10.0,
        Action::Forage => 15.0,
        Action::DrinkFromTerrain => 5.0,
        Action::Eat(_) | Action::DrinkItem(_) => 0.0,
        // Sleep is free at the action layer; phase scheduler handles it.
        Action::Sleep { .. } => 0.0,
        Action::Rescue { .. } => 15.0,
        Action::SetTrap { .. } => 15.0,
        Action::Search => 10.0,
        Action::Frozen | Action::Flashback { .. } | Action::Avoidance | Action::SearchForSubstance { .. } => 0.0,
    };

    // If base cost is 0, no need to calculate multipliers
    if base_cost == 0.0 {
        return 0;
    }

    // Terrain multiplier from movement_cost
    let terrain_multiplier = terrain.base.movement_cost();

    // Affinity modifier: 0.8 if tribute has affinity for this terrain, else 1.0
    let affinity_modifier = if tribute.terrain_affinity.contains(&terrain.base) {
        0.8
    } else {
        1.0
    };

    // Desperation multiplier: 1.0 + (0.5 × (1.0 - health%))
    let health_percent = tribute.effective_health() as f32 / 100.0;
    let desperation_multiplier = 1.0 + (0.5 * (1.0 - health_percent));

    // Calculate final cost with all multipliers
    let final_cost = base_cost * terrain_multiplier * affinity_modifier * desperation_multiplier;

    // Round to nearest integer
    final_cost.round() as u32
}

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
