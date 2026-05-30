//! PR2 integration tests: rescue action, attack-while-trapped, self-medicate gating.
//!
//! See `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md` §18.

use game::areas::AreaDetails;
use game::tributes::AfflictionDraft;
use game::tributes::Tribute;
use game::tributes::actions::Action;
use rand::prelude::*;
use rand::rngs::SmallRng;
use shared::afflictions::{AfflictionKind, AfflictionSource, Severity, TrapKind, TrappedMetadata};

fn make_tribute(name: &str) -> Tribute {
    let mut t = Tribute::new(name.into(), None, None);
    t.area = game::areas::Area::Cornucopia;
    t.attributes.strength = 30;
    t.attributes.defense = 15;
    t
}

fn add_buried(t: &mut Tribute, severity: Severity) {
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Trapped(TrapKind::Buried),
        body_part: None,
        severity,
        source: AfflictionSource::Environmental,
        trapped_metadata: Some(TrappedMetadata::fresh_for(TrapKind::Buried, None)),
    });
}

/// Test: defense halving when attacking a trapped target.
/// The target's defense is halved, making them more vulnerable.
#[test]
fn attack_while_trapped_defense_halved() {
    use game::tributes::combat::resolve::attack_contest;
    use game::tributes::combat_tuning::CombatTuning;

    let mut attacker = make_tribute("Attacker");
    attacker.attributes.strength = 50;
    let mut target = make_tribute("Target");
    target.attributes.defense = 20;
    add_buried(&mut target, Severity::Moderate);

    let mut rng = SmallRng::seed_from_u64(42);
    let mut events = Vec::new();
    let tuning = CombatTuning::default();

    let outcome = attack_contest(&mut attacker, &mut target, &mut rng, &mut events, &tuning);

    // The structural property: target's effective defense is halved.
    // With defense=20 (→10), attacker strength=50, the attacker should
    // almost always win. Check that the result is an attacker win.
    assert!(
        matches!(
            outcome.result,
            game::tributes::actions::AttackResult::AttackerWins
                | game::tributes::actions::AttackResult::AttackerWinsDecisively
                | game::tributes::actions::AttackResult::CriticalHit
        ),
        "expected attacker win vs trapped target with halved defense, got {:?}",
        outcome.result
    );
}

/// Test: rescue bonus computation works end-to-end via resolve_rescue.
#[test]
fn rescue_bonus_applied_to_trapped_target() {
    use game::tributes::rescue::resolve_rescue;

    let mut rescuer = make_tribute("Rescuer");
    rescuer.attributes.strength = 40;
    rescuer.area = game::areas::Area::Cornucopia;

    let mut target = make_tribute("Target");
    target.area = game::areas::Area::Cornucopia;
    add_buried(&mut target, Severity::Mild);

    let area = AreaDetails { area: Some(game::areas::Area::Cornucopia), ..Default::default() };

    let mut rng = SmallRng::seed_from_u64(0);
    let mut events = Vec::new();

    let resolved = resolve_rescue(&area, &rescuer, &mut target, &mut events, &mut rng);
    assert!(
        resolved,
        "rescue should resolve for co-located trapped target"
    );

    let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
    let meta = target
        .afflictions
        .get(&key)
        .and_then(|a| a.trapped_metadata.as_ref())
        .expect("trapped metadata should exist");
    assert!(
        meta.rescue_bonus_accumulated > 0.0,
        "rescue bonus should be accumulated, got {}",
        meta.rescue_bonus_accumulated
    );

    let has_event = events.iter().any(|e| {
        matches!(
            e.payload,
            game::messages::MessagePayload::RescueAttempted { .. }
        )
    });
    assert!(has_event, "expected RescueAttempted event");
}

/// Test: trapped tribute cannot move via hard gate.
#[test]
fn trapped_blocked_from_moving() {
    use game::areas::Area;
    use game::tributes::brains::affliction_override::hard_gates_with_terrain;

    let mut tribute = make_tribute("Trapped");
    add_buried(&mut tribute, Severity::Moderate);

    let result = hard_gates_with_terrain(&tribute, &Action::Move(Some(Area::Sector1)), None);
    assert_eq!(result, Some(Action::None));
}

/// Test: trapped tribute can still consume items (UseItem(None)).
#[test]
fn trapped_allows_consumable_use() {
    use game::tributes::brains::affliction_override::hard_gates_with_terrain;

    let mut tribute = make_tribute("Trapped");
    add_buried(&mut tribute, Severity::Moderate);

    let result = hard_gates_with_terrain(&tribute, &Action::UseItem(None), None);
    assert!(
        result.is_none(),
        "UseItem(None) should be allowed for trapped"
    );
}

/// Test: trapped tribute blocked from moving via affliction_override.
#[test]
fn trapped_override_blocks_move() {
    use game::tributes::brains::affliction_override::affliction_override;

    let mut tribute = make_tribute("Trapped");
    add_buried(&mut tribute, Severity::Moderate);

    let result = affliction_override(&tribute, &Action::Move(Some(game::areas::Area::Cornucopia)));
    assert_eq!(result, Some(Action::None));
}

/// Test: rescue bonus math matches spec.
#[test]
fn rescue_bonus_math() {
    use game::tributes::rescue::compute_rescue_bonus;

    let min_bonus = compute_rescue_bonus(0.0);
    assert!((min_bonus - 0.25).abs() < 1e-4);

    let max_bonus = compute_rescue_bonus(50.0);
    assert!((max_bonus - 0.55).abs() < 1e-4);

    let mid_bonus = compute_rescue_bonus(25.0);
    assert!((mid_bonus - 0.40).abs() < 1e-4);
}
