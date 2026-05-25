use super::source::TraumaSource;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Metadata attached to Trauma afflictions. Tracks observer state,
/// reinforcement history, and source. Only populated for Trauma kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraumaMetadata {
    /// The source event that caused this trauma.
    pub source: TraumaSource,
    /// Cycles since this trauma last fired/reinforced (for decay tracking).
    pub cycles_since_last_fire: u32,
    /// Tributes who have observed a flashback associated with this trauma.
    pub observed_by: BTreeSet<String>,
    /// Last cycle each observer witnessed a flashback.
    pub observer_seen_cycle: BTreeMap<String, u32>,
}
