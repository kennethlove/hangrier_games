use game::areas::events::AreaEvent;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[test]
fn test_wildfire_in_forest_kills_tributes() {
    let mut game = Game::new("test-game");

    // Create area with Forest terrain
    let area_details = AreaDetails::new_with_terrain(
        Some("Forest Area".to_string()),
        Area::North,
        TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
    );
    game.areas.push(area_details);

    let area_name = game.areas[0].area.unwrap();

    // Create tribute in forest area with low health
    let mut tribute = Tribute::random();
    tribute.area = area_name;
    tribute.attributes.health = 50; // Not desperate but vulnerable
    tribute.terrain_affinity = vec![]; // No affinity bonus
    tribute.statistics.game = game.identifier.clone(); // Set game identifier
    game.tributes.push(tribute.clone());

    // Process wildfire event (catastrophic in forest)
    let event = AreaEvent::Wildfire;

    // Run 10 times, expect at least some deaths
    let mut death_count = 0;
    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..10 {
        let mut test_game = game.clone();
        test_game
            .process_event_for_area(&area_name, &event, &mut rng)
            .unwrap();
        if test_game.tributes[0].attributes.health == 0 {
            death_count += 1;
        }
    }

    assert!(
        death_count > 0,
        "Catastrophic wildfire should kill some tributes"
    );
}

#[test]
fn test_wildfire_in_desert_minor_impact() {
    let mut game = Game::new("test-game");

    // Create area with Desert terrain
    let area_details = AreaDetails::new_with_terrain(
        Some("Desert Area".to_string()),
        Area::South,
        TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
    );
    game.areas.push(area_details);

    let area_name = game.areas[0].area.unwrap();

    // Create tribute
    let mut tribute = Tribute::random();
    tribute.area = Area::South;
    tribute.attributes.health = 50;
    tribute.terrain_affinity = vec![];
    tribute.statistics.game = game.identifier.clone();
    game.tributes.push(tribute);

    // Process wildfire (minor in desert)
    let event = AreaEvent::Wildfire;

    // Run 10 times, expect low death rate
    let mut death_count = 0;
    let mut rng = SmallRng::seed_from_u64(7);
    for _ in 0..10 {
        let mut test_game = game.clone();
        test_game
            .process_event_for_area(&area_name, &event, &mut rng)
            .unwrap();
        if test_game.tributes[0].attributes.health == 0 {
            death_count += 1;
        }
    }

    assert!(
        death_count < 5,
        "Minor wildfire should rarely kill tributes"
    );
}
