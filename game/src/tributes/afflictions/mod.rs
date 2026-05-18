//! Game-layer affliction logic: anatomy resolution, acquisition API,
//! tuning, visibility. Storage and wire types live in `shared::afflictions`.
//!
//! PR1 ships only the foundation. Cure / cascade / brain-pipeline
//! integration arrive in PR2 and PR3.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md`.

pub mod anatomy;
pub mod cascade;
pub mod cure;
pub mod effects;
pub mod trauma;
pub mod trauma_producers;
pub mod tuning;

pub use anatomy::{AcquireResolution, RejectReason, can_acquire};
pub use cascade::{CascadeOutcome, CascadeResult, apply_cascade, tick_cascade};
pub use cure::{CureOutcome, apply_cure, cure_item_to_affliction, recovery_cycles};
pub use effects::{BrainBias, StatModifiers, compute_brain_bias, compute_stat_modifiers};
pub use trauma::TraumaAcquisition;
pub use tuning::AfflictionTuning;

use crate::tributes::Tribute;
use shared::afflictions::{Affliction, Severity};

/// Returns afflictions on `target` that are visible to `observer`.
///
/// Visibility rules (spec §11):
/// - Mild = hidden (not visible to anyone)
/// - Moderate = visible only to tributes in the same area
/// - Severe = public (visible to all)
pub fn visible_afflictions_to<'a>(observer: &Tribute, target: &'a Tribute) -> Vec<&'a Affliction> {
    target
        .afflictions
        .values()
        .filter(|aff| is_visible_to(observer, target, aff))
        .collect()
}

fn is_visible_to(observer: &Tribute, target: &Tribute, aff: &Affliction) -> bool {
    match aff.severity {
        Severity::Mild => false,
        Severity::Moderate => observer.area == target.area,
        Severity::Severe => true,
    }
}

/// Returns true if `target` has a specific affliction visible to `observer`.
pub fn target_has_visible_affliction(
    observer: &Tribute,
    target: &Tribute,
    kind: shared::afflictions::AfflictionKind,
    min_severity: Severity,
) -> bool {
    visible_afflictions_to(observer, target)
        .iter()
        .any(|a| a.kind == kind && a.severity >= min_severity)
}

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod snapshot_tests;

#[cfg(test)]
mod visibility_tests {
    use super::*;
    use crate::areas::Area;
    use crate::tributes::Tribute;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, BodyPart};

    fn make_affliction(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: Some(match kind {
                AfflictionKind::MissingArm => BodyPart::Arm,
                AfflictionKind::MissingLeg => BodyPart::Leg,
                AfflictionKind::Blind => BodyPart::Eye,
                AfflictionKind::Deaf => BodyPart::Ear,
                _ => BodyPart::Rib,
            }),
            severity,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
        }
    }

    #[test]
    fn mild_affliction_hidden_from_everyone() {
        let observer = Tribute::new("Observer".to_string(), None, None);
        let mut target = Tribute::new("Target".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Mild);
        target.afflictions.insert(aff.key(), aff);

        let visible = visible_afflictions_to(&observer, &target);
        assert!(visible.is_empty());
    }

    #[test]
    fn moderate_affliction_visible_only_in_same_area() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target = Tribute::new("Target".to_string(), None, None);
        target.area = Area::Sector1;
        let aff = make_affliction(AfflictionKind::BrokenBone, Severity::Moderate);
        target.afflictions.insert(aff.key(), aff);

        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 1);

        observer.area = Area::Sector2;
        let visible = visible_afflictions_to(&observer, &target);
        assert!(visible.is_empty());
    }

    #[test]
    fn severe_affliction_visible_to_all() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target = Tribute::new("Target".to_string(), None, None);
        target.area = Area::Sector6;
        let aff = make_affliction(AfflictionKind::Blind, Severity::Severe);
        target.afflictions.insert(aff.key(), aff);

        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].kind, AfflictionKind::Blind);
    }

    #[test]
    fn mixed_severities_filter_correctly() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target = Tribute::new("Target".to_string(), None, None);
        target.area = Area::Sector1;

        let mild = make_affliction(AfflictionKind::Wounded, Severity::Mild);
        let moderate = make_affliction(AfflictionKind::BrokenBone, Severity::Moderate);
        let severe = make_affliction(AfflictionKind::Blind, Severity::Severe);
        target.afflictions.insert(mild.key(), mild);
        target.afflictions.insert(moderate.key(), moderate);
        target.afflictions.insert(severe.key(), severe);

        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 2);

        observer.area = Area::Sector2;
        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].kind, AfflictionKind::Blind);
    }

    #[test]
    fn target_has_visible_affliction_checks_correctly() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target = Tribute::new("Target".to_string(), None, None);
        target.area = Area::Sector1;
        let aff = make_affliction(AfflictionKind::Blind, Severity::Moderate);
        target.afflictions.insert(aff.key(), aff);

        assert!(target_has_visible_affliction(
            &observer,
            &target,
            AfflictionKind::Blind,
            Severity::Moderate
        ));
        assert!(!target_has_visible_affliction(
            &observer,
            &target,
            AfflictionKind::MissingArm,
            Severity::Moderate
        ));

        observer.area = Area::Sector2;
        assert!(!target_has_visible_affliction(
            &observer,
            &target,
            AfflictionKind::Blind,
            Severity::Moderate
        ));
    }
}
