use game::terrain::{BaseTerrain, Harshness, TerrainType};
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[test]
fn test_random_terrain_generates_valid_terrain() {
    let mut rng = SmallRng::seed_from_u64(12345);
    let terrain = TerrainType::random(&mut rng);

    // Should have 0-2 descriptors
    assert!(terrain.descriptors.len() <= 2);

    // All descriptors should be compatible (validated by TerrainType::new)
    assert!(TerrainType::new(terrain.base, terrain.descriptors.clone()).is_ok());
}

#[test]
fn test_random_safe_generates_mild_terrains() {
    let mut rng = SmallRng::seed_from_u64(54321);

    for _ in 0..20 {
        let terrain = TerrainType::random_safe(&mut rng);
        assert!(matches!(terrain.base.harshness(), Harshness::Mild));
    }
}

#[test]
fn test_balance_constraint_limits_harsh_terrains() {
    let mut rng = SmallRng::seed_from_u64(99999);

    // Create 5 terrains, all harsh
    let mut terrains = vec![
        TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
        TerrainType::new(BaseTerrain::Tundra, vec![]).unwrap(),
        TerrainType::new(BaseTerrain::Mountains, vec![]).unwrap(),
        TerrainType::new(BaseTerrain::Badlands, vec![]).unwrap(),
        TerrainType::new(BaseTerrain::Desert, vec![]).unwrap(),
    ];

    game::terrain::enforce_balance_constraint(&mut terrains, &mut rng);

    // Should now have at most 3 harsh terrains
    let harsh_count = terrains
        .iter()
        .filter(|t| matches!(t.base.harshness(), Harshness::Harsh))
        .count();

    assert!(
        harsh_count <= 3,
        "Found {} harsh terrains, expected <= 3",
        harsh_count
    );
}

#[test]
fn test_descriptor_generation_respects_compatibility() {
    let mut rng = SmallRng::seed_from_u64(11111);

    // Generate 100 random terrains
    for _ in 0..100 {
        let terrain = TerrainType::random(&mut rng);

        // Verify compatibility by attempting to create with validation
        let result = TerrainType::new(terrain.base, terrain.descriptors.clone());
        assert!(
            result.is_ok(),
            "Generated incompatible terrain: {:?} with {:?}",
            terrain.base,
            terrain.descriptors
        );
    }
}
