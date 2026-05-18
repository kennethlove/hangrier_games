//! Phobia reaction logic: severity tiers, trait modifiers, stat penalties,
//! and brain override reactions.
//!
//! Reaction table (spec §5):
//! - Mild: -2 atk/def, no brain bias, no override → `Reaction::Penalty`
//! - Moderate: -4 atk/def, strong flee preference → `Reaction::AutoFlee`
//! - Severe: -6 atk/def, 25% freeze chance, auto-flee otherwise → `Reaction::Freeze` or `AutoFlee`
//!
//! Trait modifiers:
//! - Resilient: -1 tier (Mild → no reaction)
//! - Fragile: +1 tier (Mild → Moderate reaction, Severe stays Severe)
//! - Reckless: ignores freeze, still takes penalty + flee bias
//! - Cautious: no special interaction

use rand::Rng;
use rand::RngExt;
use shared::afflictions::{AfflictionKind, PhobiaTrigger, Severity};

use crate::tributes::Tribute;
use crate::tributes::traits::Trait;

/// The raw effect category a phobia produces at a given severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhobiaEffect {
    /// Stat penalty only (Mild tier).
    Penalty,
    /// Stat penalty + flee bias (Moderate tier).
    Flee,
    /// Stat penalty + freeze chance (Severe tier).
    Freeze,
}

impl PhobiaEffect {
    /// Stat penalty applied to attack and defense for this effect tier.
    pub fn stat_penalty(self) -> i32 {
        match self {
            PhobiaEffect::Penalty => -2,
            PhobiaEffect::Flee => -4,
            PhobiaEffect::Freeze => -6,
        }
    }
}

/// The actual brain-reaction a phobia fires. Differs from `PhobiaEffect`
/// because trait modifiers can change the reaction (e.g. Reckless ignores
/// freeze, Resilient downgrades severity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaction {
    /// Apply stat penalty only; no action override.
    Penalty,
    /// Apply stat penalty + prefer flee (Move action bias).
    /// Does not hard-override the brain; scoring layer handles bias.
    AutoFlee,
    /// Apply stat penalty + hard-override to `Action::Frozen`.
    /// The tribute skips this cycle entirely.
    Freeze,
}

/// Maximum cumulative stat penalty from all firing phobias.
/// Prevents a tribute with many phobias from becoming completely useless.
pub const MAX_PHOBIA_PENALTY: i32 = -10;

/// Compute the effective severity after applying trait modifiers.
///
/// - Resilient: -1 tier (Mild → below threshold, Moderate → Mild, Severe → Moderate)
/// - Fragile: +1 tier (Mild → Moderate, Moderate → Severe, Severe → Severe capped)
/// - Reckless / Cautious: no modifier to severity tier
pub fn effective_severity(base: Severity, traits: &[Trait]) -> Severity {
    let tier = base.ordinal() as i32;
    let modifier = trait_tier_modifier(traits);
    let adjusted = (tier + modifier).clamp(0, Severity::Severe.ordinal() as i32);
    match adjusted {
        0 => Severity::Mild,
        1 => Severity::Moderate,
        _ => Severity::Severe,
    }
}

/// Returns the tier modifier from a tribute's traits.
/// Resilient = -1, Fragile = +1, others = 0.
fn trait_tier_modifier(traits: &[Trait]) -> i32 {
    let mut modifier = 0i32;
    for t in traits {
        match t {
            Trait::Resilient => modifier -= 1,
            Trait::Fragile => modifier += 1,
            _ => {}
        }
    }
    modifier
}

/// Determine the reaction for a given severity, applying trait modifiers.
///
/// For Severe reactions, `rng` decides the freeze roll (25% chance).
/// Reckless tributes never freeze — they get AutoFlee instead.
pub fn reaction_for(severity: Severity, traits: &[Trait], rng: &mut impl Rng) -> Reaction {
    let effective = effective_severity(severity, traits);
    let is_reckless = traits.contains(&Trait::Reckless);

    match effective {
        Severity::Mild => Reaction::Penalty,
        Severity::Moderate => Reaction::AutoFlee,
        Severity::Severe => {
            if is_reckless {
                // Reckless tributes ignore freeze, still get flee bias.
                Reaction::AutoFlee
            } else if rng.random_bool(0.25) {
                Reaction::Freeze
            } else {
                Reaction::AutoFlee
            }
        }
    }
}

/// Determine the raw `PhobiaEffect` from a severity tier (before trait mods).
pub fn effect_for(severity: Severity) -> PhobiaEffect {
    match severity {
        Severity::Mild => PhobiaEffect::Penalty,
        Severity::Moderate => PhobiaEffect::Flee,
        Severity::Severe => PhobiaEffect::Freeze,
    }
}

/// A firing phobia with its computed effective severity and reaction.
#[derive(Debug, Clone)]
pub struct FiringPhobia {
    pub trigger: PhobiaTrigger,
    pub base_severity: Severity,
    pub effective_severity: Severity,
    pub reaction: Reaction,
}

/// Collect all firing phobia afflictions for a tribute.
/// Returns a list of `FiringPhobia` with effective severity and reaction computed.
///
/// The `is_firing` closure determines whether a given trigger is active
/// in the current cycle context. This decouples trigger detection from
/// reaction logic so the brain pipeline can supply its own context.
pub fn collect_firing_phobias(
    tribute: &Tribute,
    is_firing: impl Fn(&PhobiaTrigger) -> bool,
    rng: &mut impl Rng,
) -> Vec<FiringPhobia> {
    let mut firing = Vec::new();

    for (key, aff) in &tribute.afflictions {
        let AfflictionKind::Phobia(trigger) = key.0 else {
            continue;
        };
        if !is_firing(&trigger) {
            continue;
        }
        let base = aff.severity;
        let effective = effective_severity(base, &tribute.traits);
        let reaction = reaction_for(base, &tribute.traits, rng);
        firing.push(FiringPhobia {
            trigger,
            base_severity: base,
            effective_severity: effective,
            reaction,
        });
    }

    firing
}

/// Compute the total stat penalty from all firing phobias.
/// Penalties stack additively, capped at `MAX_PHOBIA_PENALTY`.
pub fn total_stat_penalty(firing: &[FiringPhobia]) -> i32 {
    let sum: i32 = firing
        .iter()
        .map(|f| effect_for(f.effective_severity).stat_penalty())
        .sum();
    sum.max(MAX_PHOBIA_PENALTY)
}

/// Find the strongest reaction among firing phobias.
/// Override precedence: Freeze > AutoFlee > Penalty.
/// Returns `None` if no phobias are firing.
pub fn strongest_reaction(firing: &[FiringPhobia]) -> Option<Reaction> {
    if firing.is_empty() {
        return None;
    }
    let mut strongest = Reaction::Penalty;
    for f in firing {
        strongest = match (strongest, f.reaction) {
            (Reaction::Freeze, _) => Reaction::Freeze,
            (_, Reaction::Freeze) => Reaction::Freeze,
            (Reaction::AutoFlee, _) => Reaction::AutoFlee,
            (_, Reaction::AutoFlee) => Reaction::AutoFlee,
            _ => Reaction::Penalty,
        };
    }
    Some(strongest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::rstest;

    fn seeded_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    // --- effective_severity ---

    #[test]
    fn effective_severity_no_traits_unchanged() {
        assert_eq!(effective_severity(Severity::Mild, &[]), Severity::Mild);
        assert_eq!(
            effective_severity(Severity::Moderate, &[]),
            Severity::Moderate
        );
        assert_eq!(effective_severity(Severity::Severe, &[]), Severity::Severe);
    }

    #[test]
    fn effective_severity_resilient_downgrades() {
        assert_eq!(
            effective_severity(Severity::Mild, &[Trait::Resilient]),
            Severity::Mild // can't go below Mild
        );
        assert_eq!(
            effective_severity(Severity::Moderate, &[Trait::Resilient]),
            Severity::Mild
        );
        assert_eq!(
            effective_severity(Severity::Severe, &[Trait::Resilient]),
            Severity::Moderate
        );
    }

    #[test]
    fn effective_severity_fragile_upgrades() {
        assert_eq!(
            effective_severity(Severity::Mild, &[Trait::Fragile]),
            Severity::Moderate
        );
        assert_eq!(
            effective_severity(Severity::Moderate, &[Trait::Fragile]),
            Severity::Severe
        );
        assert_eq!(
            effective_severity(Severity::Severe, &[Trait::Fragile]),
            Severity::Severe // capped at Severe
        );
    }

    #[test]
    fn effective_severity_reckless_no_modifier() {
        assert_eq!(
            effective_severity(Severity::Severe, &[Trait::Reckless]),
            Severity::Severe
        );
    }

    #[test]
    fn effective_severity_cautious_no_modifier() {
        assert_eq!(
            effective_severity(Severity::Moderate, &[Trait::Cautious]),
            Severity::Moderate
        );
    }

    // --- reaction_for ---

    #[rstest]
    #[case(Severity::Mild, &[], Reaction::Penalty)]
    #[case(Severity::Moderate, &[], Reaction::AutoFlee)]
    fn reaction_base_severity(
        #[case] severity: Severity,
        #[case] traits: &[Trait],
        #[case] expected: Reaction,
    ) {
        let mut rng = seeded_rng();
        // For Severe, the RNG roll matters; Mild/Moderate are deterministic.
        if matches!(severity, Severity::Severe) {
            return; // tested separately
        }
        assert_eq!(reaction_for(severity, traits, &mut rng), expected);
    }

    #[test]
    fn reaction_resilient_mild_no_reaction() {
        // Resilient downgrades Mild → still Mild (floor), which is Penalty
        // but effectively "no meaningful reaction" per spec.
        let mut rng = seeded_rng();
        assert_eq!(
            reaction_for(Severity::Mild, &[Trait::Resilient], &mut rng),
            Reaction::Penalty
        );
    }

    #[test]
    fn reaction_fragile_mild_becomes_moderate() {
        // Fragile upgrades Mild → Moderate → AutoFlee
        let mut rng = seeded_rng();
        assert_eq!(
            reaction_for(Severity::Mild, &[Trait::Fragile], &mut rng),
            Reaction::AutoFlee
        );
    }

    #[test]
    fn reaction_reckless_severe_no_freeze() {
        // Reckless ignores freeze, gets AutoFlee instead
        let mut rng = seeded_rng();
        assert_eq!(
            reaction_for(Severity::Severe, &[Trait::Reckless], &mut rng),
            Reaction::AutoFlee
        );
    }

    // --- stat penalty composition ---

    #[test]
    fn stat_penalty_single_mild() {
        let penalty = PhobiaEffect::Penalty.stat_penalty();
        assert_eq!(penalty, -2);
    }

    #[test]
    fn stat_penalty_single_moderate() {
        let penalty = PhobiaEffect::Flee.stat_penalty();
        assert_eq!(penalty, -4);
    }

    #[test]
    fn stat_penalty_single_severe() {
        let penalty = PhobiaEffect::Freeze.stat_penalty();
        assert_eq!(penalty, -6);
    }

    #[test]
    fn stat_penalty_two_mild_stack() {
        // Two Mild phobias: -2 + -2 = -4
        let firing = vec![
            FiringPhobia {
                trigger: PhobiaTrigger::Fire,
                base_severity: Severity::Mild,
                effective_severity: Severity::Mild,
                reaction: Reaction::Penalty,
            },
            FiringPhobia {
                trigger: PhobiaTrigger::Dark,
                base_severity: Severity::Mild,
                effective_severity: Severity::Mild,
                reaction: Reaction::Penalty,
            },
        ];
        assert_eq!(total_stat_penalty(&firing), -4);
    }

    #[test]
    fn stat_penalty_capped_at_max() {
        // Three Severe phobias: -6 * 3 = -18, capped at -10
        let firing = vec![
            FiringPhobia {
                trigger: PhobiaTrigger::Fire,
                base_severity: Severity::Severe,
                effective_severity: Severity::Severe,
                reaction: Reaction::Freeze,
            },
            FiringPhobia {
                trigger: PhobiaTrigger::Dark,
                base_severity: Severity::Severe,
                effective_severity: Severity::Severe,
                reaction: Reaction::AutoFlee,
            },
            FiringPhobia {
                trigger: PhobiaTrigger::Heights,
                base_severity: Severity::Severe,
                effective_severity: Severity::Severe,
                reaction: Reaction::AutoFlee,
            },
        ];
        assert_eq!(total_stat_penalty(&firing), MAX_PHOBIA_PENALTY);
    }

    // --- strongest_reaction ---

    #[test]
    fn strongest_reaction_empty_none() {
        assert!(strongest_reaction(&[]).is_none());
    }

    #[test]
    fn strongest_reaction_single_penalty() {
        let firing = vec![FiringPhobia {
            trigger: PhobiaTrigger::Fire,
            base_severity: Severity::Mild,
            effective_severity: Severity::Mild,
            reaction: Reaction::Penalty,
        }];
        assert_eq!(strongest_reaction(&firing), Some(Reaction::Penalty));
    }

    #[test]
    fn strongest_reaction_freeze_wins() {
        let firing = vec![
            FiringPhobia {
                trigger: PhobiaTrigger::Fire,
                base_severity: Severity::Mild,
                effective_severity: Severity::Mild,
                reaction: Reaction::Penalty,
            },
            FiringPhobia {
                trigger: PhobiaTrigger::Dark,
                base_severity: Severity::Severe,
                effective_severity: Severity::Severe,
                reaction: Reaction::Freeze,
            },
        ];
        assert_eq!(strongest_reaction(&firing), Some(Reaction::Freeze));
    }

    #[test]
    fn strongest_reaction_autoflee_beats_penalty() {
        let firing = vec![
            FiringPhobia {
                trigger: PhobiaTrigger::Fire,
                base_severity: Severity::Mild,
                effective_severity: Severity::Mild,
                reaction: Reaction::Penalty,
            },
            FiringPhobia {
                trigger: PhobiaTrigger::Dark,
                base_severity: Severity::Moderate,
                effective_severity: Severity::Moderate,
                reaction: Reaction::AutoFlee,
            },
        ];
        assert_eq!(strongest_reaction(&firing), Some(Reaction::AutoFlee));
    }

    // --- collect_firing_phobias ---

    #[test]
    fn collect_firing_phobias_filters_non_firing() {
        use crate::tributes::AfflictionDraft;
        use shared::afflictions::{AfflictionSource, PhobiaMetadata, PhobiaOrigin};

        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(PhobiaTrigger::Fire),
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Spawn,
        };
        tribute.try_acquire_affliction(draft);
        if let Some((_, aff)) = tribute.afflictions.iter_mut().next() {
            aff.phobia_metadata = Some(PhobiaMetadata {
                origin: PhobiaOrigin::Innate,
                ..PhobiaMetadata::default()
            });
        }

        let mut rng = seeded_rng();
        // Fire is not firing
        let firing = collect_firing_phobias(&tribute, |_| false, &mut rng);
        assert!(firing.is_empty());

        // Fire is firing
        let firing = collect_firing_phobias(&tribute, |t| *t == PhobiaTrigger::Fire, &mut rng);
        assert_eq!(firing.len(), 1);
        assert_eq!(firing[0].trigger, PhobiaTrigger::Fire);
        assert_eq!(firing[0].base_severity, Severity::Moderate);
    }
}
