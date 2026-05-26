//! Trauma producer pipeline (spec §4, PR2).
//!
//! Scans the current phase's message log and acquires/reinforces trauma
//! afflictions on living tributes who witnessed or survived traumatic events.
//! Gated on `game.config.trauma_enabled`.
//!
//! Producers:
//! - (a)/(b) Witness ally death — Mild trauma + phobia co-acquire stub
//! - (c) Survive near-death — Moderate trauma
//! - (d) Survive betrayal — Moderate trauma + phobia co-acquire stub
//! - (f) Witness mass casualty — Moderate/Severe based on death count

mod shared;
mod survive_betrayal;
mod survive_near_death;
#[cfg(test)]
mod tests;
mod witness_ally_death;
mod witness_mass_casualty;

use crate::games::Game;

/// Run all trauma producers against the current phase's messages.
///
/// Gate: returns immediately if `game.config.trauma_enabled` is false.
pub fn run_trauma_producers(game: &mut Game) {
    if !game.config.trauma_enabled {
        return;
    }

    let phase = game.current_phase;

    witness_ally_death::produce_witness_ally_death(game, phase);
    survive_near_death::produce_survive_near_death(game, phase);
    survive_betrayal::produce_survive_betrayal(game, phase);
    witness_mass_casualty::produce_witness_mass_casualty(game, phase);
}
