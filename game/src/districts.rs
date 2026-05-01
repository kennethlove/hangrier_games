use crate::terrain::BaseTerrain;
use rand::Rng;
use rand::RngExt;

/// Represents a district's industry and terrain preferences
#[derive(Debug, Clone, Copy)]
pub struct DistrictProfile {
    pub number: u8,
    pub industry: &'static str,
    pub primary_affinity: BaseTerrain,
    pub bonus_affinity_pool: [BaseTerrain; 2],
}

/// All 12 district profiles with their industries and terrain affinities
pub const DISTRICT_PROFILES: [DistrictProfile; 12] = [
    DistrictProfile {
        number: 1,
        industry: "Luxury",
        primary_affinity: BaseTerrain::UrbanRuins,
        bonus_affinity_pool: [BaseTerrain::Clearing, BaseTerrain::Grasslands],
    },
    DistrictProfile {
        number: 2,
        industry: "Masonry",
        primary_affinity: BaseTerrain::Mountains,
        bonus_affinity_pool: [BaseTerrain::UrbanRuins, BaseTerrain::Badlands],
    },
    DistrictProfile {
        number: 3,
        industry: "Technology",
        primary_affinity: BaseTerrain::UrbanRuins,
        bonus_affinity_pool: [BaseTerrain::Mountains, BaseTerrain::Clearing],
    },
    DistrictProfile {
        number: 4,
        industry: "Fishing",
        primary_affinity: BaseTerrain::Wetlands,
        bonus_affinity_pool: [BaseTerrain::Forest, BaseTerrain::Jungle],
    },
    DistrictProfile {
        number: 5,
        industry: "Power",
        primary_affinity: BaseTerrain::Geothermal,
        bonus_affinity_pool: [BaseTerrain::UrbanRuins, BaseTerrain::Mountains],
    },
    DistrictProfile {
        number: 6,
        industry: "Transportation",
        primary_affinity: BaseTerrain::Grasslands,
        bonus_affinity_pool: [BaseTerrain::Clearing, BaseTerrain::Highlands],
    },
    DistrictProfile {
        number: 7,
        industry: "Lumber",
        primary_affinity: BaseTerrain::Forest,
        bonus_affinity_pool: [BaseTerrain::Jungle, BaseTerrain::Wetlands],
    },
    DistrictProfile {
        number: 8,
        industry: "Textiles",
        primary_affinity: BaseTerrain::Grasslands,
        bonus_affinity_pool: [BaseTerrain::Clearing, BaseTerrain::Wetlands],
    },
    DistrictProfile {
        number: 9,
        industry: "Grain",
        primary_affinity: BaseTerrain::Grasslands,
        bonus_affinity_pool: [BaseTerrain::Clearing, BaseTerrain::Highlands],
    },
    DistrictProfile {
        number: 10,
        industry: "Livestock",
        primary_affinity: BaseTerrain::Grasslands,
        bonus_affinity_pool: [BaseTerrain::Highlands, BaseTerrain::Badlands],
    },
    DistrictProfile {
        number: 11,
        industry: "Agriculture",
        primary_affinity: BaseTerrain::Grasslands,
        bonus_affinity_pool: [BaseTerrain::Clearing, BaseTerrain::Forest],
    },
    DistrictProfile {
        number: 12,
        industry: "Mining",
        primary_affinity: BaseTerrain::Tundra,
        bonus_affinity_pool: [BaseTerrain::Mountains, BaseTerrain::Badlands],
    },
];

/// Assigns terrain affinity to a tribute based on their district
///
/// # Arguments
/// * `district` - District number (1-12)
/// * `rng` - Random number generator
///
/// # Returns
/// Vec containing 1-2 terrain types:
/// - Always includes the primary affinity
/// - 40% chance to add one random terrain from the bonus pool
pub fn assign_terrain_affinity(district: u8, rng: &mut impl Rng) -> Vec<BaseTerrain> {
    // Handle invalid districts by returning empty vec
    if !(1..=12).contains(&district) {
        return vec![];
    }

    let profile = &DISTRICT_PROFILES[(district - 1) as usize];
    let mut affinities = vec![profile.primary_affinity];

    // 40% chance to add a bonus terrain
    if rng.random_bool(0.4) {
        let bonus_index = rng.random_range(0..profile.bonus_affinity_pool.len());
        affinities.push(profile.bonus_affinity_pool[bonus_index]);
    }

    affinities
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::prelude::SmallRng;
    use rstest::rstest;

    #[test]
    fn test_district_profiles_count() {
        assert_eq!(DISTRICT_PROFILES.len(), 12);
    }

    #[rstest]
    #[case(1, "Luxury", BaseTerrain::UrbanRuins)]
    #[case(2, "Masonry", BaseTerrain::Mountains)]
    #[case(3, "Technology", BaseTerrain::UrbanRuins)]
    #[case(4, "Fishing", BaseTerrain::Wetlands)]
    #[case(5, "Power", BaseTerrain::Geothermal)]
    #[case(6, "Transportation", BaseTerrain::Grasslands)]
    #[case(7, "Lumber", BaseTerrain::Forest)]
    #[case(8, "Textiles", BaseTerrain::Grasslands)]
    #[case(9, "Grain", BaseTerrain::Grasslands)]
    #[case(10, "Livestock", BaseTerrain::Grasslands)]
    #[case(11, "Agriculture", BaseTerrain::Grasslands)]
    #[case(12, "Mining", BaseTerrain::Tundra)]
    fn test_district_profile_data(
        #[case] district: u8,
        #[case] expected_industry: &str,
        #[case] expected_primary: BaseTerrain,
    ) {
        let profile = &DISTRICT_PROFILES[(district - 1) as usize];
        assert_eq!(profile.number, district);
        assert_eq!(profile.industry, expected_industry);
        assert_eq!(profile.primary_affinity, expected_primary);
        assert_eq!(profile.bonus_affinity_pool.len(), 2);
    }

    #[test]
    fn test_assign_terrain_affinity_always_includes_primary() {
        let mut rng = SmallRng::seed_from_u64(42);

        for district in 1..=12 {
            let affinities = assign_terrain_affinity(district, &mut rng);
            assert!(!affinities.is_empty());

            let profile = &DISTRICT_PROFILES[(district - 1) as usize];
            assert_eq!(affinities[0], profile.primary_affinity);
        }
    }

    #[test]
    fn test_assign_terrain_affinity_returns_one_or_two() {
        let mut rng = SmallRng::seed_from_u64(42);

        for district in 1..=12 {
            let affinities = assign_terrain_affinity(district, &mut rng);
            assert!(!affinities.is_empty() && affinities.len() <= 2);
        }
    }

    #[test]
    fn test_assign_terrain_affinity_bonus_probability() {
        let iterations = 1000;
        let mut bonus_count = 0;

        for i in 0..iterations {
            let mut rng = SmallRng::seed_from_u64(i);
            let affinities = assign_terrain_affinity(1, &mut rng);

            if affinities.len() == 2 {
                bonus_count += 1;
            }
        }

        // With 1000 iterations, expect roughly 350-450 bonuses (40% ± 5%)
        let percentage = (bonus_count as f64 / iterations as f64) * 100.0;
        assert!(
            (35.0..=45.0).contains(&percentage),
            "Expected ~40% bonus rate, got {:.1}%",
            percentage
        );
    }

    #[test]
    fn test_assign_terrain_affinity_invalid_district() {
        let mut rng = SmallRng::seed_from_u64(42);

        assert_eq!(assign_terrain_affinity(0, &mut rng).len(), 0);
        assert_eq!(assign_terrain_affinity(13, &mut rng).len(), 0);
        assert_eq!(assign_terrain_affinity(255, &mut rng).len(), 0);
    }
}
