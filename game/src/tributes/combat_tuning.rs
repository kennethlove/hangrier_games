//! Tunable knobs for combat: existing magic numbers (decisive-win multiplier,
//! stress contributions) plus the new stamina-as-combat-resource constants
//! introduced by `hangrier_games-93m`.
//!
//! All defaults preserve current behavior; tuning is a separate post-ship pass.
//! See `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatTuning {
    // --- Existing constants (verbatim from combat.rs:21-26) ---
    pub decisive_win_multiplier: f64,
    pub base_stress_no_engagements: f64,
    pub stress_sanity_normalization: f64,
    pub stress_final_divisor: f64,
    pub kill_stress_contribution: f64,
    pub non_kill_win_stress_contribution: f64,

    // --- Per-swing stamina costs (asymmetric: swinging is harder than defending) ---
    pub stamina_cost_attacker: u32,
    pub stamina_cost_target: u32,

    // --- Band thresholds (% of max_stamina; > => Fresh, > => Winded, else Exhausted) ---
    pub band_winded_pct: u8,
    pub band_exhausted_pct: u8,

    // --- Per-band roll penalties (subtracted from attack/defense rolls) ---
    pub winded_roll_penalty: i32,
    pub exhausted_roll_penalty: i32,

    // --- Per-phase recovery (gross, before survival-debuff multiplier) ---
    pub recovery_idle: u32,
    pub recovery_resting: u32,
    pub recovery_sheltered_resting: u32,
    pub recovery_starving_dehydrated_mult: f64,

    // --- Brain scoring nudges ---
    pub winded_attack_score_penalty: i32,
    pub fresh_target_visibly_tired_bonus: i32,
}

impl Default for CombatTuning {
    fn default() -> Self {
        Self {
            decisive_win_multiplier: 1.5,
            base_stress_no_engagements: 20.0,
            stress_sanity_normalization: 100.0,
            stress_final_divisor: 2.0,
            kill_stress_contribution: 50.0,
            non_kill_win_stress_contribution: 20.0,

            stamina_cost_attacker: 25,
            stamina_cost_target: 10,

            band_winded_pct: 50,
            band_exhausted_pct: 20,

            winded_roll_penalty: -2,
            exhausted_roll_penalty: -5,

            recovery_idle: 5,
            recovery_resting: 30,
            recovery_sheltered_resting: 60,
            recovery_starving_dehydrated_mult: 0.5,

            winded_attack_score_penalty: -10,
            fresh_target_visibly_tired_bonus: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_current_behavior_constants() {
        let t = CombatTuning::default();
        assert_eq!(t.decisive_win_multiplier, 1.5);
        assert_eq!(t.base_stress_no_engagements, 20.0);
        assert_eq!(t.stress_sanity_normalization, 100.0);
        assert_eq!(t.stress_final_divisor, 2.0);
        assert_eq!(t.kill_stress_contribution, 50.0);
        assert_eq!(t.non_kill_win_stress_contribution, 20.0);
    }

    #[test]
    fn default_stamina_constants_match_spec() {
        let t = CombatTuning::default();
        assert_eq!(t.stamina_cost_attacker, 25);
        assert_eq!(t.stamina_cost_target, 10);
        assert_eq!(t.band_winded_pct, 50);
        assert_eq!(t.band_exhausted_pct, 20);
        assert_eq!(t.winded_roll_penalty, -2);
        assert_eq!(t.exhausted_roll_penalty, -5);
        assert_eq!(t.recovery_idle, 5);
        assert_eq!(t.recovery_resting, 30);
        assert_eq!(t.recovery_sheltered_resting, 60);
        assert_eq!(t.recovery_starving_dehydrated_mult, 0.5);
        assert_eq!(t.winded_attack_score_penalty, -10);
        assert_eq!(t.fresh_target_visibly_tired_bonus, 5);
    }

    #[test]
    fn round_trips_through_serde_json() {
        let t = CombatTuning::default();
        let s = serde_json::to_string(&t).unwrap();
        let back: CombatTuning = serde_json::from_str(&s).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn game_default_carries_default_combat_tuning() {
        let g = crate::games::Game::default();
        assert_eq!(g.combat_tuning, CombatTuning::default());
    }
}
