//! Producer (a)/(b): Witness ally death — Mild trauma + phobia co-acquire stub.

use crate::areas::Area;
use crate::games::Game;
use crate::messages::{MessagePayload, Phase, TributeRef};
use shared::afflictions::{Severity, TraumaSource};

use super::shared::{TraumaEvent, apply_trauma_events, map_cause_to_death_cause};

pub(super) fn produce_witness_ally_death(game: &mut Game, phase: Phase) {
    // Phase 1: collect killed tributes with area info
    let killed: Vec<(String, Area, uuid::Uuid, Option<TributeRef>, String)> = game
        .messages
        .iter()
        .filter(|m| m.phase == phase)
        .filter_map(|m| match &m.payload {
            MessagePayload::TributeKilled {
                victim,
                killer,
                cause,
            } => game
                .tributes
                .iter()
                .find(|t| t.identifier == victim.identifier)
                .map(|vt| {
                    (
                        victim.identifier.clone(),
                        vt.area,
                        vt.id,
                        killer.clone(),
                        cause.clone(),
                    )
                }),
            _ => None,
        })
        .collect();

    // Phase 2: find living allies in same area
    let mut events: Vec<TraumaEvent> = Vec::new();
    for (victim_id, victim_area, victim_uuid, killer, cause) in &killed {
        for t in &game.tributes {
            if t.is_alive() && t.allies.contains(victim_uuid) && t.area == *victim_area {
                let death_cause = map_cause_to_death_cause(killer.as_ref(), cause);
                events.push(TraumaEvent {
                    tribute_id: t.identifier.clone(),
                    tribute_name: t.name.clone(),
                    source: TraumaSource::WitnessedAllyDeath {
                        ally: victim_id.clone(),
                        cause: Some(death_cause),
                    },
                    severity: Severity::Mild,
                    cause_hint: cause.clone(),
                });
            }
        }
    }

    // Phase 3: apply trauma and emit messages
    apply_trauma_events(game, events, true);
}
