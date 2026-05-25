use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phobia_metadata_default_is_innate() {
        let meta = PhobiaMetadata::default();
        assert!(matches!(meta.origin, PhobiaOrigin::Innate));
        assert!(meta.observed_by.is_empty());
        assert!(meta.observer_seen_cycle.is_empty());
        assert_eq!(meta.cycles_since_last_fire, 0);
    }
}
