use game::districts::assign_terrain_affinity;
use game::terrain::BaseTerrain;
use game::tributes::Tribute;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rstest::rstest;

#[rstest]
#[case(1, BaseTerrain::UrbanRuins, vec![BaseTerrain::Clearing, BaseTerrain::Grasslands])]
#[case(2, BaseTerrain::Mountains, vec![BaseTerrain::UrbanRuins, BaseTerrain::Badlands])]
#[case(3, BaseTerrain::UrbanRuins, vec![BaseTerrain::Mountains, BaseTerrain::Clearing])]
#[case(4, BaseTerrain::Wetlands, vec![BaseTerrain::Forest, BaseTerrain::Jungle])]
#[case(5, BaseTerrain::Geothermal, vec![BaseTerrain::UrbanRuins, BaseTerrain::Mountains])]
#[case(6, BaseTerrain::Grasslands, vec![BaseTerrain::Clearing, BaseTerrain::Highlands])]
#[case(7, BaseTerrain::Forest, vec![BaseTerrain::Jungle, BaseTerrain::Wetlands])]
#[case(8, BaseTerrain::Grasslands, vec![BaseTerrain::Clearing, BaseTerrain::Wetlands])]
#[case(9, BaseTerrain::Grasslands, vec![BaseTerrain::Clearing, BaseTerrain::Highlands])]
#[case(10, BaseTerrain::Grasslands, vec![BaseTerrain::Highlands, BaseTerrain::Badlands])]
#[case(11, BaseTerrain::Grasslands, vec![BaseTerrain::Clearing, BaseTerrain::Forest])]
#[case(12, BaseTerrain::Tundra, vec![BaseTerrain::Mountains, BaseTerrain::Badlands])]
fn test_district_primary_affinity(
    #[case] district: u8,
    #[case] expected_primary: BaseTerrain,
    #[case] expected_bonus_pool: Vec<BaseTerrain>,
) {
    // Test that primary affinity is always present

    // Run multiple times to ensure primary is always there
    for _ in 0..10 {
        let tribute = Tribute::new(
            format!("Tribute from District {}", district),
            Some(district as u32),
            None,
        );

        assert!(
            !tribute.terrain_affinity.is_empty(),
            "District {} should have at least one terrain affinity",
            district
        );

        assert_eq!(
            tribute.terrain_affinity[0], expected_primary,
            "District {} should have {:?} as primary affinity",
            district, expected_primary
        );

        // Verify any bonus terrain comes from the pool
        if tribute.terrain_affinity.len() > 1 {
            let bonus = tribute.terrain_affinity[1];
            assert!(
                expected_bonus_pool.contains(&bonus),
                "District {} bonus terrain {:?} should be in pool {:?}",
                district,
                bonus,
                expected_bonus_pool
            );
        }
    }
}

#[test]
fn test_affinity_count() {
    // Test each district multiple times
    for district in 1..=12 {
        let tribute = Tribute::new(
            format!("Tribute from District {}", district),
            Some(district),
            None,
        );

        let affinity_count = tribute.terrain_affinity.len();
        assert!(
            (1..=2).contains(&affinity_count),
            "District {} should have 1-2 terrain affinities, got {}",
            district,
            affinity_count
        );
    }
}

#[test]
fn test_bonus_affinity_probability() {
    // Use seeded RNG for determinism (mirrors unit test in districts.rs).
    // 1000 iterations gives tighter ±5% bound vs flaky 100-iter ±10%.
    let iterations = 1000;
    let mut bonus_count = 0;

    for i in 0..iterations {
        let mut rng = SmallRng::seed_from_u64(i);
        let affinities = assign_terrain_affinity(1, &mut rng);

        if affinities.len() == 2 {
            bonus_count += 1;
        }
    }

    let percentage = (bonus_count as f64 / iterations as f64) * 100.0;
    assert!(
        (35.0..=45.0).contains(&percentage),
        "Expected ~40% bonus rate, got {:.1}%",
        percentage
    );
}

#[test]
fn test_affinity_terrains_valid() {
    // Ensure all terrain affinities are valid BaseTerrain variants
    for district in 1..=12 {
        let tribute = Tribute::new(
            format!("Tribute from District {}", district),
            Some(district),
            None,
        );

        for terrain in &tribute.terrain_affinity {
            // BaseTerrain is an enum, so any value is valid
            // This test verifies the field exists and contains BaseTerrain types
            assert!(
                matches!(
                    terrain,
                    BaseTerrain::Clearing
                        | BaseTerrain::Forest
                        | BaseTerrain::Desert
                        | BaseTerrain::Tundra
                        | BaseTerrain::Wetlands
                        | BaseTerrain::Mountains
                        | BaseTerrain::UrbanRuins
                        | BaseTerrain::Jungle
                        | BaseTerrain::Grasslands
                        | BaseTerrain::Badlands
                        | BaseTerrain::Highlands
                        | BaseTerrain::Geothermal
                ),
                "Invalid terrain type for district {}: {:?}",
                district,
                terrain
            );
        }
    }
}
