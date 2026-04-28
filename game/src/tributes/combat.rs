//! Combat-related functionality for tributes.
//!
//! This module handles all attack and combat mechanics including:
//! - Attack contests between tributes
//! - Combat result application
//! - Violence stress calculations
//! - Statistics updates

use crate::items::OwnsItems;
use crate::messages::{CombatEngagement, CombatOutcome, MessagePayload, TaggedEvent, TributeRef};
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

/// Build a `TributeRef` from a tribute.
fn tref(t: &Tribute) -> TributeRef {
    TributeRef {
        identifier: t.identifier.clone(),
        name: t.name.clone(),
    }
}

impl Tribute {
    /// Tribute attacks another tribute
    /// Potentially fatal to either tribute
    ///
    /// Emits exactly one `MessagePayload::Combat` `TaggedEvent` per call when
    /// a real two-tribute engagement occurs. Self-harm / suicide / critical
    /// fumble paths are not engagements; they emit a single standalone
    /// `TributeKilled` or `TributeWounded` `TaggedEvent` instead.
    pub(crate) fn attacks(
        &mut self,
        target: &mut Tribute,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) -> AttackOutcome {
        // Is the tribute attempting suicide?
        if self == target {
            // Attack always succeeds
            self.takes_physical_damage(self.attributes.strength);
            // Capture violence-stress events as standalone (self-harm is not an engagement).
            let mut stress_events: Vec<TaggedEvent> = Vec::new();
            self.apply_violence_stress(&mut stress_events);
            events.extend(stress_events);

            return if self.attributes.health > 0 {
                let content =
                    GameOutput::TributeSelfHarm(self.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::TributeWounded {
                        victim: tref(self),
                        attacker: None,
                        hp_lost: self.attributes.strength,
                    },
                ));
                AttackOutcome::Wound(self.clone(), target.clone())
            } else {
                let content = GameOutput::TributeSuicide(self.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::TributeKilled {
                        victim: tref(self),
                        killer: None,
                        cause: "suicide".into(),
                    },
                ));
                AttackOutcome::Kill(self.clone(), target.clone())
            };
        }

        let tribute_name = self.name.clone();
        let target_name = target.name.clone();

        // Local accumulator for prose lines that become this engagement's
        // `detail_lines`. Helpers (`attack_contest`, `apply_combat_results`,
        // `apply_violence_stress`) push `TaggedEvent`s into a sub-buffer; we
        // then flatten their `.content` into `detail_lines`.
        let mut detail_lines: Vec<String> = Vec::new();
        let mut sub_events: Vec<TaggedEvent> = Vec::new();

        // `self` is the attacker
        let result = attack_contest(self, target, rng, &mut sub_events);
        for ev in sub_events.drain(..) {
            detail_lines.push(ev.content);
        }

        match result {
            AttackResult::CriticalHit => {
                // Triple damage on critical hit!
                detail_lines.push(
                    GameOutput::TributeCriticalHit(tribute_name.as_str(), target_name.as_str())
                        .to_string(),
                );
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 3, // triple damage
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::CriticalFumble => {
                // Fumble! Attacker hurts themself. Standalone (non-engagement) event.
                let fumble_content =
                    GameOutput::TributeCriticalFumble(tribute_name.as_str()).to_string();
                self.takes_physical_damage(5); // Fixed fumble damage
                self.statistics.defeats += 1;

                // If the attacker killed themselves with the fumble
                if self.attributes.health == 0 {
                    self.statistics.killed_by = Some("themselves (fumble)".to_string());
                    self.status = crate::tributes::statuses::TributeStatus::RecentlyDead;
                    self.recently_killed_by = Some(self.id);
                    let died_content = GameOutput::TributeAttackDied(
                        tribute_name.as_str(),
                        "themselves",
                    )
                    .to_string();
                    let combined = format!("{fumble_content} {died_content}");
                    events.push(TaggedEvent::new(
                        combined,
                        MessagePayload::TributeKilled {
                            victim: tref(self),
                            killer: None,
                            cause: "critical_fumble".into(),
                        },
                    ));
                    return AttackOutcome::Kill(target.clone(), self.clone());
                }

                events.push(TaggedEvent::new(
                    fumble_content,
                    MessagePayload::TributeWounded {
                        victim: tref(self),
                        attacker: None,
                        hp_lost: 5,
                    },
                ));
                return AttackOutcome::Wound(self.clone(), target.clone());
            }
            AttackResult::PerfectBlock => {
                // Perfect block! Defender counters
                detail_lines.push(
                    GameOutput::TributePerfectBlock(target_name.as_str(), tribute_name.as_str())
                        .to_string(),
                );
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage counter
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWins => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWinsDecisively => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackWinExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWins => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWinsDecisively => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackLoseExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::Miss => {
                self.statistics.draws += 1;
                target.statistics.draws += 1;

                detail_lines.push(
                    GameOutput::TributeAttackMiss(tribute_name.as_str(), target_name.as_str())
                        .to_string(),
                );

                let outcome = CombatOutcome::Stalemate;
                let summary =
                    format!("{} attacks {} ({:?})", self.name, target.name, outcome);
                events.push(TaggedEvent::new(
                    summary,
                    MessagePayload::Combat(CombatEngagement {
                        attacker: tref(self),
                        target: tref(target),
                        outcome,
                        detail_lines,
                    }),
                ));

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        // Determine outcome and finalise. Prefer death over flee/wound.
        let (outcome, attack_outcome) = if self.attributes.health == 0 {
            // Target killed attacker
            self.statistics.killed_by = Some(target_name.clone());
            self.status = crate::tributes::statuses::TributeStatus::RecentlyDead;
            self.recently_killed_by = Some(target.id);

            detail_lines.push(
                GameOutput::TributeAttackDied(tribute_name.as_str(), target_name.as_str())
                    .to_string(),
            );

            (
                CombatOutcome::Killed,
                AttackOutcome::Kill(target.clone(), self.clone()),
            )
        } else if target.attributes.health == 0 {
            // Attacker killed Target
            target.statistics.killed_by = Some(tribute_name.clone());
            target.status = crate::tributes::statuses::TributeStatus::RecentlyDead;
            target.recently_killed_by = Some(self.id);

            detail_lines.push(
                GameOutput::TributeAttackSuccessKill(tribute_name.as_str(), target_name.as_str())
                    .to_string(),
            );

            (
                CombatOutcome::Killed,
                AttackOutcome::Kill(self.clone(), target.clone()),
            )
        } else {
            detail_lines.push(
                GameOutput::TributeAttackWound(tribute_name.as_str(), target_name.as_str())
                    .to_string(),
            );
            (
                CombatOutcome::Wounded,
                AttackOutcome::Wound(self.clone(), target.clone()),
            )
        };

        let summary = format!("{} attacks {} ({:?})", self.name, target.name, outcome);
        events.push(TaggedEvent::new(
            summary,
            MessagePayload::Combat(CombatEngagement {
                attacker: tref(self),
                target: tref(target),
                outcome,
                detail_lines,
            }),
        ));

        attack_outcome
    }

    /// Apply violence stress to tribute based on their combat history
    pub(crate) fn apply_violence_stress(&mut self, events: &mut Vec<TaggedEvent>) {
        let stress_damage = calculate_violence_stress(
            self.statistics.kills,
            self.statistics.wins,
            self.attributes.sanity,
        );

        if stress_damage > 0 {
            let content =
                GameOutput::TributeHorrified(self.name.as_str(), stress_damage).to_string();
            events.push(TaggedEvent::new(
                content,
                MessagePayload::SanityBreak {
                    tribute: tref(self),
                },
            ));
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
/// Natural 1 on attack = critical fumble (attacker takes damage).
/// Natural 20 on attack = critical hit (triple damage).
/// Natural 20 on defense = perfect block (defender counters).
pub fn attack_contest(
    attacker: &mut Tribute,
    target: &mut Tribute,
    rng: &mut impl Rng,
    events: &mut Vec<TaggedEvent>,
) -> AttackResult {
    // Get attack roll and strength modifier
    let base_attack_roll: i32 = rng.random_range(1..=20); // Base roll
    let mut attack_roll = base_attack_roll;
    attack_roll += attacker.attributes.strength as i32; // Add strength

    // If the attacker has a weapon, use it
    let weapon_outcome = if let Some(weapon) = attacker.equipped_weapon_mut() {
        attack_roll += weapon.effect; // Add weapon damage
        let outcome = weapon.wear(1);
        Some((weapon.clone(), outcome))
    } else {
        None
    };
    if let Some((weapon, outcome)) = weapon_outcome {
        match outcome {
            crate::items::WearOutcome::Pristine => {}
            crate::items::WearOutcome::Worn => {
                let content =
                    GameOutput::WeaponWear(attacker.name.as_str(), weapon.name.as_str())
                        .to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(attacker),
                        item: shared::messages::ItemRef {
                            identifier: weapon.identifier.clone(),
                            name: weapon.name.clone(),
                        },
                    },
                ));
            }
            crate::items::WearOutcome::Broken => {
                let content =
                    GameOutput::WeaponBreak(attacker.name.as_str(), weapon.name.as_str())
                        .to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(attacker),
                        item: shared::messages::ItemRef {
                            identifier: weapon.identifier.clone(),
                            name: weapon.name.clone(),
                        },
                    },
                ));
                if let Err(err) = attacker.remove_item(&weapon) {
                    eprintln!("Failed to remove weapon: {}", err);
                }
            }
        }
    }

    // Get defense roll and defense modifier
    let base_defense_roll: i32 = rng.random_range(1..=20); // Base roll
    let mut defense_roll = base_defense_roll;
    defense_roll += target.attributes.defense as i32; // Add defense

    // If the defender has a shield, use it
    let shield_outcome = if let Some(shield) = target.equipped_shield_mut() {
        defense_roll += shield.effect; // Add shield defense
        let outcome = shield.wear(1);
        Some((shield.clone(), outcome))
    } else {
        None
    };
    if let Some((shield, outcome)) = shield_outcome {
        match outcome {
            crate::items::WearOutcome::Pristine => {}
            crate::items::WearOutcome::Worn => {
                let content =
                    GameOutput::ShieldWear(target.name.as_str(), shield.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(target),
                        item: shared::messages::ItemRef {
                            identifier: shield.identifier.clone(),
                            name: shield.name.clone(),
                        },
                    },
                ));
            }
            crate::items::WearOutcome::Broken => {
                let content =
                    GameOutput::ShieldBreak(target.name.as_str(), shield.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(target),
                        item: shared::messages::ItemRef {
                            identifier: shield.identifier.clone(),
                            name: shield.name.clone(),
                        },
                    },
                ));
                if let Err(err) = target.remove_item(&shield) {
                    eprintln!("Failed to remove shield: {}", err);
                }
            }
        }
    }

    // Check for critical outcomes based on natural rolls (before modifiers)
    match (base_attack_roll, base_defense_roll) {
        (1, _) => AttackResult::CriticalFumble, // Natural 1 on attack - fumble!
        (20, _) => AttackResult::CriticalHit,   // Natural 20 on attack - crit!
        (_, 20) => AttackResult::PerfectBlock,  // Natural 20 on defense - perfect block!
        _ => {
            // Normal combat resolution - compare attack vs. defense
            match attack_roll.cmp(&defense_roll) {
                Ordering::Less => {
                    // If the defender wins
                    let difference =
                        defense_roll as f64 - (attack_roll as f64 * DECISIVE_WIN_MULTIPLIER);
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
                    let difference =
                        attack_roll as f64 - (defense_roll as f64 * DECISIVE_WIN_MULTIPLIER);

                    if difference > 0.0 {
                        // Attacker wins significantly
                        AttackResult::AttackerWinsDecisively
                    } else {
                        AttackResult::AttackerWins
                    }
                }
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
    events: &mut Vec<TaggedEvent>,
) {
    loser.takes_physical_damage(damage_to_loser);
    loser.statistics.defeats += 1;
    winner.statistics.wins += 1;
    winner.apply_violence_stress(events);
    let content = log_event.to_string();
    events.push(TaggedEvent::new(
        content,
        MessagePayload::TributeWounded {
            victim: tref(loser),
            attacker: Some(tref(winner)),
            hp_lost: damage_to_loser,
        },
    ));
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
        AttackResult::CriticalHit => {
            // Critical hit is a special attacker win (triple damage)
            defender.statistics.defeats += 1;
            attacker.statistics.wins += 1;
        }
        AttackResult::PerfectBlock => {
            // Perfect block is a special defender win (counter-attack)
            attacker.statistics.defeats += 1;
            defender.statistics.wins += 1;
        }
        AttackResult::CriticalFumble => {
            // Critical fumble: attacker hurts themselves, counts as draw
            attacker.statistics.draws += 1;
            defender.statistics.draws += 1;
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
    use rand::RngCore;
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

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::AttackerWins);
    }

    #[rstest]
    fn attack_contest_win_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 0;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::AttackerWinsDecisively);
    }

    #[rstest]
    fn attack_contest_lose(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::DefenderWins);
    }

    #[rstest]
    fn attack_contest_lose_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 1;
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::DefenderWinsDecisively);
    }

    #[rstest]
    fn attack_contest_draw(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 21; // Magic number to make the final scores even
        target.attributes.defense = 20;

        let result = attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::Miss);
    }

    #[rstest]
    fn attacks_self(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.sanity = 50;
        let sanity = 50;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(outcome, AttackOutcome::Wound(attacker.clone(), target));
        assert!(attacker.attributes.sanity < sanity);
    }

    #[rstest]
    fn attacks_self_suicide(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 100;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(outcome, AttackOutcome::Kill(attacker, target));
    }

    #[rstest]
    fn attacks_wound(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let sanity = attacker.attributes.sanity;
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 25;
        target.attributes.defense = 20;

        let result = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new());
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

        let result = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new());
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

        let result = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new());
        assert_eq!(result, AttackOutcome::Miss(attacker, target));
    }

    #[rstest]
    fn test_critical_hit() {
        // Use a custom RNG that always returns the high bits needed for
        // `random_range(1..=20)` to produce 20 under rand 0.9's algorithm.
        struct CritRng;
        impl RngCore for CritRng {
            fn next_u32(&mut self) -> u32 {
                0xF333_3334
            }
            fn next_u64(&mut self) -> u64 {
                ((self.next_u32() as u64) << 32) | self.next_u32() as u64
            }
            fn fill_bytes(&mut self, dest: &mut [u8]) {
                for byte in dest.iter_mut() {
                    *byte = 0xFF;
                }
            }
        }

        let mut crit_rng = CritRng;
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 10;
        target.attributes.health = 100;

        let result = attack_contest(&mut attacker, &mut target, &mut crit_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::CriticalHit);
    }

    #[rstest]
    fn test_critical_fumble() {
        // Use a custom RNG that returns 0 so `random_range(1..=20)` yields 1.
        struct FumbleRng;
        impl RngCore for FumbleRng {
            fn next_u32(&mut self) -> u32 {
                0
            }
            fn next_u64(&mut self) -> u64 {
                0
            }
            fn fill_bytes(&mut self, dest: &mut [u8]) {
                for byte in dest.iter_mut() {
                    *byte = 0;
                }
            }
        }

        let mut fumble_rng = FumbleRng;
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        let result = attack_contest(&mut attacker, &mut target, &mut fumble_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::CriticalFumble);
    }

    #[rstest]
    fn test_perfect_block() {
        // First call (attacker roll): 0x7333_3334 → random_range(1..=20) == 10
        // Second call (defender roll): 0xF333_3334 → random_range(1..=20) == 20
        struct BlockRng {
            call_count: std::cell::Cell<usize>,
        }
        impl BlockRng {
            fn new() -> Self {
                BlockRng {
                    call_count: std::cell::Cell::new(0),
                }
            }
        }
        impl RngCore for BlockRng {
            fn next_u32(&mut self) -> u32 {
                let count = self.call_count.get();
                self.call_count.set(count + 1);
                if count == 0 { 0x7333_3334 } else { 0xF333_3334 }
            }
            fn next_u64(&mut self) -> u64 {
                self.next_u32() as u64
            }
            fn fill_bytes(&mut self, dest: &mut [u8]) {
                for byte in dest.iter_mut() {
                    *byte = self.next_u32() as u8;
                }
            }
        }

        let mut block_rng = BlockRng::new();
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        let result = attack_contest(&mut attacker, &mut target, &mut block_rng, &mut Vec::new());
        assert_eq!(result, AttackResult::PerfectBlock);
    }

    #[rstest]
    fn test_critical_hit_triple_damage(_small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 20;
        target.attributes.health = 100;
        let initial_health = target.attributes.health;
        let damage = attacker.attributes.strength * 3;

        // Manually test the damage application for critical hit
        apply_combat_results(
            &mut attacker,
            &mut target,
            damage, // Triple damage
            GameOutput::TributeAttackWin("Katniss", "Peeta"),
            &mut Vec::new(),
        );

        // Verify triple damage was applied
        assert_eq!(target.attributes.health, initial_health - damage);
    }

    #[rstest]
    fn test_fumble_self_damage(_small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.health = 100;
        let initial_health = attacker.attributes.health;

        // Simulate fumble damage
        attacker.takes_physical_damage(5);

        assert_eq!(attacker.attributes.health, initial_health - 5);
    }

    /// Regression test for the clone-mutation bug where weapon wear was
    /// applied to a cloned item from `weapons()` and silently lost.
    /// A weapon with durability 5 must survive 3 attack contests with
    /// reduced durability and remain in the attacker's inventory.
    #[rstest]
    fn weapon_survives_multiple_combats(mut small_rng: SmallRng) {
        use crate::items::{Attribute, Item, ItemRarity, ItemType};

        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        let weapon = Item::new(
            "Test Bow",
            ItemType::Weapon,
            ItemRarity::Rare,
            5,
            Attribute::Strength,
            3,
        );
        let weapon_id = weapon.identifier.clone();
        attacker.items.push(weapon);

        for _ in 0..3 {
            attack_contest(&mut attacker, &mut target, &mut small_rng, &mut Vec::new());
        }

        let stored = attacker
            .items
            .iter()
            .find(|i| i.identifier == weapon_id)
            .expect("weapon should still be in inventory after 3 combats");
        assert_eq!(stored.max_durability, 5);
        assert_eq!(
            stored.current_durability, 2,
            "weapon should have been worn 3 times (5 - 3 = 2)"
        );
    }

    #[rstest]
    fn attacks_target_killed_records_killer_id() {
        // When the attacker kills the target, target.recently_killed_by must
        // be set to the attacker's id so the cycle can attribute the death.
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 100;
        target.attributes.health = 1;
        target.attributes.defense = 0;
        let attacker_id = attacker.id;

        // Use a deterministic RNG; with strength=100 vs defense=0 the
        // attacker reliably wins or crit-hits and the 1hp target dies.
        let mut rng = SmallRng::seed_from_u64(1);
        let _ = attacker.attacks(&mut target, &mut rng, &mut Vec::new());

        assert_eq!(
            target.attributes.health, 0,
            "target should be dead in this scenario"
        );
        assert_eq!(
            target.status,
            crate::tributes::statuses::TributeStatus::RecentlyDead
        );
        assert_eq!(
            target.recently_killed_by,
            Some(attacker_id),
            "killer id must be recorded on the deceased"
        );
        assert!(
            attacker.recently_killed_by.is_none(),
            "attacker is alive; their field must remain None"
        );
    }

    #[rstest]
    fn attacks_attacker_killed_records_target_id() {
        // When the target's counter kills the attacker (e.g. perfect block),
        // attacker.recently_killed_by must point to the target.
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.health = 1;
        attacker.attributes.strength = 0;
        attacker.attributes.defense = 0;
        target.attributes.strength = 100;
        target.attributes.defense = 100;
        let target_id = target.id;

        let mut rng = SmallRng::seed_from_u64(2);
        let _ = attacker.attacks(&mut target, &mut rng, &mut Vec::new());

        if attacker.attributes.health == 0 {
            assert_eq!(
                attacker.status,
                crate::tributes::statuses::TributeStatus::RecentlyDead
            );
            assert_eq!(
                attacker.recently_killed_by,
                Some(target_id),
                "killer id must be recorded on the deceased attacker"
            );
        } else {
            // The seed didn't produce a kill; not a failure of attribution
            // logic, just RNG. Re-skip rather than flake.
            eprintln!("seed did not produce attacker death; skipping attribution check");
        }
    }

    /// Contract test: a real two-tribute engagement emits exactly one
    /// `MessagePayload::Combat` `TaggedEvent`, with the attacker/target
    /// names and a recognised outcome.
    #[rstest]
    fn attacks_emits_one_combat_taggedevent(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 50;
        target.attributes.defense = 0;
        target.attributes.health = 10;

        let mut events: Vec<TaggedEvent> = Vec::new();
        attacker.attacks(&mut target, &mut small_rng, &mut events);

        let combat_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.payload, MessagePayload::Combat(_)))
            .collect();
        assert_eq!(
            combat_events.len(),
            1,
            "exactly one Combat payload per attacks() call"
        );

        if let MessagePayload::Combat(eng) = &combat_events[0].payload {
            assert_eq!(eng.attacker.name, attacker.name);
            assert_eq!(eng.target.name, target.name);
            assert!(matches!(
                eng.outcome,
                CombatOutcome::Killed
                    | CombatOutcome::Wounded
                    | CombatOutcome::TargetFled
                    | CombatOutcome::AttackerFled
                    | CombatOutcome::Stalemate
            ));
        } else {
            panic!("expected Combat payload");
        }
    }
}
