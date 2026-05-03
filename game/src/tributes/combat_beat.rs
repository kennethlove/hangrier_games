//! Game-side narration for `shared::combat_beat::CombatBeat`.
//!
//! The data types live in `shared` so they can ride `MessagePayload`. The
//! narration depends on `crate::output::GameOutput`, so it lives here as an
//! extension trait.

pub use shared::combat_beat::{
    CombatBeat, StressReport, SwingOutcome, WearOutcomeReport, WearReport,
};

use crate::output::GameOutput;

/// Render a `CombatBeat` into the prose lines historically emitted by
/// `Tribute::attacks()`.
///
/// Order: wear lines (weapon then shield, in `wear` vec order), then outcome
/// lines, then optional trailing horrified line if `stress.stress_damage > 0`.
pub trait CombatBeatExt {
    fn to_log_lines(&self) -> Vec<String>;
}

impl CombatBeatExt for CombatBeat {
    fn to_log_lines(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(4);

        // 1. Wear lines.
        for w in &self.wear {
            match w.outcome {
                WearOutcomeReport::Pristine => {}
                WearOutcomeReport::Worn => {
                    if w.owner.identifier == self.attacker.identifier {
                        out.push(GameOutput::WeaponWear(&w.owner.name, &w.item.name).to_string());
                    } else {
                        out.push(GameOutput::ShieldWear(&w.owner.name, &w.item.name).to_string());
                    }
                }
                WearOutcomeReport::Broken => {
                    if w.owner.identifier == self.attacker.identifier {
                        out.push(GameOutput::WeaponBreak(&w.owner.name, &w.item.name).to_string());
                    } else {
                        out.push(GameOutput::ShieldBreak(&w.owner.name, &w.item.name).to_string());
                    }
                }
            }
        }

        // 2. Outcome lines.
        let a = &self.attacker.name;
        let t = &self.target.name;
        match &self.outcome {
            SwingOutcome::Miss => {
                out.push(GameOutput::TributeAttackMiss(a, t).to_string());
            }
            SwingOutcome::Wound { .. } => {
                out.push(GameOutput::TributeAttackWin(a, t).to_string());
                out.push(GameOutput::TributeAttackWound(a, t).to_string());
            }
            SwingOutcome::CriticalHitWound { .. } => {
                out.push(GameOutput::TributeCriticalHit(a, t).to_string());
                out.push(GameOutput::TributeAttackWin(a, t).to_string());
                out.push(GameOutput::TributeAttackWound(a, t).to_string());
            }
            SwingOutcome::BlockWound { .. } => {
                out.push(GameOutput::TributePerfectBlock(t, a).to_string());
                out.push(GameOutput::TributeAttackLose(t, a).to_string());
                out.push(GameOutput::TributeAttackWound(a, t).to_string());
            }
            SwingOutcome::Kill { .. } => {
                out.push(GameOutput::TributeAttackWin(a, t).to_string());
                out.push(GameOutput::TributeAttackSuccessKill(a, t).to_string());
            }
            SwingOutcome::AttackerDied { .. } => {
                out.push(GameOutput::TributeAttackLose(t, a).to_string());
                out.push(GameOutput::TributeAttackDied(a, t).to_string());
            }
            SwingOutcome::FumbleSurvive { .. } => {
                out.push(GameOutput::TributeCriticalFumble(a).to_string());
            }
            SwingOutcome::FumbleDeath { .. } => {
                out.push(GameOutput::TributeCriticalFumble(a).to_string());
                out.push(GameOutput::TributeAttackDied(a, "themselves").to_string());
            }
            SwingOutcome::SelfAttackWound { .. } => {
                out.push(GameOutput::TributeSelfHarm(a).to_string());
                out.push(GameOutput::TributeAttackWin(a, a).to_string());
                out.push(GameOutput::TributeAttackWound(a, a).to_string());
            }
            SwingOutcome::Suicide { .. } => {
                out.push(GameOutput::TributeSelfHarm(a).to_string());
                out.push(GameOutput::TributeAttackWin(a, a).to_string());
                out.push(GameOutput::TributeSuicide(a).to_string());
            }
        }

        // 3. Optional trailing horrified line.
        if self.stress.stress_damage > 0 {
            out.push(GameOutput::TributeHorrified(a, self.stress.stress_damage).to_string());
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::TributeRef;

    fn t(name: &str) -> TributeRef {
        TributeRef {
            identifier: format!("id-{name}"),
            name: name.into(),
        }
    }

    #[test]
    fn miss_renders_one_line() {
        let beat = CombatBeat {
            attacker: t("Alice"),
            target: t("Bob"),
            weapon: None,
            shield: None,
            wear: vec![],
            outcome: SwingOutcome::Miss,
            stress: StressReport::default(),
        };
        let lines = beat.to_log_lines();
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("missed") || lines[0].contains("miss"),
            "got: {}",
            lines[0]
        );
    }
}
