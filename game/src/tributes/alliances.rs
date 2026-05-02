//! Tribute alliance formation, breaks, and event queue. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §6–§7.
//!
//! Pure functions only. Phase 2 of the tribute-alliances feature. No
//! `Tribute` mutation lives here; later phases wire these helpers into
//! the simulation loop.

use rand::RngExt;
use uuid::Uuid;

use crate::tributes::traits::{REFUSERS, Trait, geometric_mean_affinity};

/// Per-tribute hard cap on direct alliances.
pub const MAX_ALLIES: usize = 5;
/// Base chance per encounter that two tributes form an alliance.
pub const BASE_ALLIANCE_CHANCE: f64 = 0.20;
/// Treacherous betrayal cadence in turns.
pub const TREACHEROUS_BETRAYAL_INTERVAL: u8 = 5;

/// Events emitted by tribute turns and drained by the game cycle. Pure data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllianceEvent {
    /// A successful alliance proposal during the proposer's turn. The game
    /// loop applies the symmetric `allies` push to both tributes and emits
    /// the `AllianceFormed` message. Carrying the deciding factor lets the
    /// game-side message reuse the same prose as the legacy pre-pass.
    FormationRecorded {
        proposer: Uuid,
        target: Uuid,
        factor: String,
    },
    BetrayalRecorded {
        betrayer: Uuid,
        victim: Uuid,
    },
    DeathRecorded {
        deceased: Uuid,
        killer: Option<Uuid>,
    },
}

/// Refuser gate per spec §6.1. Two tributes pass the gate if either both
/// have at least one positive-affinity trait, or neither has any refuser
/// trait. Empty trait sets pass (no refusers).
pub fn passes_gate(self_traits: &[Trait], target_traits: &[Trait]) -> bool {
    let has_positive = |ts: &[Trait]| ts.iter().any(|x| x.alliance_affinity() >= 1.0);
    let has_refuser = |ts: &[Trait]| ts.iter().any(|x| REFUSERS.contains(x));
    (has_positive(self_traits) && has_positive(target_traits))
        || (!has_refuser(self_traits) && !has_refuser(target_traits))
}

/// Roll chance per spec §6.2. `self_allies_len` and `target_allies_len`
/// are the current `Vec::len()` of each tribute's `allies` list. Returns
/// 0.0 if either side is at the cap; clamped at 0.95.
pub fn roll_chance(
    self_traits: &[Trait],
    target_traits: &[Trait],
    same_district: bool,
    self_allies_len: usize,
    target_allies_len: usize,
) -> f64 {
    let trait_factor = geometric_mean_affinity(self_traits);
    let target_factor = geometric_mean_affinity(target_traits);
    let district_bonus = if same_district { 1.5 } else { 1.0 };
    let self_cap_pen = ((MAX_ALLIES as f64) - (self_allies_len as f64)) / (MAX_ALLIES as f64);
    let target_cap_pen = ((MAX_ALLIES as f64) - (target_allies_len as f64)) / (MAX_ALLIES as f64);
    let raw = BASE_ALLIANCE_CHANCE
        * trait_factor
        * target_factor
        * district_bonus
        * self_cap_pen.max(0.0)
        * target_cap_pen.max(0.0);
    raw.clamp(0.0, 0.95)
}

/// Human-readable deciding factor for a successful alliance roll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecidingFactor {
    SameDistrict,
    TraitOnSelf(Trait),
    TraitOnTarget(Trait),
}

impl DecidingFactor {
    pub fn label(&self) -> &'static str {
        match self {
            DecidingFactor::SameDistrict => "same district",
            DecidingFactor::TraitOnSelf(t) | DecidingFactor::TraitOnTarget(t) => t.label(),
        }
    }
}

/// Returns the deciding factor for a successful alliance roll, or `None`
/// if no factor exceeded 1.0. Same-district contributes a 1.5 weight; each
/// trait contributes its `alliance_affinity`. Ties break by label sort to
/// keep test output deterministic.
pub fn deciding_factor(
    self_traits: &[Trait],
    target_traits: &[Trait],
    same_district: bool,
) -> Option<DecidingFactor> {
    let mut candidates: Vec<(f64, DecidingFactor)> = Vec::new();
    if same_district {
        candidates.push((1.5, DecidingFactor::SameDistrict));
    }
    for t in self_traits {
        let a = t.alliance_affinity();
        if a > 1.0 {
            candidates.push((a, DecidingFactor::TraitOnSelf(*t)));
        }
    }
    for t in target_traits {
        let a = t.alliance_affinity();
        if a > 1.0 {
            candidates.push((a, DecidingFactor::TraitOnTarget(*t)));
        }
    }
    candidates.sort_by(|(a, df_a), (b, df_b)| {
        b.partial_cmp(a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| df_a.label().cmp(df_b.label()))
    });
    candidates.into_iter().next().map(|(_, df)| df)
}

/// Per-ally sanity-break roll (spec §7.3a). Returns `true` if the
/// symmetric pair should be removed. Probability scales linearly with
/// the deficit below `low_sanity_limit`; at or above the limit the
/// function always returns `false`.
pub fn sanity_break_roll(
    current_sanity: u32,
    low_sanity_limit: u32,
    rng: &mut impl rand::Rng,
) -> bool {
    if current_sanity >= low_sanity_limit {
        return false;
    }
    let deficit_ratio =
        (low_sanity_limit.saturating_sub(current_sanity) as f64) / (low_sanity_limit.max(1) as f64);
    let p = deficit_ratio.clamp(0.0, 1.0);
    rng.random_bool(p)
}

/// Trust-shock roll for a betrayal victim (spec §7.3c1). Same threshold
/// gating as `sanity_break_roll` but with a higher baseline of
/// `0.5 + 0.5 * deficit_ratio`.
pub fn trust_shock_roll(
    current_sanity: u32,
    low_sanity_limit: u32,
    rng: &mut impl rand::Rng,
) -> bool {
    if current_sanity >= low_sanity_limit {
        return false;
    }
    let deficit_ratio =
        (low_sanity_limit.saturating_sub(current_sanity) as f64) / (low_sanity_limit.max(1) as f64);
    let p = (0.5 + 0.5 * deficit_ratio).clamp(0.0, 1.0);
    rng.random_bool(p)
}

/// Attempt to form an alliance between two tributes. Returns `true` on
/// success — gate passes, `roll_chance` is positive, and the dice roll
/// hits. The caller is responsible for mutating both sides' `allies`
/// lists and for fetching a [`DecidingFactor`] via [`deciding_factor`]
/// for human-readable messaging. Composes [`passes_gate`] and
/// [`roll_chance`] so the game cycle has a single integration point per
/// spec §6.
pub fn try_form_alliance(
    self_traits: &[Trait],
    target_traits: &[Trait],
    same_district: bool,
    self_allies_len: usize,
    target_allies_len: usize,
    rng: &mut impl rand::Rng,
) -> bool {
    if !passes_gate(self_traits, target_traits) {
        return false;
    }
    let chance = roll_chance(
        self_traits,
        target_traits,
        same_district,
        self_allies_len,
        target_allies_len,
    );
    if chance <= 0.0 {
        return false;
    }
    rng.random_bool(chance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn alliance_event_variants_distinct() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let betrayal = AllianceEvent::BetrayalRecorded {
            betrayer: a,
            victim: b,
        };
        let death = AllianceEvent::DeathRecorded {
            deceased: a,
            killer: Some(b),
        };
        assert_ne!(betrayal, death);
        let betrayal2 = AllianceEvent::BetrayalRecorded {
            betrayer: a,
            victim: b,
        };
        assert_eq!(betrayal, betrayal2);
    }

    #[test]
    fn death_event_killer_optional() {
        let id = Uuid::new_v4();
        let unattributed = AllianceEvent::DeathRecorded {
            deceased: id,
            killer: None,
        };
        if let AllianceEvent::DeathRecorded { killer, .. } = unattributed {
            assert!(killer.is_none());
        } else {
            panic!("expected DeathRecorded");
        }
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAX_ALLIES, 5);
        assert!((BASE_ALLIANCE_CHANCE - 0.20).abs() < f64::EPSILON);
        assert_eq!(TREACHEROUS_BETRAYAL_INTERVAL, 5);
    }

    #[test]
    fn paranoid_vs_paranoid_blocked() {
        assert!(!passes_gate(&[Trait::Paranoid], &[Trait::Paranoid]));
    }

    #[test]
    fn lonewolf_vs_friendly_blocked() {
        // LoneWolf affinity 0.6 (no positive) and is a refuser; Friendly is
        // 1.5. (positive AND positive) is false; (no_refuser AND no_refuser)
        // is false because LoneWolf is a refuser. Gate blocks.
        assert!(!passes_gate(&[Trait::LoneWolf], &[Trait::Friendly]));
    }

    #[test]
    fn snake_in_grass_passes_gate() {
        // [Friendly, Paranoid] paired with [Loyal]: both sides have a
        // positive-affinity trait, so the first clause holds.
        assert!(passes_gate(
            &[Trait::Friendly, Trait::Paranoid],
            &[Trait::Loyal],
        ));
    }

    #[test]
    fn empty_traits_pass_gate() {
        assert!(passes_gate(&[], &[]));
    }

    #[test]
    fn neutral_pair_passes_gate() {
        // Tough has affinity 1.0 and is not a refuser; both clauses hold.
        assert!(passes_gate(&[Trait::Tough], &[Trait::Tough]));
    }

    // ---- Task 2.3: roll_chance -------------------------------------------

    #[test]
    fn roll_chance_zero_when_self_at_cap() {
        let chance = roll_chance(&[Trait::Friendly], &[Trait::Friendly], true, MAX_ALLIES, 0);
        assert_eq!(chance, 0.0);
    }

    #[test]
    fn roll_chance_zero_when_target_at_cap() {
        let chance = roll_chance(&[Trait::Friendly], &[Trait::Friendly], true, 0, MAX_ALLIES);
        assert_eq!(chance, 0.0);
    }

    #[test]
    fn roll_chance_neutral_pair_at_base() {
        // 0.20 * 1.0 * 1.0 * 1.0 * 1.0 * 1.0 = 0.20.
        let chance = roll_chance(&[], &[], false, 0, 0);
        assert!((chance - 0.20).abs() < 1e-9);
    }

    #[test]
    fn roll_chance_friendly_same_district_higher_than_base() {
        // 0.20 * 1.5 * 1.5 * 1.5 = 0.675.
        let chance = roll_chance(&[Trait::Friendly], &[Trait::Friendly], true, 0, 0);
        assert!(chance > 0.6 && chance <= 0.95);
    }

    #[test]
    fn roll_chance_clamps_at_ceiling() {
        let chance = roll_chance(
            &[Trait::Friendly, Trait::Friendly, Trait::Friendly],
            &[Trait::Friendly, Trait::Friendly, Trait::Friendly],
            true,
            0,
            0,
        );
        assert!(chance <= 0.95 + 1e-9);
    }

    #[test]
    fn roll_chance_symmetric_in_traits() {
        let a = roll_chance(&[Trait::Friendly], &[Trait::Loyal], true, 0, 0);
        let b = roll_chance(&[Trait::Loyal], &[Trait::Friendly], true, 0, 0);
        assert!((a - b).abs() < 1e-9);
    }

    // ---- Task 2.4: deciding_factor ---------------------------------------

    #[test]
    fn deciding_factor_picks_largest_above_one() {
        let f = deciding_factor(&[Trait::Friendly], &[Trait::Loyal], true);
        assert!(f.is_some());
    }

    #[test]
    fn deciding_factor_none_when_nothing_exceeds_one() {
        let f = deciding_factor(&[], &[], false);
        assert!(f.is_none());
    }

    #[test]
    fn deciding_factor_friendly_beats_loyal() {
        // Friendly 1.5 > Loyal 1.4. Without same_district, only the trait
        // candidates are in play; Friendly wins.
        let f = deciding_factor(&[Trait::Friendly], &[Trait::Loyal], false);
        match f {
            Some(DecidingFactor::TraitOnSelf(Trait::Friendly)) => {}
            other => panic!("expected TraitOnSelf(Friendly), got {other:?}"),
        }
    }

    #[test]
    fn deciding_factor_neutral_only_loses_to_district() {
        // No trait > 1.0, but same_district adds 1.5 → SameDistrict wins.
        let f = deciding_factor(&[Trait::Tough], &[Trait::Tough], true);
        assert_eq!(f, Some(DecidingFactor::SameDistrict));
    }

    // ---- Task 2.5: sanity_break_roll -------------------------------------

    #[test]
    fn sanity_break_above_limit_no_break() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..32 {
            assert!(!sanity_break_roll(50, 20, &mut rng));
        }
    }

    #[test]
    fn sanity_break_far_below_always_breaks() {
        let mut rng = StdRng::seed_from_u64(1);
        // Sanity 0 vs limit 20: deficit ratio 1.0 → p=1.0 → always breaks.
        for _ in 0..32 {
            assert!(sanity_break_roll(0, 20, &mut rng));
        }
    }

    #[test]
    fn sanity_break_at_limit_no_break() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..32 {
            assert!(!sanity_break_roll(20, 20, &mut rng));
        }
    }

    #[test]
    fn sanity_break_zero_limit_safe() {
        let mut rng = StdRng::seed_from_u64(1);
        // Degenerate limit must not panic and must not break (sanity >= 0).
        assert!(!sanity_break_roll(0, 0, &mut rng));
    }

    // ---- Task 2.6: trust_shock_roll --------------------------------------

    #[test]
    fn trust_shock_above_limit_no_break() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..32 {
            assert!(!trust_shock_roll(50, 20, &mut rng));
        }
    }

    #[test]
    fn trust_shock_below_limit_high_baseline() {
        let mut rng = StdRng::seed_from_u64(7);
        let mut breaks = 0;
        for _ in 0..200 {
            if trust_shock_roll(10, 20, &mut rng) {
                breaks += 1;
            }
        }
        // p = 0.5 + 0.5 * 0.5 = 0.75; expect majority to break.
        assert!(breaks > 100, "expected most to break, got {breaks}");
    }

    #[test]
    fn trust_shock_at_limit_no_break() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..32 {
            assert!(!trust_shock_roll(20, 20, &mut rng));
        }
    }

    // ---- Phase 6: try_form_alliance integration helper -------------------

    #[test]
    fn try_form_alliance_returns_false_when_gate_blocks() {
        // LoneWolf vs Friendly fails passes_gate; helper must short-circuit.
        let mut rng = StdRng::seed_from_u64(313);
        let formed =
            try_form_alliance(&[Trait::LoneWolf], &[Trait::Friendly], true, 0, 0, &mut rng);
        assert!(!formed);
    }

    #[test]
    fn try_form_alliance_returns_false_when_either_at_cap() {
        let mut rng = StdRng::seed_from_u64(419);
        let r1 = try_form_alliance(
            &[Trait::Friendly],
            &[Trait::Friendly],
            true,
            MAX_ALLIES,
            0,
            &mut rng,
        );
        assert!(!r1, "self at cap blocks");
        let r2 = try_form_alliance(
            &[Trait::Friendly],
            &[Trait::Friendly],
            true,
            0,
            MAX_ALLIES,
            &mut rng,
        );
        assert!(!r2, "target at cap blocks");
    }

    #[test]
    fn try_form_alliance_can_succeed_for_high_chance_pair() {
        // Friendly + same district + 0 allies → ~0.675 chance. Sample many
        // trials and assert at least some succeed.
        let mut rng = StdRng::seed_from_u64(547);
        let mut successes = 0;
        for _ in 0..200 {
            if try_form_alliance(&[Trait::Friendly], &[Trait::Friendly], true, 0, 0, &mut rng) {
                successes += 1;
            }
        }
        assert!(
            successes > 100,
            "expected majority success at p≈0.675, got {successes}"
        );
    }

    #[test]
    fn try_form_alliance_zero_chance_never_forms() {
        // Both at cap → roll_chance = 0.0 → never forms.
        let mut rng = StdRng::seed_from_u64(101);
        for _ in 0..32 {
            assert!(!try_form_alliance(
                &[],
                &[],
                false,
                MAX_ALLIES,
                MAX_ALLIES,
                &mut rng
            ));
        }
    }
}
