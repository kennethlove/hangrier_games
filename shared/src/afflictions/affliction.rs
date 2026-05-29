use super::addiction::AddictionMetadata;
use super::fixation::FixationMetadata;
use super::kind::{AfflictionKind, BodyPart};
use super::phobia::PhobiaMetadata;
use super::severity::Severity;
use super::source::{AfflictionKey, AfflictionSource};
use super::trapped::TrappedMetadata;
use super::trauma::TraumaMetadata;
use serde::{Deserialize, Serialize};

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
    /// Optional trauma-specific metadata (source, observer state, reinforcement history).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trauma_metadata: Option<TraumaMetadata>,
    /// Optional phobia-specific metadata (origin, observer state).
    /// Only `Some` for `AfflictionKind::Phobia` variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phobia_metadata: Option<PhobiaMetadata>,
    /// Optional fixation-specific metadata (origin).
    /// Only `Some` for `AfflictionKind::Fixation` variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixation_metadata: Option<FixationMetadata>,
    /// Optional addiction-specific metadata (substance, use counters, observer state).
    /// Only `Some` for `AfflictionKind::Addiction` variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addiction_metadata: Option<AddictionMetadata>,
    /// Optional trapped-specific metadata (trap kind, cycles trapped, escape progress).
    /// Only `Some` for `AfflictionKind::Trapped` variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trapped_metadata: Option<TrappedMetadata>,
}

impl Affliction {
    /// Returns the storage key for this affliction.
    pub fn key(&self) -> AfflictionKey {
        (self.kind.clone(), self.body_part)
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
    use crate::afflictions::PhobiaTrigger;

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
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
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
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        };
        assert!(a.is_permanent());
    }

    #[test]
    fn is_permanent_returns_true_for_missing_leg() {
        let a = Affliction {
            kind: AfflictionKind::MissingLeg,
            body_part: Some(BodyPart::Leg),
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
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
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        };
        assert!(a.is_reversible());
    }

    #[test]
    fn phobia_affliction_serialization_roundtrip() {
        let aff = Affliction {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: Some(PhobiaMetadata::default()),
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        };
        let json = serde_json::to_string(&aff).unwrap();
        let restored: Affliction = serde_json::from_str(&json).unwrap();
        assert_eq!(aff, restored);
    }

    #[test]
    fn phobia_metadata_none_for_non_phobia() {
        let aff = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        };
        assert!(aff.phobia_metadata.is_none());
    }

    #[test]
    fn affliction_with_trapped_metadata_serializes() {
        use crate::afflictions::trapped::{TrapKind, TrappedMetadata};

        let a = Affliction {
            kind: AfflictionKind::Trapped(TrapKind::Drowning),
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Environmental,
            acquired_cycle: 5,
            last_progressed_cycle: 5,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: Some(TrappedMetadata::fresh_for(TrapKind::Drowning, Some(0.30))),
        };

        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("trapped_metadata"));
        assert!(json.contains("drowning"));
        let restored: Affliction = serde_json::from_str(&json).unwrap();
        assert_eq!(a, restored);
    }
}
