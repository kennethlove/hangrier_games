use crate::terrain::config::Harshness;
use crate::terrain::{BaseTerrain, TerrainDescriptor, TerrainType};
use rand::prelude::*;
use strum::IntoEnumIterator;

impl TerrainType {
    /// Generate random terrain with 0-2 compatible descriptors
    pub fn random(rng: &mut impl Rng) -> Self {
        let base = BaseTerrain::iter()
            .collect::<Vec<_>>()
            .choose(rng)
            .copied()
            .unwrap();

        let descriptors = Self::compatible_descriptors_for(base, rng);

        TerrainType { base, descriptors }
    }

    /// Generate safe/neutral terrain (Clearing or Grasslands)
    pub fn random_safe(rng: &mut impl Rng) -> Self {
        let base = if rng.random_bool(0.5) {
            BaseTerrain::Clearing
        } else {
            BaseTerrain::Grasslands
        };

        let descriptors = Self::compatible_descriptors_for(base, rng);
        TerrainType { base, descriptors }
    }

    /// Generate random Moderate harshness terrain
    pub fn random_moderate(rng: &mut impl Rng) -> Self {
        let moderate_terrains = [
            BaseTerrain::Forest,
            BaseTerrain::Jungle,
            BaseTerrain::UrbanRuins,
            BaseTerrain::Wetlands,
            BaseTerrain::Highlands,
            BaseTerrain::Geothermal,
        ];

        let base = moderate_terrains.choose(rng).copied().unwrap();
        let descriptors = Self::compatible_descriptors_for(base, rng);

        TerrainType { base, descriptors }
    }

    /// Generate random descriptors compatible with base terrain (0-2 descriptors)
    fn compatible_descriptors_for(base: BaseTerrain, rng: &mut impl Rng) -> Vec<TerrainDescriptor> {
        use BaseTerrain::*;
        use TerrainDescriptor::*;

        let compatible = match base {
            Desert => vec![Hot, Cold, Rocky, Sandy, HighAltitude, Dry],
            Tundra => vec![Cold, Frozen, Rocky, HighAltitude, Sparse],
            Forest => vec![Dense, Sparse, Wet, Dry, Temperate, Overgrown],
            Jungle => vec![Dense, Hot, Wet, Overgrown, Lowland],
            Wetlands => vec![Wet, Dense, Lowland, Overgrown],
            Mountains => vec![Rocky, HighAltitude, Sparse, Cold],
            UrbanRuins => vec![Open, Sparse, Rocky],
            Clearing => vec![Open, Temperate, Dry],
            Grasslands => vec![Open, Temperate, Dry, Sparse],
            Badlands => vec![Rocky, Dry, Hot, Sparse],
            Highlands => vec![Rocky, HighAltitude, Sparse, Cold, Temperate],
            Geothermal => vec![Hot, Rocky, Sparse],
        };

        // Pick 0-2 descriptors randomly
        let count = rng.random_range(0..=2);
        compatible.choose_multiple(rng, count).copied().collect()
    }
}

/// Ensure max 3 Harsh terrains per game
pub fn enforce_balance_constraint(terrains: &mut [TerrainType], rng: &mut impl Rng) {
    let harsh_count = terrains
        .iter()
        .filter(|t| matches!(t.base.harshness(), Harshness::Harsh))
        .count();

    if harsh_count > 3 {
        // Find harsh terrain indices
        let harsh_indices: Vec<usize> = terrains
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t.base.harshness(), Harshness::Harsh))
            .map(|(i, _)| i)
            .collect();

        // Reroll extras to Moderate terrains
        let to_reroll = harsh_count - 3;
        let reroll_indices: Vec<usize> = harsh_indices
            .choose_multiple(rng, to_reroll)
            .copied()
            .collect();

        for idx in reroll_indices {
            // Reroll to Moderate harshness terrain
            terrains[idx] = TerrainType::random_moderate(rng);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        enforce_balance_constraint(&mut terrains, &mut rng);

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

    #[test]
    fn test_random_moderate_only_generates_moderate_terrains() {
        let mut rng = SmallRng::seed_from_u64(77777);

        for _ in 0..50 {
            let terrain = TerrainType::random_moderate(&mut rng);
            assert!(
                matches!(terrain.base.harshness(), Harshness::Moderate),
                "Expected Moderate harshness, got {:?} for {:?}",
                terrain.base.harshness(),
                terrain.base
            );
        }
    }
}
