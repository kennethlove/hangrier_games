//! Wire-visible affliction types. Lives in `shared/` because `Tribute::afflictions`
//! is serialized to SurrealDB and broadcast over the WebSocket protocol.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md` §9.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

/// Categories of afflictions a tribute can carry. Permanent kinds
/// (`MissingArm`, `MissingLeg`, `Blind`, `Deaf`) cannot be cured in v1;
/// reversible kinds progress / heal via the cascade and cure paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhobiaTrigger {
    Fire,
    Water,
    Dark,
    Blood,
    Heights,
    Enclosed,
    Open,
    Animal,
    Tribute,
    TraitGroup,
}

impl fmt::Display for PhobiaTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhobiaTrigger::Fire => write!(f, "fire"),
            PhobiaTrigger::Water => write!(f, "water"),
            PhobiaTrigger::Dark => write!(f, "dark"),
            PhobiaTrigger::Blood => write!(f, "blood"),
            PhobiaTrigger::Heights => write!(f, "heights"),
            PhobiaTrigger::Enclosed => write!(f, "enclosed"),
            PhobiaTrigger::Open => write!(f, "open"),
            PhobiaTrigger::Animal => write!(f, "animal"),
            PhobiaTrigger::Tribute => write!(f, "tribute"),
            PhobiaTrigger::TraitGroup => write!(f, "trait_group"),
        }
    }
}

impl FromStr for PhobiaTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fire" => Ok(PhobiaTrigger::Fire),
            "water" => Ok(PhobiaTrigger::Water),
            "dark" => Ok(PhobiaTrigger::Dark),
            "blood" => Ok(PhobiaTrigger::Blood),
            "heights" => Ok(PhobiaTrigger::Heights),
            "enclosed" => Ok(PhobiaTrigger::Enclosed),
            "open" => Ok(PhobiaTrigger::Open),
            "animal" => Ok(PhobiaTrigger::Animal),
            "tribute" => Ok(PhobiaTrigger::Tribute),
            "trait_group" => Ok(PhobiaTrigger::TraitGroup),
            other => Err(format!("unknown PhobiaTrigger: {other}")),
        }
    }
}

/// Origin of a phobia affliction. Innate phobias are lifelong dispositions;
/// Traumatic phobias are learned through adverse events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhobiaOrigin {
    Innate,
    Traumatic { event_ref: String },
}

/// Metadata attached to Phobia afflictions. Tracks observer state,
/// reinforcement history, and origin. Only populated for Phobia kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhobiaMetadata {
    pub origin: PhobiaOrigin,
    /// Tributes who have observed this phobia firing.
    pub observed_by: BTreeSet<String>,
    /// Last cycle each observer saw this phobia fire.
    pub observer_seen_cycle: BTreeMap<String, u32>,
    /// Cycles since this phobia last fired (for decay tracking).
    pub cycles_since_last_fire: u32,
}

impl Default for PhobiaMetadata {
    fn default() -> Self {
        Self {
            origin: PhobiaOrigin::Innate,
            observed_by: BTreeSet::new(),
            observer_seen_cycle: BTreeMap::new(),
            cycles_since_last_fire: 0,
        }
    }
}

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
    Phobia(PhobiaTrigger),
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
            AfflictionKind::Phobia(trigger) => write!(f, "phobia:{trigger}"),
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
            rest if rest.starts_with("phobia:") => {
                let trigger_str = rest.strip_prefix("phobia:").unwrap();
                let trigger = PhobiaTrigger::from_str(trigger_str)?;
                Ok(AfflictionKind::Phobia(trigger))
            }
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

    /// Steps up one severity tier. `Severe` caps at `Severe`.
    pub fn next_tier(&self) -> Self {
        match self {
            Severity::Mild => Severity::Moderate,
            Severity::Moderate => Severity::Severe,
            Severity::Severe => Severity::Severe,
        }
    }

    /// Steps down one severity tier. `Mild` returns `None` (cured).
    pub fn prev_tier(&self) -> Option<Self> {
        match self {
            Severity::Severe => Some(Severity::Moderate),
            Severity::Moderate => Some(Severity::Mild),
            Severity::Mild => None,
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

/// Outcome of a traumatic reinforcement roll.
/// `escalated` is true only when severity actually increased.
pub struct ReinforcementOutcome {
    pub escalated: bool,
    pub new_severity: Severity,
}

/// Outcome of a decay tick.
/// `new_severity` is `None` when the affliction is cured (dropped off the bottom).
pub struct DecayOutcome {
    pub decayed: bool,
    pub new_severity: Option<Severity>,
}

/// Apply traumatic reinforcement to an affliction severity.
///
/// Rolls against `escalation_chance` (0.0–1.0). On success the severity
/// steps up one tier; `Severe` is the cap and never rolls.
///
/// # Arguments
/// * `current_severity` — the affliction's current severity tier.
/// * `escalation_chance` — probability of stepping up (e.g. 0.12 for 12%).
/// * `rng` — any type implementing `rand::Rng`.
pub fn apply_traumatic_reinforcement(
    current_severity: Severity,
    escalation_chance: f64,
    rng: &mut impl rand::Rng,
) -> ReinforcementOutcome {
    if current_severity == Severity::Severe {
        return ReinforcementOutcome {
            escalated: false,
            new_severity: Severity::Severe,
        };
    }
    if rng.random_bool(escalation_chance) {
        ReinforcementOutcome {
            escalated: true,
            new_severity: current_severity.next_tier(),
        }
    } else {
        ReinforcementOutcome {
            escalated: false,
            new_severity: current_severity,
        }
    }
}

/// Tick decay for a tier-scaled affliction.
///
/// If `cycles_since_last` has not reached `decay_threshold` the affliction
/// holds. Once the threshold is met the severity steps down one tier;
/// `Mild` decays to `None` (cured).
///
/// # Arguments
/// * `current_severity` — the affliction's current severity tier.
/// * `cycles_since_last` — cycles elapsed since the affliction last fired.
/// * `decay_threshold` — cycles required before decay triggers (5 for
///   phobia/fixation, 10 for trauma).
pub fn tick_decay(
    current_severity: Severity,
    cycles_since_last: u32,
    decay_threshold: u32,
) -> DecayOutcome {
    if cycles_since_last < decay_threshold {
        return DecayOutcome {
            decayed: false,
            new_severity: Some(current_severity),
        };
    }
    DecayOutcome {
        decayed: true,
        new_severity: current_severity.prev_tier(),
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
    /// Optional phobia-specific metadata (origin, observer state).
    /// Only `Some` for `AfflictionKind::Phobia` variants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phobia_metadata: Option<PhobiaMetadata>,
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
            phobia_metadata: None,
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

    #[test]
    fn phobia_trigger_display_roundtrip() {
        for trigger in [
            PhobiaTrigger::Fire,
            PhobiaTrigger::Water,
            PhobiaTrigger::Dark,
            PhobiaTrigger::Blood,
            PhobiaTrigger::Heights,
            PhobiaTrigger::Enclosed,
            PhobiaTrigger::Open,
            PhobiaTrigger::Animal,
            PhobiaTrigger::Tribute,
            PhobiaTrigger::TraitGroup,
        ] {
            let s = trigger.to_string();
            let parsed: PhobiaTrigger = s.parse().unwrap();
            assert_eq!(trigger, parsed);
        }
    }

    #[test]
    fn phobia_trigger_from_str_invalid() {
        assert!(PhobiaTrigger::from_str("spiders").is_err());
    }

    #[test]
    fn affliction_kind_phobia_display() {
        let kind = AfflictionKind::Phobia(PhobiaTrigger::Fire);
        assert_eq!(kind.to_string(), "phobia:fire");

        let kind = AfflictionKind::Phobia(PhobiaTrigger::Heights);
        assert_eq!(kind.to_string(), "phobia:heights");
    }

    #[test]
    fn affliction_kind_phobia_from_str() {
        let kind: AfflictionKind = "phobia:fire".parse().unwrap();
        assert_eq!(kind, AfflictionKind::Phobia(PhobiaTrigger::Fire));

        let kind: AfflictionKind = "phobia:dark".parse().unwrap();
        assert_eq!(kind, AfflictionKind::Phobia(PhobiaTrigger::Dark));
    }

    #[test]
    fn affliction_kind_phobia_from_str_invalid() {
        assert!(AfflictionKind::from_str("phobia:unknown").is_err());
    }

    #[test]
    fn phobia_metadata_default_is_innate() {
        let meta = PhobiaMetadata::default();
        assert!(matches!(meta.origin, PhobiaOrigin::Innate));
        assert!(meta.observed_by.is_empty());
        assert!(meta.observer_seen_cycle.is_empty());
        assert_eq!(meta.cycles_since_last_fire, 0);
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
        };
        assert!(aff.phobia_metadata.is_none());
    }

    // ── Severity tier helpers ──────────────────────────────────────────

    #[test]
    fn severity_next_tier_steps_up_correctly() {
        assert_eq!(Severity::Mild.next_tier(), Severity::Moderate);
        assert_eq!(Severity::Moderate.next_tier(), Severity::Severe);
        assert_eq!(Severity::Severe.next_tier(), Severity::Severe);
    }

    #[test]
    fn severity_prev_tier_steps_down_correctly() {
        assert_eq!(Severity::Severe.prev_tier(), Some(Severity::Moderate));
        assert_eq!(Severity::Moderate.prev_tier(), Some(Severity::Mild));
        assert_eq!(Severity::Mild.prev_tier(), None);
    }

    // ── Traumatic reinforcement ────────────────────────────────────────

    /// Deterministic RNG that always yields the same `u64`.
    struct FixedRng(u64);
    impl rand::RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            self.next_u64() as u32
        }
        fn next_u64(&mut self) -> u64 {
            self.0
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            for chunk in dest.chunks_mut(8) {
                let bytes = self.0.to_le_bytes();
                chunk.copy_from_slice(&bytes[..chunk.len()]);
            }
        }
    }

    #[test]
    fn reinforcement_mild_to_moderate_on_success() {
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Moderate);
    }

    #[test]
    fn reinforcement_mild_stays_mild_on_failure() {
        // Use 50% chance; FixedRng(0) produces f64 ≈ 0.0 which is < 0.5 → true.
        // To force false, use a chance the fixed value beats.
        // FixedRng(u64::MAX) → f64 ≈ 1.0, so random_bool(0.5) → 1.0 < 0.5 = false.
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 0.5, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Mild);
    }

    #[test]
    fn reinforcement_moderate_to_severe_on_success() {
        let mut rng = FixedRng(0); // f64 ≈ 0.0, < 1.0 → true
        let outcome = apply_traumatic_reinforcement(Severity::Moderate, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    #[test]
    fn reinforcement_severe_stays_severe_capped() {
        let mut rng = FixedRng(0);
        let outcome = apply_traumatic_reinforcement(Severity::Severe, 1.0, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    #[test]
    fn reinforcement_chance_zero_never_esculates() {
        let mut rng = FixedRng(0);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 0.0, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Mild);
    }

    #[test]
    fn reinforcement_chance_one_always_esculates() {
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Moderate, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    // ── Decay ──────────────────────────────────────────────────────────

    #[test]
    fn decay_below_threshold_no_decay() {
        let outcome = tick_decay(Severity::Severe, 3, 5);
        assert!(!outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Severe));
    }

    #[test]
    fn decay_at_threshold_severe_to_moderate() {
        let outcome = tick_decay(Severity::Severe, 5, 5);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Moderate));
    }

    #[test]
    fn decay_at_threshold_moderate_to_mild() {
        let outcome = tick_decay(Severity::Moderate, 5, 5);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Mild));
    }

    #[test]
    fn decay_at_threshold_mild_to_cured() {
        let outcome = tick_decay(Severity::Mild, 5, 5);
        assert!(outcome.decayed);
        assert!(outcome.new_severity.is_none());
    }

    #[test]
    fn decay_above_threshold_same_as_at_threshold() {
        let outcome_severe = tick_decay(Severity::Severe, 100, 5);
        assert!(outcome_severe.decayed);
        assert_eq!(outcome_severe.new_severity, Some(Severity::Moderate));

        let outcome_mild = tick_decay(Severity::Mild, 100, 5);
        assert!(outcome_mild.decayed);
        assert!(outcome_mild.new_severity.is_none());
    }

    #[test]
    fn decay_trauma_threshold_10() {
        let outcome = tick_decay(Severity::Severe, 9, 10);
        assert!(!outcome.decayed);

        let outcome = tick_decay(Severity::Severe, 10, 10);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Moderate));
    }
}
