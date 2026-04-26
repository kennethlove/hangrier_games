//! Tribute trait system. Replaces `BrainPersonality`. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §5.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
