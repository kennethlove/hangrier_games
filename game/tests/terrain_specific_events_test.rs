use game::areas::events::AreaEvent;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::terrain::{BaseTerrain, TerrainType};
use std::collections::HashMap;

/// Test that game loop generates terrain-appropriate events
/// Forest should get mostly wildfires, desert should get sandstorms, etc.
#[test]
fn test_game_loop_generates_terrain_appropriate_events() {
    let mut game = Game::new("test-terrain-events");
    let _ = game.start();
    game.day = Some(2); // Day 2 to avoid special day behavior

    // Create areas with different terrains
    let terrains = vec![
        (Area::North, BaseTerrain::Forest, "Forest"),
        (Area::South, BaseTerrain::Desert, "Desert"),
        (Area::East, BaseTerrain::Mountains, "Mountains"),
        (Area::West, BaseTerrain::Wetlands, "Wetlands"),
    ];

    for (area, terrain, name) in terrains {
        let area_details = AreaDetails::new_with_terrain(
            Some(name.to_string()),
            area,
            TerrainType::new(terrain, vec![]).unwrap(),
        );
        game.areas.push(area_details);
    }

    // Track which events occur in which areas over many cycles
    let mut event_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

    // Initialize counters
    for area_detail in &game.areas {
        let area_name = area_detail.area.as_ref().unwrap().to_string();
        event_counts.insert(area_name, HashMap::new());
    }

    // Run many cycles to get statistically significant results
    for _ in 0..100 {
        // Clear previous events
        for area_detail in &mut game.areas {
            area_detail.events.clear();
        }

        // Manually trigger events (we can't directly call private trigger_cycle_events)
        // Instead, we'll directly test the terrain-specific generation
        let mut rng = rand::rng();
        for area_detail in &mut game.areas {
            let event = AreaEvent::random_for_terrain(&area_detail.terrain.base, &mut rng);
            area_detail.events.push(event.clone());

            let area_name = area_detail.area.as_ref().unwrap().to_string();
            let event_name = event.to_string();

            *event_counts
                .get_mut(&area_name)
                .unwrap()
                .entry(event_name)
                .or_insert(0) += 1;
        }
    }

    // Verify terrain-appropriate events are most common

    // Forest should get mostly wildfires
    let forest_events = event_counts.get("North").unwrap();
    let forest_wildfire = forest_events.get("wildfire").unwrap_or(&0);
    assert!(
        *forest_wildfire > 20,
        "Forest should get many wildfires (got {})",
        forest_wildfire
    );

    // Desert should get mostly sandstorms or heatwaves
    let desert_events = event_counts.get("South").unwrap();
    let desert_sandstorm = desert_events.get("sandstorm").unwrap_or(&0);
    let desert_heatwave = desert_events.get("heatwave").unwrap_or(&0);
    assert!(
        desert_sandstorm + desert_heatwave > 40,
        "Desert should get many sandstorms/heatwaves (got {}/{})",
        desert_sandstorm,
        desert_heatwave
    );

    // Mountains should get mostly avalanches or rockslides
    let mountain_events = event_counts.get("East").unwrap();
    let mountain_avalanche = mountain_events.get("avalanche").unwrap_or(&0);
    let mountain_rockslide = mountain_events.get("rockslide").unwrap_or(&0);
    assert!(
        mountain_avalanche + mountain_rockslide > 40,
        "Mountains should get many avalanches/rockslides (got {}/{})",
        mountain_avalanche,
        mountain_rockslide
    );

    // Wetlands should get mostly floods
    let wetland_events = event_counts.get("West").unwrap();
    let wetland_flood = wetland_events.get("flood").unwrap_or(&0);
    assert!(
        *wetland_flood > 30,
        "Wetlands should get many floods (got {})",
        wetland_flood
    );
}

/// Test that different terrain types get different event distributions
#[test]
fn test_terrain_event_diversity() {
    let mut rng = rand::rng();

    // Forest events
    let mut forest_events = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Forest, &mut rng);
        *forest_events.entry(event.to_string()).or_insert(0) += 1;
    }

    // Desert events
    let mut desert_events = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Desert, &mut rng);
        *desert_events.entry(event.to_string()).or_insert(0) += 1;
    }

    // Forests and deserts should have significantly different event distributions
    // Forest should have more wildfires, desert should have more sandstorms
    let forest_wildfire = forest_events.get("wildfire").unwrap_or(&0);
    let desert_wildfire = desert_events.get("wildfire").unwrap_or(&0);
    let desert_sandstorm = desert_events.get("sandstorm").unwrap_or(&0);

    assert!(
        forest_wildfire > desert_wildfire,
        "Forest should get more wildfires than desert ({} vs {})",
        forest_wildfire,
        desert_wildfire
    );

    assert!(
        *desert_sandstorm > 20,
        "Desert should get many sandstorms (got {})",
        desert_sandstorm
    );
}

/// Test that terrain-specific events have appropriate severity for that terrain
#[test]
fn test_terrain_specific_events_have_logical_severity() {
    use game::areas::events::EventSeverity;

    // Wildfire in forest should be catastrophic
    let wildfire_in_forest = AreaEvent::Wildfire.severity_in_terrain(&BaseTerrain::Forest);
    assert_eq!(wildfire_in_forest, EventSeverity::Catastrophic);

    // Sandstorm in desert should be catastrophic
    let sandstorm_in_desert = AreaEvent::Sandstorm.severity_in_terrain(&BaseTerrain::Desert);
    assert_eq!(sandstorm_in_desert, EventSeverity::Catastrophic);

    // Avalanche in mountains should be catastrophic
    let avalanche_in_mountains = AreaEvent::Avalanche.severity_in_terrain(&BaseTerrain::Mountains);
    assert_eq!(avalanche_in_mountains, EventSeverity::Catastrophic);

    // Flood in wetlands should be catastrophic
    let flood_in_wetlands = AreaEvent::Flood.severity_in_terrain(&BaseTerrain::Wetlands);
    assert_eq!(flood_in_wetlands, EventSeverity::Catastrophic);
}
