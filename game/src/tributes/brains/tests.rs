use super::*;
use crate::items::Item;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use rand::prelude::*;
use rstest::{fixture, rstest};
use shared::messages::Phase;

#[fixture]
fn tribute() -> Tribute {
    // Build a fully-deterministic Tribute for AI tests:
    //   - Brain::default() = Balanced personality with seed=0 thresholds
    //     (low_health=18, mid_health=38, low_sanity=16, mid_sanity=34,
    //     low_movement=8, high_intelligence=40, low_intelligence=91)
    //   - Attributes::default() = maxed-out attributes
    // Tribute::new() randomizes both, so override after construction.
    let mut tribute = Tribute::new("Katniss".to_string(), None, None);
    tribute.brain = Brain::default();
    tribute.attributes = crate::tributes::Attributes::default();
    // Fixed identifier ensures hash-based trap check is deterministic.
    // "safe-test-id" sums to 1243, not divisible by 7 — never triggers trap.
    tribute.identifier = "safe-test-id".to_string();
    tribute
}

#[fixture]
fn small_rng() -> SmallRng {
    // Use a fixed seed so brain decision tests are deterministic.
    // Otherwise low-probability branches (e.g. wants_to_propose_alliance
    // at ~5%) cause occasional CI flakes.
    SmallRng::seed_from_u64(0xA11CE5EED)
}

#[rstest]
fn decide_on_action_default(tribute: Tribute, mut small_rng: SmallRng) {
    // If there are no enemies nearby, the tribute should move
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Move(None));
}

#[rstest]
fn decide_on_action_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
    // If the tribute has low health, they should rest
    tribute.attributes.set_health(10);
    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Move(None));
}

#[rstest]
fn decide_on_action_no_health(mut tribute: Tribute, mut small_rng: SmallRng) {
    // If the tribute has no health, they should do nothing
    tribute.attributes.set_health(0);
    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::None);
}

#[rstest]
fn decide_on_action_no_movement_alone(mut tribute: Tribute, mut small_rng: SmallRng) {
    // If the tribute has no movement and is alone, they should rest
    tribute.attributes.movement = 0;
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Rest);
}

#[rstest]
fn decide_on_action_no_movement_surrounded_low_health(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    // If the tribute has no movement and is not alone, they should hide
    tribute.attributes.movement = 1;
    tribute.attributes.set_health(10);
    let action = tribute.brain.act(
        &tribute.clone(),
        5,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Hide);
}

#[rstest]
fn decide_on_action_enemies(tribute: Tribute, mut small_rng: SmallRng) {
    // If there are enemies nearby, the tribute should attack
    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn decide_on_action_enemies_medium_health(mut tribute: Tribute, mut small_rng: SmallRng) {
    // If there are enemies nearby, but the tribute is low on health
    // the tribute should hide
    tribute.attributes.set_health(20);
    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Move(None));
}

#[rstest]
fn decide_on_action_preferred_action(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.brain.set_preferred_action(Action::Rest, 1.0);
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Rest);
}

#[rstest]
fn clear_preferred_action(mut tribute: Tribute) {
    tribute.brain.set_preferred_action(Action::Rest, 1.0);
    assert_eq!(tribute.brain.preferred_action, Some(Action::Rest));
    assert_eq!(tribute.brain.preferred_action_percentage, 1.0);

    tribute.brain.clear_preferred_action();
    assert_eq!(tribute.brain.preferred_action, None);
    assert_eq!(tribute.brain.preferred_action_percentage, 0.0);
}

#[rstest]
fn prefer_to_use_item_if_available(mut tribute: Tribute, mut small_rng: SmallRng) {
    let item = Item::new_random_consumable();
    tribute.items.push(item.clone());
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::UseItem(None));
}

#[rstest]
fn prefer_to_hide_at_mid_health_and_visible(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.attributes.set_health(25);
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Hide);
}

#[rstest]
fn prefer_to_move_at_mid_health_and_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.attributes.set_health(25);
    tribute.attributes.set_sanity(15);
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Move(None));
}

#[rstest]
fn decide_on_action_alone_healthy_no_movement(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.attributes.movement = 0;
    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Rest);
}

#[rstest]
fn decide_on_action_surrounded_low_health_low_movement_low_sanity(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    tribute.attributes.set_health(10);
    tribute.attributes.movement = 0;
    tribute.attributes.set_sanity(15);
    let action = tribute.brain.act(
        &tribute.clone(),
        3,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn decide_on_action_surrounded_low_health_low_sanity(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    tribute.attributes.set_health(15);
    tribute.attributes.set_sanity(10);
    let action = tribute.brain.act(
        &tribute.clone(),
        3,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn decide_on_action_surrounded_hidden_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.attributes.is_hidden = true;
    tribute.attributes.set_health(10);
    let action = tribute.brain.act(
        &tribute.clone(),
        3,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::None);
}

#[rstest]
fn decide_on_action_surrounded_ok_health_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
    tribute.attributes.set_health(25);
    tribute.attributes.set_sanity(15);
    let action = tribute.brain.act(
        &tribute.clone(),
        3,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn decide_on_action_heavily_surrounded_normal_sanity_and_intelligence(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    tribute.attributes.intelligence = 50;
    tribute.attributes.set_sanity(50);
    let action = tribute.brain.act(
        &tribute.clone(),
        6,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Move(None));
}

#[rstest]
fn decide_on_action_heavily_surrounded_low_sanity_and_intelligence(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    tribute.attributes.intelligence = 20;
    tribute.attributes.set_sanity(20);
    let action = tribute.brain.act(
        &tribute.clone(),
        6,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Hide);
}

#[rstest]
fn decide_on_action_heavily_surrounded_no_sanity_and_intelligence(
    mut tribute: Tribute,
    mut small_rng: SmallRng,
) {
    // recklessness = 100 - intelligence - sanity must reach low_intelligence
    // threshold (~91 for Balanced after variance) for the Attack branch.
    tribute.attributes.intelligence = 5;
    tribute.attributes.set_sanity(0);
    let action = tribute.brain.act(
        &tribute.clone(),
        6,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn test_psychotic_break_triggers_at_low_sanity(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.attributes.set_sanity(3); // Below typical break threshold

    tribute
        .brain
        .check_psychotic_break(tribute.attributes.sanity(), &mut small_rng);

    assert!(tribute.brain.psychotic_break.is_some());
}

#[rstest]
fn test_psychotic_break_doesnt_trigger_at_normal_sanity(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.attributes.set_sanity(50); // Well above break threshold

    tribute
        .brain
        .check_psychotic_break(tribute.attributes.sanity(), &mut small_rng);

    assert!(tribute.brain.psychotic_break.is_none());
}

#[rstest]
fn test_psychotic_break_recovery(mut small_rng: SmallRng) {
    // Use deterministic Brain so psychotic_break_threshold is fixed
    // (Balanced base = 7) and the +20 recovery margin is predictable.
    let mut tribute = Tribute {
        brain: Brain::default(),
        ..Tribute::default()
    };
    tribute.attributes.set_sanity(3);

    // Trigger break
    tribute
        .brain
        .check_psychotic_break(tribute.attributes.sanity(), &mut small_rng);
    assert!(tribute.brain.psychotic_break.is_some());

    // Sanity recovers significantly (needs to be 20+ above threshold)
    tribute.attributes.set_sanity(30);
    tribute.brain.check_recovery(tribute.attributes.sanity());

    assert!(tribute.brain.psychotic_break.is_none());
}

#[rstest]
fn test_psychotic_break_no_recovery_insufficient_sanity(mut small_rng: SmallRng) {
    // Deterministic Brain: Balanced psychotic_break_threshold = 7,
    // recovery requires sanity >= 27. sanity = 15 is below that.
    let mut tribute = Tribute {
        brain: Brain::default(),
        ..Tribute::default()
    };
    tribute.attributes.set_sanity(3);

    // Trigger break
    tribute
        .brain
        .check_psychotic_break(tribute.attributes.sanity(), &mut small_rng);
    assert!(tribute.brain.psychotic_break.is_some());

    // Sanity recovers but not enough
    tribute.attributes.set_sanity(15);
    tribute.brain.check_recovery(tribute.attributes.sanity());

    // Should still be broken
    assert!(tribute.brain.psychotic_break.is_some());
}

#[rstest]
fn test_berserk_break_attacks(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.brain.psychotic_break = Some(PsychoticBreakType::Berserk);

    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn test_paranoid_break_hides(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.brain.psychotic_break = Some(PsychoticBreakType::Paranoid);

    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::Hide);
}

#[rstest]
fn test_catatonic_break_does_nothing(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.brain.psychotic_break = Some(PsychoticBreakType::Catatonic);

    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    assert_eq!(action, Action::None);
}

#[rstest]
fn test_self_destructive_break_attacks(mut small_rng: SmallRng) {
    let mut tribute = Tribute::default();
    tribute.brain.psychotic_break = Some(PsychoticBreakType::SelfDestructive);
    tribute.attributes.set_health(5); // Very low health - normally would rest/hide

    let action = tribute.brain.act(
        &tribute.clone(),
        2,
        &[],
        &[],
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );
    // Self-destructive ignores health and attacks
    assert_eq!(action, Action::Attack);
}

#[rstest]
fn from_traits_empty_uses_balanced_baseline() {
    // With no traits and zero variance, thresholds collapse to the
    // documented baseline values (the original `Balanced` numbers).
    let mut rng = SmallRng::seed_from_u64(101);
    let thresholds = PersonalityThresholds::from_traits(&[], &mut rng);
    // Each base ±20% — assert each lies in the expected window.
    assert!(
        (16..=24).contains(&thresholds.low_health),
        "low_health={}",
        thresholds.low_health
    );
    assert!((32..=48).contains(&thresholds.mid_health));
    assert!((8..=12).contains(&thresholds.extreme_low_sanity));
    assert!((16..=24).contains(&thresholds.low_sanity));
    assert!((28..=42).contains(&thresholds.mid_sanity));
    assert!((8..=12).contains(&thresholds.low_movement));
    assert!((28..=42).contains(&thresholds.high_intelligence));
    assert!((64..=96).contains(&thresholds.low_intelligence));
    assert!((6..=10).contains(&thresholds.psychotic_break_threshold));
}

#[rstest]
fn from_traits_aggressive_lowers_health_thresholds() {
    // Aggressive: low_health -5 (→15), mid_health -10 (→30) before variance.
    // Use many seeds and check the mean is shifted below baseline.
    let aggressive = vec![Trait::Aggressive];
    let mut total_low: u32 = 0;
    let mut total_mid: u32 = 0;
    for seed in 0..50 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let t = PersonalityThresholds::from_traits(&aggressive, &mut rng);
        total_low += t.low_health;
        total_mid += t.mid_health;
    }
    // Mean low_health ≈ 15, mean mid_health ≈ 30.
    let mean_low = total_low / 50;
    let mean_mid = total_mid / 50;
    assert!((12..=18).contains(&mean_low), "mean_low={}", mean_low);
    assert!((25..=35).contains(&mean_mid), "mean_mid={}", mean_mid);
}

#[rstest]
fn from_traits_clamps_to_minimum_one() {
    // Stack many sanity-lowering traits to push past the clamp boundary.
    let traits = vec![Trait::Reckless, Trait::Aggressive];
    let mut rng = SmallRng::seed_from_u64(17);
    let t = PersonalityThresholds::from_traits(&traits, &mut rng);
    assert!(t.extreme_low_sanity >= 1);
    assert!(t.low_health >= 1);
}

/// 8pq: when scoring picks a non-neighbor goal, brain.act should return the
/// *first hop* of the planned path (not the goal itself).
/// Goal Sector4 from Sector1 must route via Cornucopia.
#[rstest]
fn brain_act_routes_first_hop_to_non_neighbor_goal(mut tribute: Tribute, mut small_rng: SmallRng) {
    use crate::areas::Area;

    // Place the tribute in Sector1.
    tribute.area = Area::Sector1;
    // Avoid the items branch & alliance branch — strip both.
    tribute.items.clear();
    tribute.brain.preferred_action = None;

    // Build a 7-area world. Make Sector4 carry the tribute's terrain
    // affinity so choose_destination prefers it; everything else is
    // plain Clearing so neighbor scoring is uniform.
    let mk = |a: Area, base: BaseTerrain| {
        AreaDetails::new_with_terrain(
            Some(format!("{a:?}")),
            a,
            TerrainType::new(base, vec![]).unwrap(),
        )
    };
    // Give the tribute a Desert affinity, then make Sector4 a Desert.
    tribute.terrain_affinity = vec![BaseTerrain::Desert];
    let all_areas = vec![
        mk(Area::Cornucopia, BaseTerrain::Clearing),
        mk(Area::Sector1, BaseTerrain::Clearing),
        mk(Area::Sector2, BaseTerrain::Clearing),
        mk(Area::Sector3, BaseTerrain::Clearing),
        mk(Area::Sector4, BaseTerrain::Desert),
        mk(Area::Sector5, BaseTerrain::Clearing),
        mk(Area::Sector6, BaseTerrain::Clearing),
    ];

    let action = tribute.brain.act(
        &tribute.clone(),
        0,
        &[],
        &all_areas,
        &[],
        &HashMap::new(),
        Phase::Day,
        &mut small_rng,
    );

    match action {
        Action::Move(Some(first_hop)) => {
            // Sector1's clockwise neighbors are Sector2 and Cornucopia.
            // The shortest path to Sector4 goes Sector1 -> Cornucopia
            // -> Sector4, so the first hop must be Cornucopia.
            assert_eq!(
                first_hop,
                Area::Cornucopia,
                "expected first hop toward Sector4 to be Cornucopia, got {first_hop:?}"
            );
        }
        other => panic!("expected Move(Some(_)), got {other:?}"),
    }
}

/// hangrier_games-4wnj — When two areas score identically on terrain
/// signals but one is empty and the other holds enemies, the crowd
/// penalty in `choose_destination` must steer the tribute toward the
/// empty one. Excludes the tribute itself from its own area's count.
#[rstest]
fn choose_destination_avoids_crowded_areas(tribute: Tribute) {
    use crate::areas::Area;

    let mk = |a: Area| {
        AreaDetails::new_with_terrain(
            Some(format!("{a:?}")),
            a,
            TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
        )
    };
    let areas = vec![mk(Area::Sector1), mk(Area::Sector2)];

    let mut density = HashMap::new();
    density.insert(Area::Sector1, 4); // crowded
    density.insert(Area::Sector2, 0); // empty

    let chosen = tribute
        .brain
        .choose_destination(&areas, &tribute, &density)
        .expect("expected a destination");
    assert_eq!(
        chosen,
        Area::Sector2,
        "should pick empty Sector2 over crowded Sector1"
    );
}

/// The tribute's own area excludes itself from the crowd penalty so a
/// solo tribute does not penalize staying put. Equal "others" counts
/// (0 vs 0) tie-break to scoring order, but the home area must not
/// score *worse* than an equally-empty foreign one.
#[rstest]
fn choose_destination_excludes_self_from_own_area_density(tribute: Tribute) {
    use crate::areas::Area;

    let mut t = tribute;
    t.area = Area::Sector1;

    let mk = |a: Area| {
        AreaDetails::new_with_terrain(
            Some(format!("{a:?}")),
            a,
            TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
        )
    };
    let areas = vec![mk(Area::Sector1), mk(Area::Sector2)];

    let mut density = HashMap::new();
    density.insert(Area::Sector1, 1); // just the tribute itself
    density.insert(Area::Sector2, 0);

    let chosen = t
        .brain
        .choose_destination(&areas, &t, &density)
        .expect("expected a destination");
    // Tie on "others" (0 vs 0): scoring order picks the first area.
    assert_eq!(chosen, Area::Sector1);
}

// ---- Sleep gating (PR2c.1, bd-9sjj) ----

#[rstest]
fn should_sleep_dominant_threshold_overrides_safety(tribute: Tribute, mut small_rng: SmallRng) {
    use shared::messages::Phase;
    let mut t = tribute.clone();
    t.cycles_awake = SLEEP_DOMINANT_THRESHOLD;
    let action = t.brain.should_sleep(&t, 5, Phase::Day, &mut small_rng);
    assert!(matches!(action, Some(Action::Sleep { duration_phases: 4 })));
}

#[rstest]
fn should_sleep_want_threshold_requires_safety_and_night(
    tribute: Tribute,
    mut small_rng: SmallRng,
) {
    use shared::messages::Phase;
    let mut t = tribute.clone();
    t.cycles_awake = SLEEP_WANT_THRESHOLD;
    let action = t.brain.should_sleep(&t, 0, Phase::Night, &mut small_rng);
    assert!(matches!(action, Some(Action::Sleep { duration_phases: 3 })));
    let action = t.brain.should_sleep(&t, 1, Phase::Night, &mut small_rng);
    assert!(action.is_none());
    let action = t.brain.should_sleep(&t, 0, Phase::Day, &mut small_rng);
    assert!(action.is_none());
}

#[rstest]
fn should_sleep_exhausted_naps_when_safe_off_day(tribute: Tribute, mut small_rng: SmallRng) {
    use shared::messages::Phase;
    let mut t = tribute.clone();
    t.cycles_awake = 1;
    t.stamina = 10;
    let action = t.brain.should_sleep(&t, 0, Phase::Dusk, &mut small_rng);
    assert!(matches!(action, Some(Action::Sleep { duration_phases: 2 })));
    let action = t.brain.should_sleep(&t, 0, Phase::Day, &mut small_rng);
    assert!(action.is_none());
}

#[rstest]
fn should_sleep_psychotic_break_blocks_sleep(tribute: Tribute, mut small_rng: SmallRng) {
    use shared::messages::Phase;
    let mut t = tribute.clone();
    t.cycles_awake = SLEEP_DOMINANT_THRESHOLD + 4;
    t.brain.psychotic_break = Some(PsychoticBreakType::Berserk);
    let action = t.brain.should_sleep(&t, 0, Phase::Night, &mut small_rng);
    assert!(action.is_none());
}

#[rstest]
fn should_sleep_already_sleeping_returns_none(tribute: Tribute, mut small_rng: SmallRng) {
    use shared::messages::Phase;
    let mut t = tribute.clone();
    t.sleeping = true;
    t.sleep_remaining = 2;
    t.cycles_awake = SLEEP_DOMINANT_THRESHOLD + 10;
    let action = t.brain.should_sleep(&t, 0, Phase::Night, &mut small_rng);
    assert!(action.is_none());
}

// ---- Survival override tests ----

pub(crate) mod survival_override_tests {
    use super::*;
    use crate::areas::weather::Weather;
    use crate::items::Item;
    use crate::terrain::BaseTerrain;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;

    #[test]
    fn override_dehydrated_at_water_terrain_picks_drink() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.thirst = 3;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, Some(Action::DrinkFromTerrain));
    }

    #[test]
    fn override_dehydrated_with_water_item_picks_drink_item() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.thirst = 3;
        t.items.push(Item::new_water(None, 2));
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::DrinkItem(_))));
    }

    #[test]
    fn override_starving_with_food_picks_eat() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        t.items.push(Item::new_food(None, 3));
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::Eat(_))));
    }

    #[test]
    fn override_starving_at_forageable_terrain_no_inventory_picks_forage() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, Some(Action::Forage));
    }

    #[test]
    fn override_starving_in_combat_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, true);
        assert_eq!(action, None);
    }

    #[test]
    fn override_hungry_not_starving_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 3;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, None);
    }

    mod stamina {
        use super::*;
        use crate::tributes::combat_tuning::CombatTuning;

        fn make(name: &str, stamina: u32) -> Tribute {
            let mut t = Tribute::new(name.to_string(), None, None);
            t.brain = Brain::default();
            t.attributes = crate::tributes::Attributes::default();
            t.stamina = stamina;
            t.max_stamina = 100;
            t
        }

        #[test]
        fn fresh_returns_none() {
            let t = make("F", 100);
            let result = stamina_override(&t, &[], false, &CombatTuning::default());
            assert_eq!(result, None);
        }

        #[test]
        fn winded_returns_none() {
            let t = make("W", 30);
            let result = stamina_override(&t, &[], false, &CombatTuning::default());
            assert_eq!(result, None);
        }

        #[test]
        fn exhausted_in_shelter_returns_rest() {
            let t = make("E", 10);
            let result = stamina_override(&t, &[], true, &CombatTuning::default());
            assert_eq!(result, Some(Action::Rest));
        }

        #[test]
        fn exhausted_no_shelter_no_threats_rests() {
            let t = make("E", 10);
            let result = stamina_override(&t, &[], false, &CombatTuning::default());
            assert_eq!(result, Some(Action::Rest));
        }

        #[test]
        fn exhausted_with_fresh_threat_flees() {
            let actor = make("E", 10);
            let threat = make("Hunter", 100);
            let result = stamina_override(&actor, &[threat], false, &CombatTuning::default());
            assert!(matches!(result, Some(Action::Move(_))));
        }

        #[test]
        fn exhausted_self_does_not_count_as_threat() {
            // Same identifier as actor → not a threat.
            let mut actor = make("E", 10);
            actor.identifier = "self".to_string();
            let mut clone = make("E", 100);
            clone.identifier = "self".to_string();
            let result = stamina_override(&actor, &[clone], false, &CombatTuning::default());
            assert_eq!(result, Some(Action::Rest));
        }

        #[test]
        fn fresh_actor_gets_predator_bonus_against_winded_target() {
            let tuning = CombatTuning::default();
            let actor = make("Fresh", 100);
            let fresh_target = make("FreshT", 100);
            let winded_target = make("WindedT", 30);
            let s_fresh = target_attack_score(&actor, &fresh_target, &tuning);
            let s_winded = target_attack_score(&actor, &winded_target, &tuning);
            assert_eq!(s_winded - s_fresh, tuning.fresh_target_visibly_tired_bonus);
        }

        #[test]
        fn fresh_actor_gets_predator_bonus_against_exhausted_target() {
            let tuning = CombatTuning::default();
            let actor = make("Fresh", 100);
            let fresh_target = make("FreshT", 100);
            let exhausted_target = make("ExT", 10);
            let s_fresh = target_attack_score(&actor, &fresh_target, &tuning);
            let s_ex = target_attack_score(&actor, &exhausted_target, &tuning);
            assert_eq!(s_ex - s_fresh, tuning.fresh_target_visibly_tired_bonus);
        }

        #[test]
        fn winded_actor_no_predator_bonus() {
            let tuning = CombatTuning::default();
            let actor = make("WActor", 30);
            let fresh_target = make("FreshT", 100);
            let winded_target = make("WindedT", 30);
            let s_fresh = target_attack_score(&actor, &fresh_target, &tuning);
            let s_winded = target_attack_score(&actor, &winded_target, &tuning);
            assert_eq!(s_fresh, s_winded);
        }

        #[test]
        fn action_gate_blocks_attack_when_stamina_below_cost() {
            let tuning = CombatTuning::default();
            let mut actor = make("Low", 100);
            actor.stamina = tuning.stamina_cost_attacker - 1;
            let score = action_score(&actor, &Action::Attack, &[], &tuning);
            assert_eq!(score, i32::MIN);
        }

        #[test]
        fn winded_actor_attack_score_lowered_by_penalty() {
            let tuning = CombatTuning::default();
            let fresh = make("F", 100);
            let winded = make("W", 30);
            let s_fresh = action_score(&fresh, &Action::Attack, &[], &tuning);
            let s_winded = action_score(&winded, &Action::Attack, &[], &tuning);
            assert_eq!(s_winded - s_fresh, tuning.winded_attack_score_penalty);
        }
    }
}
