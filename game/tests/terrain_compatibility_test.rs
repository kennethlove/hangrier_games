use game::terrain::{BaseTerrain, TerrainDescriptor, TerrainType};

#[test]
fn test_desert_cannot_be_wet() {
    let result = TerrainType::new(BaseTerrain::Desert, vec![TerrainDescriptor::Wet]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot have Wet descriptor"));
}

#[test]
fn test_tundra_cannot_be_hot() {
    let result = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Hot]);
    assert!(result.is_err());
}

#[test]
fn test_tundra_must_be_cold_or_frozen() {
    let cold_tundra = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Cold]);
    assert!(cold_tundra.is_ok());

    let frozen_tundra = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Frozen]);
    assert!(frozen_tundra.is_ok());
}

#[test]
fn test_geothermal_must_be_hot() {
    let cold_geo = TerrainType::new(BaseTerrain::Geothermal, vec![TerrainDescriptor::Cold]);
    assert!(cold_geo.is_err());

    let hot_geo = TerrainType::new(BaseTerrain::Geothermal, vec![TerrainDescriptor::Hot]);
    assert!(hot_geo.is_ok());
}

#[test]
fn test_forest_can_be_wet() {
    let rainforest = TerrainType::new(
        BaseTerrain::Forest,
        vec![TerrainDescriptor::Wet, TerrainDescriptor::Dense],
    );
    assert!(rainforest.is_ok());
}

#[test]
fn test_empty_descriptors_valid() {
    let plain_clearing = TerrainType::new(BaseTerrain::Clearing, vec![]);
    assert!(plain_clearing.is_ok());
}
