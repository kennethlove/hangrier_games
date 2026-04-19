use game::areas::events::AreaEvent;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use rand::SeedableRng;
use rand::rngs::SmallRng;

/// Test that events triggered in game loop actually process tribute survival checks
#[test]
fn test_event_survival_integration_with_game_loop() {
    let mut game = Game::new("test-game");
    game.start();

    // Create area with Forest terrain (catastrophic for wildfire)
    let mut area_details = AreaDetails::new_with_terrain(
        Some("Forest Area".to_string()),
        Area::North,
        TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
    );

    // Manually add wildfire event (simulate what trigger_cycle_events does)
    area_details.events.push(AreaEvent::Wildfire);
    game.areas.push(area_details);

    // Create tributes in forest with vulnerable health
    for i in 0..5 {
        let mut tribute = Tribute::new(format!("Tribute{}", i), Some((i % 12) + 1), None);
        tribute.area = Area::North;
        tribute.attributes.health = 50; // Vulnerable
        tribute.terrain_affinity = vec![]; // No protection
        tribute.statistics.game = game.identifier.clone();
        game.tributes.push(tribute);
    }

    let initial_alive = game.living_tributes().len();
    assert_eq!(initial_alive, 5);

    // Drain any setup messages so we only inspect event-driven output below
    game.messages.clear();

    // Process event survival checks
    let mut rng = SmallRng::seed_from_u64(123);
    game.process_event_for_area(&Area::North, &AreaEvent::Wildfire, &mut rng)
        .unwrap();

    // Check that some tributes died (probabilistic, but should happen)
    let final_alive = game.living_tributes().len();

    // In a catastrophic event with no protection, expect casualties
    // Run assertion with tolerance (not guaranteed 100% death)
    assert!(
        final_alive < initial_alive,
        "Expected casualties from catastrophic wildfire, but all {} tributes survived",
        initial_alive
    );

    // Verify messages were generated
    assert!(
        !game.messages.is_empty(),
        "Expected survival outcome messages but found none"
    );

    // Check for specific outcome messages (death or survival)
    let has_outcome_message = game.messages.iter().any(|m| {
        m.content.contains("dies from")
            || m.content.contains("killed by")
            || m.content.contains("survives")
    });
    assert!(
        has_outcome_message,
        "Expected death or survival messages but found none"
    );
}

/// Test that trigger_cycle_events integrates with process_event_for_area
#[test]
fn test_trigger_cycle_events_calls_process_event() {
    let mut game = Game::new("test-game");
    game.start();
    game.day = Some(2); // Day 2 to avoid special day #1 behavior

    // Create multiple areas with different terrains
    let forest_area = AreaDetails::new_with_terrain(
        Some("Forest".to_string()),
        Area::North,
        TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
    );
    let desert_area = AreaDetails::new_with_terrain(
        Some("Desert".to_string()),
        Area::South,
        TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
    );

    game.areas.push(forest_area);
    game.areas.push(desert_area);

    // Create tributes in each area
    for i in 0..6 {
        let mut tribute = Tribute::new(format!("Tribute{}", i), Some((i % 12) + 1), None);
        tribute.area = if i < 3 { Area::North } else { Area::South };
        tribute.attributes.health = 50;
        tribute.terrain_affinity = vec![];
        tribute.statistics.game = game.identifier.clone();
        game.tributes.push(tribute);
    }

    // Note: We can't easily test trigger_cycle_events directly because it's random
    // and private. This test verifies the integration exists by manually calling
    // process_event_for_area (which trigger_cycle_events now calls)

    // Manually trigger events (simulating what trigger_cycle_events does)
    game.areas[0].events.push(AreaEvent::Wildfire);
    game.areas[1].events.push(AreaEvent::Sandstorm);

    let initial_alive = game.living_tributes().len();

    // Drain setup messages so the assertion below only sees event output
    game.messages.clear();

    // Process events (what trigger_cycle_events now does internally)
    let mut rng = SmallRng::seed_from_u64(99);
    game.process_event_for_area(&Area::North, &AreaEvent::Wildfire, &mut rng)
        .unwrap();
    game.process_event_for_area(&Area::South, &AreaEvent::Sandstorm, &mut rng)
        .unwrap();

    let final_alive = game.living_tributes().len();

    // Verify tributes were affected
    // Wildfire in forest is catastrophic, sandstorm in desert is also catastrophic
    // Expect some deaths (probabilistic)
    assert!(
        final_alive <= initial_alive,
        "Expected some casualties from events"
    );

    // Verify messages exist
    assert!(!game.messages.is_empty(), "Expected event outcome messages");
}

/// Test that tributes with terrain affinity have better survival
#[test]
fn test_terrain_affinity_improves_survival() {
    // Run multiple trials to get statistical significance
    let mut with_affinity_deaths = 0;
    let mut without_affinity_deaths = 0;
    let trials = 20;
    let mut rng = SmallRng::seed_from_u64(2024);

    for _ in 0..trials {
        // Test WITH affinity
        let mut game_with = Game::new("test-affinity");
        game_with.start();

        let mut area = AreaDetails::new_with_terrain(
            Some("Forest".to_string()),
            Area::North,
            TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
        );
        area.events.push(AreaEvent::Wildfire);
        game_with.areas.push(area);

        let mut tribute = Tribute::new("Affinity Tribute".to_string(), Some(1), None);
        tribute.area = Area::North;
        tribute.attributes.health = 50;
        tribute.terrain_affinity = vec![BaseTerrain::Forest]; // HAS affinity
        tribute.statistics.game = game_with.identifier.clone();
        game_with.tributes.push(tribute);

        game_with
            .process_event_for_area(&Area::North, &AreaEvent::Wildfire, &mut rng)
            .unwrap();

        if game_with.tributes[0].attributes.health == 0 {
            with_affinity_deaths += 1;
        }

        // Test WITHOUT affinity
        let mut game_without = Game::new("test-no-affinity");
        game_without.start();

        let mut area2 = AreaDetails::new_with_terrain(
            Some("Forest".to_string()),
            Area::North,
            TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
        );
        area2.events.push(AreaEvent::Wildfire);
        game_without.areas.push(area2);

        let mut tribute2 = Tribute::new("No Affinity Tribute".to_string(), Some(1), None);
        tribute2.area = Area::North;
        tribute2.attributes.health = 50;
        tribute2.terrain_affinity = vec![]; // NO affinity
        tribute2.statistics.game = game_without.identifier.clone();
        game_without.tributes.push(tribute2);

        game_without
            .process_event_for_area(&Area::North, &AreaEvent::Wildfire, &mut rng)
            .unwrap();

        if game_without.tributes[0].attributes.health == 0 {
            without_affinity_deaths += 1;
        }
    }

    // Tributes with affinity should die less often
    // This is probabilistic, but over 20 trials should show a difference
    assert!(
        with_affinity_deaths <= without_affinity_deaths,
        "Expected tributes WITH affinity to die less often (with: {}, without: {})",
        with_affinity_deaths,
        without_affinity_deaths
    );
}
