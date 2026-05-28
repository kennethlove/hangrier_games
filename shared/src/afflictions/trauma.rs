use super::source::TraumaSource;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Runtime state for a Trauma affliction (spec §4). Stored as
/// `Affliction.trauma_metadata = Some(...)` on Trauma kinds; `None` for all
/// other affliction kinds. Mutable (counter ticks, observer set grows/decays).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraumaMetadata {
    /// All producer events that have contributed to this trauma. Grows over
    /// the trauma's lifetime; never shrinks. Used by §7.1 source matching and
    /// §12 UI to describe the carried sources.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub sources: BTreeSet<TraumaSource>,

    /// Cycles since any producer fired for this tribute. Reset to 0 on
    /// reinforcement (§6.1); incremented otherwise (§6.2). Decay tier-step at
    /// `tuning.decay_threshold_cycles` (default 10).
    #[serde(default)]
    pub cycles_since_last_event: u32,

    /// Tributes who have personally witnessed a flashback or Severe avoidance
    /// (§9 visibility moments). Pruned by observer-decay tick.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub observed_by: BTreeSet<String>,

    /// Last cycle each observer saw this trauma fire. Used to prune
    /// `observed_by` at the observer-decay threshold (default 10 cycles).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub observer_seen_cycle: BTreeMap<String, u32>,
}
