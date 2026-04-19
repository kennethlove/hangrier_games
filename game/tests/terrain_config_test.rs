use game::terrain::{BaseTerrain, Harshness, Visibility};
use strum::IntoEnumIterator;

#[test]
fn test_movement_costs_within_range() {
    for terrain in BaseTerrain::iter() {
        let cost = terrain.movement_cost();
        assert!(
            (0.5..=3.0).contains(&cost),
            "Movement cost out of range for {:?}",
            terrain
        );
    }
}

#[test]
fn test_clearing_is_mild() {
    assert_eq!(BaseTerrain::Clearing.harshness(), Harshness::Mild);
}

#[test]
fn test_tundra_is_harsh() {
    assert_eq!(BaseTerrain::Tundra.harshness(), Harshness::Harsh);
}

#[test]
fn test_forest_is_concealed() {
    assert_eq!(BaseTerrain::Forest.visibility(), Visibility::Concealed);
}

#[test]
fn test_desert_is_exposed() {
    assert_eq!(BaseTerrain::Desert.visibility(), Visibility::Exposed);
}

#[test]
fn test_item_weights_sum_to_one() {
    for terrain in BaseTerrain::iter() {
        let weights = terrain.item_weights();
        let sum = weights.weapons + weights.shields + weights.consumables;
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Item weights don't sum to 1.0 for {:?}: {}",
            terrain,
            sum
        );
    }
}

#[test]
fn test_item_spawn_modifiers_reasonable() {
    for terrain in BaseTerrain::iter() {
        let modifier = terrain.item_spawn_modifier();
        assert!(
            (0.5..=1.5).contains(&modifier),
            "Item modifier out of range for {:?}",
            terrain
        );
    }
}
