//! Combat resolution: attack contest, damage application, violence stress.
//!
//! This submodule owns the pure combat-logic functions:
//! - `attack_contest` — rolls, modifiers, wear, outcome determination
//! - `apply_combat_results` — damage application, stat bookkeeping
//! - `calculate_violence_stress` / `apply_violence_stress` — mental damage
//! - `AttackContestOutcome` — the value object returned by `attack_contest`

use crate::items::{Item, OwnsItems};
use crate::messages::{MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::Tribute;
use crate::tributes::actions::{AttackOutcome, AttackResult};
use crate::tributes::combat::inflict_table::{
    HitSeverity, WeaponKind, lookup_break_mid_swing_inflict, lookup_inflicts,
};
use crate::tributes::stamina_band::stamina_band;
use rand::RngExt;
use rand::prelude::*;
use shared::combat_beat::{CombatBeat, StressReport, SwingOutcome, WearOutcomeReport, WearReport};
use shared::messages::{ItemRef, StaminaBand};
use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// Helpers shared between resolve.rs and mod.rs (via pub(super))
// ---------------------------------------------------------------------------

/// Build a `TributeRef` from a tribute.
pub(super) fn tref(t: &Tribute) -> TributeRef {
    TributeRef {
        identifier: t.identifier.clone(),
        name: t.name.clone(),
    }
}

/// Build an `ItemRef` from an item.
pub(super) fn iref(i: &Item) -> ItemRef {
    ItemRef {
        identifier: i.identifier.clone(),
        name: i.name.clone(),
    }
}

/// Build a baseline `CombatBeat` (no wear, no stress) for the current swing.
/// Callers fill in `outcome` and (eventually) `wear` / `stress` per branch.
pub(super) fn new_beat(
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

// ---------------------------------------------------------------------------
// AttackContestOutcome
// ---------------------------------------------------------------------------

/// Outcome of a single `attack_contest` invocation.
///
/// Carries the high-level `AttackResult` plus enough wear/penalty data for
/// the caller to assemble a `CombatBeat` without re-snapshotting equipment
/// state.
pub struct AttackContestOutcome {
    pub result: AttackResult,
    /// Wear records emitted in attack-roll order: weapon (if equipped) then
    /// shield (if equipped). Items that were `Pristine` are omitted.
    pub wear: Vec<WearReport>,
    /// Affliction drafts to apply to the target (loser of the contest).
    pub inflicts: Vec<crate::tributes::AfflictionDraft>,
    /// Affliction drafts to apply to the attacker (e.g. BreakMidSwing recoil).
    pub attacker_inflicts: Vec<crate::tributes::AfflictionDraft>,
}

// ---------------------------------------------------------------------------
// attack_contest
// ---------------------------------------------------------------------------

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
    tuning: &crate::tributes::combat_tuning::CombatTuning,
) -> AttackContestOutcome {
    // Get attack roll and strength modifier
    let base_attack_roll: i32 = rng.random_range(1..=20); // Base roll
    let mut attack_roll = base_attack_roll;
    attack_roll += attacker.attributes.strength as i32; // Add strength
    {
        let band = stamina_band(attacker.stamina, attacker.max_stamina, tuning);
        let penalty = match band {
            StaminaBand::Fresh => 0,
            StaminaBand::Winded => tuning.winded_roll_penalty,
            StaminaBand::Exhausted => tuning.exhausted_roll_penalty,
        };
        attack_roll += penalty;
    }

    let mut wear: Vec<WearReport> = Vec::new();

    // Track weapon kind and whether it broke for inflict table lookup.
    let mut weapon_kind = WeaponKind::Unarmed;
    let mut weapon_broken = false;

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
        weapon_kind = classify_weapon(&weapon);
        let attacker_ref = tref(attacker);
        let item_ref = ItemRef {
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
                weapon_broken = true;
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
    {
        let band = stamina_band(target.stamina, target.max_stamina, tuning);
        let penalty = match band {
            StaminaBand::Fresh => 0,
            StaminaBand::Winded => tuning.winded_roll_penalty,
            StaminaBand::Exhausted => tuning.exhausted_roll_penalty,
        };
        defense_roll += penalty;
    }

    // Trapped tributes cannot dodge effectively — halve defense
    if target
        .afflictions
        .values()
        .any(|a| matches!(a.kind, shared::afflictions::AfflictionKind::Trapped(_)))
    {
        defense_roll = (defense_roll as f32 / 2.0).ceil() as i32;
    }

    // TODO(dvd): apply sponsor_affinity_penalty(attacker, SPONSOR_PENALTY_ATTACK_TRAPPED)
    //            when attacking a tribute with any AfflictionKind::Trapped(_)

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
        let item_ref = ItemRef {
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

    // Phase 3: affliction inflict table lookup.
    let hit_severity = result_to_hit_severity(&result);
    let attacker_id = attacker.identifier.as_str();
    let (inflicts, attacker_inflicts) =
        if matches!(result, AttackResult::Miss | AttackResult::CriticalFumble) {
            (Vec::new(), Vec::new())
        } else {
            let target_inflicts = lookup_inflicts(weapon_kind, hit_severity, attacker_id, rng);
            // BreakMidSwing follow-through: when weapon shatters, attacker suffers
            // a recoil injury in addition to any target inflicts.
            let attacker_inflicts = if weapon_broken {
                lookup_break_mid_swing_inflict(weapon_kind, attacker_id, rng)
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            (target_inflicts, attacker_inflicts)
        };

    // TODO(dvd): emit SponsorEvent::AttackOnTrapped when attacker wins against
    //            a trapped target, so the sponsorship system can apply affinity
    //            penalties and generate audience-disapproval narration.
    AttackContestOutcome {
        result,
        wear,
        inflicts,
        attacker_inflicts,
    }
}

/// Map an AttackResult to the inflict table's severity band.
fn result_to_hit_severity(result: &AttackResult) -> HitSeverity {
    match result {
        AttackResult::CriticalHit => HitSeverity::Critical,
        AttackResult::AttackerWinsDecisively
        | AttackResult::DefenderWinsDecisively
        | AttackResult::PerfectBlock => HitSeverity::Heavy,
        AttackResult::AttackerWins | AttackResult::DefenderWins => HitSeverity::Normal,
        AttackResult::CriticalFumble | AttackResult::Miss => HitSeverity::Normal,
    }
}

/// Classify an Item into a WeaponKind for the inflict table.
fn classify_weapon(item: &Item) -> WeaponKind {
    let name_lower = item.name.to_lowercase();
    if name_lower.contains("bow") || name_lower.contains("arrow") || name_lower.contains("spear") {
        WeaponKind::Ranged
    } else if name_lower.contains("knife")
        || name_lower.contains("sword")
        || name_lower.contains("blade")
        || name_lower.contains("dagger")
        || name_lower.contains("axe")
    {
        WeaponKind::Bladed
    } else if name_lower.contains("club")
        || name_lower.contains("hammer")
        || name_lower.contains("mace")
        || name_lower.contains("bat")
    {
        WeaponKind::Blunt
    } else {
        WeaponKind::Bladed // Default for unknown weapons
    }
}

// ---------------------------------------------------------------------------
// apply_combat_results
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Violence stress
// ---------------------------------------------------------------------------

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

impl Tribute {
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

    /// Handle self-attack (suicide/self-harm) path.
    ///
    /// Deducts `strength` as physical damage, applies violence stress, and
    /// emits the appropriate events. Returns the outcome immediately.
    pub(crate) fn handle_self_attack(
        &mut self,
        target: &mut Tribute,
        events: &mut Vec<TaggedEvent>,
        tuning: &crate::tributes::combat_tuning::CombatTuning,
    ) -> AttackOutcome {
        let damage = self.attributes.strength;
        self.takes_physical_damage(damage);
        let mut stress_events: Vec<TaggedEvent> = Vec::new();
        let stress_damage = self.apply_violence_stress(&mut stress_events, tuning);
        events.extend(stress_events);

        if self.attributes.health > 0 {
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
        }
    }
}
