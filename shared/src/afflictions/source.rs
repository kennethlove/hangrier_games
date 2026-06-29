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

impl std::fmt::Display for BeastKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeastKind::Tracker => write!(f, "tracker"),
            BeastKind::Mutt => write!(f, "mutt"),
            BeastKind::Wolf => write!(f, "wolf"),
            BeastKind::Bear => write!(f, "bear"),
            BeastKind::Snake => write!(f, "snake"),
            BeastKind::Bird => write!(f, "bird"),
            BeastKind::Other => write!(f, "beast"),
        }
    }
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

impl std::fmt::Display for HazardKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HazardKind::Trap => write!(f, "trap"),
            HazardKind::Pit => write!(f, "pit"),
            HazardKind::FallingDebris => write!(f, "falling debris"),
            HazardKind::ToxicGas => write!(f, "toxic gas"),
            HazardKind::Quicksand => write!(f, "quicksand"),
            HazardKind::Other => write!(f, "hazard"),
        }
    }
}

/// Coarse classification of how a tribute died, used as the keying source for
/// trauma acquisition (spec §4). Stored on `TraumaSource` variants and as
/// the `cause` field of `MessagePayload::TributeKilled`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DeathCause {
    Tribute(String),
    Combat,
    Suicide,
    CriticalFumble,
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

impl std::fmt::Display for DeathCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeathCause::Tribute(name) => write!(f, "{name}"),
            DeathCause::Combat => write!(f, "combat"),
            DeathCause::Suicide => write!(f, "suicide"),
            DeathCause::CriticalFumble => write!(f, "critical_fumble"),
            DeathCause::Fire => write!(f, "fire"),
            DeathCause::Drowning => write!(f, "drowning"),
            DeathCause::Beast(kind) => write!(f, "{kind}"),
            DeathCause::Hazard(kind) => write!(f, "{kind}"),
            DeathCause::Starvation => write!(f, "starvation"),
            DeathCause::Dehydration => write!(f, "dehydration"),
            DeathCause::Affliction(kind) => write!(f, "{kind}"),
            DeathCause::Gamemaker => write!(f, "gamemaker"),
            DeathCause::Unknown => write!(f, "unknown"),
        }
    }
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
