//! Post-combat statistics updates.

use crate::tributes::Tribute;
use crate::tributes::actions::AttackResult;

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
