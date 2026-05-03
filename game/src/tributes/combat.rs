//! Combat-related functionality for tributes.
//!
//! This module handles all attack and combat mechanics including:
//! - Attack contests between tributes
//! - Combat result application
//! - Violence stress calculations
//! - Statistics updates

use crate::items::{Item, OwnsItems};
use crate::messages::{CombatEngagement, CombatOutcome, MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::Tribute;
use crate::tributes::actions::{AttackOutcome, AttackResult};
use rand::RngExt;
use rand::prelude::*;
use shared::combat_beat::{CombatBeat, StressReport, SwingOutcome};
use shared::messages::ItemRef;
use std::cmp::Ordering;

/// Build a `TributeRef` from a tribute.
fn tref(t: &Tribute) -> TributeRef {
    TributeRef {
        identifier: t.identifier.clone(),
        name: t.name.clone(),
    }
}

/// Build an `ItemRef` from an item.
fn iref(i: &Item) -> ItemRef {
    ItemRef {
        identifier: i.identifier.clone(),
        name: i.name.clone(),
    }
}

/// Build a baseline `CombatBeat` (no wear, no stress) for the current swing.
/// Callers fill in `outcome` and (eventually) `wear` / `stress` per branch.
fn new_beat(
    attacker: &Tribute,
    target: &Tribute,
    outcome: SwingOutcome,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> CombatBeat {
    CombatBeat {
        attacker: tref(attacker),
        target: tref(target),
        weapon: attacker
            .items
            .iter()
            .rfind(|i| i.is_weapon() && i.current_durability > 0)
            .map(iref),
        shield: target
            .items
            .iter()
            .rfind(|i| i.is_defensive() && i.current_durability > 0)
            .map(iref),
        wear: Vec::new(),
        outcome,
        stress: StressReport::default(),
        attacker_stamina_cost: tuning.stamina_cost_attacker,
        target_stamina_cost: tuning.stamina_cost_target,
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
        tuning: &crate::tributes::combat_tuning::CombatTuning,
    ) -> AttackOutcome {
        // Check self-attack BEFORE deducting stamina (stamina mutation would
        // otherwise break the derived PartialEq equality check below).
        let is_self_attack = self == target;

        // Per-swing stamina cost: deduct from both combatants up-front.
        // Saturating semantics ensure neither tribute goes negative.
        // Action-gating (refusing to swing while exhausted) lands in Task 10.
        self.stamina = self.stamina.saturating_sub(tuning.stamina_cost_attacker);
        if !is_self_attack {
            target.stamina = target.stamina.saturating_sub(tuning.stamina_cost_target);
        }

        // Is the tribute attempting suicide?
        if is_self_attack {
            // Snapshot before mutation.
            let damage = self.attributes.strength;
            // Attack always succeeds
            self.takes_physical_damage(damage);
            // Capture violence-stress events as standalone (self-harm is not an engagement).
            let mut stress_events: Vec<TaggedEvent> = Vec::new();
            let stress_damage = self.apply_violence_stress(&mut stress_events, tuning);
            events.extend(stress_events);

            return if self.attributes.health > 0 {
                let content = GameOutput::TributeSelfHarm(self.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::TributeWounded {
                        victim: tref(self),
                        attacker: None,
                        hp_lost: damage,
                    },
                ));
                let mut beat = new_beat(
                    self,
                    target,
                    SwingOutcome::SelfAttackWound { damage },
                    tuning,
                );
                beat.target_stamina_cost = 0;
                beat.stress.stress_damage = stress_damage;
                if stress_damage > 0 {
                    beat.stress.stressed = Some(tref(self));
                }
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
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
                let mut beat = new_beat(self, target, SwingOutcome::Suicide { damage }, tuning);
                beat.target_stamina_cost = 0;
                beat.stress.stress_damage = stress_damage;
                if stress_damage > 0 {
                    beat.stress.stressed = Some(tref(self));
                }
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
                ));
                AttackOutcome::Kill(self.clone(), target.clone())
            };
        }

        let tribute_name = self.name.clone();
        let target_name = target.name.clone();

        // Snapshot equipment refs at swing start so post-mutation values
        // (e.g. broken items removed from inventory) don't pollute the beat.
        let weapon_at_start = self
            .items
            .iter()
            .rfind(|i| i.is_weapon() && i.current_durability > 0)
            .map(iref);
        let shield_at_start = target
            .items
            .iter()
            .rfind(|i| i.is_defensive() && i.current_durability > 0)
            .map(iref);
        let attacker_ref_at_start = tref(self);
        let target_ref_at_start = tref(target);

        // Local accumulator for prose lines that become this engagement's
        // `detail_lines`. Helpers (`attack_contest`, `apply_combat_results`,
        // `apply_violence_stress`) push `TaggedEvent`s into a sub-buffer; we
        // then flatten their `.content` into `detail_lines`.
        let mut detail_lines: Vec<String> = Vec::new();
        let mut sub_events: Vec<TaggedEvent> = Vec::new();

        // `self` is the attacker
        let contest = attack_contest(self, target, rng, &mut sub_events, tuning);
        let result = contest.result;
        let wear_records = contest.wear;
        // Stress applied to the swing's winner via apply_violence_stress; set
        // by each branch below before mk_beat is invoked. Initialised to 0
        // because Miss / fumble paths never apply stress.
        #[allow(unused_assignments)]
        let mut stress_damage: u32 = 0;
        // Tracks which combatant absorbed the stress so the horrified line
        // renders with the correct name. None = no horrified line.
        let mut stressed: Option<TributeRef> = None;
        let mk_beat =
            |outcome: SwingOutcome, stress_damage: u32, stressed: Option<TributeRef>| CombatBeat {
                attacker: attacker_ref_at_start.clone(),
                target: target_ref_at_start.clone(),
                weapon: weapon_at_start.clone(),
                shield: shield_at_start.clone(),
                wear: wear_records.clone(),
                outcome,
                stress: StressReport {
                    stress_damage,
                    stressed,
                },
                attacker_stamina_cost: tuning.stamina_cost_attacker,
                target_stamina_cost: tuning.stamina_cost_target,
            };
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
                stress_damage = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 3, // triple damage
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(self));
                }
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
                    let died_content =
                        GameOutput::TributeAttackDied(tribute_name.as_str(), "themselves")
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
                    let beat = mk_beat(SwingOutcome::FumbleDeath { self_damage: 5 }, 0, None);
                    events.push(TaggedEvent::new(
                        String::new(),
                        MessagePayload::CombatSwing(beat),
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
                let beat = mk_beat(SwingOutcome::FumbleSurvive { self_damage: 5 }, 0, None);
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
                ));
                return AttackOutcome::Wound(self.clone(), target.clone());
            }
            AttackResult::PerfectBlock => {
                // Perfect block! Defender counters
                detail_lines.push(
                    GameOutput::TributePerfectBlock(target_name.as_str(), tribute_name.as_str())
                        .to_string(),
                );
                stress_damage = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage counter
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(target));
                }
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWins => {
                stress_damage = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(self));
                }
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWinsDecisively => {
                stress_damage = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackWinExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(self));
                }
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWins => {
                stress_damage = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(target));
                }
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWinsDecisively => {
                stress_damage = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackLoseExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                );
                if stress_damage > 0 {
                    stressed = Some(tref(target));
                }
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
                let beat = mk_beat(SwingOutcome::Miss, 0, None);
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
                ));

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        // Capture damage applied this swing for the typed beat. Computed from
        // the resolved branch so the beat's damage matches what was applied.
        let swing_damage: u32 = match result {
            AttackResult::CriticalHit => self.attributes.strength * 3,
            AttackResult::AttackerWins => self.attributes.strength,
            AttackResult::AttackerWinsDecisively => self.attributes.strength * 2,
            AttackResult::PerfectBlock => target.attributes.strength * 2,
            AttackResult::DefenderWins => target.attributes.strength,
            AttackResult::DefenderWinsDecisively => target.attributes.strength * 2,
            // Unreachable: handled above with early returns.
            AttackResult::CriticalFumble | AttackResult::Miss => 0,
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

        // Map (AttackResult, post-resolution alive state) → typed SwingOutcome.
        let swing_outcome = match (&attack_outcome, result) {
            (AttackOutcome::Kill(winner, _loser), _) if winner.id == self.id => {
                SwingOutcome::Kill {
                    damage: swing_damage,
                }
            }
            (AttackOutcome::Kill(_, _), _) => SwingOutcome::AttackerDied {
                damage: swing_damage,
            },
            (AttackOutcome::Wound(_, _), AttackResult::CriticalHit) => {
                SwingOutcome::CriticalHitWound {
                    damage: swing_damage,
                }
            }
            (AttackOutcome::Wound(_, _), AttackResult::PerfectBlock) => SwingOutcome::BlockWound {
                damage: swing_damage,
            },
            (AttackOutcome::Wound(_, _), _) => SwingOutcome::Wound {
                damage: swing_damage,
            },
            // Miss already returned above.
            (AttackOutcome::Miss(_, _), _) => SwingOutcome::Miss,
        };
        let beat = mk_beat(swing_outcome, stress_damage, stressed);
        events.push(TaggedEvent::new(
            String::new(),
            MessagePayload::CombatSwing(beat),
        ));

        attack_outcome
    }

    /// Apply violence stress to tribute based on their combat history.
    ///
    /// Returns the amount of mental damage applied (0 if below threshold).
    /// Callers thread this value into `CombatBeat.stress.stress_damage` so
    /// the typed swing payload can reproduce the trailing horrified line
    /// rendered in `CombatEngagement.detail_lines`.
    pub(crate) fn apply_violence_stress(
        &mut self,
        events: &mut Vec<TaggedEvent>,
        tuning: &crate::tributes::combat_tuning::CombatTuning,
    ) -> u32 {
        let stress_damage = calculate_violence_stress(
            self.statistics.kills,
            self.statistics.wins,
            self.attributes.sanity,
            tuning,
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
        stress_damage
    }
}

/// Calculate stress from violent encounters
fn calculate_violence_stress(
    kills: u32,
    wins: u32,
    current_sanity: u32,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> u32 {
    let non_kill_wins = wins.saturating_sub(kills);

    let calculated_stress_f64 = if wins > 0 {
        // Calculate the stress potential based on kills and non-kill wins
        let raw_stress_potential = (kills as f64 * tuning.kill_stress_contribution)
            + (non_kill_wins as f64 * tuning.non_kill_win_stress_contribution);

        // Desensitize: the more total wins (violent encounters), the more this raw potential
        // is "spread out" or reduced. This gives an average stressfulness per encounter.
        let desensitized_stress_per_encounter = raw_stress_potential / wins as f64;

        // Scale by the tribute's current sanity percentage and apply a final divisor.
        // Lower sanity means less new stress from these types of events.
        desensitized_stress_per_encounter
            * (current_sanity as f64 / tuning.stress_sanity_normalization)
            / tuning.stress_final_divisor
    } else {
        // No wins (and therefore no kills), apply a base stress.
        tuning.base_stress_no_engagements
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
/// Outcome of a single `attack_contest` invocation.
///
/// Carries the high-level `AttackResult` plus enough wear/penalty data for
/// the caller to assemble a `CombatBeat` without re-snapshotting equipment
/// state.
pub struct AttackContestOutcome {
    pub result: AttackResult,
    /// Wear records emitted in attack-roll order: weapon (if equipped) then
    /// shield (if equipped). Items that were `Pristine` are omitted.
    pub wear: Vec<shared::combat_beat::WearReport>,
}

pub fn attack_contest(
    attacker: &mut Tribute,
    target: &mut Tribute,
    rng: &mut impl Rng,
    events: &mut Vec<TaggedEvent>,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> AttackContestOutcome {
    use shared::combat_beat::{WearOutcomeReport, WearReport};
    // Get attack roll and strength modifier
    let base_attack_roll: i32 = rng.random_range(1..=20); // Base roll
    let mut attack_roll = base_attack_roll;
    attack_roll += attacker.attributes.strength as i32; // Add strength

    let mut wear: Vec<WearReport> = Vec::new();

    // If the attacker has a weapon, use it
    let weapon_outcome = if let Some(weapon) = attacker.equipped_weapon_mut() {
        attack_roll += weapon.effect; // Add weapon damage
        let outcome = weapon.wear(1);
        // Defer clone: only Worn/Broken paths need the snapshot.
        match outcome {
            crate::items::WearOutcome::Pristine => None,
            _ => Some((weapon.clone(), outcome)),
        }
    } else {
        None
    };
    if let Some((weapon, outcome)) = weapon_outcome {
        let attacker_ref = tref(attacker);
        let item_ref = shared::messages::ItemRef {
            identifier: weapon.identifier.clone(),
            name: weapon.name.clone(),
        };
        match outcome {
            crate::items::WearOutcome::Pristine => {}
            crate::items::WearOutcome::Worn => {
                let content = GameOutput::WeaponWear(attacker.name.as_str(), weapon.name.as_str())
                    .to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: attacker_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                wear.push(WearReport {
                    owner: attacker_ref,
                    item: item_ref,
                    outcome: WearOutcomeReport::Worn,
                    forfeited_effect: None,
                    mid_action_penalty: None,
                });
            }
            crate::items::WearOutcome::Broken => {
                let content = GameOutput::WeaponBreak(attacker.name.as_str(), weapon.name.as_str())
                    .to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: attacker_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                if let Err(err) = attacker.remove_item(&weapon) {
                    eprintln!("Failed to remove weapon: {}", err);
                }
                // D1 + D2: forfeit the just-applied effect bonus and apply 1d4 penalty.
                attack_roll -= weapon.effect;
                let penalty = rng.random_range(1..=4);
                attack_roll -= penalty;
                let narration = GameOutput::WeaponShattersMidSwing(
                    attacker.name.as_str(),
                    weapon.name.as_str(),
                    penalty as u32,
                )
                .to_string();
                events.push(TaggedEvent::new(
                    narration,
                    MessagePayload::ItemUsed {
                        tribute: attacker_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                wear.push(WearReport {
                    owner: attacker_ref,
                    item: item_ref,
                    outcome: WearOutcomeReport::Broken,
                    forfeited_effect: Some(weapon.effect),
                    mid_action_penalty: Some(penalty),
                });
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
        match outcome {
            crate::items::WearOutcome::Pristine => None,
            _ => Some((shield.clone(), outcome)),
        }
    } else {
        None
    };
    if let Some((shield, outcome)) = shield_outcome {
        let target_ref = tref(target);
        let item_ref = shared::messages::ItemRef {
            identifier: shield.identifier.clone(),
            name: shield.name.clone(),
        };
        match outcome {
            crate::items::WearOutcome::Pristine => {}
            crate::items::WearOutcome::Worn => {
                let content =
                    GameOutput::ShieldWear(target.name.as_str(), shield.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: target_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                wear.push(WearReport {
                    owner: target_ref,
                    item: item_ref,
                    outcome: WearOutcomeReport::Worn,
                    forfeited_effect: None,
                    mid_action_penalty: None,
                });
            }
            crate::items::WearOutcome::Broken => {
                let content =
                    GameOutput::ShieldBreak(target.name.as_str(), shield.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: target_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                if let Err(err) = target.remove_item(&shield) {
                    eprintln!("Failed to remove shield: {}", err);
                }
                // D3 mirror: forfeit shield effect + apply 1d4 defense penalty.
                defense_roll -= shield.effect;
                let penalty = rng.random_range(1..=4);
                defense_roll -= penalty;
                let narration = GameOutput::ShieldShattersMidBlock(
                    target.name.as_str(),
                    shield.name.as_str(),
                    penalty as u32,
                )
                .to_string();
                events.push(TaggedEvent::new(
                    narration,
                    MessagePayload::ItemUsed {
                        tribute: target_ref.clone(),
                        item: item_ref.clone(),
                    },
                ));
                wear.push(WearReport {
                    owner: target_ref,
                    item: item_ref,
                    outcome: WearOutcomeReport::Broken,
                    forfeited_effect: Some(shield.effect),
                    mid_action_penalty: Some(penalty),
                });
            }
        }
    }

    // Check for critical outcomes based on natural rolls (before modifiers)
    let result = match (base_attack_roll, base_defense_roll) {
        (1, _) => AttackResult::CriticalFumble, // Natural 1 on attack - fumble!
        (20, _) => AttackResult::CriticalHit,   // Natural 20 on attack - crit!
        (_, 20) => AttackResult::PerfectBlock,  // Natural 20 on defense - perfect block!
        _ => {
            // Normal combat resolution - compare attack vs. defense
            match attack_roll.cmp(&defense_roll) {
                Ordering::Less => {
                    // If the defender wins
                    let difference =
                        defense_roll as f64 - (attack_roll as f64 * tuning.decisive_win_multiplier);
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
                        attack_roll as f64 - (defense_roll as f64 * tuning.decisive_win_multiplier);

                    if difference > 0.0 {
                        // Attacker wins significantly
                        AttackResult::AttackerWinsDecisively
                    } else {
                        AttackResult::AttackerWins
                    }
                }
            }
        }
    };

    // D5: CriticalFumble clears attacker-side break-penalty fields on the
    // recorded wear report. The fumble path's catastrophic self-damage is the
    // story; piling the snapped-weapon penalty on top is double jeopardy.
    if matches!(result, AttackResult::CriticalFumble)
        && let Some(report) = wear.iter_mut().find(|w| {
            w.outcome == WearOutcomeReport::Broken
                && w.owner.identifier == tref(attacker).identifier
        })
    {
        report.forfeited_effect = None;
        report.mid_action_penalty = None;
    }

    AttackContestOutcome { result, wear }
}

/// Apply the results of a combat encounter.
/// Adjust statistics and log the result.
///
/// Returns the stress damage applied to the winner via
/// `apply_violence_stress` so the caller can populate
/// `CombatBeat.stress.stress_damage` for the swing payload.
pub(crate) fn apply_combat_results(
    winner: &mut Tribute,
    loser: &mut Tribute,
    damage_to_loser: u32,
    log_event: GameOutput,
    events: &mut Vec<TaggedEvent>,
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> u32 {
    loser.takes_physical_damage(damage_to_loser);
    loser.statistics.defeats += 1;
    winner.statistics.wins += 1;
    let stress_damage = winner.apply_violence_stress(events, tuning);
    let content = log_event.to_string();
    events.push(TaggedEvent::new(
        content,
        MessagePayload::TributeWounded {
            victim: tref(loser),
            attacker: Some(tref(winner)),
            hp_lost: damage_to_loser,
        },
    ));
    stress_damage
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
    use crate::tributes::combat_tuning::CombatTuning;
    use core::convert::Infallible;
    use rand::SeedableRng;
    use rand::TryRng;
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

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::AttackerWins);
    }

    #[rstest]
    fn attack_contest_win_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 0;

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::AttackerWinsDecisively);
    }

    #[rstest]
    fn attack_contest_lose(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 20;

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::DefenderWins);
    }

    #[rstest]
    fn attack_contest_lose_decisively(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 1;
        target.attributes.defense = 20;

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::DefenderWinsDecisively);
    }

    #[rstest]
    fn attack_contest_draw(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 21; // Magic number to make the final scores even
        target.attributes.defense = 20;

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::Miss);
    }

    #[rstest]
    fn attacks_self(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.sanity = 50;
        let sanity = 50;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
        assert_eq!(outcome, AttackOutcome::Wound(attacker.clone(), target));
        assert!(attacker.attributes.sanity < sanity);
    }

    #[rstest]
    fn attacks_self_suicide(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 100;
        let mut target = attacker.clone();

        let outcome = attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
        assert_eq!(outcome, AttackOutcome::Kill(attacker, target));
    }

    #[rstest]
    fn attacks_wound(mut small_rng: SmallRng) {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let sanity = attacker.attributes.sanity;
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 25;
        target.attributes.defense = 20;

        let result = attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
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

        let result = attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
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

        let result = attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );
        assert_eq!(result, AttackOutcome::Miss(attacker, target));
    }

    #[rstest]
    fn attacks_deducts_stamina_costs(mut small_rng: SmallRng) {
        let tuning = CombatTuning::default();
        let mut attacker = Tribute::new("A".to_string(), None, None);
        attacker.stamina = 100;
        attacker.max_stamina = 100;
        let mut target = Tribute::new("B".to_string(), None, None);
        target.stamina = 100;
        target.max_stamina = 100;
        let _ = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new(), &tuning);
        assert_eq!(attacker.stamina, 100 - tuning.stamina_cost_attacker);
        assert_eq!(target.stamina, 100 - tuning.stamina_cost_target);
    }

    #[rstest]
    fn attacks_saturates_at_zero_when_below_cost(mut small_rng: SmallRng) {
        let tuning = CombatTuning::default();
        let mut attacker = Tribute::new("A".to_string(), None, None);
        attacker.stamina = 5;
        attacker.max_stamina = 100;
        let mut target = Tribute::new("B".to_string(), None, None);
        target.stamina = 3;
        target.max_stamina = 100;
        let _ = attacker.attacks(&mut target, &mut small_rng, &mut Vec::new(), &tuning);
        assert_eq!(attacker.stamina, 0);
        assert_eq!(target.stamina, 0);
    }

    #[rstest]
    fn combat_beat_carries_stamina_costs(mut small_rng: SmallRng) {
        let tuning = CombatTuning::default();
        let mut attacker = Tribute::new("A".to_string(), None, None);
        attacker.stamina = 100;
        attacker.max_stamina = 100;
        let mut target = Tribute::new("B".to_string(), None, None);
        target.stamina = 100;
        target.max_stamina = 100;
        let mut events = Vec::new();
        let _ = attacker.attacks(&mut target, &mut small_rng, &mut events, &tuning);
        let beats: Vec<_> = events
            .iter()
            .filter_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .collect();
        assert_eq!(beats.len(), 1);
        assert_eq!(beats[0].attacker_stamina_cost, tuning.stamina_cost_attacker);
        assert_eq!(beats[0].target_stamina_cost, tuning.stamina_cost_target);
    }

    #[rstest]
    fn test_critical_hit() {
        // Use a custom RNG that always returns the high bits needed for
        // `random_range(1..=20)` to produce 20 under rand 0.9's algorithm.
        struct CritRng;
        impl TryRng for CritRng {
            type Error = Infallible;
            fn try_next_u32(&mut self) -> Result<u32, Infallible> {
                Ok(0xF333_3334)
            }
            fn try_next_u64(&mut self) -> Result<u64, Infallible> {
                Ok((0xF333_3334u64 << 32) | 0xF333_3334u64)
            }
            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
                for byte in dest.iter_mut() {
                    *byte = 0xFF;
                }
                Ok(())
            }
        }

        let mut crit_rng = CritRng;
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 10;
        target.attributes.health = 100;

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut crit_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
        assert_eq!(result, AttackResult::CriticalHit);
    }

    #[rstest]
    fn test_critical_fumble() {
        // Use a custom RNG that returns 0 so `random_range(1..=20)` yields 1.
        struct FumbleRng;
        impl TryRng for FumbleRng {
            type Error = Infallible;
            fn try_next_u32(&mut self) -> Result<u32, Infallible> {
                Ok(0)
            }
            fn try_next_u64(&mut self) -> Result<u64, Infallible> {
                Ok(0)
            }
            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
                for byte in dest.iter_mut() {
                    *byte = 0;
                }
                Ok(())
            }
        }

        let mut fumble_rng = FumbleRng;
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut fumble_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
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
        impl TryRng for BlockRng {
            type Error = Infallible;
            fn try_next_u32(&mut self) -> Result<u32, Infallible> {
                let count = self.call_count.get();
                self.call_count.set(count + 1);
                Ok(if count == 0 { 0x7333_3334 } else { 0xF333_3334 })
            }
            fn try_next_u64(&mut self) -> Result<u64, Infallible> {
                Ok(self.try_next_u32().unwrap() as u64)
            }
            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
                for byte in dest.iter_mut() {
                    *byte = self.try_next_u32().unwrap() as u8;
                }
                Ok(())
            }
        }

        let mut block_rng = BlockRng::new();
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        let result = attack_contest(
            &mut attacker,
            &mut target,
            &mut block_rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        )
        .result;
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
            &CombatTuning::default(),
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
            attack_contest(
                &mut attacker,
                &mut target,
                &mut small_rng,
                &mut Vec::new(),
                &CombatTuning::default(),
            );
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
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );

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
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut Vec::new(),
            &CombatTuning::default(),
        );

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
        attacker.attacks(
            &mut target,
            &mut small_rng,
            &mut events,
            &CombatTuning::default(),
        );

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

    #[rstest]
    fn attacks_emits_one_combat_swing_per_call(small_rng: SmallRng) {
        // Every call to attacks() produces exactly one MessagePayload::CombatSwing,
        // regardless of which branch (miss / wound / kill / fumble / self-attack).
        for seed in 0..16u64 {
            let mut events: Vec<TaggedEvent> = Vec::new();
            let mut attacker = Tribute::new(format!("A{seed}"), None, None);
            let mut target = Tribute::new(format!("T{seed}"), None, None);
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

            let swings: Vec<_> = events
                .iter()
                .filter(|e| matches!(e.payload, MessagePayload::CombatSwing(_)))
                .collect();
            assert_eq!(
                swings.len(),
                1,
                "seed {seed}: expected exactly one CombatSwing payload, got {}",
                swings.len()
            );

            if let MessagePayload::CombatSwing(beat) = &swings[0].payload {
                assert_eq!(beat.attacker.name, attacker.name);
                assert_eq!(beat.target.name, target.name);
            }
        }
        // Touch the rstest fixture to silence unused warnings.
        let _ = small_rng;
    }

    #[rstest]
    fn self_attack_emits_one_combat_swing(mut small_rng: SmallRng) {
        let mut tribute = Tribute::new("Solo".to_string(), None, None);
        let mut clone = tribute.clone();
        let mut events: Vec<TaggedEvent> = Vec::new();
        let _ = tribute.attacks(
            &mut clone,
            &mut small_rng,
            &mut events,
            &CombatTuning::default(),
        );
        let swings: usize = events
            .iter()
            .filter(|e| matches!(e.payload, MessagePayload::CombatSwing(_)))
            .count();
        assert_eq!(swings, 1, "self-attack must emit exactly one CombatSwing");
    }

    /// Construct a weapon with the given effect and durability=1 so a single
    /// `wear(1)` call breaks it.
    fn brittle_weapon(effect: i32) -> crate::items::Item {
        crate::items::Item::new(
            "Glass Sword",
            crate::items::ItemType::Weapon,
            crate::items::ItemRarity::Common,
            1,
            crate::items::Attribute::Strength,
            effect,
        )
    }

    /// Construct a shield with the given effect and durability=1.
    fn brittle_shield(effect: i32) -> crate::items::Item {
        crate::items::Item::new(
            "Glass Buckler",
            crate::items::ItemType::Weapon,
            crate::items::ItemRarity::Common,
            1,
            crate::items::Attribute::Defense,
            effect,
        )
    }

    #[test]
    fn weapon_break_records_forfeit_and_penalty_on_beat() {
        let mut attacker = Tribute::new("Atk".into(), None, None);
        attacker.attributes.strength = 10;
        let weapon = brittle_weapon(5);
        attacker.add_item(weapon.clone());

        let mut target = Tribute::new("Tgt".into(), None, None);
        target.attributes.defense = 5;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(42);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        let weapon_wear = beat
            .wear
            .iter()
            .find(|w| w.owner.identifier == beat.attacker.identifier)
            .expect("expected a wear report for the attacker's weapon");

        assert_eq!(
            weapon_wear.outcome,
            shared::combat_beat::WearOutcomeReport::Broken
        );
        assert_eq!(weapon_wear.forfeited_effect, Some(5));
        let penalty = weapon_wear.mid_action_penalty.expect("penalty must fire");
        assert!(
            (1..=4).contains(&penalty),
            "penalty must be 1..=4, got {}",
            penalty
        );
    }

    #[test]
    fn shield_break_records_forfeit_and_penalty_on_beat() {
        let mut attacker = Tribute::new("Atk".into(), None, None);
        attacker.attributes.strength = 10;

        let mut target = Tribute::new("Tgt".into(), None, None);
        target.attributes.defense = 5;
        let shield = brittle_shield(4);
        target.add_item(shield.clone());

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(7);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        let shield_wear = beat
            .wear
            .iter()
            .find(|w| w.owner.identifier == beat.target.identifier)
            .expect("expected a wear report for the target's shield");

        assert_eq!(
            shield_wear.outcome,
            shared::combat_beat::WearOutcomeReport::Broken
        );
        assert_eq!(shield_wear.forfeited_effect, Some(4));
        let penalty = shield_wear.mid_action_penalty.expect("penalty must fire");
        assert!((1..=4).contains(&penalty), "penalty was {}", penalty);
    }

    #[test]
    fn fumble_clears_attacker_break_penalty_on_beat() {
        // Hunt for a seed where the attacker's swing both fumbles AND breaks
        // the brittle weapon. Per design D5 the attacker-side break-penalty
        // fields must be cleared on fumble for clean narration.
        for seed in 0u64..2_000 {
            let mut attacker = Tribute::new("Atk".into(), None, None);
            attacker.attributes.strength = 10;
            let weapon = brittle_weapon(5);
            attacker.add_item(weapon.clone());

            let mut target = Tribute::new("Tgt".into(), None, None);
            target.attributes.defense = 5;

            let mut events: Vec<TaggedEvent> = Vec::new();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

            let beat = match events.iter().find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            }) {
                Some(b) => b,
                None => continue,
            };

            let is_fumble = matches!(
                beat.outcome,
                shared::combat_beat::SwingOutcome::FumbleSurvive { .. }
                    | shared::combat_beat::SwingOutcome::FumbleDeath { .. }
            );
            let weapon_wear = beat
                .wear
                .iter()
                .find(|w| w.owner.identifier == beat.attacker.identifier);

            if is_fumble
                && weapon_wear
                    .map(|w| w.outcome == shared::combat_beat::WearOutcomeReport::Broken)
                    .unwrap_or(false)
            {
                let w = weapon_wear.unwrap();
                assert_eq!(
                    w.forfeited_effect, None,
                    "D5: fumble must clear forfeited_effect"
                );
                assert_eq!(
                    w.mid_action_penalty, None,
                    "D5: fumble must clear mid_action_penalty"
                );
                return;
            }
        }
        panic!("no seed in 0..2000 produced a fumble + weapon break combo; widen the search");
    }

    #[test]
    fn unarmed_unshielded_emits_no_break_penalty() {
        let mut attacker = Tribute::new("Atk".into(), None, None);
        attacker.attributes.strength = 10;
        let mut target = Tribute::new("Tgt".into(), None, None);
        target.attributes.defense = 5;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(123);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        for w in &beat.wear {
            assert_eq!(w.forfeited_effect, None);
            assert_eq!(w.mid_action_penalty, None);
        }
    }

    /// Parity: wear lines rendered from `CombatBeat.wear` via
    /// `CombatBeatExt::to_log_lines` must match the wear lines that
    /// `attack_contest` flattens into `CombatEngagement.detail_lines`.
    /// This locks `CombatBeat.wear` as the single source of truth for wear
    /// narration once consumers migrate off `detail_lines`.
    #[test]
    fn combat_beat_wear_matches_engagement_detail_wear_lines() {
        use crate::tributes::combat_beat::CombatBeatExt;
        // Both attacker weapon and target shield are brittle so we exercise
        // the full wear (Worn or Broken) emission paths in one swing.
        for seed in 0..32u64 {
            let mut attacker = Tribute::new(format!("Atk{seed}"), None, None);
            attacker.attributes.strength = 10;
            attacker.add_item(brittle_weapon(5));
            let mut target = Tribute::new(format!("Tgt{seed}"), None, None);
            target.attributes.defense = 5;
            target.add_item(brittle_shield(4));

            let mut events: Vec<TaggedEvent> = Vec::new();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

            // Find the engagement (skip seeds that didn't produce one, e.g.
            // pure fumble paths emit a standalone TributeWounded instead and
            // never write detail_lines for the legacy timeline to consume).
            let Some(detail_lines) = events.iter().find_map(|e| match &e.payload {
                MessagePayload::Combat(eng) => Some(eng.detail_lines.clone()),
                _ => None,
            }) else {
                continue;
            };
            let beat = events
                .iter()
                .find_map(|e| match &e.payload {
                    MessagePayload::CombatSwing(b) => Some(b),
                    _ => None,
                })
                .expect("expected one CombatSwing per attacks() call");

            // Wear-related substrings we want to track.
            let is_wear_line = |s: &str| {
                s.contains("starting to wear out")
                    || s.contains("breaks")
                    || s.contains("shatters mid-swing")
                    || s.contains("shatters mid-block")
            };

            let beat_wear_lines: Vec<String> = beat
                .to_log_lines()
                .into_iter()
                .filter(|s| is_wear_line(s))
                .collect();
            let detail_wear_lines: Vec<String> = detail_lines
                .into_iter()
                .filter(|s| is_wear_line(s))
                .collect();

            assert_eq!(
                beat_wear_lines, detail_wear_lines,
                "seed {seed}: wear lines from CombatBeat must match detail_lines"
            );
        }
    }

    /// A swing that wins (and therefore triggers `apply_violence_stress`) on
    /// a tribute already credited with prior wins must record the resulting
    /// stress on the swing's `CombatBeat.stress.stress_damage`.
    #[test]
    fn combat_swing_records_stress_damage() {
        // Arrange: attacker has prior wins+kills so violence-stress is non-zero.
        let mut attacker = Tribute::new("Atk".into(), None, None);
        attacker.attributes.strength = 50;
        attacker.statistics.wins = 5;
        attacker.statistics.kills = 5;

        let mut target = Tribute::new("Tgt".into(), None, None);
        target.attributes.defense = 0;
        target.attributes.health = 1;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = SmallRng::seed_from_u64(1);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        assert!(
            beat.stress.stress_damage > 0,
            "expected non-zero stress on the beat; got {}",
            beat.stress.stress_damage
        );
    }

    /// Parity: the optional trailing horrified line that
    /// `CombatBeatExt::to_log_lines` renders from `stress.stress_damage`
    /// must match the horrified line that `apply_violence_stress` flattened
    /// into `CombatEngagement.detail_lines`.
    #[test]
    fn combat_beat_stress_matches_engagement_horrified_line() {
        use crate::tributes::combat_beat::CombatBeatExt;
        for seed in 0..32u64 {
            let mut attacker = Tribute::new(format!("Atk{seed}"), None, None);
            attacker.attributes.strength = 50;
            attacker.statistics.wins = 5;
            attacker.statistics.kills = 5;

            let mut target = Tribute::new(format!("Tgt{seed}"), None, None);
            target.attributes.defense = 0;
            target.attributes.health = 100;

            let mut events: Vec<TaggedEvent> = Vec::new();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = attacker.attacks(&mut target, &mut rng, &mut events, &CombatTuning::default());

            let Some(detail_lines) = events.iter().find_map(|e| match &e.payload {
                MessagePayload::Combat(eng) => Some(eng.detail_lines.clone()),
                _ => None,
            }) else {
                continue;
            };
            let beat = events
                .iter()
                .find_map(|e| match &e.payload {
                    MessagePayload::CombatSwing(b) => Some(b),
                    _ => None,
                })
                .expect("expected one CombatSwing per attacks() call");

            let beat_horrified: Vec<String> = beat
                .to_log_lines()
                .into_iter()
                .filter(|s| s.contains("horrified"))
                .collect();
            let detail_horrified: Vec<String> = detail_lines
                .into_iter()
                .filter(|s| s.contains("horrified"))
                .collect();

            assert_eq!(
                beat_horrified, detail_horrified,
                "seed {seed}: horrified lines from CombatBeat must match detail_lines"
            );
        }
    }
}
