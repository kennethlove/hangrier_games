//! Producer (c): Survive near-death — Moderate trauma.

use crate::games::Game;
use crate::messages::{MessagePayload, Phase};
use shared::afflictions::{DeathCause, Severity, TraumaSource};

use super::shared::{TraumaEvent, apply_trauma_events};

pub(super) fn produce_survive_near_death(game: &mut Game, phase: Phase) {
    // Collect wound events that pushed a tribute to <= 10% HP
    let mut events: Vec<TraumaEvent> = Vec::new();

    for msg in &game.messages {
        if msg.phase != phase {
            continue;
        }
        let MessagePayload::TributeWounded {
            victim,
            attacker,
            hp_lost: _,
        } = &msg.payload
        else {
            continue;
        };

        let Some(tribute) = game
            .tributes
            .iter()
            .find(|t| victim.identifier == t.identifier)
        else {
            continue;
        };

        let hp_after = tribute.effective_health();
        let hp_percent = hp_after;

        if hp_percent <= 10 && tribute.is_alive() {
            let death_cause = attacker
                .as_ref()
                .map(|a| DeathCause::Tribute(a.identifier.to_string()))
                .unwrap_or(DeathCause::Unknown);

            events.push(TraumaEvent {
                tribute_id: tribute.identifier.to_string(),
                tribute_name: tribute.name.clone(),
                source: TraumaSource::NearDeath { cause: death_cause },
                severity: Severity::Moderate,
                cause_hint: shared::afflictions::DeathCause::Unknown,
            });
        }
    }

    apply_trauma_events(game, events, false);
}
