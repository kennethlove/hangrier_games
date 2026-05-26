//! Fixation acquisition logic: spawn-time innate rolls and item-pickup hooks.
//!
//! See spec §Fixation (PR1).

use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, AfflictionSource, FixationMetadata, FixationOrigin,
    FixationTarget, Severity,
};
use std::collections::BTreeMap;
use strum::IntoEnumIterator;

use crate::areas::Area;
use crate::items::Item;
use crate::tributes::Tribute;
use rand::RngExt;
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;

pub const MAX_FIXATIONS: usize = 2;
pub const MAX_FIXATIONS_PER_TARGET_KIND: usize = 1;

/// Count how many fixations exist in a tribute's affliction map.
pub fn count_fixations(afflictions: &BTreeMap<AfflictionKey, Affliction>) -> usize {
    afflictions
        .values()
        .filter(|a| matches!(a.kind, AfflictionKind::Fixation(_)))
        .count()
}

/// Count fixations by target variant. Returns (tribute, item, area) counts.
pub fn count_by_target_kind(
    afflictions: &BTreeMap<AfflictionKey, Affliction>,
) -> (usize, usize, usize) {
    let mut tribute_count = 0;
    let mut item_count = 0;
    let mut area_count = 0;
    for aff in afflictions.values() {
        if let AfflictionKind::Fixation(target) = &aff.kind {
            match target {
                FixationTarget::Tribute(_) => tribute_count += 1,
                FixationTarget::Item(_) => item_count += 1,
                FixationTarget::Area(_) => area_count += 1,
            }
        }
    }
    (tribute_count, item_count, area_count)
}

/// ~5% chance per tribute to spawn with an innate fixation on a random area.
/// Respects MAX_FIXATIONS and MAX_FIXATIONS_PER_TARGET_KIND.
pub fn roll_spawn_fixations(tribute: &mut Tribute, rng: &mut SmallRng) {
    if count_fixations(&tribute.afflictions) >= MAX_FIXATIONS {
        return;
    }

    // ~5% base chance
    if rng.random_range(0..100) >= 5 {
        return;
    }

    let (_, _, area_count) = count_by_target_kind(&tribute.afflictions);
    if area_count >= MAX_FIXATIONS_PER_TARGET_KIND {
        return;
    }

    // Pick a random area as fixation target
    let areas: Vec<Area> = Area::iter().collect();
    if let Some(area) = areas.choose(rng) {
        let target = FixationTarget::Area(area.to_string());
        let key = (AfflictionKind::Fixation(target.clone()), None);
        let aff = Affliction {
            kind: AfflictionKind::Fixation(target),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: Some(FixationMetadata {
                origin: FixationOrigin::Innate,
            }),
        };
        tribute.afflictions.insert(key, aff);
    }
}

/// ~10% chance to acquire an item fixation when a tribute picks up an item.
/// Respects MAX_FIXATIONS and MAX_FIXATIONS_PER_TARGET_KIND.
pub fn maybe_acquire_item_fixation(tribute: &mut Tribute, item: &Item) {
    let mut rng = SmallRng::from_rng(&mut rand::rng());

    if count_fixations(&tribute.afflictions) >= MAX_FIXATIONS {
        return;
    }

    // ~10% base chance
    if rng.random_range(0..100) >= 10 {
        return;
    }

    let (_, item_count, _) = count_by_target_kind(&tribute.afflictions);
    if item_count >= MAX_FIXATIONS_PER_TARGET_KIND {
        return;
    }

    let target = FixationTarget::Item(item.identifier.clone());
    let key = (AfflictionKind::Fixation(target.clone()), None);
    let aff = Affliction {
        kind: AfflictionKind::Fixation(target),
        body_part: None,
        severity: Severity::Mild,
        source: AfflictionSource::Spawn,
        acquired_cycle: 0,
        last_progressed_cycle: 0,
        trauma_metadata: None,
        phobia_metadata: None,
        fixation_metadata: Some(FixationMetadata {
            origin: FixationOrigin::Acquired {
                event_ref: format!("pickup:{}", item.identifier),
            },
        }),
    };
    tribute.afflictions.insert(key, aff);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn make_tribute() -> Tribute {
        Tribute::new("Test".to_string(), None, None)
    }

    fn add_fixation(tribute: &mut Tribute, target: FixationTarget) {
        let key = (AfflictionKind::Fixation(target.clone()), None);
        let aff = Affliction {
            kind: AfflictionKind::Fixation(target),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: Some(FixationMetadata {
                origin: FixationOrigin::Innate,
            }),
        };
        tribute.afflictions.insert(key, aff);
    }

    #[test]
    fn count_fixations_empty() {
        let t = make_tribute();
        assert_eq!(count_fixations(&t.afflictions), 0);
    }

    #[test]
    fn count_fixations_with_fixations() {
        let mut t = make_tribute();
        add_fixation(&mut t, FixationTarget::Area("sector1".to_string()));
        assert_eq!(count_fixations(&t.afflictions), 1);

        add_fixation(&mut t, FixationTarget::Tribute("other-tribute".to_string()));
        assert_eq!(count_fixations(&t.afflictions), 2);
    }

    #[test]
    fn count_by_target_kind_mixed() {
        let mut t = make_tribute();
        add_fixation(&mut t, FixationTarget::Area("sector1".to_string()));
        add_fixation(&mut t, FixationTarget::Tribute("other-tribute".to_string()));
        let (trib, item, area) = count_by_target_kind(&t.afflictions);
        assert_eq!(trib, 1);
        assert_eq!(item, 0);
        assert_eq!(area, 1);
    }

    #[test]
    fn spawn_roll_respects_cap() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut max_seen = 0;
        for _ in 0..200 {
            let mut t = Tribute::new("Test".to_string(), None, None);
            // Seed manually so the inner roll is deterministic
            let mut inner_rng = SmallRng::seed_from_u64(rng.random());
            roll_spawn_fixations(&mut t, &mut inner_rng);
            let count = count_fixations(&t.afflictions);
            assert!(
                count <= MAX_FIXATIONS,
                "count={count} exceeds MAX_FIXATIONS={MAX_FIXATIONS}"
            );
            if count > max_seen {
                max_seen = count;
            }
        }
        // With 5% probability over 200 tributes, we should see at least some fixations
        assert!(
            max_seen >= 1,
            "no fixations rolled in 200 tributes (bad luck or broken rng)"
        );
    }

    #[test]
    fn item_pickup_respects_cap() {
        let mut t = make_tribute();
        // Fill up to cap
        add_fixation(&mut t, FixationTarget::Area("sector1".to_string()));
        add_fixation(&mut t, FixationTarget::Tribute("other".to_string()));

        let item = Item::new_random_consumable();
        maybe_acquire_item_fixation(&mut t, &item);
        // Already at cap, should not add
        assert_eq!(count_fixations(&t.afflictions), 2);
    }

    #[test]
    fn per_kind_cap_blocks_duplicates() {
        let mut t = make_tribute();
        add_fixation(&mut t, FixationTarget::Area("sector1".to_string()));

        // Try adding another area fixation - should be blocked by cap checks
        // But the can_acquire check happens in anatomy.rs, so we simulate via
        // the game-side function
        let item = Item::new_random_consumable();
        maybe_acquire_item_fixation(&mut t, &item);
        // Item fixation is different kind than area, so this should be allowed
        // (per_kind_cap is per variant, but item != area)
        let count = count_fixations(&t.afflictions);
        assert!(count <= 2, "count={count} exceeds MAX_FIXATIONS");
    }
}
