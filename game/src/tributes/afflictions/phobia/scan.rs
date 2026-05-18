//! Per-cycle phobia scan skeleton.
//!
//! Walks each tribute's phobia afflictions and checks whether their trigger
//! is present in the current cycle context. Pure detection — no reactions.
//!
//! PR1 returns nothing. PR2 will produce reaction data for the brain pipeline.
//!
//! See spec §4, §7.

use shared::afflictions::{AfflictionKind, PhobiaMetadata, Severity};

use crate::tributes::Tribute;

use super::triggers::{PhobiaContext, is_present};

/// Result of scanning a single tribute's phobias for one cycle.
#[derive(Debug, Clone, Default)]
pub struct PhobiaScanResult {
    /// Number of phobias that fired this cycle.
    pub firing_count: u32,
    /// Highest severity among firing phobias.
    pub highest_firing_severity: Option<Severity>,
}

/// Scans all tributes for phobia triggers in the current cycle.
///
/// For each tribute with Phobia afflictions:
/// 1. Checks if each trigger is present via `is_present()`
/// 2. Updates `cycles_since_last_fire` counter on metadata
/// 3. Tracks observer state (PR3 will flesh this out)
///
/// This is a pure detection pass. Reactions (stat penalties, flee bias,
/// freeze) are implemented in PR2.
pub fn scan_phobias<'a>(
    tributes: &'a mut [Tribute],
    ctx: &PhobiaContext<'_>,
    cycle: u32,
) -> Vec<(&'a Tribute, PhobiaScanResult)> {
    let mut results = Vec::new();

    for tribute in tributes.iter_mut() {
        if !tribute.is_alive() {
            continue;
        }

        let result = scan_tribute_phobias(tribute, ctx, cycle);
        if result.firing_count > 0 {
            results.push((&*tribute, result));
        }
    }

    results
}

/// Scans a single tribute's phobia afflictions.
fn scan_tribute_phobias(
    tribute: &mut Tribute,
    ctx: &PhobiaContext<'_>,
    cycle: u32,
) -> PhobiaScanResult {
    let mut result = PhobiaScanResult::default();

    // Collect phobia data first to avoid borrow conflict between
    // immutable tribute borrow (is_present) and mutable affliction borrow.
    let phobia_data: Vec<_> = tribute
        .afflictions
        .iter()
        .filter(|(_, aff)| aff.phobia_metadata.is_some())
        .map(|(key, aff)| (*key, aff.severity))
        .collect();

    // Check which phobias are firing (immutable borrow of tribute).
    let firing: Vec<_> = phobia_data
        .iter()
        .filter(|(key, _)| {
            let AfflictionKind::Phobia(trigger) = key.0 else {
                return false;
            };
            is_present(&trigger, tribute, ctx)
        })
        .map(|(key, severity)| (*key, *severity))
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
            on_phobia_fire(meta, cycle);
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
            on_phobia_idle(meta);
        }
    }

    result
}

/// Called when a phobia fires this cycle.
///
/// Resets the decay counter and (for Traumatic phobias) would trigger
/// the sensitization escalation roll in PR3.
fn on_phobia_fire(meta: &mut PhobiaMetadata, cycle: u32) {
    meta.cycles_since_last_fire = 0;
    meta.observer_seen_cycle
        .retain(|_observer_id, last_seen| cycle.saturating_sub(*last_seen) <= 5);
}

/// Called when a phobia does not fire this cycle.
///
/// Increments the decay counter. At threshold (5 cycles), Traumatic
/// phobias decay one tier in PR3.
fn on_phobia_idle(meta: &mut PhobiaMetadata) {
    meta.cycles_since_last_fire = meta.cycles_since_last_fire.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::areas::events::AreaEvent;
    use crate::terrain::{BaseTerrain, TerrainType};
    use crate::tributes::AfflictionDraft;
    use shared::afflictions::{AfflictionSource, PhobiaOrigin, PhobiaTrigger};

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
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
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
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.firing_count, 1);
    }

    #[test]
    fn scan_no_firing_when_trigger_absent() {
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);

        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let ctx = make_context_clear();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1);
        assert!(results.is_empty());
    }

    #[test]
    fn scan_dead_tribute_skipped() {
        use crate::tributes::statuses::TributeStatus;
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        tribute.status = TributeStatus::RecentlyDead;

        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);

        let ctx = make_context_with_fire();
        let results = scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1);
        assert!(results.is_empty());
    }

    #[test]
    fn cycles_since_last_fire_increments_on_idle() {
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
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
        scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1);

        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.phobia_metadata.as_ref().unwrap().cycles_since_last_fire,
            3
        );
    }

    #[test]
    fn cycles_since_last_fire_resets_on_fire() {
        let mut tribute = crate::tributes::Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
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
        scan_phobias(std::slice::from_mut(&mut tribute), &ctx, 1);

        let (_, aff) = tribute.afflictions.iter().next().unwrap();
        assert_eq!(
            aff.phobia_metadata.as_ref().unwrap().cycles_since_last_fire,
            0
        );
    }
}
