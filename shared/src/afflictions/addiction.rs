use super::kind::Substance;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Metadata attached to Addiction afflictions (spec §4).
///
/// Tracks use timing, High/Withdrawal mode, observer state, and the
/// use-count snapshot that enables relapse messaging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddictionMetadata {
    /// The specific substance (duplicates kind payload for ergonomic access).
    pub substance: Substance,
    /// Cycles since the tribute last used this substance. Reset on use,
    /// incremented otherwise. Drives decay and Withdrawal mode.
    pub cycles_since_last_use: u32,
    /// Cycles remaining in High mode. Decremented each cycle; 0 = Withdrawal.
    pub high_cycles_remaining: u32,
    /// Snapshot of `addiction_use_count[substance]` at acquisition moment.
    /// Used for relapse messaging ("back on it after N uses").
    pub use_count_at_acquisition: u32,
    /// Tributes who have observed a substance use or craving action.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub observed_by: BTreeSet<String>,
    /// Last cycle each observer witnessed a use or craving.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub observer_seen_cycle: BTreeMap<String, u32>,
}

/// Reasons an addiction acquisition attempt was resisted (spec §10).
///
/// Future variants: `TraitResistance`, `EquippedTalisman`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddictionResistReason {
    /// Tribute already has MAX_ACTIVE_ADDICTIONS (2) active addictions.
    AtCap,
}
