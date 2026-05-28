//! Fixation override layer for the brain pipeline.
//!
//! Runs after stamina_override, before phobia_override. Provides:
//! 1. Per-tier override semantics — Mild = tiebreaker, Moderate = strong-bias,
//!    Severe = compulsion (overrides all unless target unreachable)
//! 2. Per-target hooks — Tribute → Attack, Item → Loot/Gather, Area → Move
//! 3. Trait modifiers — Resilient/Fragile/Loyal affect effective tier
//!
//! Pipeline order: [..., survival, stamina, **fixation**, phobia, trauma, affliction, preferred, alliance, consumable]
//!
//! See spec §8 (fixation brain layer).

use shared::afflictions::{AfflictionKind, FixationTarget, Severity};

use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::traits::Trait;

/// Context available to the fixation override layer.
#[derive(Clone, Debug)]
pub struct FixationOverrideContext {
    /// Whether the tribute's fixation target is reachable this cycle.
    /// `None` when the target is unknown/unavailable.
    pub target_reachable: bool,
}

/// Compute effective severity for a fixation, applying trait modifiers.
///
/// - Resilient: -1 tier (fixation feels less urgent)
/// - Fragile: +1 tier (fixation feels more urgent)
/// - Loyal: +1 tier when target is a Tribute (ally fixation stronger)
pub fn effective_tier(severity: Severity, traits: &[Trait], kind: &AfflictionKind) -> Severity {
    let tier = severity.ordinal() as i32;
    let mut modifier = 0i32;

    for t in traits {
        match t {
            Trait::Resilient => modifier -= 1,
            Trait::Fragile => modifier += 1,
            Trait::Loyal => {
                if matches!(kind, AfflictionKind::Fixation(FixationTarget::Tribute(_))) {
                    modifier += 1;
                }
            }
            _ => {}
        }
    }

    let adjusted = (tier + modifier).clamp(0, Severity::Severe.ordinal() as i32);
    match adjusted {
        0 => Severity::Mild,
        1 => Severity::Moderate,
        _ => Severity::Severe,
    }
}

/// Determine which action a fixation pushes toward, based on the target type.
pub fn fixation_target_action(kind: &AfflictionKind) -> Option<Action> {
    match kind {
        AfflictionKind::Fixation(FixationTarget::Tribute(_)) => Some(Action::Attack),
        AfflictionKind::Fixation(FixationTarget::Item(_)) => Some(Action::TakeItem),
        AfflictionKind::Fixation(FixationTarget::Area(_)) => Some(Action::Move(None)),
        _ => None,
    }
}

/// Fixation override layer entry point for the pre-decision pipeline.
///
/// Returns `Some(action)` to short-circuit the brain pipeline, or `None`
/// to fall through.
///
/// Per-tier semantics:
/// - Mild: Only overrides when no strong preference exists (tiebreaker).
///   Returns `None` to let normal scoring decide — the bias layer handles it.
/// - Moderate: Strong-bias — overrides equal or slightly-better alternatives.
///   Returns the fixation action.
/// - Severe: Compulsion — overrides all alternatives unless the target
///   is unreachable. Returns the fixation action unconditionally.
///
/// Per-target action mapping:
/// - Fixation(Tribute(id)) → Attack
/// - Fixation(Item(id)) → TakeItem  
/// - Fixation(Area(name)) → Move
pub fn fixation_override(tribute: &Tribute, ctx: &FixationOverrideContext) -> Option<Action> {
    if tribute.afflictions.is_empty() {
        return None;
    }

    // Find the highest-severity fixation.
    let fixation_kind = find_strongest_fixation(tribute)?;

    let effective = effective_tier(
        strongest_fixation_severity(tribute)?,
        &tribute.traits,
        &fixation_kind,
    );

    match effective {
        Severity::Mild => {
            // Tiebreaker only — let normal scoring handle it.
            // The brain bias layer provides the mild nudge.
            None
        }
        Severity::Moderate => {
            // Strong-bias: override when target is reachable.
            if ctx.target_reachable {
                fixation_target_action(&fixation_kind)
            } else {
                None
            }
        }
        Severity::Severe => {
            // Compulsion: override unless target is unreachable.
            if ctx.target_reachable {
                fixation_target_action(&fixation_kind)
            } else {
                None
            }
        }
    }
}

/// Find the affliction kind of the strongest fixation.
fn find_strongest_fixation(tribute: &Tribute) -> Option<AfflictionKind> {
    tribute
        .afflictions
        .values()
        .filter(|a| matches!(a.kind, AfflictionKind::Fixation(_)))
        .max_by_key(|a| a.severity)
        .map(|a| a.kind.clone())
}

/// Get the severity of the strongest fixation.
fn strongest_fixation_severity(tribute: &Tribute) -> Option<Severity> {
    tribute
        .afflictions
        .values()
        .filter(|a| matches!(a.kind, AfflictionKind::Fixation(_)))
        .max_by_key(|a| a.severity)
        .map(|a| a.severity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::{Affliction, AfflictionSource, FixationMetadata, FixationOrigin};
    use std::collections::{BTreeMap, BTreeSet};

    fn make_fixation(target: FixationTarget, severity: Severity) -> Affliction {
        Affliction {
            kind: AfflictionKind::Fixation(target),
            body_part: None,
            severity,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: Some(FixationMetadata {
                origin: FixationOrigin::Innate,
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 0,
            }),
            addiction_metadata: None,
        }
    }

    fn make_tribute_with_fixation(target: FixationTarget, severity: Severity) -> Tribute {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = Tribute::new_with_rng("Test".to_string(), None, None, &mut rng);
        let aff = make_fixation(target, severity);
        tribute.afflictions.insert(aff.key(), aff);
        tribute
    }

    // --- effective_tier ---

    #[test]
    fn effective_tier_no_traits_unchanged() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        assert_eq!(effective_tier(Severity::Mild, &[], &kind), Severity::Mild);
        assert_eq!(
            effective_tier(Severity::Moderate, &[], &kind),
            Severity::Moderate
        );
        assert_eq!(
            effective_tier(Severity::Severe, &[], &kind),
            Severity::Severe
        );
    }

    #[test]
    fn effective_tier_resilient_downgrades() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        assert_eq!(
            effective_tier(Severity::Mild, &[Trait::Resilient], &kind),
            Severity::Mild
        );
        assert_eq!(
            effective_tier(Severity::Moderate, &[Trait::Resilient], &kind),
            Severity::Mild
        );
        assert_eq!(
            effective_tier(Severity::Severe, &[Trait::Resilient], &kind),
            Severity::Moderate
        );
    }

    #[test]
    fn effective_tier_fragile_upgrades() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        assert_eq!(
            effective_tier(Severity::Mild, &[Trait::Fragile], &kind),
            Severity::Moderate
        );
        assert_eq!(
            effective_tier(Severity::Moderate, &[Trait::Fragile], &kind),
            Severity::Severe
        );
        assert_eq!(
            effective_tier(Severity::Severe, &[Trait::Fragile], &kind),
            Severity::Severe
        );
    }

    #[test]
    fn effective_tier_loyal_boosts_tribute_fixation() {
        let tribute_kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        let item_kind = AfflictionKind::Fixation(FixationTarget::Item("i-1".into()));
        // Loyal + Tribute: +1 tier
        assert_eq!(
            effective_tier(Severity::Mild, &[Trait::Loyal], &tribute_kind),
            Severity::Moderate
        );
        // Loyal + Item: no boost
        assert_eq!(
            effective_tier(Severity::Mild, &[Trait::Loyal], &item_kind),
            Severity::Mild
        );
    }

    #[test]
    fn effective_tier_loyal_and_resilient_cancel() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        // Loyal +1, Resilient -1 = net 0
        assert_eq!(
            effective_tier(Severity::Mild, &[Trait::Loyal, Trait::Resilient], &kind),
            Severity::Mild
        );
    }

    // --- fixation_target_action ---

    #[test]
    fn fixation_target_action_tribute_returns_attack() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into()));
        assert_eq!(fixation_target_action(&kind), Some(Action::Attack));
    }

    #[test]
    fn fixation_target_action_item_returns_takeitem() {
        let kind = AfflictionKind::Fixation(FixationTarget::Item("i-1".into()));
        assert_eq!(fixation_target_action(&kind), Some(Action::TakeItem));
    }

    #[test]
    fn fixation_target_action_area_returns_move() {
        let kind = AfflictionKind::Fixation(FixationTarget::Area("cornucopia".into()));
        assert_eq!(fixation_target_action(&kind), Some(Action::Move(None)));
    }

    #[test]
    fn fixation_target_action_non_fixation_returns_none() {
        let kind = AfflictionKind::Wounded;
        assert_eq!(fixation_target_action(&kind), None);
    }

    // --- fixation_override ---

    #[test]
    fn mild_fixation_no_override() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Tribute("u-1".into()), Severity::Mild);
        tribute.traits.clear(); // Clear random traits to avoid Loyal boosting tier
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        assert!(fixation_override(&tribute, &ctx).is_none());
    }

    #[test]
    fn moderate_fixation_overrides_when_reachable() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Item("i-1".into()), Severity::Moderate);
        tribute.traits.clear();
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        assert_eq!(fixation_override(&tribute, &ctx), Some(Action::TakeItem));
    }

    #[test]
    fn moderate_fixation_no_override_when_unreachable() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Item("i-1".into()), Severity::Moderate);
        tribute.traits.clear();
        let ctx = FixationOverrideContext {
            target_reachable: false,
        };
        assert!(fixation_override(&tribute, &ctx).is_none());
    }

    #[test]
    fn severe_fixation_compulsion_when_reachable() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Area("cornucopia".into()), Severity::Severe);
        tribute.traits.clear();
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        assert_eq!(fixation_override(&tribute, &ctx), Some(Action::Move(None)));
    }

    #[test]
    fn severe_fixation_no_override_when_unreachable() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Area("cornucopia".into()), Severity::Severe);
        tribute.traits.clear();
        let ctx = FixationOverrideContext {
            target_reachable: false,
        };
        assert!(fixation_override(&tribute, &ctx).is_none());
    }

    #[test]
    fn no_fixation_no_override() {
        let tribute = Tribute::new("Test".to_string(), None, None);
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        assert!(fixation_override(&tribute, &ctx).is_none());
    }

    #[test]
    fn fragile_mild_becomes_moderate_overrides() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Tribute("u-1".into()), Severity::Mild);
        tribute.traits.clear();
        tribute.traits.push(Trait::Fragile);
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        // Fragile: Mild → Moderate → override to Attack
        assert_eq!(fixation_override(&tribute, &ctx), Some(Action::Attack));
    }

    #[test]
    fn resilient_severe_becomes_moderate_still_overrides() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Tribute("u-1".into()), Severity::Severe);
        tribute.traits.clear();
        tribute.traits.push(Trait::Resilient);
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        // Resilient: Severe → Moderate → still overrides (strong-bias)
        assert_eq!(fixation_override(&tribute, &ctx), Some(Action::Attack));
    }

    #[test]
    fn override_picks_highest_severity_fixation() {
        let mut tribute =
            make_tribute_with_fixation(FixationTarget::Item("i-1".into()), Severity::Mild);
        tribute.traits.clear();
        let aff = make_fixation(FixationTarget::Tribute("u-2".into()), Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let ctx = FixationOverrideContext {
            target_reachable: true,
        };
        // Should pick the Moderate (Tribute) fixation → Attack
        assert_eq!(fixation_override(&tribute, &ctx), Some(Action::Attack));
    }
}
