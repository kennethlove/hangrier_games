//! Per-cycle phobia scan skeleton.
//!
//! Walks each tribute's phobia afflictions and checks whether their trigger
//! is present in the current cycle context. Pure detection — no reactions.
//!
//! PR1 returns nothing. PR2 will produce reaction data for the brain pipeline.
//!
//! See spec §4, §7.

use shared::afflictions::{AfflictionKind, Severity};
use shared::messages::MessagePayload;

use crate::tributes::Tribute;

use super::outcomes::{on_phobia_fire, on_phobia_idle};
use super::triggers::{PhobiaContext, is_present};

/// Result of scanning a single tribute's phobias for one cycle.
#[derive(Debug, Clone, Default)]
pub struct PhobiaScanResult {
    /// Number of phobias that fired this cycle.
    pub firing_count: u32,
    /// Highest severity among firing phobias.
    pub highest_firing_severity: Option<Severity>,
    /// Messages produced by this scan (observations, escalations, habituations).
    pub messages: Vec<MessagePayload>,
}

/// Scans all tributes for phobia triggers in the current cycle.
///
/// For each tribute with Phobia afflictions:
/// 1. Checks if each trigger is present via `is_present()`
/// 2. Updates `cycles_since_last_fire` counter on metadata
/// 3. Tracks observer state
/// 4. Handles traumatic reinforcement on fire, traumatic decay on idle
///
/// Reactions (stat penalties, flee bias, freeze) are implemented in PR2.
pub fn scan_phobias<'a>(
    tributes: &'a mut [Tribute],
    ctx: &PhobiaContext<'_>,
    cycle: u32,
    rng: &mut impl rand::Rng,
) -> Vec<(&'a Tribute, PhobiaScanResult)> {
    let mut results = Vec::new();

    for tribute in tributes.iter_mut() {
        if !tribute.is_alive() {
            continue;
        }

        let result = scan_tribute_phobias(tribute, ctx, cycle, rng);
        if result.firing_count > 0 || !result.messages.is_empty() {
            results.push((&*tribute, result));
        }
    }

    results
}

/// Scan a single tribute's phobia afflictions.
///
/// Convenience wrapper for calling from the game cycle where tributes are
/// processed area-by-area.
pub fn scan_tribute(
    tribute: &mut Tribute,
    ctx: &PhobiaContext<'_>,
    cycle: u32,
    rng: &mut impl rand::Rng,
) -> PhobiaScanResult {
    scan_tribute_phobias(tribute, ctx, cycle, rng)
}

/// Scans a single tribute's phobia afflictions.
fn scan_tribute_phobias(
    tribute: &mut Tribute,
    ctx: &PhobiaContext<'_>,
    cycle: u32,
    rng: &mut impl rand::Rng,
) -> PhobiaScanResult {
    let mut result = PhobiaScanResult::default();

    // Collect phobia data first to avoid borrow conflict between
    // immutable tribute borrow (is_present) and mutable affliction borrow.
    let phobia_data: Vec<_> = tribute
        .afflictions
        .iter()
        .filter(|(_, aff)| aff.phobia_metadata.is_some())
        .map(|(key, aff)| (key.clone(), aff.severity))
        .collect();

    // Check which phobias are firing (immutable borrow of tribute).
    let firing: Vec<_> = phobia_data
        .iter()
        .filter(|(key, _)| {
            let AfflictionKind::Phobia(ref trigger) = key.0 else {
                return false;
            };
            is_present(trigger, tribute, ctx)
        })
        .map(|(key, severity)| (key.clone(), *severity))
        .collect();

    // Update metadata based on firing state.
    for (key, severity) in &firing {
        result.firing_count += 1;
        if result.highest_firing_severity.is_none_or(|s| *severity > s) {
            result.highest_firing_severity = Some(*severity);
        }
        if let Some(aff) = tribute.afflictions.get_mut(key)
            && let Some(meta) = &mut aff.phobia_metadata
        {
            let AfflictionKind::Phobia(ref trigger) = aff.kind else {
                unreachable!()
            };
            let msgs = on_phobia_fire(
                meta,
                cycle,
                &mut aff.severity,
                trigger,
                tribute.identifier.as_str(),
                ctx,
                rng,
            );
            result.messages.extend(msgs);
        }
    }

    // Tick idle phobias.
    for (key, _) in &phobia_data {
        if firing.iter().any(|(k, _)| k == key) {
            continue;
        }
        if let Some(aff) = tribute.afflictions.get_mut(key)
            && let Some(meta) = &mut aff.phobia_metadata
        {
            let AfflictionKind::Phobia(ref trigger) = aff.kind else {
                unreachable!()
            };
            let (msgs, should_remove) = on_phobia_idle(
                meta,
                &mut aff.severity,
                trigger,
                tribute.identifier.as_str(),
            );
            result.messages.extend(msgs);
            if should_remove {
                tribute.afflictions.remove(key);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::areas::events::AreaEvent;
    use crate::terrain::{BaseTerrain, TerrainType};
    use crate::tributes::AfflictionDraft;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::{AfflictionSource, PhobiaMetadata, PhobiaOrigin, PhobiaTrigger};

    fn make_context_with_fire() -> PhobiaContext<'static> {
        use std::sync::LazyLock;
        static AREA: LazyLock<crate::areas::AreaDetails> = LazyLock::new(|| {
            let mut area = crate::areas::AreaDetails::new(None, Area::Cornucopia);
            area.terrain = TerrainType::new(BaseTerrain::Forest, vec![]).unwrap();
            area.events = vec![AreaEvent::Wildfire];
            area
        });
        PhobiaContext {
            area: &AREA,
            is_night: false,
            other_tributes_in_area: &[],
            cycle_messages: &[],
            cycle: 1,
        }
    }

    fn make_context_clear() -> PhobiaContext<'static> {
        use std::sync::LazyLock;
        static AREA: LazyLock<crate::areas::AreaDetails> = LazyLock::new(|| {
            let mut area = crate::areas::AreaDetails::new(None, Area::Cornucopia);
            area.terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
            area
        });
        PhobiaContext {
            area: &AREA,
            is_night: false,
            other_tributes_in_area: &[],
            cycle_messages: &[],
            cycle: 1,
        }
    }

    #[test]
    fn scan_detects_firing_phobia() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        // Add phobia metadata
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_with_fire();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.firing_count, 1);
    }

    #[test]
    fn scan_no_firing_when_trigger_absent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_clear();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);
        assert!(results.is_empty());
    }

    #[test]
    fn scan_dead_tribute_skipped() {
        let mut rng = SmallRng::seed_from_u64(42);
        use crate::tributes::statuses::TributeStatus;
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        tribute.status = TributeStatus::RecentlyDead;

        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        let ctx = make_context_with_fire();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);
        assert!(results.is_empty());
    }

    #[test]
    fn cycles_since_last_fire_increments_on_idle() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                cycles_since_last_fire: 2,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_clear();
        scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.phobia_metadata.as_ref().unwrap().cycles_since_last_fire,
            3
        );
    }

    fn make_context_with_others(others: &[Tribute]) -> PhobiaContext<'_> {
        use std::sync::LazyLock;
        static AREA: LazyLock<crate::areas::AreaDetails> = LazyLock::new(|| {
            let mut area = crate::areas::AreaDetails::new(None, Area::Cornucopia);
            area.terrain = TerrainType::new(BaseTerrain::Forest, vec![]).unwrap();
            area.events = vec![AreaEvent::Wildfire];
            area
        });
        PhobiaContext {
            area: &AREA,
            is_night: false,
            other_tributes_in_area: others,
            cycle_messages: &[],
            cycle: 1,
        }
    }

    #[test]
    fn innate_phobia_can_escalate_with_amendment() {
        // Spec amendment: all phobia firings roll ~12% escalation.
        // Innate phobias can now escalate. Use seed that triggers escalation.
        let mut rng = SmallRng::seed_from_u64(3);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        // Add Innate phobia metadata
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        } else {
            panic!("no affliction after acquire");
        }

        let ctx = make_context_with_fire();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        assert!(
            !results.is_empty(),
            "innate phobia should fire in fire context"
        );

        let has_escalation = results[0].1.messages.iter().any(|m| {
            matches!(
                m,
                MessagePayload::PhobiaEscalated {
                    from_severity,
                    to_severity,
                    ..
                } if from_severity == "mild" && to_severity == "moderate"
            )
        });
        assert!(
            has_escalation,
            "innate phobia should escalate with spec amendment"
        );

        // Verify severity was escalated.
        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.severity,
            Severity::Moderate,
            "innate phobia severity must escalate to Moderate"
        );
    }

    #[test]
    fn traumatic_phobia_escalates_at_seed_zero() {
        let mut rng = SmallRng::seed_from_u64(3);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        // Add Traumatic phobia metadata.
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Traumatic {
                    event_ref: "forest_fire".into(),
                },
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_with_fire();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        assert!(
            !results.is_empty(),
            "traumatic phobia should fire in fire context"
        );
        assert!(
            !results[0].1.messages.is_empty(),
            "should have messages after scan"
        );

        let has_escalation = results[0].1.messages.iter().any(|m| {
            matches!(
                m,
                MessagePayload::PhobiaEscalated {
                    from_severity,
                    to_severity,
                    ..
                } if from_severity == "mild" && to_severity == "moderate"
            )
        });
        assert!(
            has_escalation,
            "expected PhobiaEscalated from mild to moderate"
        );

        // Verify severity was updated on the affliction.
        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.severity,
            Severity::Moderate,
            "traumatic phobia severity must escalate to Moderate"
        );
    }

    #[test]
    fn traumatic_habituation_decays_severity() {
        let mut rng = SmallRng::seed_from_u64(99);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        // Add Traumatic phobia metadata with idle counter at threshold.
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Traumatic {
                    event_ref: "forest_fire".into(),
                },
                cycles_since_last_fire: 5,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_clear();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        assert!(
            !results.is_empty(),
            "should produce result even with firing_count=0"
        );
        let result = &results[0].1;

        let has_habituation = result.messages.iter().any(|m| {
            matches!(
                m,
                MessagePayload::PhobiaHabituated {
                    from_severity,
                    to_severity: Some(to),
                    ..
                } if from_severity == "severe" && to == "moderate"
            )
        });
        assert!(
            has_habituation,
            "expected PhobiaHabituated from severe to moderate"
        );

        // Verify severity decayed on the affliction.
        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.severity,
            Severity::Moderate,
            "severity must decay one tier"
        );
    }

    #[test]
    fn traumatic_habituation_cures_mild() {
        let mut rng = SmallRng::seed_from_u64(99);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        let key = (AfflictionKind::Phobia(PhobiaTrigger::Fire), None);

        // Add Traumatic phobia metadata with idle counter at threshold.
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Traumatic {
                    event_ref: "forest_fire".into(),
                },
                cycles_since_last_fire: 5,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_clear();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        assert!(
            !results.is_empty(),
            "should produce result even with firing_count=0"
        );
        let result = &results[0].1;

        let has_cure = result.messages.iter().any(|m| {
            matches!(
                m,
                MessagePayload::PhobiaHabituated {
                    from_severity,
                    to_severity: None,
                    ..
                } if from_severity == "mild"
            )
        });
        assert!(
            has_cure,
            "expected PhobiaHabituated with to_severity: None (cured)"
        );

        // Affliction should be removed from the map.
        assert!(
            !tribute.afflictions.contains_key(&key),
            "phobia should be removed (cured) from afflictions"
        );
    }

    #[test]
    fn observer_added_on_moderate_fire() {
        let mut main_tribute = crate::tributes::Tribute::new("MainTribute".to_string(), None, None);
        let observer_tribute = crate::tributes::Tribute::new("Observer".to_string(), None, None);

        // Capture UUID-generated identifiers before they're moved.
        let main_id = main_tribute.identifier.clone();
        let observer_id = observer_tribute.identifier.clone();

        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        main_tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = main_tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let mut rng = SmallRng::seed_from_u64(42);
        let others = [observer_tribute];
        let ctx = make_context_with_others(&others);
        let results = scan_phobias(std::slice::from_mut(&mut main_tribute), &ctx, 1, &mut rng);

        assert!(!results.is_empty(), "should fire in fire context");
        let result = &results[0].1;

        let has_observed = result.messages.iter().any(|m| {
            matches!(
                m,
                MessagePayload::PhobiaObserved {
                    observer,
                    subject,
                    ..
                } if observer_id == *observer && main_id == *subject
            )
        });
        assert!(
            has_observed,
            "should emit PhobiaObserved for Moderate firing"
        );

        // Check metadata was updated.
        let (_, aff) = main_tribute.afflictions.iter().next().unwrap();
        let meta = aff.phobia_metadata.as_ref().unwrap();
        assert!(
            meta.observed_by.contains(observer_id.as_str()),
            "observer should be tracked in observed_by"
        );
    }

    #[test]
    fn no_observer_on_mild_fire() {
        let mut main_tribute = crate::tributes::Tribute::new("MainTribute".to_string(), None, None);
        let observer_tribute = crate::tributes::Tribute::new("Observer".to_string(), None, None);

        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        main_tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = main_tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let mut rng = SmallRng::seed_from_u64(42);
        let others = [observer_tribute];
        let ctx = make_context_with_others(&others);
        let results = scan_phobias(std::slice::from_mut(&mut main_tribute), &ctx, 1, &mut rng);

        assert!(!results.is_empty(), "should fire in fire context");
        let result = &results[0].1;

        let has_observed = result
            .messages
            .iter()
            .any(|m| matches!(m, MessagePayload::PhobiaObserved { .. }));
        assert!(!has_observed, "Mild phobia must not emit PhobiaObserved");

        // observed_by should be empty.
        let (_, aff) = main_tribute.afflictions.iter().next().unwrap();
        let meta = aff.phobia_metadata.as_ref().unwrap();
        assert!(
            meta.observed_by.is_empty(),
            "no observers should be recorded for Mild phobia"
        );
    }

    #[test]
    fn cycles_since_last_fire_resets_on_fire() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            trapped_metadata: None,
        };
        tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                cycles_since_last_fire: 5,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_with_fire();
        scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1, &mut rng);

        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.phobia_metadata.as_ref().unwrap().cycles_since_last_fire,
            0
        );
    }
}
