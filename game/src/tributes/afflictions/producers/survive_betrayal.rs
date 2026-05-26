//! Producer (d): Survive betrayal — Moderate trauma + phobia co-acquire stub.

use crate::games::Game;
use crate::messages::{MessagePayload, Phase};
use shared::afflictions::{Severity, TraumaSource};

use super::shared::{TraumaEvent, apply_trauma_events};

pub(super) fn produce_survive_betrayal(game: &mut Game, phase: Phase) {
    let mut events: Vec<TraumaEvent> = Vec::new();

    for msg in &game.messages {
        if msg.phase != phase {
            continue;
        }
        let MessagePayload::BetrayalTriggered { betrayer, victim } = &msg.payload else {
            continue;
        };

        let Some(tribute) = game
            .tributes
            .iter()
            .find(|t| t.identifier == victim.identifier)
        else {
            continue;
        };

        if !tribute.is_alive() {
            continue;
        }

        events.push(TraumaEvent {
            tribute_id: tribute.identifier.clone(),
            tribute_name: tribute.name.clone(),
            source: TraumaSource::Betrayal {
                by: betrayer.identifier.clone(),
            },
            severity: Severity::Moderate,
            cause_hint: betrayer.name.clone(),
        });
    }

    apply_trauma_events(game, events, true);
}
