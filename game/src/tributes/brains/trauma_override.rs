//! Trauma override layer for the brain pipeline.
//!
//! Runs after phobia_override, before affliction_override. Provides:
//! 1. Hard override — Severe trauma → `Action::Avoidance`
//!
//! Pipeline order: [..., survival, stamina, phobia, **trauma**, affliction, preferred, alliance, consumable]
//!
//! See spec §7 (trauma brain layer).

use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use shared::afflictions::{AfflictionKind, Severity};

/// Trauma override layer entry point for the pre-decision pipeline.
///
/// Returns `Some(Action::Avoidance)` to short-circuit the brain pipeline for
/// Severe trauma (hard veto), or `None` to fall through.
///
/// Gated on `config.trauma_enabled` — caller must check before invoking.
pub fn trauma_override(tribute: &Tribute) -> Option<Action> {
    // Early exit: no trauma, no override
    if !tribute
        .afflictions
        .values()
        .any(|a| matches!(a.kind, AfflictionKind::Trauma))
    {
        return None;
    }

    // Severe trauma → hard veto (avoidance)
    if tribute
        .afflictions
        .values()
        .any(|a| matches!(a.kind, AfflictionKind::Trauma) && a.severity >= Severity::Severe)
    {
        return Some(Action::Avoidance);
    }

    // Moderate/Mild → bias modifier at scoring level (no hard override)
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use shared::afflictions::{Affliction, AfflictionSource, TraumaMetadata, TraumaSource};
    use std::collections::{BTreeMap, BTreeSet};

    fn make_trauma(severity: Severity) -> Affliction {
        Affliction {
            kind: AfflictionKind::Trauma,
            body_part: None,
            severity,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: Some(TraumaMetadata {
                sources: BTreeSet::from([TraumaSource::NearDeath {
                    cause: shared::afflictions::DeathCause::Unknown,
                }]),
                cycles_since_last_event: 0,
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
            }),
            phobia_metadata: None,
            fixation_metadata: None,
        }
    }

    #[test]
    fn no_trauma_no_override() {
        let t = Tribute::new("Test".to_string(), None, None);
        assert!(trauma_override(&t).is_none());
    }

    #[test]
    fn mild_trauma_no_override() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.afflictions.insert(
            make_trauma(Severity::Mild).key(),
            make_trauma(Severity::Mild),
        );
        assert!(trauma_override(&t).is_none());
    }

    #[test]
    fn moderate_trauma_no_override() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.afflictions.insert(
            make_trauma(Severity::Moderate).key(),
            make_trauma(Severity::Moderate),
        );
        assert!(trauma_override(&t).is_none());
    }

    #[test]
    fn severe_trauma_returns_avoidance() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.afflictions.insert(
            make_trauma(Severity::Severe).key(),
            make_trauma(Severity::Severe),
        );
        assert_eq!(trauma_override(&t), Some(Action::Avoidance));
    }
}
