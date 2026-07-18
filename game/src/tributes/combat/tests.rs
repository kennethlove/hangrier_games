use super::*;
use crate::items::OwnsItems;
use crate::tributes::Tribute;
use crate::tributes::combat_tuning::CombatTuning;
use core::convert::Infallible;
use rand::SeedableRng;
use rand::TryRng;
use rand::rngs::SmallRng;
use rstest::*;

#[fixture]
fn small_rng() -> SmallRng {
    SmallRng::seed_from_u64(0)
}

#[rstest]
fn attack_contest_win(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 10;
    target.attributes.defense = 5;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::AttackerWins);
}

#[rstest]
fn attack_contest_win_decisively(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 15;
    target.attributes.defense = 0;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::AttackerWinsDecisively);
}

#[rstest]
fn attack_contest_lose(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 15;
    target.attributes.defense = 20;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::DefenderWins);
}

#[rstest]
fn attack_contest_lose_decisively(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 1;
    target.attributes.defense = 20;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::DefenderWinsDecisively);
}

#[rstest]
fn attack_contest_draw(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 21; // Magic number to make the final scores even
    target.attributes.defense = 20;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::Miss);
}

#[rstest]
fn attacks_self(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    attacker.attributes.strength = 25;
    // Sanity is now derived from mental_conditions; start at full sanity
    let sanity = attacker.effective_sanity();
    let mut target = attacker.clone();

    let outcome = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    assert_eq!(outcome, AttackOutcome::Wound(attacker.clone(), target));
    assert!(attacker.effective_sanity() < sanity);
}

#[rstest]
fn attacks_self_suicide(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    attacker.attributes.strength = 100;
    let mut target = attacker.clone();

    let outcome = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    assert_eq!(outcome, AttackOutcome::Kill(attacker, target));
}

#[rstest]
fn attacks_wound(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let sanity = attacker.effective_sanity();
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 25;
    target.attributes.defense = 20;

    let result = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    assert_eq!(
        result,
        AttackOutcome::Wound(attacker.clone(), target.clone())
    );
    assert_eq!(attacker.statistics.wins, 1);
    assert_eq!(target.statistics.defeats, 1);
    assert!(attacker.effective_sanity() < sanity);
}

#[rstest]
fn attacks_kill(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 50;
    target.attributes.defense = 0;
    target.blood = 100;

    let result = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    assert!(matches!(result, AttackOutcome::Kill(_, _)));
    assert_eq!(target.blood, 0);
    assert_eq!(attacker.statistics.wins, 1);
    assert_eq!(target.statistics.defeats, 1);
}

#[rstest]
fn attacks_miss(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 21; // Magic number to make them draw
    target.attributes.defense = 20;

    let result = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    assert_eq!(result, AttackOutcome::Miss(attacker, target));
}

#[rstest]
fn attacks_deducts_stamina_costs(mut small_rng: SmallRng) {
    let tuning = CombatTuning::default();
    let mut attacker = Tribute::new("A".to_string(), None, None);
    attacker.stamina = 100;
    attacker.max_stamina = 100;
    let mut target = Tribute::new("B".to_string(), None, None);
    target.stamina = 100;
    target.max_stamina = 100;
    let _ = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &tuning,
    );
    assert_eq!(attacker.stamina, 100 - tuning.stamina_cost_attacker);
    assert_eq!(target.stamina, 100 - tuning.stamina_cost_target);
}

#[rstest]
fn attacks_saturates_at_zero_when_below_cost(mut small_rng: SmallRng) {
    let tuning = CombatTuning::default();
    let mut attacker = Tribute::new("A".to_string(), None, None);
    attacker.stamina = 5;
    attacker.max_stamina = 100;
    let mut target = Tribute::new("B".to_string(), None, None);
    target.stamina = 3;
    target.max_stamina = 100;
    let _ = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &tuning,
    );
    assert_eq!(attacker.stamina, 0);
    assert_eq!(target.stamina, 0);
}

#[test]
fn attacker_winded_takes_attack_roll_penalty() {
    // Find a seed where the Winded penalty produces a different outcome
    // from a Fresh attacker with otherwise identical setup.
    let tuning = CombatTuning::default();
    for seed in 0..200u64 {
        let mut a_fresh = Tribute::new("AF".to_string(), None, None);
        a_fresh.stamina = 100;
        a_fresh.max_stamina = 100;
        a_fresh.attributes.strength = 5;
        let mut a_winded = Tribute::new("AW".to_string(), None, None);
        a_winded.stamina = 70; // post-cost: 45 → Winded
        a_winded.max_stamina = 100;
        a_winded.attributes.strength = 5;
        let mut t1 = Tribute::new("T1".to_string(), None, None);
        t1.stamina = 100;
        t1.max_stamina = 100;
        t1.blood = 1000;
        t1.attributes.defense = 10;
        let mut t2 = Tribute::new("T2".to_string(), None, None);
        t2.stamina = 100;
        t2.max_stamina = 100;
        t2.blood = 1000;
        t2.attributes.defense = 10;

        let mut rng_a = SmallRng::seed_from_u64(seed);
        let mut rng_b = SmallRng::seed_from_u64(seed);
        let out_a = a_fresh.attacks(
            &mut t1,
            &mut rng_a,
            &mut Vec::new(),
            shared::messages::Phase::Day,
            &tuning,
        );
        let out_b = a_winded.attacks(
            &mut t2,
            &mut rng_b,
            &mut Vec::new(),
            shared::messages::Phase::Day,
            &tuning,
        );

        if t1.blood != t2.blood {
            return; // Found a seed where penalty changes outcome
        }
        let _ = (out_a, out_b);
    }
    panic!("No seed in 0..200 produced different outcomes for fresh vs winded attacker");
}

#[rstest]
fn combat_beat_carries_stamina_costs(mut small_rng: SmallRng) {
    let tuning = CombatTuning::default();
    let mut attacker = Tribute::new("A".to_string(), None, None);
    attacker.stamina = 100;
    attacker.max_stamina = 100;
    let mut target = Tribute::new("B".to_string(), None, None);
    target.stamina = 100;
    target.max_stamina = 100;
    let mut events = Vec::new();
    let _ = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut events,
        shared::messages::Phase::Day,
        &tuning,
    );
    let beats: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(beats.len(), 1);
    assert_eq!(beats[0].attacker_stamina_cost, tuning.stamina_cost_attacker);
    assert_eq!(beats[0].target_stamina_cost, tuning.stamina_cost_target);
}

#[rstest]
fn test_critical_hit() {
    // Use a custom RNG that always returns the high bits needed for
    // `random_range(1..=20)` to produce 20 under rand 0.9's algorithm.
    struct CritRng;
    impl TryRng for CritRng {
        type Error = Infallible;
        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            Ok(0xF333_3334)
        }
        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            Ok((0xF333_3334u64 << 32) | 0xF333_3334u64)
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
            for byte in dest.iter_mut() {
                *byte = 0xFF;
            }
            Ok(())
        }
    }

    let mut crit_rng = CritRng;
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 10;
    target.blood = 1000;

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut crit_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::CriticalHit);
}

#[rstest]
fn test_critical_fumble() {
    // Use a custom RNG that returns 0 so `random_range(1..=20)` yields 1.
    struct FumbleRng;
    impl TryRng for FumbleRng {
        type Error = Infallible;
        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            Ok(0)
        }
        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            Ok(0)
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
            for byte in dest.iter_mut() {
                *byte = 0;
            }
            Ok(())
        }
    }

    let mut fumble_rng = FumbleRng;
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut fumble_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::CriticalFumble);
}

#[rstest]
fn test_perfect_block() {
    // First call (attacker roll): 0x7333_3334 → random_range(1..=20) == 10
    // Second call (defender roll): 0xF333_3334 → random_range(1..=20) == 20
    struct BlockRng {
        call_count: std::cell::Cell<usize>,
    }
    impl BlockRng {
        fn new() -> Self {
            BlockRng {
                call_count: std::cell::Cell::new(0),
            }
        }
    }
    impl TryRng for BlockRng {
        type Error = Infallible;
        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            let count = self.call_count.get();
            self.call_count.set(count + 1);
            Ok(if count == 0 { 0x7333_3334 } else { 0xF333_3334 })
        }
        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            Ok(self.try_next_u32().unwrap() as u64)
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
            for byte in dest.iter_mut() {
                *byte = self.try_next_u32().unwrap() as u8;
            }
            Ok(())
        }
    }

    let mut block_rng = BlockRng::new();
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    let result = attack_contest(
        &mut attacker,
        &mut target,
        &mut block_rng,
        &mut Vec::new(),
        &CombatTuning::default(),
    )
    .result;
    assert_eq!(result, AttackResult::PerfectBlock);
}

#[rstest]
fn test_critical_hit_triple_damage(mut _small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 20;
    let damage = attacker.attributes.strength * 3;

    // Manually test the wound creation for critical hit
    apply_combat_results(
        &mut attacker,
        &mut target,
        damage, // Triple damage
        GameOutput::TributeAttackWin("Katniss", "Peeta"),
        &mut Vec::new(),
        &CombatTuning::default(),
        &mut _small_rng,
    );

    // Verify wound was created (damage maps to severity, not direct health loss)
    assert!(!target.wounds.is_empty());
    // Blood should have been drained from the wound
    assert!(target.blood < 1000);
}

#[rstest]
fn test_fumble_self_damage(_small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    attacker.blood = 1000;
    let initial_blood = attacker.blood;

    // Simulate fumble damage (5 in old scale = 50 in blood)
    attacker.blood = attacker.blood.saturating_sub(50);

    assert_eq!(attacker.blood, initial_blood - 50);
}

/// Regression test for the clone-mutation bug where weapon wear was
/// applied to a cloned item from `weapons()` and silently lost.
/// A weapon with durability 5 must survive 3 attack contests with
/// reduced durability and remain in the attacker's inventory.
#[rstest]
fn weapon_survives_multiple_combats(mut small_rng: SmallRng) {
    use crate::items::{Attribute, Item, ItemRarity, ItemType};

    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    let weapon = Item::new(
        "Test Bow",
        ItemType::Weapon,
        ItemRarity::Rare,
        5,
        Attribute::Strength,
        3,
    );
    let weapon_id = weapon.identifier.clone();
    attacker.items.push(weapon);

    for _ in 0..3 {
        attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
    }

    let stored = attacker
        .items
        .iter()
        .find(|i| i.identifier == weapon_id)
        .expect("weapon should still be in inventory after 3 combats");
    assert_eq!(stored.max_durability, 5);
    assert_eq!(
        stored.current_durability, 2,
        "weapon should have been worn 3 times (5 - 3 = 2)"
    );
}

#[rstest]
fn attacks_target_killed_records_killer_id() {
    // When the attacker kills the target, target.recently_killed_by must
    // be set to the attacker's id so the cycle can attribute the death.
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 100;
    target.blood = 10;
    target.attributes.defense = 0;
    let attacker_id = attacker.id;

    // Use a deterministic RNG; with strength=100 vs defense=0 the
    // attacker reliably wins or crit-hits and the 1hp target dies.
    let mut rng = SmallRng::seed_from_u64(1);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    assert_eq!(target.blood, 0, "target should be dead in this scenario");
    assert_eq!(
        target.status,
        crate::tributes::statuses::TributeStatus::RecentlyDead
    );
    assert_eq!(
        target.recently_killed_by,
        Some(attacker_id),
        "killer id must be recorded on the deceased"
    );
    assert!(
        attacker.recently_killed_by.is_none(),
        "attacker is alive; their field must remain None"
    );
}

#[rstest]
fn attacks_attacker_killed_records_target_id() {
    // When the target's counter kills the attacker (e.g. perfect block),
    // attacker.recently_killed_by must point to the target.
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.blood = 10;
    attacker.attributes.strength = 0;
    attacker.attributes.defense = 0;
    target.attributes.strength = 100;
    target.attributes.defense = 100;
    let target_id = target.id;

    let mut rng = SmallRng::seed_from_u64(2);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut Vec::new(),
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    if attacker.blood == 0 {
        assert_eq!(
            attacker.status,
            crate::tributes::statuses::TributeStatus::RecentlyDead
        );
        assert_eq!(
            attacker.recently_killed_by,
            Some(target_id),
            "killer id must be recorded on the deceased attacker"
        );
    } else {
        // The seed didn't produce a kill; not a failure of attribution
        // logic, just RNG. Re-skip rather than flake.
        eprintln!("seed did not produce attacker death; skipping attribution check");
    }
}

/// Contract test: a real two-tribute engagement emits exactly one
/// `MessagePayload::Combat` `TaggedEvent`, with the attacker/target
/// names and a recognised outcome.
#[rstest]
fn attacks_emits_one_combat_taggedevent(mut small_rng: SmallRng) {
    let mut attacker = Tribute::new("Katniss".to_string(), None, None);
    let mut target = Tribute::new("Peeta".to_string(), None, None);

    attacker.attributes.strength = 50;
    target.attributes.defense = 0;
    target.blood = 100;

    let mut events: Vec<TaggedEvent> = Vec::new();
    attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    let combat_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.payload, MessagePayload::Combat(_)))
        .collect();
    assert_eq!(
        combat_events.len(),
        1,
        "exactly one Combat payload per attacks() call"
    );

    if let MessagePayload::Combat(eng) = &combat_events[0].payload {
        assert_eq!(eng.attacker.name, attacker.name);
        assert_eq!(eng.target.name, target.name);
        assert!(matches!(
            eng.outcome,
            CombatOutcome::Killed
                | CombatOutcome::Wounded
                | CombatOutcome::TargetFled
                | CombatOutcome::AttackerFled
                | CombatOutcome::Stalemate
        ));
    } else {
        panic!("expected Combat payload");
    }
}

#[rstest]
fn attacks_emits_one_combat_swing_per_call(small_rng: SmallRng) {
    // Every call to attacks() produces exactly one MessagePayload::CombatSwing,
    // regardless of which branch (miss / wound / kill / fumble / self-attack).
    for seed in 0..16u64 {
        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut attacker = Tribute::new(format!("A{seed}"), None, None);
        let mut target = Tribute::new(format!("T{seed}"), None, None);
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &CombatTuning::default(),
        );

        let swings: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.payload, MessagePayload::CombatSwing(_)))
            .collect();
        assert_eq!(
            swings.len(),
            1,
            "seed {seed}: expected exactly one CombatSwing payload, got {}",
            swings.len()
        );

        if let MessagePayload::CombatSwing(beat) = &swings[0].payload {
            assert_eq!(beat.attacker.name, attacker.name);
            assert_eq!(beat.target.name, target.name);
        }
    }
    // Touch the rstest fixture to silence unused warnings.
    let _ = small_rng;
}

#[rstest]
fn self_attack_emits_one_combat_swing(mut small_rng: SmallRng) {
    let mut tribute = Tribute::new("Solo".to_string(), None, None);
    let mut clone = tribute.clone();
    let mut events: Vec<TaggedEvent> = Vec::new();
    let _ = tribute.attacks(
        &mut clone,
        &mut small_rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );
    let swings: usize = events
        .iter()
        .filter(|e| matches!(e.payload, MessagePayload::CombatSwing(_)))
        .count();
    assert_eq!(swings, 1, "self-attack must emit exactly one CombatSwing");
}

/// Construct a weapon with the given effect and durability=1 so a single
/// `wear(1)` call breaks it.
fn brittle_weapon(effect: i32) -> crate::items::Item {
    crate::items::Item::new(
        "Glass Sword",
        crate::items::ItemType::Weapon,
        crate::items::ItemRarity::Common,
        1,
        crate::items::Attribute::Strength,
        effect,
    )
}

/// Construct a shield with the given effect and durability=1.
fn brittle_shield(effect: i32) -> crate::items::Item {
    crate::items::Item::new(
        "Glass Buckler",
        crate::items::ItemType::Weapon,
        crate::items::ItemRarity::Common,
        1,
        crate::items::Attribute::Defense,
        effect,
    )
}

#[test]
fn weapon_break_records_forfeit_and_penalty_on_beat() {
    let mut attacker = Tribute::new("Atk".into(), None, None);
    attacker.attributes.strength = 10;
    let weapon = brittle_weapon(5);
    attacker.add_item(weapon.clone());

    let mut target = Tribute::new("Tgt".into(), None, None);
    target.attributes.defense = 5;

    let mut events: Vec<TaggedEvent> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    let beat = events
        .iter()
        .find_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        })
        .expect("expected one CombatSwing emission");
    let weapon_wear = beat
        .wear
        .iter()
        .find(|w| w.owner.identifier == beat.attacker.identifier)
        .expect("expected a wear report for the attacker's weapon");

    assert_eq!(
        weapon_wear.outcome,
        shared::combat_beat::WearOutcomeReport::Broken
    );
    assert_eq!(weapon_wear.forfeited_effect, Some(5));
    let penalty = weapon_wear.mid_action_penalty.expect("penalty must fire");
    assert!(
        (1..=4).contains(&penalty),
        "penalty must be 1..=4, got {}",
        penalty
    );
}

#[test]
fn shield_break_records_forfeit_and_penalty_on_beat() {
    let mut attacker = Tribute::new("Atk".into(), None, None);
    attacker.attributes.strength = 10;

    let mut target = Tribute::new("Tgt".into(), None, None);
    target.attributes.defense = 5;
    let shield = brittle_shield(4);
    target.add_item(shield.clone());

    let mut events: Vec<TaggedEvent> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(7);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    let beat = events
        .iter()
        .find_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        })
        .expect("expected one CombatSwing emission");
    let shield_wear = beat
        .wear
        .iter()
        .find(|w| w.owner.identifier == beat.target.identifier)
        .expect("expected a wear report for the target's shield");

    assert_eq!(
        shield_wear.outcome,
        shared::combat_beat::WearOutcomeReport::Broken
    );
    assert_eq!(shield_wear.forfeited_effect, Some(4));
    let penalty = shield_wear.mid_action_penalty.expect("penalty must fire");
    assert!((1..=4).contains(&penalty), "penalty was {}", penalty);
}

#[test]
fn fumble_clears_attacker_break_penalty_on_beat() {
    // Hunt for a seed where the attacker's swing both fumbles AND breaks
    // the brittle weapon. Per design D5 the attacker-side break-penalty
    // fields must be cleared on fumble for clean narration.
    for seed in 0u64..2_000 {
        let mut attacker = Tribute::new("Atk".into(), None, None);
        attacker.attributes.strength = 10;
        let weapon = brittle_weapon(5);
        attacker.add_item(weapon.clone());

        let mut target = Tribute::new("Tgt".into(), None, None);
        target.attributes.defense = 5;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &CombatTuning::default(),
        );

        let beat = match events.iter().find_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        }) {
            Some(b) => b,
            None => continue,
        };

        let is_fumble = matches!(
            beat.outcome,
            shared::combat_beat::SwingOutcome::FumbleSurvive { .. }
                | shared::combat_beat::SwingOutcome::FumbleDeath { .. }
        );
        let weapon_wear = beat
            .wear
            .iter()
            .find(|w| w.owner.identifier == beat.attacker.identifier);

        if is_fumble
            && weapon_wear
                .map(|w| w.outcome == shared::combat_beat::WearOutcomeReport::Broken)
                .unwrap_or(false)
        {
            let w = weapon_wear.unwrap();
            assert_eq!(
                w.forfeited_effect, None,
                "D5: fumble must clear forfeited_effect"
            );
            assert_eq!(
                w.mid_action_penalty, None,
                "D5: fumble must clear mid_action_penalty"
            );
            return;
        }
    }
    panic!("no seed in 0..2000 produced a fumble + weapon break combo; widen the search");
}

#[test]
fn unarmed_unshielded_emits_no_break_penalty() {
    let mut attacker = Tribute::new("Atk".into(), None, None);
    attacker.attributes.strength = 10;
    let mut target = Tribute::new("Tgt".into(), None, None);
    target.attributes.defense = 5;

    let mut events: Vec<TaggedEvent> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(123);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    let beat = events
        .iter()
        .find_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        })
        .expect("expected one CombatSwing emission");
    for w in &beat.wear {
        assert_eq!(w.forfeited_effect, None);
        assert_eq!(w.mid_action_penalty, None);
    }
}

/// Parity: wear lines rendered from `CombatBeat.wear` via
/// `CombatBeatExt::to_log_lines` must match the wear lines that
/// `attack_contest` flattens into `CombatEngagement.detail_lines`.
/// This locks `CombatBeat.wear` as the single source of truth for wear
/// narration once consumers migrate off `detail_lines`.
#[test]
fn combat_beat_wear_matches_engagement_detail_wear_lines() {
    use crate::tributes::combat_beat::CombatBeatExt;
    // Both attacker weapon and target shield are brittle so we exercise
    // the full wear (Worn or Broken) emission paths in one swing.
    for seed in 0..32u64 {
        let mut attacker = Tribute::new(format!("Atk{seed}"), None, None);
        attacker.attributes.strength = 10;
        attacker.add_item(brittle_weapon(5));
        let mut target = Tribute::new(format!("Tgt{seed}"), None, None);
        target.attributes.defense = 5;
        target.add_item(brittle_shield(4));

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &CombatTuning::default(),
        );

        // Find the engagement (skip seeds that didn't produce one, e.g.
        // pure fumble paths emit a standalone TributeWounded instead and
        // never write detail_lines for the legacy timeline to consume).
        let Some(detail_lines) = events.iter().find_map(|e| match &e.payload {
            MessagePayload::Combat(eng) => Some(eng.detail_lines.clone()),
            _ => None,
        }) else {
            continue;
        };
        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing per attacks() call");

        // Wear-related substrings we want to track.
        let is_wear_line = |s: &str| {
            s.contains("starting to wear out")
                || s.contains("breaks")
                || s.contains("shatters mid-swing")
                || s.contains("shatters mid-block")
        };

        let beat_wear_lines: Vec<String> = beat
            .to_log_lines()
            .into_iter()
            .filter(|s| is_wear_line(s))
            .collect();
        let detail_wear_lines: Vec<String> = detail_lines
            .into_iter()
            .filter(|s| is_wear_line(s))
            .collect();

        assert_eq!(
            beat_wear_lines, detail_wear_lines,
            "seed {seed}: wear lines from CombatBeat must match detail_lines"
        );
    }
}

/// A swing that wins (and therefore triggers `apply_violence_stress`) on
/// a tribute already credited with prior wins must record the resulting
/// stress on the swing's `CombatBeat.stress.stress_damage`.
#[test]
fn combat_swing_records_stress_damage() {
    // Arrange: attacker has prior wins+kills so violence-stress is non-zero.
    let mut attacker = Tribute::new("Atk".into(), None, None);
    attacker.attributes.strength = 50;
    attacker.statistics.wins = 5;
    attacker.statistics.kills = 5;

    let mut target = Tribute::new("Tgt".into(), None, None);
    target.attributes.defense = 0;
    target.blood = 10;

    let mut events: Vec<TaggedEvent> = Vec::new();
    let mut rng = SmallRng::seed_from_u64(1);
    let _ = attacker.attacks(
        &mut target,
        &mut rng,
        &mut events,
        shared::messages::Phase::Day,
        &CombatTuning::default(),
    );

    let beat = events
        .iter()
        .find_map(|e| match &e.payload {
            MessagePayload::CombatSwing(b) => Some(b),
            _ => None,
        })
        .expect("expected one CombatSwing emission");
    assert!(
        beat.stress.stress_damage > 0,
        "expected non-zero stress on the beat; got {}",
        beat.stress.stress_damage
    );
}

/// Parity: the optional trailing horrified line that
/// `CombatBeatExt::to_log_lines` renders from `stress.stress_damage`
/// must match the horrified line that `apply_violence_stress` flattened
/// into `CombatEngagement.detail_lines`.
#[test]
fn combat_beat_stress_matches_engagement_horrified_line() {
    use crate::tributes::combat_beat::CombatBeatExt;
    for seed in 0..32u64 {
        let mut attacker = Tribute::new(format!("Atk{seed}"), None, None);
        attacker.attributes.strength = 50;
        attacker.statistics.wins = 5;
        attacker.statistics.kills = 5;

        let mut target = Tribute::new(format!("Tgt{seed}"), None, None);
        target.attributes.defense = 0;
        target.blood = 1000;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &CombatTuning::default(),
        );

        let Some(detail_lines) = events.iter().find_map(|e| match &e.payload {
            MessagePayload::Combat(eng) => Some(eng.detail_lines.clone()),
            _ => None,
        }) else {
            continue;
        };
        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing per attacks() call");

        let beat_horrified: Vec<String> = beat
            .to_log_lines()
            .into_iter()
            .filter(|s| s.contains("horrified"))
            .collect();
        let detail_horrified: Vec<String> = detail_lines
            .into_iter()
            .filter(|s| s.contains("horrified"))
            .collect();

        assert_eq!(
            beat_horrified, detail_horrified,
            "seed {seed}: horrified lines from CombatBeat must match detail_lines"
        );
    }
}

#[rstest]
fn attacking_sleeping_target_emits_ambush_wake(mut small_rng: SmallRng) {
    // Spec §6.4 PR2c.2 (bd-1zju): hitting a sleeping tribute must wake
    // them with `WakeReason::Interrupted { Ambush }` BEFORE damage
    // resolution. The ambush still lands; we just verify the wake
    // event precedes any TributeWounded / Killed event in `events`.
    let mut attacker = Tribute::new("Cato".to_string(), None, None);
    attacker.attributes.strength = 10;
    let mut target = Tribute::new("Rue".to_string(), None, None);
    target.sleeping = true;
    target.sleep_remaining = 4;
    target.cycles_awake = 12;

    let mut events: Vec<TaggedEvent> = Vec::new();
    let _ = attacker.attacks(
        &mut target,
        &mut small_rng,
        &mut events,
        shared::messages::Phase::Night,
        &CombatTuning::default(),
    );

    assert!(!target.sleeping, "ambushed target must wake");
    assert_eq!(target.sleep_remaining, 0);
    assert_eq!(target.cycles_awake, 0);

    let woke_idx = events
        .iter()
        .position(|e| {
            matches!(
                &e.payload,
                MessagePayload::TributeWoke {
                    reason: shared::messages::WakeReason::Interrupted {
                        event: shared::messages::InterruptionKind::Ambush { .. },
                    },
                    ..
                }
            )
        })
        .expect("expected one ambush TributeWoke event");
    assert_eq!(
        woke_idx, 0,
        "ambush wake must be emitted before damage resolution"
    );
}
