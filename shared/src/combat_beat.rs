//! Typed payload describing one combat swing.
//!
//! One `CombatBeat` is produced per `Tribute::attacks()` call. The data lives
//! in `shared` so it can ride `MessagePayload::CombatSwing(CombatBeat)`. The
//! narration `to_log_lines()` lives in the `game` crate, since it depends on
//! `GameOutput` rendering.

use crate::messages::{ItemRef, TributeRef};
use serde::{Deserialize, Serialize};

/// What happened to a piece of equipment during the swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WearOutcomeReport {
    Pristine,
    Worn,
    Broken,
}

/// Wear/break record for one piece of equipment used in the swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WearReport {
    /// Owner of the item (attacker for weapon, target for shield).
    pub owner: TributeRef,
    pub item: ItemRef,
    pub outcome: WearOutcomeReport,
}

/// High-level outcome of one swing.
///
/// Mirrors the post-resolution branches in the legacy `attacks()`. New variant
/// `FumbleDeath` covers the previously implicit "fumble killed the attacker"
/// path that the old code hid inside `AttackOutcome::Kill(target.clone(), self)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwingOutcome {
    /// Attack missed entirely.
    Miss,
    /// Attacker landed a hit; target survived.
    Wound { damage: u32 },
    /// Attacker scored a critical hit; target survived.
    CriticalHitWound { damage: u32 },
    /// Defender countered (PerfectBlock); attacker took damage; attacker survived.
    BlockWound { damage: u32 },
    /// Target was killed by the attacker.
    Kill { damage: u32 },
    /// Attacker was killed by the target's counter (PerfectBlock or DefenderWins killed self).
    AttackerDied { damage: u32 },
    /// Attacker fumbled (nat-1) and survived self-damage.
    FumbleSurvive { self_damage: u32 },
    /// Attacker fumbled (nat-1) and killed themselves.
    FumbleDeath { self_damage: u32 },
    /// Attacker == target. Self-attack that wounded.
    SelfAttackWound { damage: u32 },
    /// Attacker == target. Self-attack that killed.
    Suicide { damage: u32 },
}

/// Stress damage applied to attacker after the swing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StressReport {
    /// Mental damage applied via `apply_violence_stress`. 0 means no horrified line.
    pub stress_damage: u32,
}

/// Full record of one swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatBeat {
    pub attacker: TributeRef,
    pub target: TributeRef,
    /// Weapon equipped by attacker at swing start (None if unarmed).
    pub weapon: Option<ItemRef>,
    /// Shield equipped by target at swing start (None if unshielded).
    pub shield: Option<ItemRef>,
    /// Wear/break records emitted in attack-roll order: weapon first, then shield.
    pub wear: Vec<WearReport>,
    /// Final outcome of the swing.
    pub outcome: SwingOutcome,
    /// Stress applied to attacker after the resolution (may be 0).
    pub stress: StressReport,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(name: &str) -> TributeRef {
        TributeRef {
            identifier: "id".into(),
            name: name.into(),
        }
    }

    #[test]
    fn beat_roundtrips_via_serde() {
        let beat = CombatBeat {
            attacker: t("A"),
            target: t("B"),
            weapon: None,
            shield: None,
            wear: vec![],
            outcome: SwingOutcome::Miss,
            stress: StressReport::default(),
        };
        let json = serde_json::to_string(&beat).unwrap();
        let back: CombatBeat = serde_json::from_str(&json).unwrap();
        assert_eq!(beat, back);
    }
}
