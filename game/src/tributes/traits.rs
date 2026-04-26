//! Tribute trait system. Replaces `BrainPersonality`. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §5.

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    // Combat stance
    Aggressive,
    Defensive,
    Cautious,
    Reckless,
    // Social
    Friendly,
    Loyal,
    Paranoid,
    LoneWolf,
    Treacherous,
    // Mental
    Resilient,
    Fragile,
    Cunning,
    Dim,
    // Physical
    Asthmatic,
    Nearsighted,
    Tough,
}

impl Trait {
    pub fn label(&self) -> &'static str {
        match self {
            Trait::Aggressive => "aggressive",
            Trait::Defensive => "defensive",
            Trait::Cautious => "cautious",
            Trait::Reckless => "reckless",
            Trait::Friendly => "friendly",
            Trait::Loyal => "loyal",
            Trait::Paranoid => "paranoid",
            Trait::LoneWolf => "a lone wolf",
            Trait::Treacherous => "treacherous",
            Trait::Resilient => "resilient",
            Trait::Fragile => "fragile",
            Trait::Cunning => "cunning",
            Trait::Dim => "dim",
            Trait::Asthmatic => "asthmatic",
            Trait::Nearsighted => "nearsighted",
            Trait::Tough => "tough",
        }
    }

    pub fn alliance_affinity(&self) -> f64 {
        match self {
            Trait::Friendly => 1.5,
            Trait::Loyal => 1.4,
            Trait::Treacherous => 1.2,
            Trait::LoneWolf => 0.6,
            Trait::Paranoid => 0.5,
            _ => 1.0,
        }
    }
}

pub const REFUSERS: &[Trait] = &[Trait::Paranoid, Trait::LoneWolf];

/// Geometric mean of trait affinity values. Returns 1.0 for empty input.
pub fn geometric_mean_affinity(traits: &[Trait]) -> f64 {
    if traits.is_empty() {
        return 1.0;
    }
    let n = traits.len() as f64;
    let product: f64 = traits.iter().map(|t| t.alliance_affinity()).product();
    product.powf(1.0 / n)
}

pub const CONFLICTS: &[(Trait, Trait)] = &[
    (Trait::Friendly, Trait::Paranoid),
    (Trait::Loyal, Trait::Treacherous),
    (Trait::Loyal, Trait::LoneWolf),
    (Trait::Aggressive, Trait::Cautious),
    (Trait::Aggressive, Trait::Defensive),
    (Trait::Reckless, Trait::Cautious),
    (Trait::Resilient, Trait::Fragile),
    (Trait::Cunning, Trait::Dim),
];

pub fn conflicts_with(a: Trait, b: Trait) -> bool {
    CONFLICTS
        .iter()
        .any(|(x, y)| (*x == a && *y == b) || (*x == b && *y == a))
}

pub const DISTRICT_1_POOL: &[(Trait, u8)] = &[
    (Trait::Loyal, 4),
    (Trait::Aggressive, 4),
    (Trait::Paranoid, 3),
    (Trait::Tough, 2),
];
pub const DISTRICT_2_POOL: &[(Trait, u8)] = &[
    (Trait::Aggressive, 4),
    (Trait::Defensive, 4),
    (Trait::Loyal, 3),
    (Trait::Tough, 2),
];
pub const DISTRICT_3_POOL: &[(Trait, u8)] = &[
    (Trait::Cunning, 4),
    (Trait::Cautious, 3),
    (Trait::Dim, 2),
    (Trait::Nearsighted, 2),
    (Trait::Asthmatic, 1),
];
pub const DISTRICT_4_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4),
    (Trait::Aggressive, 3),
    (Trait::Loyal, 3),
    (Trait::Tough, 2),
];
pub const DISTRICT_5_POOL: &[(Trait, u8)] = &[
    (Trait::Cunning, 4),
    (Trait::Cautious, 3),
    (Trait::Treacherous, 2),
];
pub const DISTRICT_6_POOL: &[(Trait, u8)] = &[
    (Trait::Fragile, 3),
    (Trait::Friendly, 3),
    (Trait::Asthmatic, 2),
    (Trait::Nearsighted, 2),
];
pub const DISTRICT_7_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4),
    (Trait::Defensive, 3),
    (Trait::Tough, 3),
];
pub const DISTRICT_8_POOL: &[(Trait, u8)] = &[
    (Trait::Fragile, 2),
    (Trait::Friendly, 4),
    (Trait::Loyal, 3),
    (Trait::Asthmatic, 2),
];
pub const DISTRICT_9_POOL: &[(Trait, u8)] = &[
    (Trait::Cautious, 3),
    (Trait::Friendly, 3),
    (Trait::Asthmatic, 2),
];
pub const DISTRICT_10_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4),
    (Trait::Defensive, 3),
    (Trait::Tough, 3),
];
pub const DISTRICT_11_POOL: &[(Trait, u8)] = &[
    (Trait::Loyal, 3),
    (Trait::Friendly, 4),
    (Trait::Resilient, 3),
    (Trait::Tough, 2),
];
pub const DISTRICT_12_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 3),
    (Trait::LoneWolf, 3),
    (Trait::Cunning, 3),
    (Trait::Asthmatic, 2),
];

pub fn pool_for(district: u8) -> &'static [(Trait, u8)] {
    match district {
        1 => DISTRICT_1_POOL,
        2 => DISTRICT_2_POOL,
        3 => DISTRICT_3_POOL,
        4 => DISTRICT_4_POOL,
        5 => DISTRICT_5_POOL,
        6 => DISTRICT_6_POOL,
        7 => DISTRICT_7_POOL,
        8 => DISTRICT_8_POOL,
        9 => DISTRICT_9_POOL,
        10 => DISTRICT_10_POOL,
        11 => DISTRICT_11_POOL,
        12 => DISTRICT_12_POOL,
        _ => DISTRICT_1_POOL,
    }
}

/// Generate a trait set for a tribute in `district`. Rolls 2–6 uniformly,
/// then draws weighted picks from the district pool, rejecting conflicts and
/// duplicates. Stops early if the pool cannot satisfy the count; never spins.
pub fn generate_traits(district: u8, rng: &mut impl Rng) -> Vec<Trait> {
    let pool = pool_for(district);
    let target_count = rng.random_range(2..=6);
    let mut chosen: Vec<Trait> = Vec::with_capacity(target_count);

    let mut remaining: Vec<(Trait, u8)> = pool.to_vec();

    while chosen.len() < target_count && !remaining.is_empty() {
        let total: u32 = remaining.iter().map(|(_, w)| *w as u32).sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.random_range(0..total);
        let mut picked_idx: Option<usize> = None;
        for (i, (_, w)) in remaining.iter().enumerate() {
            if roll < *w as u32 {
                picked_idx = Some(i);
                break;
            }
            roll -= *w as u32;
        }
        let idx = picked_idx.expect("weighted pick must succeed when total > 0");
        let (candidate, _) = remaining.remove(idx);

        if chosen.iter().any(|t| conflicts_with(*t, candidate)) {
            continue;
        }
        chosen.push(candidate);
    }

    chosen
}

/// Additive deltas applied to `PersonalityThresholds`. `i32` so deltas can be
/// signed; final values clamp to u32 ranges in `compute_thresholds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThresholdDelta {
    pub low_health_limit: i32,
    pub mid_health_limit: i32,
    pub low_sanity_limit: i32,
    pub mid_sanity_limit: i32,
    pub high_sanity_limit: i32,
    pub movement_limit: i32,
    pub low_intelligence_limit: i32,
    pub high_intelligence_limit: i32,
    pub psychotic_break_threshold: i32,
}

impl std::ops::Add for ThresholdDelta {
    type Output = ThresholdDelta;
    fn add(self, rhs: Self) -> Self {
        ThresholdDelta {
            low_health_limit: self.low_health_limit + rhs.low_health_limit,
            mid_health_limit: self.mid_health_limit + rhs.mid_health_limit,
            low_sanity_limit: self.low_sanity_limit + rhs.low_sanity_limit,
            mid_sanity_limit: self.mid_sanity_limit + rhs.mid_sanity_limit,
            high_sanity_limit: self.high_sanity_limit + rhs.high_sanity_limit,
            movement_limit: self.movement_limit + rhs.movement_limit,
            low_intelligence_limit: self.low_intelligence_limit + rhs.low_intelligence_limit,
            high_intelligence_limit: self.high_intelligence_limit + rhs.high_intelligence_limit,
            psychotic_break_threshold: self.psychotic_break_threshold
                + rhs.psychotic_break_threshold,
        }
    }
}

impl std::iter::Sum for ThresholdDelta {
    fn sum<I: Iterator<Item = ThresholdDelta>>(iter: I) -> Self {
        iter.fold(ThresholdDelta::default(), |a, b| a + b)
    }
}

impl Trait {
    pub fn threshold_modifiers(&self) -> ThresholdDelta {
        match self {
            Trait::Aggressive => ThresholdDelta {
                low_health_limit: -5,
                mid_health_limit: -10,
                low_sanity_limit: -2,
                psychotic_break_threshold: 2,
                ..Default::default()
            },
            Trait::Defensive => ThresholdDelta {
                low_health_limit: 10,
                mid_health_limit: 10,
                psychotic_break_threshold: -2,
                ..Default::default()
            },
            Trait::Cautious => ThresholdDelta {
                low_health_limit: 15,
                mid_health_limit: 15,
                low_sanity_limit: 10,
                mid_sanity_limit: 10,
                psychotic_break_threshold: -3,
                ..Default::default()
            },
            Trait::Reckless => ThresholdDelta {
                low_health_limit: -10,
                low_sanity_limit: -10,
                psychotic_break_threshold: 4,
                ..Default::default()
            },
            Trait::Resilient => ThresholdDelta {
                psychotic_break_threshold: -3,
                low_sanity_limit: -3,
                ..Default::default()
            },
            Trait::Fragile => ThresholdDelta {
                psychotic_break_threshold: 3,
                low_sanity_limit: 5,
                ..Default::default()
            },
            Trait::Cunning => ThresholdDelta {
                low_intelligence_limit: -5,
                high_intelligence_limit: -5,
                ..Default::default()
            },
            Trait::Dim => ThresholdDelta {
                low_intelligence_limit: 10,
                high_intelligence_limit: 5,
                ..Default::default()
            },
            // Social and physical traits leave thresholds untouched.
            _ => ThresholdDelta::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn affinity_known_values() {
        assert_eq!(Trait::Friendly.alliance_affinity(), 1.5);
        assert_eq!(Trait::Loyal.alliance_affinity(), 1.4);
        assert_eq!(Trait::Treacherous.alliance_affinity(), 1.2);
        assert_eq!(Trait::Aggressive.alliance_affinity(), 1.0);
        assert_eq!(Trait::Tough.alliance_affinity(), 1.0);
        assert_eq!(Trait::LoneWolf.alliance_affinity(), 0.6);
        assert_eq!(Trait::Paranoid.alliance_affinity(), 0.5);
    }

    #[test]
    fn refusers_membership() {
        assert!(REFUSERS.contains(&Trait::Paranoid));
        assert!(REFUSERS.contains(&Trait::LoneWolf));
        assert!(!REFUSERS.contains(&Trait::Friendly));
    }

    #[test]
    fn geometric_mean_empty_is_one() {
        assert_eq!(geometric_mean_affinity(&[]), 1.0);
    }

    #[test]
    fn geometric_mean_single_is_identity() {
        assert!((geometric_mean_affinity(&[Trait::Friendly]) - 1.5).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn geometric_mean_two_friendly_one_lonewolf() {
        let g = geometric_mean_affinity(&[Trait::Friendly, Trait::Friendly, Trait::LoneWolf]);
        let expected = (1.5_f64 * 1.5 * 0.6).powf(1.0 / 3.0);
        assert!((g - expected).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn conflict_symmetry() {
        let pairs = [
            (Trait::Friendly, Trait::Paranoid),
            (Trait::Loyal, Trait::Treacherous),
            (Trait::Loyal, Trait::LoneWolf),
            (Trait::Aggressive, Trait::Cautious),
            (Trait::Aggressive, Trait::Defensive),
            (Trait::Reckless, Trait::Cautious),
            (Trait::Resilient, Trait::Fragile),
            (Trait::Cunning, Trait::Dim),
        ];
        for (a, b) in pairs {
            assert!(conflicts_with(a, b), "{a:?} should conflict with {b:?}");
            assert!(
                conflicts_with(b, a),
                "{b:?} should conflict with {a:?} (symmetry)"
            );
        }
    }

    #[test]
    fn conflict_allowed_combos_do_not_conflict() {
        assert!(!conflicts_with(Trait::Friendly, Trait::Treacherous));
        assert!(!conflicts_with(Trait::Paranoid, Trait::LoneWolf));
    }

    #[test]
    fn pool_for_returns_correct_pool_per_district() {
        let p1 = pool_for(1);
        assert!(p1.iter().any(|(t, _)| *t == Trait::Loyal));
        let p12 = pool_for(12);
        assert!(p12.iter().any(|(t, _)| *t == Trait::LoneWolf));
    }

    #[test]
    fn pool_for_unknown_district_falls_back() {
        // Districts outside 1..=12 fall back to district 1's pool; assert non-panic.
        let _ = pool_for(99);
    }

    #[test]
    fn generate_respects_count_when_pool_supports() {
        let mut rng = StdRng::seed_from_u64(42);
        let traits = generate_traits(1, &mut rng);
        assert!(traits.len() >= 2 && traits.len() <= 6);
        for i in 0..traits.len() {
            for j in (i + 1)..traits.len() {
                assert!(!conflicts_with(traits[i], traits[j]));
            }
        }
    }

    #[test]
    fn generate_no_duplicates() {
        let mut rng = StdRng::seed_from_u64(7);
        let traits = generate_traits(2, &mut rng);
        let mut sorted: Vec<_> = traits.clone();
        sorted.sort_by_key(|t| *t as u8);
        sorted.dedup();
        assert_eq!(sorted.len(), traits.len());
    }

    #[test]
    fn threshold_delta_aggressive_lowers_health_threshold() {
        let d = Trait::Aggressive.threshold_modifiers();
        assert!(d.low_health_limit < 0);
    }

    #[test]
    fn threshold_delta_zero_traits_is_identity() {
        let total: ThresholdDelta = [].iter().map(|t: &Trait| t.threshold_modifiers()).sum();
        assert_eq!(total, ThresholdDelta::default());
    }
}
