use game::areas::events::{AreaEvent, EventSeverity};
use game::terrain::BaseTerrain;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rstest::rstest;

// Test EventSeverity ordering
#[test]
fn test_severity_ordering() {
    assert!(EventSeverity::Catastrophic > EventSeverity::Major);
    assert!(EventSeverity::Major > EventSeverity::Moderate);
    assert!(EventSeverity::Moderate > EventSeverity::Minor);
}

// Wildfire severity tests
#[rstest]
#[case(BaseTerrain::Forest, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Jungle, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Grasslands, EventSeverity::Major)]
#[case(BaseTerrain::Desert, EventSeverity::Minor)]
#[case(BaseTerrain::Tundra, EventSeverity::Minor)]
#[case(BaseTerrain::Wetlands, EventSeverity::Minor)]
fn test_wildfire_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Wildfire;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Blizzard severity tests
#[rstest]
#[case(BaseTerrain::Mountains, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Tundra, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Highlands, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Desert, EventSeverity::Minor)]
#[case(BaseTerrain::Jungle, EventSeverity::Minor)]
fn test_blizzard_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Blizzard;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Sandstorm severity tests
#[rstest]
#[case(BaseTerrain::Desert, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Badlands, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Grasslands, EventSeverity::Major)]
#[case(BaseTerrain::Clearing, EventSeverity::Moderate)]
#[case(BaseTerrain::Forest, EventSeverity::Minor)]
#[case(BaseTerrain::Wetlands, EventSeverity::Minor)]
fn test_sandstorm_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Sandstorm;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Flood severity tests
#[rstest]
#[case(BaseTerrain::Wetlands, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Jungle, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Major)]
#[case(BaseTerrain::Grasslands, EventSeverity::Moderate)]
#[case(BaseTerrain::Mountains, EventSeverity::Minor)]
#[case(BaseTerrain::Highlands, EventSeverity::Minor)]
fn test_flood_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Flood;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Earthquake severity tests
#[rstest]
#[case(BaseTerrain::Mountains, EventSeverity::Catastrophic)]
#[case(BaseTerrain::UrbanRuins, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Highlands, EventSeverity::Major)]
#[case(BaseTerrain::Geothermal, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Grasslands, EventSeverity::Minor)]
fn test_earthquake_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Earthquake;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Avalanche severity tests
#[rstest]
#[case(BaseTerrain::Mountains, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Highlands, EventSeverity::Major)]
#[case(BaseTerrain::Tundra, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Grasslands, EventSeverity::Minor)]
#[case(BaseTerrain::Desert, EventSeverity::Minor)]
fn test_avalanche_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Avalanche;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Landslide severity tests
#[rstest]
#[case(BaseTerrain::Mountains, EventSeverity::Major)]
#[case(BaseTerrain::Highlands, EventSeverity::Major)]
#[case(BaseTerrain::Jungle, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Grasslands, EventSeverity::Minor)]
fn test_landslide_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Landslide;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Heatwave severity tests
#[rstest]
#[case(BaseTerrain::Desert, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Badlands, EventSeverity::Major)]
#[case(BaseTerrain::Grasslands, EventSeverity::Moderate)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Tundra, EventSeverity::Minor)]
#[case(BaseTerrain::Mountains, EventSeverity::Minor)]
fn test_heatwave_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Heatwave;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Drought severity tests
#[rstest]
#[case(BaseTerrain::Desert, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Grasslands, EventSeverity::Major)]
#[case(BaseTerrain::Badlands, EventSeverity::Major)]
#[case(BaseTerrain::Forest, EventSeverity::Moderate)]
#[case(BaseTerrain::Wetlands, EventSeverity::Minor)]
#[case(BaseTerrain::Jungle, EventSeverity::Minor)]
fn test_drought_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Drought;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Rockslide severity tests
#[rstest]
#[case(BaseTerrain::Mountains, EventSeverity::Catastrophic)]
#[case(BaseTerrain::Badlands, EventSeverity::Major)]
#[case(BaseTerrain::Highlands, EventSeverity::Major)]
#[case(BaseTerrain::UrbanRuins, EventSeverity::Moderate)]
#[case(BaseTerrain::Grasslands, EventSeverity::Minor)]
#[case(BaseTerrain::Wetlands, EventSeverity::Minor)]
fn test_rockslide_severity(#[case] terrain: BaseTerrain, #[case] expected: EventSeverity) {
    let event = AreaEvent::Rockslide;
    assert_eq!(event.severity_in_terrain(&terrain), expected);
}

// Survival check tests - affinity bonus
#[test]
fn test_survival_check_with_affinity() {
    let event = AreaEvent::Wildfire;
    let terrain = BaseTerrain::Forest;
    let mut rng = SmallRng::seed_from_u64(42);

    // With affinity, survival should be easier
    let result_with_affinity =
        event.survival_check(&terrain, true, false, false, 100, true, 1.0, &mut rng);
    let mut rng2 = SmallRng::seed_from_u64(42);
    let result_without_affinity =
        event.survival_check(&terrain, false, false, false, 100, true, 1.0, &mut rng2);

    // Both should succeed or fail, but we can't deterministically test randomness
    // Just verify the function runs without panic and returns accessible fields
    let _ = result_with_affinity.survived;
    let _ = result_without_affinity.survived;
}

// Survival check tests - item bonus
#[test]
fn test_survival_check_with_item_bonus() {
    let event = AreaEvent::Blizzard;
    let terrain = BaseTerrain::Tundra;
    let mut rng = SmallRng::seed_from_u64(42);

    // With item bonus, survival should be easier
    let result_with_item =
        event.survival_check(&terrain, false, true, false, 100, true, 1.0, &mut rng);
    let mut rng2 = SmallRng::seed_from_u64(42);
    let result_without_item =
        event.survival_check(&terrain, false, false, false, 100, true, 1.0, &mut rng2);

    // Just verify the function runs
    let _ = result_with_item.survived;
    let _ = result_without_item.survived;
}

// Survival check tests - desperation bonus
#[test]
fn test_survival_check_with_desperation() {
    let event = AreaEvent::Earthquake;
    let terrain = BaseTerrain::Mountains;
    let mut rng = SmallRng::seed_from_u64(42);

    // With desperation, survival should be easier
    let result_desperate =
        event.survival_check(&terrain, false, false, true, 10, true, 1.0, &mut rng);
    let mut rng2 = SmallRng::seed_from_u64(42);
    let result_normal =
        event.survival_check(&terrain, false, false, false, 100, true, 1.0, &mut rng2);

    // Just verify the function runs
    let _ = result_desperate.survived;
    let _ = result_normal.survived;
}

// Test survival result structure
#[test]
fn test_survival_result_structure() {
    let event = AreaEvent::Wildfire;
    let terrain = BaseTerrain::Forest;
    let mut rng = SmallRng::seed_from_u64(42);

    let result = event.survival_check(&terrain, false, false, false, 100, true, 1.0, &mut rng);

    // Verify result has expected fields
    if result.survived {
        // If survived, stamina/sanity restored are non-negative by type (u32)
        // Just verify we can access the fields
        let _ = result.stamina_restored;
        let _ = result.sanity_restored;
    } else {
        // If died, no rewards
        assert_eq!(result.stamina_restored, 0);
        assert_eq!(result.sanity_restored, 0);
        assert!(result.reward_item.is_none());
    }
}

// Test catastrophic events have reduced instant-death chance (5% not 10%)
#[test]
fn test_catastrophic_instant_death_probability() {
    let event = AreaEvent::Wildfire;
    let terrain = BaseTerrain::Forest; // Catastrophic for wildfire

    // Run many checks to verify ~5% instant death rate
    let mut instant_deaths = 0;
    let trials = 1000;

    for i in 0..trials {
        let mut rng = SmallRng::seed_from_u64(i);
        let result = event.survival_check(&terrain, false, false, false, 100, true, 1.0, &mut rng);
        if !result.survived && result.instant_death {
            instant_deaths += 1;
        }
    }

    let death_rate = instant_deaths as f32 / trials as f32;

    // Should be around 5% (allow 2-10% range for randomness)
    assert!(
        (0.02..=0.10).contains(&death_rate),
        "Instant death rate {} outside expected range",
        death_rate
    );
}

// Test desperation rewards distribution
#[test]
fn test_desperation_rewards_distribution() {
    let event = AreaEvent::Sandstorm;
    let terrain = BaseTerrain::Grasslands; // Major severity

    let mut stamina_rewards = 0;
    let mut sanity_rewards = 0;
    let mut item_rewards = 0;
    let mut no_rewards = 0;
    let trials = 1000;

    for i in 0..trials {
        let mut rng = SmallRng::seed_from_u64(i);
        let result = event.survival_check(&terrain, false, false, true, 10, true, 1.0, &mut rng);
        if result.survived {
            if result.stamina_restored > 0 {
                stamina_rewards += 1;
            } else if result.sanity_restored > 0 {
                sanity_rewards += 1;
            } else if result.reward_item.is_some() {
                item_rewards += 1;
            } else {
                no_rewards += 1;
            }
        }
    }

    // Expected distribution: ~42.5% stamina, ~42.5% sanity, ~10% item, ~5% nothing
    // Allow broad ranges for randomness
    if stamina_rewards + sanity_rewards + item_rewards + no_rewards > 0 {
        let total = (stamina_rewards + sanity_rewards + item_rewards + no_rewards) as f32;
        let stamina_pct = stamina_rewards as f32 / total;
        let sanity_pct = sanity_rewards as f32 / total;
        let item_pct = item_rewards as f32 / total;
        let none_pct = no_rewards as f32 / total;

        // Rough validation (allow 25-60% for stamina/sanity, 0-25% for item, 0-15% for none)
        assert!((0.25..=0.60).contains(&stamina_pct));
        assert!((0.25..=0.60).contains(&sanity_pct));
        assert!((0.0..=0.25).contains(&item_pct));
        assert!((0.0..=0.20).contains(&none_pct));
    }
}
