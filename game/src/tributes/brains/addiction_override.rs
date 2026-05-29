//! Addiction override layer for the brain pipeline.
//!
//! Pipeline order: [..., survival, stamina, fixation, phobia, trauma, **addiction**, affliction, ...]
//!
//! See spec §7-8.

use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use shared::afflictions::{AfflictionKind, Severity};

/// Addiction override layer entry point.
///
/// Checks all addictions on the tribute. For each addiction:
/// - High mode: no action override (stat effects only)
/// - Withdrawal mode: applies severity-tiered craving/compulsion
///
/// Returns `Some(Action::SearchForSubstance)` to override the pipeline,
/// or `None` to fall through.
///
/// Gated on `config.addiction_enabled` — caller must check before invoking.
pub fn addiction_override(tribute: &Tribute) -> Option<Action> {
    for aff in tribute.afflictions.values() {
        let kind = match &aff.kind {
            AfflictionKind::Addiction(sub) => sub,
            _ => continue,
        };
        let Some(meta) = &aff.addiction_metadata else {
            continue;
        };

        // High mode: no override — stat effects applied via visible_modifiers
        if meta.high_cycles_remaining > 0 {
            continue;
        }

        // Withdrawal mode: severity-gated compulsion
        match aff.severity {
            Severity::Severe => {
                // Hard override — seek the substance
                return Some(Action::SearchForSubstance { substance: *kind });
            }
            Severity::Moderate | Severity::Mild => {
                // Bias only — scoring weight modifier, no hard override here
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use shared::afflictions::{
        AddictionMetadata, Affliction, AfflictionSource, Severity, Substance,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn make_addiction(substance: Substance, severity: Severity, high_remaining: u32) -> Affliction {
        Affliction {
            kind: AfflictionKind::Addiction(substance),
            body_part: None,
            severity,
            source: AfflictionSource::Environmental,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: Some(AddictionMetadata {
                substance,
                cycles_since_last_use: 10,
                high_cycles_remaining: high_remaining,
                use_count_at_acquisition: 3,
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
            }),
            trapped_metadata: None,
        }
    }

    #[test]
    fn no_addiction_no_override() {
        let t = Tribute::new("Test".to_string(), None, None);
        assert!(addiction_override(&t).is_none());
    }

    #[test]
    fn high_mode_no_override() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        let aff = make_addiction(Substance::Stimulant, Severity::Severe, 2);
        t.afflictions.insert(aff.key(), aff);
        assert!(addiction_override(&t).is_none());
    }

    #[test]
    fn severe_withdrawal_returns_search() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        let aff = make_addiction(Substance::Stimulant, Severity::Severe, 0);
        t.afflictions.insert(aff.key(), aff);
        assert_eq!(
            addiction_override(&t),
            Some(Action::SearchForSubstance {
                substance: Substance::Stimulant
            })
        );
    }

    #[test]
    fn moderate_withdrawal_no_override() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        let aff = make_addiction(Substance::Morphling, Severity::Moderate, 0);
        t.afflictions.insert(aff.key(), aff);
        assert!(addiction_override(&t).is_none());
    }

    #[test]
    fn mild_withdrawal_no_override() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        let aff = make_addiction(Substance::Alcohol, Severity::Mild, 0);
        t.afflictions.insert(aff.key(), aff);
        assert!(addiction_override(&t).is_none());
    }

    #[test]
    fn multiple_addictions_severe_wins() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.afflictions.insert(
            make_addiction(Substance::Alcohol, Severity::Mild, 0).key(),
            make_addiction(Substance::Alcohol, Severity::Mild, 0),
        );
        t.afflictions.insert(
            make_addiction(Substance::Stimulant, Severity::Severe, 0).key(),
            make_addiction(Substance::Stimulant, Severity::Severe, 0),
        );
        assert_eq!(
            addiction_override(&t),
            Some(Action::SearchForSubstance {
                substance: Substance::Stimulant
            })
        );
    }
}
