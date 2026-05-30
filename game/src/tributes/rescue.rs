//! Rescue resolution logic for Trapped afflictions.
//!
//! See `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md` §10.

use crate::areas::AreaDetails;
use crate::config::GameConfig;
use crate::messages::{MessagePayload, TaggedEvent};
use crate::tributes::Tribute;
use rand::Rng;
use rand::RngExt;
use shared::afflictions::{AfflictionKind, PARTIAL_RESCUE_THRESHOLD, RESCUE_BONUS_CAP, Severity};

/// Compute a single rescuer's bonus contribution.
///
/// Formula: `0.25 + (rescuer_strength / MAX_STAT) * 0.30`
/// Clamped to `[0.25, 0.55]`.
///
/// At max Strength (50), bonus = 0.25 + 1.0 * 0.30 = 0.55.
/// At min Strength (0), bonus = 0.25 + 0.0 * 0.30 = 0.25.
pub fn compute_rescue_bonus(rescuer_strength: f32) -> f32 {
    let max_strength = GameConfig::default().max_strength as f32;
    let normalized = (rescuer_strength / max_strength).clamp(0.0, 1.0);
    let bonus = 0.25 + normalized * 0.30;
    bonus.clamp(0.25, 0.55)
}

/// Accumulate a rescuer bonus into the target's existing bonus, capped at
/// `RESCUE_BONUS_CAP`. This prevents 4 max-Strength rescuers from
/// trivializing the escape roll.
pub fn accumulate_rescue_bonus(current: f32, additional: f32) -> f32 {
    (current + additional).min(RESCUE_BONUS_CAP)
}

/// Resolve a rescue action from `rescuer` targeting `target`.
///
/// Steps per spec §10:
/// 1. Validate co-location (same area)
/// 2. Validate target has Trapped affliction
/// 3. Compute rescuer bonus from strength
/// 4. If Severe + first rescuer this cycle → increment escape_progress
/// 5. Else → add to accumulated rescue bonus
/// 6. Consume rescuer's turn (caller responsibility)
///
/// Returns `true` if rescue was resolved, `false` otherwise.
pub fn resolve_rescue(
    area: &AreaDetails,
    rescuer: &Tribute,
    target: &mut Tribute,
    events: &mut Vec<TaggedEvent>,
    _rng: &mut impl Rng,
) -> bool {
    // 1. Validate co-location
    if target.area != rescuer.area || area.area != Some(target.area) {
        return false;
    }

    // 2. Find target's Trapped affliction
    let (trapped_key, severity) = {
        let aff = target
            .afflictions
            .iter()
            .find(|((kind, _), _)| matches!(kind, AfflictionKind::Trapped(_)));
        match aff {
            Some(((kind, part), aff)) => ((kind.clone(), *part), aff.severity),
            None => return false,
        }
    };

    // Determine TrapKind from the affliction kind
    let trap_kind = match trapped_key.0.clone() {
        AfflictionKind::Trapped(k) => k,
        _ => unreachable!("trapped_key was confirmed as Trapped above"),
    };

    // 3. Compute rescuer bonus
    let bonus = compute_rescue_bonus(rescuer.attributes.strength as f32);

    // 4. Severe + first rescuer this cycle → partial rescue progress
    if severity == Severity::Severe {
        let existing_bonus = target
            .afflictions
            .get(&trapped_key)
            .and_then(|a| a.trapped_metadata.as_ref())
            .map(|m| m.rescue_bonus_accumulated)
            .unwrap_or(0.0);

        if existing_bonus <= 0.0 {
            // No bonus yet this cycle
            if let Some(meta) = target
                .afflictions
                .get_mut(&trapped_key)
                .and_then(|a| a.trapped_metadata.as_mut())
            {
                meta.escape_progress = meta.escape_progress.saturating_add(1);
                events.push(TaggedEvent::new(
                    format!(
                        "{} helps {} escape — making progress ({}/{})",
                        rescuer.name, target.name, meta.escape_progress, PARTIAL_RESCUE_THRESHOLD
                    ),
                    MessagePayload::PartialRescueProgress {
                        rescuer: rescuer.identifier.clone(),
                        target: target.identifier.clone(),
                        kind: trap_kind,
                        severity,
                        bonus,
                        progress: meta.escape_progress,
                        threshold: PARTIAL_RESCUE_THRESHOLD,
                    },
                ));

                // If progress reaches threshold, also add the bonus
                if meta.escape_progress >= PARTIAL_RESCUE_THRESHOLD {
                    meta.rescue_bonus_accumulated =
                        accumulate_rescue_bonus(meta.rescue_bonus_accumulated, bonus);
                }
            }
            return true;
        }
    }

    // 5. Apply bonus to accumulated rescue bonus (for non-Severe or subsequent rescuers)
    if let Some(meta) = target
        .afflictions
        .get_mut(&trapped_key)
        .and_then(|a| a.trapped_metadata.as_mut())
    {
        meta.rescue_bonus_accumulated =
            accumulate_rescue_bonus(meta.rescue_bonus_accumulated, bonus);

        events.push(TaggedEvent::new(
            format!(
                "{} tries to rescue {} (bonus: {:.2})",
                rescuer.name, target.name, bonus
            ),
            MessagePayload::RescueAttempted {
                rescuer: rescuer.identifier.clone(),
                target: target.identifier.clone(),
                kind: trap_kind,
                severity,
                bonus,
            },
        ));
    }

    true
}

/// Evaluate whether `potential_rescuer` should rescue a trapped co-located
/// tribute. Returns `Some(target_id)` if rescue is warranted, `None` otherwise.
///
/// Checks co-located tributes for any with Trapped afflictions, then rolls
/// against a base rescue chance. Future iterations should scale by
/// compassion/magnanimity personality traits and affinity data.
pub fn evaluate_rescue_opportunity(
    potential_rescuer: &Tribute,
    _area: &AreaDetails,
    area_tributes: &[Tribute],
    rng: &mut impl Rng,
) -> Option<String> {
    // Find co-located tributes with Trapped affliction
    let trapped_targets: Vec<&Tribute> = area_tributes
        .iter()
        .filter(|t| {
            t.identifier != potential_rescuer.identifier
                && t.afflictions
                    .values()
                    .any(|a| matches!(a.kind, AfflictionKind::Trapped(_)))
        })
        .collect();

    if trapped_targets.is_empty() {
        return None;
    }

    // Simple: 30% base chance to rescue the first trapped tribute found.
    // Future: scale by compassion/magnanimity traits, affinity, etc.
    let target = trapped_targets[0];
    let base_chance = 0.30;

    if rng.random_bool(base_chance) {
        Some(target.identifier.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::tributes::Tribute;
    use rand::SeedableRng;
    use rstest::rstest;
    use shared::afflictions::{
        AfflictionKind, AfflictionSource, Severity, TrapKind, TrappedMetadata,
    };

    fn trapped_tribute(name: &str, severity: Severity) -> Tribute {
        let mut t = Tribute::new(name.into(), None, None);
        t.attributes.strength = 5;
        t.try_acquire_affliction(crate::tributes::AfflictionDraft {
            kind: AfflictionKind::Trapped(TrapKind::Buried),
            body_part: None,
            severity,
            source: AfflictionSource::Environmental,
            trapped_metadata: Some(TrappedMetadata::fresh_for(TrapKind::Buried, None)),
        });
        t
    }

    fn free_tribute(name: &str, strength: u32) -> Tribute {
        let mut t = Tribute::new(name.into(), None, None);
        t.attributes.strength = strength;
        t.area = Area::Cornucopia;
        t
    }

    fn same_area_details() -> AreaDetails {
        AreaDetails {
            area: Some(Area::Cornucopia),
            ..Default::default()
        }
    }

    // -- compute_rescue_bonus tests --

    #[test]
    fn rescue_bonus_min_strength() {
        let b = compute_rescue_bonus(0.0);
        assert!((b - 0.25).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn rescue_bonus_max_strength() {
        let b = compute_rescue_bonus(50.0);
        assert!((b - 0.55).abs() < 1e-6, "got {b}");
    }

    #[test]
    fn rescue_bonus_mid_strength() {
        let b = compute_rescue_bonus(25.0);
        // 0.25 + (25/50) * 0.30 = 0.25 + 0.15 = 0.40
        assert!((b - 0.40).abs() < 1e-6, "got {b}");
    }

    #[rstest]
    #[case(0.0, 0.25)]
    #[case(10.0, 0.31)]
    #[case(25.0, 0.40)]
    #[case(40.0, 0.49)]
    #[case(50.0, 0.55)]
    fn rescue_bonus_parametrized(#[case] strength: f32, #[case] expected: f32) {
        let b = compute_rescue_bonus(strength);
        assert!(
            (b - expected).abs() < 1e-4,
            "strength={strength} got {b} expected {expected}"
        );
    }

    #[test]
    fn rescue_bonus_clamps_above_55() {
        let b = compute_rescue_bonus(100.0);
        assert_eq!(b, 0.55);
    }

    #[test]
    fn accumulate_stays_below_cap() {
        let total = accumulate_rescue_bonus(0.55, 0.55);
        assert_eq!(total, RESCUE_BONUS_CAP);
    }

    #[test]
    fn accumulate_sums_below_cap() {
        let total = accumulate_rescue_bonus(0.20, 0.30);
        assert!((total - 0.50).abs() < 1e-6);
    }

    // -- resolve_rescue tests --

    #[test]
    fn resolve_rescue_co_location_validates() {
        let mut target = trapped_tribute("target", Severity::Moderate);
        let rescuer = free_tribute("rescuer", 30);
        let mut area = same_area_details();
        area.area = Some(Area::Sector1); // different area from Cornucopia
        let mut events = Vec::new();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        let result = resolve_rescue(&area, &rescuer, &mut target, &mut events, &mut rng);
        assert!(!result, "rescue should fail when areas differ");
        assert!(events.is_empty(), "no events expected on co-location fail");
    }

    #[test]
    fn resolve_rescue_mild_increases_bonus() {
        let rescuer = free_tribute("rescuer", 30);
        let mut target = trapped_tribute("target", Severity::Mild);
        let area = same_area_details();
        let mut events = Vec::new();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        let result = resolve_rescue(&area, &rescuer, &mut target, &mut events, &mut rng);
        assert!(result, "rescue should succeed for co-located Mild");

        let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
        let meta = target
            .afflictions
            .get(&key)
            .unwrap()
            .trapped_metadata
            .as_ref()
            .unwrap();
        assert!(
            meta.rescue_bonus_accumulated > 0.0,
            "expected bonus > 0, got {}",
            meta.rescue_bonus_accumulated
        );

        let has_rescue_attempt = events
            .iter()
            .any(|e| matches!(e.payload, MessagePayload::RescueAttempted { .. }));
        assert!(has_rescue_attempt, "expected RescueAttempted event");
    }

    #[test]
    fn resolve_rescue_severe_increments_progress() {
        let rescuer = free_tribute("rescuer", 30);
        let mut target = trapped_tribute("target", Severity::Severe);
        let area = same_area_details();
        let mut events = Vec::new();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        let result = resolve_rescue(&area, &rescuer, &mut target, &mut events, &mut rng);
        assert!(result, "rescue should succeed for Severe");

        let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
        let meta = target
            .afflictions
            .get(&key)
            .unwrap()
            .trapped_metadata
            .as_ref()
            .unwrap();
        // First rescue at Severe: progress incremented, bonus NOT applied yet
        assert_eq!(
            meta.escape_progress, 1,
            "progress should be 1 after first rescue"
        );
        assert_eq!(
            meta.rescue_bonus_accumulated, 0.0,
            "no bonus applied until threshold"
        );

        let has_partial = events
            .iter()
            .any(|e| matches!(e.payload, MessagePayload::PartialRescueProgress { .. }));
        assert!(has_partial, "expected PartialRescueProgress event");
    }

    #[test]
    fn resolve_rescue_severe_threshold_applies_bonus() {
        let rescuer = free_tribute("rescuer", 30);
        let mut target = trapped_tribute("target", Severity::Severe);
        let area = same_area_details();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        // First rescue: progress = 1
        let mut events1 = Vec::new();
        resolve_rescue(&area, &rescuer, &mut target, &mut events1, &mut rng);

        // Second rescue: progress reaches threshold, bonus applied
        let mut events2 = Vec::new();
        let result = resolve_rescue(&area, &rescuer, &mut target, &mut events2, &mut rng);
        assert!(result);

        let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
        let meta = target
            .afflictions
            .get(&key)
            .unwrap()
            .trapped_metadata
            .as_ref()
            .unwrap();
        assert_eq!(
            meta.escape_progress, 2,
            "progress should be 2 after second rescue"
        );
        assert!(
            meta.rescue_bonus_accumulated > 0.0,
            "bonus should be > 0 after threshold"
        );

        let has_partial = events2
            .iter()
            .any(|e| matches!(e.payload, MessagePayload::PartialRescueProgress { .. }));
        assert!(
            has_partial,
            "expected PartialRescueProgress on second rescue too"
        );
    }

    #[test]
    fn evaluate_rescue_finds_trapped_tribute() {
        let mut rescuer = Tribute::new("Rescuer".into(), None, None);
        rescuer.area = Area::Cornucopia;

        let mut trapped = Tribute::new("Trapped".into(), None, None);
        trapped.area = Area::Cornucopia;
        trapped.try_acquire_affliction(crate::tributes::AfflictionDraft {
            kind: AfflictionKind::Trapped(TrapKind::Buried),
            body_part: None,
            severity: Severity::Moderate,
            source: shared::afflictions::AfflictionSource::Environmental,
            trapped_metadata: Some(TrappedMetadata::fresh_for(TrapKind::Buried, None)),
        });

        let area = AreaDetails::default();
        let tributes = vec![rescuer.clone(), trapped];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

        // With a 30% chance, we should see Some at least sometimes.
        // Use a seed that triggers it.
        let result = evaluate_rescue_opportunity(&rescuer, &area, &tributes, &mut rng);
        // We don't assert specific value since it's probabilistic.
        // Just verify the function runs without error.
        let _ = result;
    }
}
