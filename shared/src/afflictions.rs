//! Wire-visible affliction types. Lives in `shared/` because `Tribute::afflictions`
//! is serialized to SurrealDB and broadcast over the WebSocket protocol.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md` §9.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Categories of afflictions a tribute can carry. Permanent kinds
/// (`MissingArm`, `MissingLeg`, `Blind`, `Deaf`) cannot be cured in v1;
/// reversible kinds progress / heal via the cascade and cure paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfflictionKind {
    Wounded,
    Infected,
    MissingArm,
    MissingLeg,
    Blind,
    Deaf,
    BrokenBone,
    Poisoned,
    Starving,
    Dehydrated,
    Frozen,
    Overheated,
    Burned,
    Sick,
    Electrocuted,
    Drowned,
    Buried,
    Trauma,
}

impl fmt::Display for AfflictionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AfflictionKind::Wounded => write!(f, "wounded"),
            AfflictionKind::Infected => write!(f, "infected"),
            AfflictionKind::MissingArm => write!(f, "missing_arm"),
            AfflictionKind::MissingLeg => write!(f, "missing_leg"),
            AfflictionKind::Blind => write!(f, "blind"),
            AfflictionKind::Deaf => write!(f, "deaf"),
            AfflictionKind::BrokenBone => write!(f, "broken_bone"),
            AfflictionKind::Poisoned => write!(f, "poisoned"),
            AfflictionKind::Starving => write!(f, "starving"),
            AfflictionKind::Dehydrated => write!(f, "dehydrated"),
            AfflictionKind::Frozen => write!(f, "frozen"),
            AfflictionKind::Overheated => write!(f, "overheated"),
            AfflictionKind::Burned => write!(f, "burned"),
            AfflictionKind::Sick => write!(f, "sick"),
            AfflictionKind::Electrocuted => write!(f, "electrocuted"),
            AfflictionKind::Drowned => write!(f, "drowned"),
            AfflictionKind::Buried => write!(f, "buried"),
            AfflictionKind::Trauma => write!(f, "trauma"),
        }
    }
}

impl FromStr for AfflictionKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wounded" => Ok(AfflictionKind::Wounded),
            "infected" => Ok(AfflictionKind::Infected),
            "missing_arm" => Ok(AfflictionKind::MissingArm),
            "missing_leg" => Ok(AfflictionKind::MissingLeg),
            "blind" => Ok(AfflictionKind::Blind),
            "deaf" => Ok(AfflictionKind::Deaf),
            "broken_bone" => Ok(AfflictionKind::BrokenBone),
            "poisoned" => Ok(AfflictionKind::Poisoned),
            "starving" => Ok(AfflictionKind::Starving),
            "dehydrated" => Ok(AfflictionKind::Dehydrated),
            "frozen" => Ok(AfflictionKind::Frozen),
            "overheated" => Ok(AfflictionKind::Overheated),
            "burned" => Ok(AfflictionKind::Burned),
            "sick" => Ok(AfflictionKind::Sick),
            "electrocuted" => Ok(AfflictionKind::Electrocuted),
            "drowned" => Ok(AfflictionKind::Drowned),
            "buried" => Ok(AfflictionKind::Buried),
            "trauma" => Ok(AfflictionKind::Trauma),
            other => Err(format!("unknown AfflictionKind: {other}")),
        }
    }
}

/// Anatomical attachment points for body-part-specific afflictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPart {
    Arm,
    Leg,
    Eye,
    Ear,
    Skull,
    Rib,
    Hand,
    Foot,
}

impl fmt::Display for BodyPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyPart::Arm => write!(f, "arm"),
            BodyPart::Leg => write!(f, "leg"),
            BodyPart::Eye => write!(f, "eye"),
            BodyPart::Ear => write!(f, "ear"),
            BodyPart::Skull => write!(f, "skull"),
            BodyPart::Rib => write!(f, "rib"),
            BodyPart::Hand => write!(f, "hand"),
            BodyPart::Foot => write!(f, "foot"),
        }
    }
}

impl FromStr for BodyPart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arm" => Ok(BodyPart::Arm),
            "leg" => Ok(BodyPart::Leg),
            "eye" => Ok(BodyPart::Eye),
            "ear" => Ok(BodyPart::Ear),
            "skull" => Ok(BodyPart::Skull),
            "rib" => Ok(BodyPart::Rib),
            "hand" => Ok(BodyPart::Hand),
            "foot" => Ok(BodyPart::Foot),
            other => Err(format!("unknown BodyPart: {other}")),
        }
    }
}

/// Severity tier for tier-scaled afflictions. Permanent kinds are always
/// `Severe` in practice; tier ordering is total (Mild < Moderate < Severe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
}

impl Severity {
    /// Returns the ordinal value for this severity: 0 for Mild, 1 for Moderate, 2 for Severe.
    pub fn ordinal(&self) -> u8 {
        match self {
            Severity::Mild => 0,
            Severity::Moderate => 1,
            Severity::Severe => 2,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Mild => write!(f, "mild"),
            Severity::Moderate => write!(f, "moderate"),
            Severity::Severe => write!(f, "severe"),
        }
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mild" => Ok(Severity::Mild),
            "moderate" => Ok(Severity::Moderate),
            "severe" => Ok(Severity::Severe),
            other => Err(format!("unknown Severity: {other}")),
        }
    }
}

/// Storage discriminator. Same kind on different parts is independent;
/// same kind on the same part collapses to one slot.
pub type AfflictionKey = (AfflictionKind, Option<BodyPart>);

/// Classification of trauma cause for mass casualty events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseClass {
    Combat,
    Environmental,
    Mixed,
}

/// Specific cause of death, used in trauma source metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    Tribute(String),
    Fire,
    Drowning,
    Starvation,
    Dehydration,
    Unknown,
}

/// Source of a trauma affliction, capturing the triggering event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// A single affliction slot on a tribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Affliction {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub source: AfflictionSource,
    /// Cycle number when this affliction was acquired.
    pub acquired_cycle: u32,
    /// Last cycle this affliction progressed (stepped up or spawned successor).
    pub last_progressed_cycle: u32,
    /// Optional trauma-specific metadata (source, reinforcement history).
    pub trauma_metadata: Option<TraumaSource>,
}

impl Affliction {
    /// Returns the storage key for this affliction.
    pub fn key(&self) -> AfflictionKey {
        (self.kind, self.body_part)
    }

    /// Returns true if this affliction kind is permanent and cannot be cured in v1.
    pub fn is_permanent(&self) -> bool {
        matches!(
            self.kind,
            AfflictionKind::MissingArm
                | AfflictionKind::MissingLeg
                | AfflictionKind::Blind
                | AfflictionKind::Deaf
        )
    }

    /// Returns true if this affliction can be reversed (cured).
    pub fn is_reversible(&self) -> bool {
        !self.is_permanent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affliction_key_returns_correct_tuple() {
        let a = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
        };
        assert_eq!(a.key(), (AfflictionKind::Wounded, Some(BodyPart::Arm)));
    }

    #[test]
    fn is_permanent_returns_true_for_missing_arm() {
        let a = Affliction {
            kind: AfflictionKind::MissingArm,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
        };
        assert!(a.is_permanent());
    }

    #[test]
    fn is_reversible_returns_true_for_wounded() {
        let a = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
        };
        assert!(a.is_reversible());
    }

    #[test]
    fn severity_ordering_is_correct() {
        assert_eq!(Severity::Mild.ordinal(), 0);
        assert_eq!(Severity::Moderate.ordinal(), 1);
        assert_eq!(Severity::Severe.ordinal(), 2);
        assert!(Severity::Mild < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Severe);
    }
}
