//! Combat-related functionality for tributes.
//!
//! This module handles all attack and combat mechanics including:
//! - Attack contests between tributes
//! - Combat result application
//! - Violence stress calculations
//! - Statistics updates

use crate::items::OwnsItems;
use crate::output::GameOutput;
use crate::tributes::Tribute;
use crate::tributes::actions::{AttackOutcome, AttackResult};
use rand::prelude::*;
use std::cmp::Ordering;

/// Constants for combat calculations
const DECISIVE_WIN_MULTIPLIER: f64 = 1.5;
const BASE_STRESS_NO_ENGAGEMENTS: f64 = 20.0;
const STRESS_SANITY_NORMALIZATION: f64 = 100.0;
const STRESS_FINAL_DIVISOR: f64 = 2.0;
const KILL_STRESS_CONTRIBUTION: f64 = 50.0;
const NON_KILL_WIN_STRESS_CONTRIBUTION: f64 = 20.0;

impl Tribute {
    /// Tribute attacks another tribute
    /// Potentially fatal to either tribute
    pub(crate) fn attacks(&mut self, target: &mut Tribute, rng: &mut impl Rng) -> AttackOutcome {
        // Is the tribute attempting suicide?
        if self == target {
            self.try_log_action(GameOutput::TributeSelfHarm(self.name.as_str()), "self-harm");

            // Attack always succeeds
            self.takes_physical_damage(self.attributes.strength);
            self.apply_violence_stress();

            self.try_log_action(
                GameOutput::TributeAttackWin(self.name.as_str(), target.name.as_str()),
                "attack against self",
            );

            return if self.attributes.health > 0 {
                self.try_log_action(
                    GameOutput::TributeAttackWound(self.name.as_str(), target.name.as_str()),
                    "wounded self",
                );
                AttackOutcome::Wound(self.clone(), target.clone())
            } else {
                self.try_log_action(
                    GameOutput::TributeSuicide(self.name.as_str()),
                    "successful suicide",
                );
                AttackOutcome::Kill(self.clone(), target.clone())
            };
        }

        let tribute_name = self.name.clone();
        let target_name = target.name.clone();
        // `self` is the attacker
        match attack_contest(self, target, rng) {
            AttackResult::AttackerWins => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    "attack win",
                );
            }
            AttackResult::AttackerWinsDecisively => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackWinExtra(tribute_name.as_str(), target_name.as_str()),
                    "attack win extra",
                );
            }
            AttackResult::DefenderWins => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    "attack lose",
                );
            }
            AttackResult::DefenderWinsDecisively => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackLoseExtra(tribute_name.as_str(), target_name.as_str()),
                    "attack lose extra",
                );
            }
            AttackResult::Miss => {
                self.statistics.draws += 1;
                target.statistics.draws += 1;

                self.try_log_action(
                    GameOutput::TributeAttackMiss(tribute_name.as_str(), target_name.as_str()),
                    "missed attack",
                );

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        if self.attributes.health == 0 {
            // Target killed attacker
            self.statistics.killed_by = Some(target_name.clone());
            self.status = crate::tributes::statuses::TributeStatus::RecentlyDead;

            self.try_log_action(
                GameOutput::TributeAttackDied(tribute_name.as_str(), target_name.as_str()),
                "attacker died",
            );

            AttackOutcome::Kill(target.clone(), self.clone())
        } else if target.attributes.health == 0 {
            // Attacker killed Target
            target.statistics.killed_by = Some(tribute_name.clone());
            target.status = crate::tributes::statuses::TributeStatus::RecentlyDead;

            self.try_log_action(
                GameOutput::TributeAttackSuccessKill(tribute_name.as_str(), target_name.as_str()),
                "killed target",
            );

            AttackOutcome::Kill(self.clone(), target.clone())
        } else {
            self.try_log_action(
                GameOutput::TributeAttackWound(tribute_name.as_str(), target_name.as_str()),
                "wounded target",
            );
            AttackOutcome::Wound(self.clone(), target.clone())
        }
    }

    /// Apply violence stress to tribute based on their combat history
    pub(crate) fn apply_violence_stress(&mut self) {
        let stress_damage = calculate_violence_stress(
            self.statistics.kills,
            self.statistics.wins,
            self.attributes.sanity,
        );

        if stress_damage > 0 {
            self.try_log_action(
                GameOutput::TributeHorrified(self.name.as_str(), stress_damage),
                "violence stress",
            );
            self.takes_mental_damage(stress_damage);
        }
    }
}

/// Calculate stress from violent encounters
fn calculate_violence_stress(kills: u32, wins: u32, current_sanity: u32) -> u32 {
    let non_kill_wins = wins.saturating_sub(kills);

    let calculated_stress_f64 = if wins > 0 {
        // Calculate the stress potential based on kills and non-kill wins
        let raw_stress_potential = (kills as f64 * KILL_STRESS_CONTRIBUTION)
            + (non_kill_wins as f64 * NON_KILL_WIN_STRESS_CONTRIBUTION);

        // Desensitize: the more total wins (violent encounters), the more this raw potential
        // is "spread out" or reduced. This gives an average stressfulness per encounter.
        let desensitized_stress_per_encounter = raw_stress_potential / wins as f64;

        // Scale by the tribute's current sanity percentage and apply a final divisor.
        // Lower sanity means less new stress from these types of events.
        desensitized_stress_per_encounter * (current_sanity as f64 / STRESS_SANITY_NORMALIZATION)
            / STRESS_FINAL_DIVISOR
    } else {
        // No wins (and therefore no kills), apply a base stress.
        BASE_STRESS_NO_ENGAGEMENTS
    };

    let rounded_stress = calculated_stress_f64.round();

    // Only apply stress if it's at least 1
    if rounded_stress >= 1.0 {
        rounded_stress as u32
    } else {
        0
    }
}

/// Generate attack data for each tribute.
/// Each rolls a d20 to decide a basic attack / defense value.
/// Strength and any weapon are added to the attack roll.
/// Defense and any shield are added to the defense roll.
/// If either roll is more than 1.5x the other, that triggers a "decisive" victory.
pub fn attack_contest(
    attacker: &mut Tribute,
    target: &mut Tribute,
    rng: &mut impl Rng,
) -> AttackResult {
    // Get attack roll and strength modifier
    let mut attack_roll: i32 = rng.random_range(1..=20); // Base roll
    attack_roll += attacker.attributes.strength as i32; // Add strength

    // If the attacker has a weapon, use it
    if let Some(weapon) = attacker.weapons().iter_mut().last() {
        attack_roll += weapon.effect; // Add weapon damage
        weapon.quantity = weapon.quantity.saturating_sub(1);
        if weapon.quantity == 0 {
            attacker.try_log_action(
                GameOutput::WeaponBreak(attacker.name.as_str(), weapon.name.as_str()),
                "weapon break",
            );
            if let Err(err) = attacker.remove_item(weapon) {
                eprintln!("Failed to remove weapon: {}", err);
            }
        }
    }

    // Get defense roll and defense modifier
    let mut defense_roll: i32 = rng.random_range(1..=20); // Base roll
    defense_roll += target.attributes.defense as i32; // Add defense

    // If the defender has a shield, use it
    if let Some(shield) = target.shields().iter_mut().last() {
        defense_roll += shield.effect; // Add shield defense
        shield.quantity = shield.quantity.saturating_sub(1);
        if shield.quantity == 0 {
            target.try_log_action(
                GameOutput::ShieldBreak(target.name.as_str(), shield.name.as_str()),
                "shield break",
            );
            if let Err(err) = target.remove_item(shield) {
                eprintln!("Failed to remove shield: {}", err);
            };
        }
    }

    // Compare attack vs. defense
    match attack_roll.cmp(&defense_roll) {
        Ordering::Less => {
            // If the defender wins
            let difference = defense_roll as f64 - (attack_roll as f64 * DECISIVE_WIN_MULTIPLIER);
            if difference > 0.0 {
                // Defender wins significantly
                AttackResult::DefenderWinsDecisively
            } else {
                AttackResult::DefenderWins
            }
        }
        Ordering::Equal => AttackResult::Miss, // If they tie
        Ordering::Greater => {
            // If the attacker wins
            let difference = attack_roll as f64 - (defense_roll as f64 * DECISIVE_WIN_MULTIPLIER);

            if difference > 0.0 {
                // Attacker wins significantly
                AttackResult::AttackerWinsDecisively
            } else {
                AttackResult::AttackerWins
            }
        }
    }
}

/// Apply the results of a combat encounter.
/// Adjust statistics and log the result.
pub(crate) fn apply_combat_results(
    winner: &mut Tribute,
    loser: &mut Tribute,
    damage_to_loser: u32,
    log_event: GameOutput,
    log_description: &str,
) {
    loser.takes_physical_damage(damage_to_loser);
    loser.statistics.defeats += 1;
    winner.statistics.wins += 1;
    winner.apply_violence_stress();
    winner.try_log_action(log_event, log_description);
}

/// Update statistics for a pair of tributes based on the attack result
pub fn update_stats(attacker: &mut Tribute, defender: &mut Tribute, result: AttackResult) {
    match result {
        AttackResult::AttackerWins | AttackResult::AttackerWinsDecisively => {
            defender.statistics.defeats += 1;
            attacker.statistics.wins += 1;
        }
        AttackResult::DefenderWins | AttackResult::DefenderWinsDecisively => {
            attacker.statistics.defeats += 1;
            defender.statistics.wins += 1;
        }
        AttackResult::Miss => {
            attacker.statistics.draws += 1;
            defender.statistics.draws += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::*;

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[rstest]
    fn attack_contest_win(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 10;
        target.attributes.defense = 5;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng);
        assert_eq!(result, AttackResult::AttackerWins);
    }

    #[rstest]
    fn attack_contest_win_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 0;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng);
        assert_eq!(result, AttackResult::AttackerWinsDecisively);
    }

    #[rstest]
    fn attack_contest_lose(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng);
        assert_eq!(result, AttackResult::DefenderWins);
    }

    #[rstest]
    fn attack_contest_lose_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 1;
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng);
        assert_eq!(result, AttackResult::DefenderWinsDecisively);
    }

    #[rstest]
    fn attack_contest_draw(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 21; // Magic number to make the final scores even
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng);
        assert_eq!(result, AttackResult::Miss);
    }

    #[rstest]
    fn attacks_self(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.sanity = 50;
        let sanity = 50;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(&mut target, &mut small_rng);
        assert_eq!(outcome, AttackOutcome::Wound(attacker.clone(), target));
        assert!(attacker.attributes.sanity < sanity);
    }

    #[rstest]
    fn attacks_self_suicide(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 100;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(&mut target, &mut small_rng);
        assert_eq!(outcome, AttackOutcome::Kill(attacker, target));
    }

    #[rstest]
    fn attacks_wound(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let sanity = attacker.attributes.sanity;
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 25;
        target.attributes.defense = 20;

        let result = attacker.attacks(&mut target, &mut small_rng);
        assert_eq!(
            result,
            AttackOutcome::Wound(attacker.clone(), target.clone())
        );
        assert_eq!(attacker.statistics.wins, 1);
        assert_eq!(target.statistics.defeats, 1);
        assert!(attacker.attributes.sanity < sanity);
    }

    #[rstest]
    fn attacks_kill(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 50;
        target.attributes.defense = 0;
        target.attributes.health = 10;

        let result = attacker.attacks(&mut target, &mut small_rng);
        assert!(matches!(result, AttackOutcome::Kill(_, _)));
        assert_eq!(target.attributes.health, 0);
        assert_eq!(attacker.statistics.wins, 1);
        assert_eq!(target.statistics.defeats, 1);
    }

    #[rstest]
    fn attacks_miss(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 21; // Magic number to make them draw
        target.attributes.defense = 20;

        let result = attacker.attacks(&mut target, &mut small_rng);
        assert_eq!(result, AttackOutcome::Miss(attacker, target));
    }
}
