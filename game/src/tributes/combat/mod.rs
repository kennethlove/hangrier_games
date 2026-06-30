//! Combat-related functionality for tributes.
//!
//! This module handles all attack and combat mechanics including:
//! - Attack contests between tributes
//! - Combat result application
//! - Violence stress calculations
//! - Statistics updates

pub mod inflict_table;
pub mod resolve;
pub mod stats;

#[cfg(test)]
mod tests;

// Re-exports for public API.
pub use resolve::{AttackContestOutcome, attack_contest, resolve};
pub use stats::update_stats;

// Re-export for tests (pub(crate) items from resolve).
pub(crate) use resolve::apply_combat_results;

use crate::messages::{CombatEngagement, CombatOutcome, MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::Tribute;
use crate::tributes::actions::{AttackOutcome, AttackResult};
use rand::prelude::*;
use resolve::tref;

// ---------------------------------------------------------------------------
// Tribute::attacks — the combat orchestrator
// ---------------------------------------------------------------------------

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
        phase: shared::messages::Phase,
        tuning: &crate::tributes::combat_tuning::CombatTuning,
    ) -> AttackOutcome {
        // Check self-attack BEFORE deducting stamina (stamina mutation would
        // otherwise break the derived PartialEq equality check below).
        let is_self_attack = self == target;

        // Sleep ambush (PR2c.2, bd-1zju). If the target is asleep we wake
        // them with `InterruptionKind::Ambush` BEFORE damage resolution so
        // the wake-event precedes any TributeWounded / TributeKilled
        // emission. The ambush still lands — sleeping targets still take
        // the swing — but at least the timeline reflects the rude awakening.
        if !is_self_attack && target.sleeping {
            target.was_ambushed = true; // Signal to attack_contest: 0 defense
            let attacker_ref = TributeRef {
                identifier: self.identifier.clone(),
                name: self.name.clone(),
            };
            target.wake_interrupted(
                shared::messages::InterruptionKind::Ambush {
                    attacker: attacker_ref,
                },
                phase,
                events,
            );
        }

        // Per-swing stamina cost: deduct from both combatants up-front.
        // Saturating semantics ensure neither tribute goes negative.
        // Action-gating (refusing to swing while exhausted) lands in Task 10.
        self.stamina = self.stamina.saturating_sub(tuning.stamina_cost_attacker);
        if !is_self_attack {
            target.stamina = target.stamina.saturating_sub(tuning.stamina_cost_target);
        }

        // Is the tribute attempting suicide?
        if is_self_attack {
            return self.handle_self_attack(target, events, tuning);
        }

        let tribute_name = self.name.clone();
        let target_name = target.name.clone();

        let mut detail_lines: Vec<String> = Vec::new();
        let mut sub_events: Vec<TaggedEvent> = Vec::new();

        let (contest, beat) = resolve(self, target, rng, &mut sub_events, tuning);
        let result = contest.result;
        let target_inflicts = contest.inflicts;
        let attacker_inflicts = contest.attacker_inflicts;

        for ev in sub_events.drain(..) {
            detail_lines.push(ev.content);
        }

        match result {
            AttackResult::CriticalHit => {
                detail_lines.push(
                    GameOutput::TributeCriticalHit(tribute_name.as_str(), target_name.as_str())
                        .to_string(),
                );
                let _ = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 3,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::CriticalFumble => {
                let fumble_content =
                    GameOutput::TributeCriticalFumble(tribute_name.as_str()).to_string();
                self.attributes.health = self.attributes.health.saturating_sub(5);
                self.statistics.defeats += 1;

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
                            cause: shared::afflictions::DeathCause::CriticalFumble,
                        },
                    ));
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
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
                ));
                return AttackOutcome::Wound(self.clone(), target.clone());
            }
            AttackResult::PerfectBlock => {
                detail_lines.push(
                    GameOutput::TributePerfectBlock(target_name.as_str(), tribute_name.as_str())
                        .to_string(),
                );
                let _ = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWins => {
                let _ = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::AttackerWinsDecisively => {
                let _ = apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 2,
                    GameOutput::TributeAttackWinExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWins => {
                let _ = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
                );
                for ev in sub_events.drain(..) {
                    detail_lines.push(ev.content);
                }
            }
            AttackResult::DefenderWinsDecisively => {
                let _ = apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2,
                    GameOutput::TributeAttackLoseExtra(tribute_name.as_str(), target_name.as_str()),
                    &mut sub_events,
                    tuning,
                    rng,
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
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::CombatSwing(beat),
                ));

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        for draft in &target_inflicts {
            let resolution = target.try_acquire_affliction(draft.clone());
            if matches!(
                resolution,
                crate::tributes::afflictions::AcquireResolution::Insert
                    | crate::tributes::afflictions::AcquireResolution::Upgrade(_)
                    | crate::tributes::afflictions::AcquireResolution::Supersede(_)
            ) {
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::AfflictionAcquired {
                        tribute_id: target.identifier.clone(),
                        affliction: draft.kind.to_string(),
                        severity: draft.severity.to_string(),
                    },
                ));
            }
        }
        for draft in &attacker_inflicts {
            let resolution = self.try_acquire_affliction(draft.clone());
            if matches!(
                resolution,
                crate::tributes::afflictions::AcquireResolution::Insert
                    | crate::tributes::afflictions::AcquireResolution::Upgrade(_)
                    | crate::tributes::afflictions::AcquireResolution::Supersede(_)
            ) {
                events.push(TaggedEvent::new(
                    String::new(),
                    MessagePayload::AfflictionAcquired {
                        tribute_id: self.identifier.clone(),
                        affliction: draft.kind.to_string(),
                        severity: draft.severity.to_string(),
                    },
                ));
            }
        }

        let (outcome, attack_outcome) = if self.attributes.health == 0 {
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

        events.push(TaggedEvent::new(
            String::new(),
            MessagePayload::CombatSwing(beat),
        ));

        attack_outcome
    }
}
