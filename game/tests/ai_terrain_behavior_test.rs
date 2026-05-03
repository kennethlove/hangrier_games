use game::areas::{Area, AreaDetails};
use game::items::Item;
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use game::tributes::brains::Brain;
use rand::prelude::*;
use rstest::{fixture, rstest};

#[fixture]
fn small_rng() -> SmallRng {
    // Deterministic seed: behavioral tests assert on a narrow set of
    // valid actions, so the RNG must be reproducible across CI runs.
    SmallRng::seed_from_u64(0xA17EBA11)
}

#[fixture]
fn tribute_with_forest_affinity() -> Tribute {
    let mut tribute = Tribute::new("Katniss".to_string(), Some(7), None);
    tribute.terrain_affinity = vec![BaseTerrain::Forest];
    tribute
}

#[fixture]
fn areas_with_terrain() -> Vec<AreaDetails> {
    vec![
        AreaDetails::new_with_terrain(
            Some("Forest Area".to_string()),
            Area::Sector1,
            TerrainType::new(BaseTerrain::Forest, vec![]).unwrap(),
        ),
        AreaDetails::new_with_terrain(
            Some("Desert Area".to_string()),
            Area::Sector4,
            TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
        ),
        AreaDetails::new_with_terrain(
            Some("Grasslands Area".to_string()),
            Area::Sector2,
            TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap(),
        ),
    ]
}

/// Test that destination scoring favors affinity terrain
#[rstest]
fn test_destination_scoring_favors_affinity_terrain(
    tribute_with_forest_affinity: Tribute,
    areas_with_terrain: Vec<AreaDetails>,
) {
    let brain = Brain::default();
    let chosen = brain.choose_destination(
        &areas_with_terrain,
        &tribute_with_forest_affinity,
        &std::collections::HashMap::new(),
    );

    assert!(chosen.is_some());
    let chosen_area = chosen.unwrap();
    assert_eq!(chosen_area, Area::Sector1); // Forest area should be chosen
}

/// Test that harsh terrain receives penalty in scoring
#[rstest]
fn test_harsh_terrain_penalty_applied() {
    let brain = Brain::default();
    let tribute = Tribute::new("Peeta".to_string(), Some(12), None);

    let areas = vec![
        // Grasslands (Mild harshness) - should score higher
        AreaDetails::new_with_terrain(
            Some("Safe Grasslands".to_string()),
            Area::Sector1,
            TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap(),
        ),
        // Desert (Harsh) - should score lower
        AreaDetails::new_with_terrain(
            Some("Harsh Desert".to_string()),
            Area::Sector4,
            TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
        ),
    ];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    assert_eq!(chosen.unwrap(), Area::Sector1); // Grasslands should be preferred
}

/// Test that concealed terrain boosts hiding preference
#[rstest]
fn test_concealed_terrain_boosts_hiding(mut small_rng: SmallRng) {
    let brain = Brain::default();
    let tribute = Tribute::new("Rue".to_string(), Some(11), None);

    // Forest has Concealed visibility
    let forest_terrain = TerrainType::new(BaseTerrain::Forest, vec![]).unwrap();

    // Test that action weights favor Hide in concealed terrain
    let action = brain.decide_action_with_terrain(&tribute, 3, forest_terrain, &mut small_rng);

    // With few enemies and concealed terrain, should prefer hiding
    // (This tests the weight boost logic)
    assert!(matches!(
        action,
        game::tributes::actions::Action::Hide | game::tributes::actions::Action::Move(_)
    ));
}

/// Test that resource-scarce terrain boosts search behavior
#[rstest]
fn test_resource_scarce_terrain_boosts_search(mut small_rng: SmallRng) {
    let brain = Brain::default();
    let mut tribute = Tribute::new("Foxface".to_string(), Some(5), None);
    tribute.attributes.health = 50;

    // Desert is resource-scarce
    let desert_terrain = TerrainType::new(BaseTerrain::Desert, vec![]).unwrap();

    // The brain should recognize Desert as resource-scarce and boost search weight
    // This is a behavioral test - we expect search/move actions in scarce terrain
    let action = brain.decide_action_with_terrain(&tribute, 0, desert_terrain, &mut small_rng);

    // Should prefer moving/searching in resource-scarce terrain when alone
    assert!(matches!(action, game::tributes::actions::Action::Move(_)));
}

/// Test that desperate tributes (health < 30) flee to affinity terrain
#[rstest]
fn test_desperate_tributes_flee_to_affinity_terrain() {
    let brain = Brain::default();
    let mut tribute = Tribute::new("Thresh".to_string(), Some(11), None);
    tribute.attributes.health = 25; // Desperate (< 30)
    tribute.terrain_affinity = vec![BaseTerrain::Grasslands];

    let areas = vec![
        AreaDetails::new_with_terrain(
            Some("Safe Grasslands".to_string()),
            Area::Sector1,
            TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap(),
        ),
        AreaDetails::new_with_terrain(
            Some("Dangerous Mountains".to_string()),
            Area::Sector4,
            TerrainType::new(BaseTerrain::Mountains, vec![]).unwrap(),
        ),
    ];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    assert_eq!(chosen.unwrap(), Area::Sector1); // Should flee to affinity terrain
}

/// Test that concealed visibility gives bonus to hiding spots
#[rstest]
fn test_concealed_visibility_bonus() {
    let brain = Brain::default();
    let tribute = Tribute::new("Katniss".to_string(), Some(12), None);

    let areas = vec![
        // Jungle is Concealed - good for hiding
        AreaDetails::new_with_terrain(
            Some("Dense Jungle".to_string()),
            Area::Sector1,
            TerrainType::new(BaseTerrain::Jungle, vec![]).unwrap(),
        ),
        // Desert is Exposed - bad for hiding
        AreaDetails::new_with_terrain(
            Some("Open Desert".to_string()),
            Area::Sector4,
            TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
        ),
    ];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    // Concealed terrain should score higher
    assert_eq!(chosen.unwrap(), Area::Sector1);
}

/// Test that areas with items get scoring bonus
#[rstest]
fn test_areas_with_items_bonus() {
    let brain = Brain::default();
    let tribute = Tribute::new("Foxface".to_string(), Some(5), None);

    let mut area_with_items = AreaDetails::new_with_terrain(
        Some("Supply Area".to_string()),
        Area::Sector1,
        TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
    );
    area_with_items.items.push(Item::new_random_weapon());

    let area_without_items = AreaDetails::new_with_terrain(
        Some("Empty Area".to_string()),
        Area::Sector4,
        TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
    );

    let areas = vec![area_with_items, area_without_items];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    assert_eq!(chosen.unwrap(), Area::Sector1); // Should prefer area with items
}

/// Test scoring with multiple factors combined
#[rstest]
fn test_combined_scoring_factors() {
    let brain = Brain::default();
    let mut tribute = Tribute::new("Cato".to_string(), Some(2), None);
    tribute.terrain_affinity = vec![BaseTerrain::UrbanRuins];
    tribute.attributes.health = 80; // Not desperate

    let mut perfect_area = AreaDetails::new_with_terrain(
        Some("Ideal Ruins".to_string()),
        Area::Sector1,
        TerrainType::new(BaseTerrain::UrbanRuins, vec![]).unwrap(), // Affinity + Concealed
    );
    perfect_area.items.push(Item::new_random_weapon()); // Has items

    let poor_area = AreaDetails::new_with_terrain(
        Some("Harsh Tundra".to_string()),
        Area::Sector4,
        TerrainType::new(BaseTerrain::Tundra, vec![]).unwrap(), // Harsh + Exposed
    );

    let areas = vec![perfect_area, poor_area];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    assert_eq!(chosen.unwrap(), Area::Sector1); // Should strongly prefer perfect area
}

/// Test that desperate modifier significantly boosts affinity terrain
#[rstest]
fn test_desperate_modifier_strength() {
    let brain = Brain::default();
    let mut tribute = Tribute::new("Marvel".to_string(), Some(1), None);
    tribute.terrain_affinity = vec![BaseTerrain::UrbanRuins];
    tribute.attributes.health = 20; // Very desperate

    // Even with items in the non-affinity area, desperate tribute should prefer affinity
    let affinity_area = AreaDetails::new_with_terrain(
        Some("Safe Ruins".to_string()),
        Area::Sector1,
        TerrainType::new(BaseTerrain::UrbanRuins, vec![]).unwrap(),
    );

    let mut tempting_area = AreaDetails::new_with_terrain(
        Some("Supply Grasslands".to_string()),
        Area::Sector4,
        TerrainType::new(BaseTerrain::Grasslands, vec![]).unwrap(),
    );
    tempting_area.items.push(Item::new_random_weapon());
    tempting_area.items.push(Item::new_random_consumable());

    let areas = vec![affinity_area, tempting_area];

    let chosen = brain.choose_destination(&areas, &tribute, &std::collections::HashMap::new());
    assert!(chosen.is_some());
    // Desperate (3.0x) boost should overcome item bonuses
    assert_eq!(chosen.unwrap(), Area::Sector1);
}
