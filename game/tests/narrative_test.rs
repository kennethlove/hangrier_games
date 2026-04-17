use game::messages::{hiding_spot_narrative, movement_narrative, stamina_narrative};
use game::terrain::BaseTerrain;

/// Test movement narrative for Desert terrain.
#[test]
fn test_desert_movement_narrative() {
    let narrative = movement_narrative(BaseTerrain::Desert, "Alice");
    assert!(narrative.contains("Alice"));
    assert!(narrative.contains("desert"));
    assert!(narrative.contains("scorching") || narrative.contains("sands"));
}

/// Test movement narrative for Forest terrain.
#[test]
fn test_forest_movement_narrative() {
    let narrative = movement_narrative(BaseTerrain::Forest, "Bob");
    assert!(narrative.contains("Bob"));
    assert!(narrative.contains("forest"));
    assert!(narrative.contains("dense") || narrative.contains("branches"));
}

/// Test movement narrative for Mountains terrain.
#[test]
fn test_mountains_movement_narrative() {
    let narrative = movement_narrative(BaseTerrain::Mountains, "Charlie");
    assert!(narrative.contains("Charlie"));
    assert!(narrative.contains("mountain"));
    assert!(narrative.contains("climb") || narrative.contains("steep"));
}

/// Test hiding narrative for concealed terrain (Forest).
#[test]
fn test_forest_hiding_narrative() {
    let narrative = hiding_spot_narrative(BaseTerrain::Forest, "Diana");
    assert!(narrative.contains("Diana"));
    assert!(narrative.contains("foliage") || narrative.contains("concealed"));
}

/// Test hiding narrative for concealed terrain (UrbanRuins).
#[test]
fn test_urban_ruins_hiding_narrative() {
    let narrative = hiding_spot_narrative(BaseTerrain::UrbanRuins, "Eve");
    assert!(narrative.contains("Eve"));
    assert!(
        narrative.contains("building")
            || narrative.contains("shadows")
            || narrative.contains("cover")
    );
}

/// Test hiding narrative for exposed terrain (Desert).
#[test]
fn test_desert_hiding_narrative() {
    let narrative = hiding_spot_narrative(BaseTerrain::Desert, "Frank");
    assert!(narrative.contains("Frank"));
    assert!(
        narrative.contains("desert") || narrative.contains("dune") || narrative.contains("exposed")
    );
}

/// Test hiding narrative for exposed terrain (Tundra).
#[test]
fn test_tundra_hiding_narrative() {
    let narrative = hiding_spot_narrative(BaseTerrain::Tundra, "Grace");
    assert!(narrative.contains("Grace"));
    assert!(
        narrative.contains("snow") || narrative.contains("exposed") || narrative.contains("white")
    );
}

/// Test stamina narrative with high stamina in harsh terrain.
#[test]
fn test_high_stamina_harsh_terrain() {
    let narrative = stamina_narrative(BaseTerrain::Mountains, 80);
    assert!(narrative.contains("mountain"));
    // Should mention terrain but not exhaustion
    assert!(!narrative.contains("collapse"));
}

/// Test stamina narrative with low stamina in harsh terrain.
#[test]
fn test_low_stamina_harsh_terrain() {
    let narrative = stamina_narrative(BaseTerrain::Desert, 15);
    assert!(narrative.contains("desert"));
    assert!(narrative.contains("collapse") || narrative.contains("severe"));
}

/// Test stamina narrative with medium stamina in moderate terrain.
#[test]
fn test_medium_stamina_moderate_terrain() {
    let narrative = stamina_narrative(BaseTerrain::Wetlands, 50);
    assert!(narrative.contains("wetland"));
    // Should mention some fatigue but not severe
    assert!(!narrative.contains("collapse"));
}

/// Test stamina narrative with low stamina in mild terrain.
#[test]
fn test_low_stamina_mild_terrain() {
    let narrative = stamina_narrative(BaseTerrain::Clearing, 10);
    assert!(narrative.contains("clearing"));
    assert!(narrative.contains("exhaustion") || narrative.contains("collapse"));
}

/// Test that all terrains produce valid movement narratives.
#[test]
fn test_all_terrains_movement_narrative() {
    let terrains = vec![
        BaseTerrain::Desert,
        BaseTerrain::Tundra,
        BaseTerrain::Forest,
        BaseTerrain::Jungle,
        BaseTerrain::Mountains,
        BaseTerrain::Clearing,
        BaseTerrain::UrbanRuins,
        BaseTerrain::Grasslands,
        BaseTerrain::Wetlands,
        BaseTerrain::Badlands,
        BaseTerrain::Highlands,
        BaseTerrain::Geothermal,
    ];

    for terrain in terrains {
        let narrative = movement_narrative(terrain, "Test");
        assert!(
            narrative.contains("Test"),
            "Movement narrative should contain tribute name for {:?}",
            terrain
        );
        assert!(
            narrative.len() > 20,
            "Movement narrative should be descriptive for {:?}",
            terrain
        );
    }
}

/// Test that all terrains produce valid hiding narratives.
#[test]
fn test_all_terrains_hiding_narrative() {
    let terrains = vec![
        BaseTerrain::Desert,
        BaseTerrain::Tundra,
        BaseTerrain::Forest,
        BaseTerrain::Jungle,
        BaseTerrain::Mountains,
        BaseTerrain::Clearing,
        BaseTerrain::UrbanRuins,
        BaseTerrain::Grasslands,
        BaseTerrain::Wetlands,
        BaseTerrain::Badlands,
        BaseTerrain::Highlands,
        BaseTerrain::Geothermal,
    ];

    for terrain in terrains {
        let narrative = hiding_spot_narrative(terrain, "Test");
        assert!(
            narrative.contains("Test"),
            "Hiding narrative should contain tribute name for {:?}",
            terrain
        );
        assert!(
            narrative.len() > 20,
            "Hiding narrative should be descriptive for {:?}",
            terrain
        );
    }
}

/// Test that concealed terrains mention better hiding.
#[test]
fn test_concealed_terrains_better_hiding() {
    let forest_hide = hiding_spot_narrative(BaseTerrain::Forest, "Alice");
    let jungle_hide = hiding_spot_narrative(BaseTerrain::Jungle, "Bob");
    let urban_hide = hiding_spot_narrative(BaseTerrain::UrbanRuins, "Charlie");

    // Concealed terrains should use positive hiding language
    assert!(
        forest_hide.contains("conceals")
            || forest_hide.contains("hidden")
            || forest_hide.contains("invisible")
    );
    assert!(
        jungle_hide.contains("disappears")
            || jungle_hide.contains("hidden")
            || jungle_hide.contains("conceals")
    );
    assert!(
        urban_hide.contains("cover")
            || urban_hide.contains("shadows")
            || urban_hide.contains("hidden")
    );
}

/// Test that exposed terrains mention poor hiding.
#[test]
fn test_exposed_terrains_poor_hiding() {
    let desert_hide = hiding_spot_narrative(BaseTerrain::Desert, "Diana");
    let tundra_hide = hiding_spot_narrative(BaseTerrain::Tundra, "Eve");

    // Exposed terrains should indicate difficulty hiding
    assert!(
        desert_hide.contains("exposed")
            || desert_hide.contains("barely")
            || desert_hide.contains("open")
    );
    assert!(
        tundra_hide.contains("exposed")
            || tundra_hide.contains("standing out")
            || tundra_hide.contains("visible")
    );
}

/// Test stamina narrative at different thresholds.
#[test]
fn test_stamina_threshold_differences() {
    let terrain = BaseTerrain::Forest;

    let high = stamina_narrative(terrain, 80);
    let medium = stamina_narrative(terrain, 50);
    let low = stamina_narrative(terrain, 25);
    let critical = stamina_narrative(terrain, 10);

    // Each level should be different
    assert_ne!(high, medium);
    assert_ne!(medium, low);
    assert_ne!(low, critical);

    // Critical should be most severe
    assert!(critical.len() > 0);
}

/// Test that harsh terrain stamina narratives are more severe.
#[test]
fn test_harsh_terrain_more_severe_stamina() {
    let stamina = 30;

    let harsh = stamina_narrative(BaseTerrain::Mountains, stamina);
    let mild = stamina_narrative(BaseTerrain::Clearing, stamina);

    // Both should be non-empty
    assert!(harsh.len() > 0);
    assert!(mild.len() > 0);

    // Harsh should mention difficulty more prominently
    assert!(harsh.contains("harsh") || harsh.contains("brutal") || harsh.contains("demanding"));
}
