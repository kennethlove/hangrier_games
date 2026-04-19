use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::actions::Action;
use game::tributes::{Tribute, calculate_stamina_cost};
use rstest::rstest;

/// Test base stamina costs for each action type
#[rstest]
#[case(Action::Move(None), 20)]
#[case(Action::Hide, 15)]
#[case(Action::TakeItem, 10)]
#[case(Action::Attack, 25)]
#[case(Action::Rest, 0)]
#[case(Action::UseItem(None), 10)]
#[case(Action::None, 0)]
fn test_base_stamina_costs(#[case] action: Action, #[case] expected_base: u32) {
    // Neutral conditions: Clearing (1.0x), no affinity (1.0x), full health (1.0x)
    let terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100; // Full health for no desperation

    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // With all multipliers at 1.0, cost should equal base cost
    assert_eq!(
        cost, expected_base,
        "Base cost for {:?} should be {}",
        action, expected_base
    );
}

/// Test terrain multiplier application
#[rstest]
#[case(BaseTerrain::Grasslands, 0.9)] // Easiest
#[case(BaseTerrain::Clearing, 1.0)] // Neutral
#[case(BaseTerrain::UrbanRuins, 1.2)]
#[case(BaseTerrain::Forest, 1.3)]
#[case(BaseTerrain::Jungle, 1.4)]
#[case(BaseTerrain::Wetlands, 1.5)]
#[case(BaseTerrain::Highlands, 1.6)]
#[case(BaseTerrain::Badlands, 1.7)]
#[case(BaseTerrain::Mountains, 1.8)]
#[case(BaseTerrain::Desert, 2.0)] // Hardest
#[case(BaseTerrain::Tundra, 2.0)] // Hardest
fn test_terrain_multiplier(#[case] base_terrain: BaseTerrain, #[case] multiplier: f32) {
    let terrain = TerrainType::new(base_terrain, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100; // Full health

    let action = Action::Move(None); // Base cost 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    let expected = (20.0 * multiplier).round() as u32;
    assert_eq!(
        cost, expected,
        "Move action in {:?} should cost {} (20 * {})",
        base_terrain, expected, multiplier
    );
}

/// Test affinity modifier reduces cost by 20%
#[test]
fn test_affinity_modifier_with_affinity() {
    let terrain = TerrainType::new(BaseTerrain::Desert, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;
    tribute.terrain_affinity = vec![BaseTerrain::Desert]; // Has affinity

    let action = Action::Move(None); // Base 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 20 * 2.0 (terrain) * 0.8 (affinity) * 1.0 (desperation) = 32
    assert_eq!(cost, 32, "Affinity should reduce desert move cost to 32");
}

#[test]
fn test_affinity_modifier_without_affinity() {
    let terrain = TerrainType::new(BaseTerrain::Desert, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;
    tribute.terrain_affinity = vec![BaseTerrain::Forest]; // Different affinity

    let action = Action::Move(None); // Base 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 20 * 2.0 (terrain) * 1.0 (no affinity) * 1.0 (desperation) = 40
    assert_eq!(cost, 40, "Without affinity, desert move cost should be 40");
}

/// Test desperation multiplier at various health levels
#[rstest]
#[case(100, 1.0)] // 1.0 + 0.5 * (1.0 - 1.0) = 1.0
#[case(80, 1.1)] // 1.0 + 0.5 * (1.0 - 0.8) = 1.1
#[case(60, 1.2)] // 1.0 + 0.5 * (1.0 - 0.6) = 1.2
#[case(50, 1.25)] // 1.0 + 0.5 * (1.0 - 0.5) = 1.25
#[case(40, 1.3)] // 1.0 + 0.5 * (1.0 - 0.4) = 1.3
#[case(20, 1.4)] // 1.0 + 0.5 * (1.0 - 0.2) = 1.4
#[case(10, 1.45)] // 1.0 + 0.5 * (1.0 - 0.1) = 1.45
#[case(1, 1.495)] // 1.0 + 0.5 * (1.0 - 0.01) = 1.495
#[case(0, 1.5)] // 1.0 + 0.5 * (1.0 - 0.0) = 1.5
fn test_desperation_multiplier(#[case] health: u32, #[case] desperation: f32) {
    let terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = health;

    let action = Action::Move(None); // Base 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    let expected = (20.0 * 1.0 * 1.0 * desperation).round() as u32;
    assert_eq!(
        cost, expected,
        "At {}% health, desperation should be {}, cost {}",
        health, desperation, expected
    );
}

/// Test all multipliers combined
#[test]
fn test_all_multipliers_combined() {
    // Worst case: Desert (2.0x), no affinity (1.0x), near death (1.5x)
    let terrain = TerrainType::new(BaseTerrain::Desert, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 0; // Near death
    tribute.terrain_affinity = vec![]; // No affinity

    let action = Action::Attack; // Base 25
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 25 * 2.0 * 1.0 * 1.5 = 75
    assert_eq!(cost, 75, "Worst case attack should cost 75 stamina");
}

#[test]
fn test_best_case_combined() {
    // Best case: Grasslands (0.9x), with affinity (0.8x), full health (1.0x)
    let terrain = TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;
    tribute.terrain_affinity = vec![BaseTerrain::Grasslands];

    let action = Action::Hide; // Base 15
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 15 * 0.9 * 0.8 * 1.0 = 10.8 -> rounds to 11
    assert_eq!(cost, 11, "Best case hide should cost 11 stamina");
}

/// Edge case: stamina restoration
#[test]
fn test_stamina_restoration() {
    let mut tribute = Tribute::default();

    // Initial state should be full stamina
    assert_eq!(tribute.stamina, 100, "Initial stamina should be 100");
    assert_eq!(tribute.max_stamina, 100, "Max stamina should be 100");

    // Deplete stamina
    tribute.stamina = 25;

    // Restore
    tribute.restore_stamina();

    assert_eq!(
        tribute.stamina, tribute.max_stamina,
        "Stamina should be fully restored"
    );
}

/// Edge case: zero stamina
#[test]
fn test_zero_stamina_calculation() {
    let terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
    let mut tribute = Tribute {
        stamina: 0, // Depleted stamina
        ..Tribute::default()
    };
    tribute.attributes.health = 100;

    let action = Action::Rest; // Base cost 0
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    assert_eq!(cost, 0, "Rest action should always cost 0 stamina");
}

/// Edge case: action costs more than available stamina pool
#[test]
fn test_cost_exceeds_max_stamina() {
    let terrain = TerrainType::new(BaseTerrain::Desert, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 1; // Near death for max desperation
    tribute.max_stamina = 50; // Lower max stamina
    tribute.stamina = 50;

    let action = Action::Attack; // Base 25
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 25 * 2.0 * 1.0 * ~1.495 = ~74.75 -> 75
    assert_eq!(
        cost, 75,
        "Cost calculation should not be capped by max stamina"
    );
}

/// Edge case: multiple terrain affinities
#[test]
fn test_multiple_terrain_affinities() {
    let terrain = TerrainType::new(BaseTerrain::Forest, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;
    tribute.terrain_affinity = vec![
        BaseTerrain::Desert,
        BaseTerrain::Forest,
        BaseTerrain::Jungle,
    ];

    let action = Action::Move(None); // Base 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 20 * 1.3 * 0.8 * 1.0 = 20.8 -> 21
    assert_eq!(
        cost, 21,
        "Should apply affinity modifier if any affinity matches"
    );
}

/// Edge case: negative health (should clamp to 0 for calculation)
#[test]
fn test_negative_health_clamped() {
    let terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 0; // Dead tribute

    let action = Action::Move(None); // Base 20
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 20 * 1.0 * 1.0 * 1.5 = 30
    assert_eq!(cost, 30, "Zero health should use max desperation (1.5x)");
}

/// Test stamina fields initialization
#[test]
fn test_tribute_new_initializes_stamina() {
    let tribute = Tribute::new("Test".to_string(), Some(5), None);

    assert_eq!(tribute.stamina, 100, "New tribute should have 100 stamina");
    assert_eq!(
        tribute.max_stamina, 100,
        "New tribute should have 100 max stamina"
    );
}

/// Test different action types with same conditions
#[test]
fn test_action_type_ordering() {
    let terrain = TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;

    let attack_cost = calculate_stamina_cost(&Action::Attack, &terrain, &tribute);
    let move_cost = calculate_stamina_cost(&Action::Move(None), &terrain, &tribute);
    let hide_cost = calculate_stamina_cost(&Action::Hide, &terrain, &tribute);
    let take_cost = calculate_stamina_cost(&Action::TakeItem, &terrain, &tribute);
    let rest_cost = calculate_stamina_cost(&Action::Rest, &terrain, &tribute);

    assert!(attack_cost > move_cost, "Attack should cost more than Move");
    assert!(move_cost > hide_cost, "Move should cost more than Hide");
    assert!(hide_cost > take_cost, "Hide should cost more than Take");
    assert!(take_cost > rest_cost, "Take should cost more than Rest");
    assert_eq!(rest_cost, 0, "Rest should cost 0");
}

/// Test rounding behavior
#[test]
fn test_rounding_behavior() {
    let terrain = TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 100;
    tribute.terrain_affinity = vec![BaseTerrain::Grasslands];

    let action = Action::TakeItem; // Base 10
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 10 * 0.9 * 0.8 * 1.0 = 7.2 -> should round to 7
    assert_eq!(cost, 7, "Should round 7.2 down to 7");
}

#[test]
fn test_rounding_up() {
    let terrain = TerrainType::new(BaseTerrain::UrbanRuins, vec![]).unwrap();
    let mut tribute = Tribute::default();
    tribute.attributes.health = 90;
    tribute.terrain_affinity = vec![];

    let action = Action::Hide; // Base 15
    let cost = calculate_stamina_cost(&action, &terrain, &tribute);

    // 15 * 1.2 * 1.0 * 1.05 = 18.9 -> should round to 19
    assert_eq!(cost, 19, "Should round 18.9 up to 19");
}
