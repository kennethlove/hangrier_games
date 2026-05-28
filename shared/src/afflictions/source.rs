use super::kind::{AfflictionKind, BodyPart};
use serde::{Deserialize, Serialize};

/// Storage discriminator. Same kind on different parts is independent;
/// same kind on the same part collapses to one slot.
pub type AfflictionKey = (AfflictionKind, Option<BodyPart>);

/// Coarse beast taxonomy used by `DeathCause::Beast`. Trauma source matching
/// is at this granularity (no per-individual beast keying in v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeastKind {
    Tracker,
    Mutt,
    Wolf,
    Bear,
    Snake,
    Bird,
    Other,
}

/// Coarse hazard taxonomy used by `DeathCause::Hazard`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardKind {
    Trap,
    Pit,
    FallingDebris,
    ToxicGas,
    Quicksand,
    Other,
}

/// Coarse classification of how a tribute died, used as the keying source for
/// trauma acquisition (spec §4). Stored on `TraumaSource` variants. Resolution
/// finer than this lives in messages, not in trauma state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DeathCause {
    Tribute(String),
    Fire,
    Drowning,
    Beast(BeastKind),
    Hazard(HazardKind),
    Starvation,
    Dehydration,
    Affliction(AfflictionKind),
    Gamemaker,
    Unknown,
}

/// Coarse class of mass-casualty event source (spec §4, §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseClass {
    Combat,
    Environmental,
    Gamemaker,
    Mixed,
}

/// Source of a trauma affliction, capturing the triggering event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraumaSource {
    WitnessedAllyDeath {
        ally: String,
        cause: Option<DeathCause>,
    },
    NearDeath {
        cause: DeathCause,
    },
    Betrayal {
        by: String,
    },
    MassCasualty {
        cause_class: CauseClass,
        deaths_this_cycle: u32,
    },
}

/// Origin of an affliction. `Sponsor` and `Gamemaker` variants are reserved
/// for future systems but ship in v1 to avoid enum churn (per spec §3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfflictionSource {
    Spawn,
    Combat { attacker_id: String },
    Environmental,
    Cascade { from: AfflictionKey },
    Sponsor,
    Gamemaker,
}
