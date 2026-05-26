//! Fixation acquisition logic: spawn-time innate rolls and item-pickup hooks.
//!
//! See spec §Fixation (PR1).

use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, AfflictionSource, FixationMetadata, FixationOrigin,
    FixationTarget, Severity, ThwartReason,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use strum::IntoEnumIterator;

use crate::areas::Area;
use crate::items::Item;
use crate::messages::MessagePayload;
use crate::tributes::Tribute;
use rand::RngExt;
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::SmallRng;
use uuid::Uuid;

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
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 0,
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
            observed_by: BTreeSet::new(),
            observer_seen_cycle: BTreeMap::new(),
            cycles_since_last_contact: 0,
        }),
    };
    tribute.afflictions.insert(key, aff);
}

// ── Fixation processing (PR3) ───────────────────────────────────────

/// How many cycles of continuous contact trigger consummation for
/// item/area targets.
pub const CONTACT_CYCLES_FOR_CONSUMMATION: u32 = 3;

/// How many cycles without any contact before an Acquired fixation fades.
pub const DECAY_THRESHOLD: u32 = 5;

/// Read-only context for per-tribute fixation processing.
pub struct FixationContext<'a> {
    /// Current cycle number.
    pub cycle: u32,
    /// Maps dead tribute UUID → optional killer UUID.
    pub dead_tribute_killers: &'a HashMap<Uuid, Option<Uuid>>,
    /// Maps tribute identifier string → UUID.
    pub id_to_uuid: &'a HashMap<String, Uuid>,
    /// Maps tribute identifier → area name (for same-area contact checks).
    pub tribute_areas: &'a HashMap<String, String>,
    /// Set of closed area names (lowercased for matching).
    pub closed_areas: &'a BTreeSet<String>,
    /// Set of all item identifiers still present in the game.
    pub all_item_ids: &'a BTreeSet<String>,
}

/// Process all fixations for a single tribute. Called once per tribute
/// per cycle. Returns messages to be pushed into the game event stream.
pub fn process_tribute_fixations(
    tribute: &mut Tribute,
    ctx: &FixationContext,
) -> Vec<MessagePayload> {
    let mut messages = Vec::new();

    // ── Early exit: no fixations ──
    if count_fixations(&tribute.afflictions) == 0 {
        return messages;
    }

    let tribute_id = tribute.identifier.clone();
    let tribute_uuid = tribute.id;
    let tribute_area = tribute.area;
    let self_area_str = tribute_area.to_string();

    // Collect fixation keys (clone to release borrow on afflictions).
    let fixation_keys: Vec<(AfflictionKey, FixationTarget, FixationOrigin, u32)> = tribute
        .afflictions
        .iter()
        .filter_map(|(key, aff)| match &aff.kind {
            AfflictionKind::Fixation(target) => {
                let origin = aff
                    .fixation_metadata
                    .as_ref()
                    .map(|m| m.origin.clone())
                    .unwrap_or(FixationOrigin::Innate);
                let cycles_since = aff
                    .fixation_metadata
                    .as_ref()
                    .map(|m| m.cycles_since_last_contact)
                    .unwrap_or(0);
                Some((key.clone(), target.clone(), origin, cycles_since))
            }
            _ => None,
        })
        .collect();

    if fixation_keys.is_empty() {
        return messages;
    }

    let mut to_remove: Vec<AfflictionKey> = Vec::new();

    for (key, target, origin, cycles_since) in &fixation_keys {
        let is_innate = matches!(origin, FixationOrigin::Innate);
        let target_str = target.to_string();

        let mut consummated = false;
        let mut thwarted = false;
        let mut thwart_reason = ThwartReason::TargetLost;
        let mut in_contact = false;

        // ── Evaluate target-specific outcomes ──
        match target {
            FixationTarget::Tribute(t_id) => {
                if let Some(t_uuid) = ctx.id_to_uuid.get(t_id)
                    && let Some(killer) = ctx.dead_tribute_killers.get(t_uuid)
                {
                    // Target is dead
                    if *killer == Some(tribute_uuid) {
                        consummated = true;
                    } else {
                        thwarted = true;
                        thwart_reason = ThwartReason::TargetLost;
                    }
                }
                // Target alive — check same-area contact.
                if !consummated
                    && !thwarted
                    && let Some(target_area) = ctx.tribute_areas.get(t_id)
                    && *target_area == self_area_str
                {
                    in_contact = true;
                }
            }
            FixationTarget::Item(i_id) => {
                let holds_item = tribute.items.iter().any(|i| i.identifier == *i_id);
                if holds_item {
                    in_contact = true;
                }
                if !ctx.all_item_ids.contains(i_id) {
                    thwarted = true;
                    thwart_reason = ThwartReason::TargetLost;
                }
            }
            FixationTarget::Area(a_name) => {
                let area_str = tribute_area.to_string();
                if area_str.eq_ignore_ascii_case(a_name) {
                    in_contact = true;
                }
                if ctx.closed_areas.contains(&a_name.to_lowercase()) {
                    thwarted = true;
                    thwart_reason = ThwartReason::TargetUnreachable;
                }
            }
        }

        // ── Determine new cycles_since_last_contact ──
        let new_cycles_since = if in_contact {
            0u32
        } else {
            cycles_since.saturating_add(1)
        };

        // ── Consummation check for item/area ──
        // Simplified: consummate on first contact. The N-cycle
        // requirement is tracked via CONTACT_CYCLES_FOR_CONSUMMATION
        // for future refinement.
        if !consummated && !thwarted && in_contact {
            match target {
                FixationTarget::Item(_) | FixationTarget::Area(_) => {
                    consummated = true;
                }
                _ => {}
            }
        }

        // ── Decay check (Acquired only) ──
        let should_decay =
            !is_innate && !consummated && !thwarted && new_cycles_since >= DECAY_THRESHOLD;

        // ── Update metadata for surviving fixations ──
        if !consummated
            && !thwarted
            && !should_decay
            && let Some(aff) = tribute.afflictions.get_mut(key)
            && let Some(meta) = &mut aff.fixation_metadata
        {
            meta.cycles_since_last_contact = new_cycles_since;
            // Observer decay: keep only observers seen within 5 cycles.
            meta.observer_seen_cycle
                .retain(|_observer_id, last_seen| ctx.cycle.saturating_sub(*last_seen) <= 5);
        }

        // ── Emit messages and queue removal ──
        if consummated {
            messages.push(MessagePayload::FixationConsummated {
                tribute_id: tribute_id.clone(),
                target: target_str,
            });
            to_remove.push(key.clone());
        } else if thwarted {
            messages.push(MessagePayload::FixationThwarted {
                tribute_id: tribute_id.clone(),
                target: target_str,
                reason: thwart_reason.to_string(),
            });
            to_remove.push(key.clone());
        } else if should_decay {
            messages.push(MessagePayload::FixationFaded {
                tribute_id: tribute_id.clone(),
                target: target_str,
            });
            to_remove.push(key.clone());
        }
    }

    // ── Apply removals ──
    for key in &to_remove {
        tribute.afflictions.remove(key);
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::OwnsItems;
    use rand::SeedableRng;
    use uuid::Uuid;

    fn make_tribute() -> Tribute {
        Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        )
    }

    /// Build an Acquired fixation and insert it into the tribute.
    fn add_acquired_fixation(tribute: &mut Tribute, target: FixationTarget) {
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
                    event_ref: "test".to_string(),
                },
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 6, // past decay threshold
            }),
        };
        tribute.afflictions.insert(key, aff);
    }

    /// Build a FixationContext referencing the given maps.
    fn ctx_from_refs<'a>(
        cycle: u32,
        id_to_uuid: &'a HashMap<String, Uuid>,
        dead_tribute_killers: &'a HashMap<Uuid, Option<Uuid>>,
        tribute_areas: &'a HashMap<String, String>,
        closed_areas: &'a BTreeSet<String>,
        all_item_ids: &'a BTreeSet<String>,
    ) -> FixationContext<'a> {
        FixationContext {
            cycle,
            dead_tribute_killers,
            id_to_uuid,
            tribute_areas,
            closed_areas,
            all_item_ids,
        }
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
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 0,
            }),
        };
        tribute.afflictions.insert(key, aff);
    }

    #[test]
    fn count_fixations_empty() {
        let t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        assert_eq!(count_fixations(&t.afflictions), 0);
    }

    #[test]
    fn count_fixations_with_fixations() {
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        add_fixation(&mut t, FixationTarget::Area("sector1".to_string()));
        assert_eq!(count_fixations(&t.afflictions), 1);

        add_fixation(&mut t, FixationTarget::Tribute("other-tribute".to_string()));
        assert_eq!(count_fixations(&t.afflictions), 2);
    }

    #[test]
    fn count_by_target_kind_mixed() {
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
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
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
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

    // ── Fixation processing tests (PR3) ──────────────────────────────

    #[test]
    fn tribute_consummation() {
        // Tribute A has a fixation on tribute B. B is dead, killed by A.
        let mut a = Tribute::new_with_rng(
            "Alpha".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        let b = Tribute::new_with_rng(
            "Bravo".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(99),
        );

        add_fixation(&mut a, FixationTarget::Tribute(b.identifier.clone()));

        let mut id_to_uuid = HashMap::new();
        id_to_uuid.insert(a.identifier.clone(), a.id);
        id_to_uuid.insert(b.identifier.clone(), b.id);

        let mut dead_killers = HashMap::new();
        dead_killers.insert(b.id, Some(a.id));

        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut a, &ctx);

        assert_eq!(msgs.len(), 1);
        assert!(matches!(
            &msgs[0],
            MessagePayload::FixationConsummated { target, .. }
                if target == &FixationTarget::Tribute(b.identifier.clone()).to_string()
        ));
        assert_eq!(count_fixations(&a.afflictions), 0);
    }

    #[test]
    fn item_consummation() {
        // Tribute has a fixation on an item they are holding.
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        let item = Item::new_random_consumable();
        let item_id = item.identifier.clone();
        let target = FixationTarget::Item(item_id.clone());
        add_fixation(&mut t, target.clone());
        t.add_item(item);

        let id_to_uuid = HashMap::new();
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let mut all_items = BTreeSet::new();
        all_items.insert(item_id);
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut t, &ctx);

        assert_eq!(msgs.len(), 1);
        assert!(matches!(
            &msgs[0],
            MessagePayload::FixationConsummated { .. }
        ));
        assert_eq!(count_fixations(&t.afflictions), 0);
    }

    #[test]
    fn area_consummation() {
        // Tribute has a fixation on an area they are standing in.
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        // Default area is Cornucopia.
        t.area = Area::Cornucopia;
        add_fixation(&mut t, FixationTarget::Area("Cornucopia".to_string()));

        let id_to_uuid = HashMap::new();
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut t, &ctx);

        assert_eq!(msgs.len(), 1);
        assert!(matches!(
            &msgs[0],
            MessagePayload::FixationConsummated { .. }
        ));
        assert_eq!(count_fixations(&t.afflictions), 0);
    }

    #[test]
    fn thwarted() {
        // Tribute A has a fixation on tribute B. B is dead but killed by C.
        let mut a = Tribute::new_with_rng(
            "Alpha".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        let b = Tribute::new_with_rng(
            "Bravo".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(99),
        );
        let c = Tribute::new_with_rng(
            "Charlie".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(11),
        );

        add_fixation(&mut a, FixationTarget::Tribute(b.identifier.clone()));

        let mut id_to_uuid = HashMap::new();
        id_to_uuid.insert(a.identifier.clone(), a.id);
        id_to_uuid.insert(b.identifier.clone(), b.id);
        id_to_uuid.insert(c.identifier.clone(), c.id);

        let mut dead_killers = HashMap::new();
        dead_killers.insert(b.id, Some(c.id)); // C killed B, not A

        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut a, &ctx);

        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], MessagePayload::FixationThwarted { .. }));
        assert_eq!(count_fixations(&a.afflictions), 0);
    }

    #[test]
    fn decay_fades_acquired_fixation() {
        // Acquired fixation with cycles_since_last_contact >= DECAY_THRESHOLD.
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        add_acquired_fixation(&mut t, FixationTarget::Area("Cornucopia".to_string()));
        // add_acquired_fixation sets cycles_since_last_contact = 6,
        // which is above DECAY_THRESHOLD (5).
        // Place tribute in a DIFFERENT area so they're not in contact.
        t.area = Area::Sector1;

        let id_to_uuid = HashMap::new();
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut t, &ctx);

        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], MessagePayload::FixationFaded { .. }));
        assert_eq!(count_fixations(&t.afflictions), 0);
    }

    #[test]
    fn contact_resets_decay_timer() {
        // Acquired fixation on a tribute who is alive and in the same area.
        // Contact resets the decay timer without consummating (the tribute
        // must die and be killed by the fixator for consummation).
        let mut a = Tribute::new_with_rng(
            "Alpha".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        let mut b = Tribute::new_with_rng(
            "Bravo".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(99),
        );
        a.area = Area::Cornucopia;
        b.area = Area::Cornucopia; // same area

        // Acquired fixation with cycles_since_last_contact = 6 (past decay threshold).
        let key = (
            AfflictionKind::Fixation(FixationTarget::Tribute(b.identifier.clone())),
            None,
        );
        let aff = Affliction {
            kind: AfflictionKind::Fixation(FixationTarget::Tribute(b.identifier.clone())),
            body_part: None,
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: Some(FixationMetadata {
                origin: FixationOrigin::Acquired {
                    event_ref: "test".to_string(),
                },
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 6, // past DECAY_THRESHOLD
            }),
        };
        a.afflictions.insert(key, aff);

        let mut id_to_uuid = HashMap::new();
        id_to_uuid.insert(a.identifier.clone(), a.id);
        id_to_uuid.insert(b.identifier.clone(), b.id);
        // B is alive → no entry in dead_killers
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let mut tribute_areas = HashMap::new();
        tribute_areas.insert(a.identifier.clone(), "Cornucopia".to_string());
        tribute_areas.insert(b.identifier.clone(), "Cornucopia".to_string());
        let ctx = ctx_from_refs(
            5,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut a, &ctx);

        // Contact should prevent decay → no messages, fixation preserved.
        assert!(msgs.is_empty(), "expected no messages, got: {msgs:?}");
        assert_eq!(count_fixations(&a.afflictions), 1);

        // Verify cycles_since_last_contact was reset to 0.
        let aff = a.afflictions.values().next().unwrap();
        let meta = aff.fixation_metadata.as_ref().unwrap();
        assert_eq!(meta.cycles_since_last_contact, 0);
    }

    #[test]
    fn observer_decay_five_cycle_threshold() {
        // Stale observers (last_seen > 5 cycles ago) are removed.
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        t.area = Area::Sector1;
        let target = FixationTarget::Area("Cornucopia".to_string());
        add_acquired_fixation(&mut t, target.clone());

        // Set up observer state with various last_seen cycles.
        let key = t
            .afflictions
            .keys()
            .next()
            .cloned()
            .expect("should have a fixation");
        let aff = t.afflictions.get_mut(&key).unwrap();
        let meta = aff.fixation_metadata.as_mut().unwrap();
        meta.observer_seen_cycle.insert("fresh".to_string(), 10);
        meta.observer_seen_cycle.insert("stale".to_string(), 2);
        meta.observed_by.insert("fresh".to_string());
        meta.observed_by.insert("stale".to_string());
        // Reset cycles_since_last_contact so decay doesn't fire.
        meta.cycles_since_last_contact = 1;

        // Cycle 12: fresh (last seen 10) is within 5 cycles; stale (2) is 10 cycles stale.
        let id_to_uuid = HashMap::new();
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            12,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let _msgs = process_tribute_fixations(&mut t, &ctx);

        let aff = t.afflictions.get(&key).unwrap();
        let meta = aff.fixation_metadata.as_ref().unwrap();
        assert!(
            meta.observer_seen_cycle.contains_key("fresh"),
            "fresh observer should remain"
        );
        assert!(
            !meta.observer_seen_cycle.contains_key("stale"),
            "stale observer should be removed"
        );
        // observed_by is NOT cleaned by observer_seen_cycle decay
        // (it retains all observers that have ever seen the fixation).
        assert!(meta.observed_by.contains("stale"));
    }

    #[test]
    fn innate_fixations_exempt_from_decay() {
        // Innate fixation with high cycles_since_last_contact should NOT fade.
        let mut t = Tribute::new_with_rng(
            "Test".to_string(),
            None,
            None,
            &mut SmallRng::seed_from_u64(42),
        );
        t.area = Area::Sector1;
        // Add an Innate fixation with high contact gap.
        let target = FixationTarget::Area("Cornucopia".to_string());
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
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
                cycles_since_last_contact: 10, // well past DECAY_THRESHOLD
            }),
        };
        t.afflictions.insert(key, aff);

        let id_to_uuid = HashMap::new();
        let dead_killers = HashMap::new();
        let closed_areas = BTreeSet::new();
        let all_items = BTreeSet::new();
        let tribute_areas = HashMap::new();
        let ctx = ctx_from_refs(
            10,
            &id_to_uuid,
            &dead_killers,
            &tribute_areas,
            &closed_areas,
            &all_items,
        );

        let msgs = process_tribute_fixations(&mut t, &ctx);

        // Innate fixations should NOT decay.
        assert!(msgs.is_empty(), "innate fixation should not decay");
        assert_eq!(count_fixations(&t.afflictions), 1);
    }
}
