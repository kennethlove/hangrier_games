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

use crate::areas::Area;
use crate::games::Game;
use crate::messages::{MessagePayload, MessageSource, Phase, TributeRef};
use shared::afflictions::{CauseClass, DeathCause, Severity, TraumaSource};

/// Maximum health value used for near-death percentage calculations.
const MAX_HEALTH: u32 = 100;

/// Collected trauma event before application (avoids borrow conflicts).
struct TraumaEvent {
    tribute_id: String,
    tribute_name: String,
    source: TraumaSource,
    severity: Severity,
    /// Raw cause string for phobia co-acquire stub.
    cause_hint: String,
}

/// Message data collected during trauma application, pushed afterwards.
struct TraumaMessage {
    tribute_id: String,
    tribute_name: String,
    acquisition: crate::tributes::afflictions::TraumaAcquisition,
}

/// Run all trauma producers against the current phase's messages.
///
/// Gate: returns immediately if `game.config.trauma_enabled` is false.
pub fn run_trauma_producers(game: &mut Game) {
    if !game.config.trauma_enabled {
        return;
    }

    let phase = game.current_phase;

    produce_witness_ally_death(game, phase);
    produce_survive_near_death(game, phase);
    produce_survive_betrayal(game, phase);
    produce_witness_mass_casualty(game, phase);
}

// ── Producer (a)/(b): Witness ally death ────────────────────────────────────

fn produce_witness_ally_death(game: &mut Game, phase: Phase) {
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

// ── Producer (c): Survive near-death ────────────────────────────────────────

fn produce_survive_near_death(game: &mut Game, phase: Phase) {
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
            .find(|t| t.identifier == victim.identifier)
        else {
            continue;
        };

        let hp_after = tribute.attributes.health;
        let hp_percent = hp_after * 100 / MAX_HEALTH;

        if hp_percent <= 10 && tribute.is_alive() {
            let death_cause = attacker
                .as_ref()
                .map(|a| DeathCause::Tribute(a.identifier.clone()))
                .unwrap_or(DeathCause::Unknown);

            events.push(TraumaEvent {
                tribute_id: tribute.identifier.clone(),
                tribute_name: tribute.name.clone(),
                source: TraumaSource::NearDeath { cause: death_cause },
                severity: Severity::Moderate,
                cause_hint: String::new(),
            });
        }
    }

    apply_trauma_events(game, events, false);
}

// ── Producer (d): Survive betrayal ──────────────────────────────────────────

fn produce_survive_betrayal(game: &mut Game, phase: Phase) {
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

// ── Producer (f): Witness mass casualty ─────────────────────────────────────

fn produce_witness_mass_casualty(game: &mut Game, phase: Phase) {
    // Phase 1: count deaths per area
    let mut deaths_by_area: std::collections::HashMap<Area, u32> = std::collections::HashMap::new();

    for msg in &game.messages {
        if msg.phase != phase {
            continue;
        }
        let MessagePayload::TributeKilled { victim, .. } = &msg.payload else {
            continue;
        };

        let Some(vt) = game
            .tributes
            .iter()
            .find(|t| t.identifier == victim.identifier)
        else {
            continue;
        };

        *deaths_by_area.entry(vt.area).or_insert(0) += 1;
    }

    // Phase 2: find living tributes in areas with >= 3 deaths
    let mut events: Vec<TraumaEvent> = Vec::new();

    for (area, death_count) in &deaths_by_area {
        if *death_count < 3 {
            continue;
        }

        let severity = if *death_count >= 5 {
            Severity::Severe
        } else {
            Severity::Moderate
        };

        for t in &game.tributes {
            if t.is_alive() && t.area == *area {
                events.push(TraumaEvent {
                    tribute_id: t.identifier.clone(),
                    tribute_name: t.name.clone(),
                    source: TraumaSource::MassCasualty {
                        cause_class: CauseClass::Mixed,
                        deaths_this_cycle: *death_count,
                    },
                    severity,
                    cause_hint: String::new(),
                });
            }
        }
    }

    apply_trauma_events(game, events, false);
}

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Apply collected trauma events, emitting appropriate messages.
/// When `with_phobia_stub` is true, calls the phobia co-acquire stub after
/// each acquisition.
fn apply_trauma_events(game: &mut Game, events: Vec<TraumaEvent>, with_phobia_stub: bool) {
    let mut messages: Vec<TraumaMessage> = Vec::new();

    for event in events {
        let Some(tribute) = game
            .tributes
            .iter_mut()
            .find(|t| t.identifier == event.tribute_id)
        else {
            continue;
        };

        let acquisition = tribute.try_acquire_trauma(event.source, event.severity);
        messages.push(TraumaMessage {
            tribute_id: event.tribute_id.clone(),
            tribute_name: event.tribute_name.clone(),
            acquisition,
        });

        if with_phobia_stub {
            try_co_acquire_phobia(tribute, &event.cause_hint);
        }
    }

    // Push messages after mutable borrows end
    for msg in messages {
        push_trauma_message(game, &msg.tribute_name, &msg.tribute_id, &msg.acquisition);
    }
}

/// Push a `TraumaAcquired` or `TraumaReinforced` message onto the game log.
fn push_trauma_message(
    game: &mut Game,
    tribute_name: &str,
    tribute_id: &str,
    acquisition: &crate::tributes::afflictions::TraumaAcquisition,
) {
    use crate::tributes::afflictions::TraumaAcquisition;

    let payload = match acquisition {
        TraumaAcquisition::Acquired { severity, source } => MessagePayload::TraumaAcquired {
            tribute: tribute_id.to_string(),
            severity: severity.to_string(),
            source: format!("{source:?}"),
        },
        TraumaAcquisition::Reinforced {
            from_severity,
            to_severity,
            floor_bumped,
        } => MessagePayload::TraumaReinforced {
            tribute: tribute_id.to_string(),
            from_severity: from_severity.to_string(),
            to_severity: to_severity.to_string(),
            floor_bumped: *floor_bumped,
        },
    };

    let tick = game.tick_counter.next();
    game.push_message(
        MessageSource::Tribute(tribute_id.to_string()),
        format!("tribute:{tribute_id}"),
        format_trauma_content(tribute_name, acquisition),
        payload,
        tick,
    );
}

/// Format human-readable content for a trauma message.
fn format_trauma_content(
    name: &str,
    acquisition: &crate::tributes::afflictions::TraumaAcquisition,
) -> String {
    use crate::tributes::afflictions::TraumaAcquisition;

    match acquisition {
        TraumaAcquisition::Acquired { severity, .. } => {
            format!("{name} acquires trauma ({severity}).")
        }
        TraumaAcquisition::Reinforced {
            from_severity,
            to_severity,
            floor_bumped,
        } => {
            if *floor_bumped {
                format!("{name}'s trauma reinforced: {from_severity} → {to_severity}.")
            } else {
                format!("{name}'s trauma reinforced ({to_severity}).")
            }
        }
    }
}

/// Map a killer reference and cause string to a `DeathCause`.
fn map_cause_to_death_cause(killer: Option<&TributeRef>, cause: &str) -> DeathCause {
    if let Some(k) = killer {
        return DeathCause::Tribute(k.identifier.clone());
    }
    match cause.to_lowercase().as_str() {
        "fire" | "wildfire" => DeathCause::Fire,
        "drowning" | "flood" => DeathCause::Drowning,
        "starvation" => DeathCause::Starvation,
        "dehydration" => DeathCause::Dehydration,
        _ => DeathCause::Unknown,
    }
}

/// Stub for phobia co-acquisition. No-op until phobia PR lands.
///
/// TODO(phobia-pr1): wire to `try_acquire_phobia` when the phobia affliction
/// system is implemented.
#[allow(dead_code)]
fn try_co_acquire_phobia(_tribute: &mut crate::tributes::Tribute, _cause: &str) {
    // No-op stub. Will trigger phobia acquisition once the phobia system exists.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::messages::GameMessage;
    use crate::tributes::Tribute;
    use shared::messages::Phase;
    use uuid::Uuid;

    fn make_tribute(name: &str) -> Tribute {
        let mut t = Tribute::new(name.to_string(), None, None);
        t.attributes.health = 100;
        t
    }

    fn make_killed_msg(victim_id: &str, victim_name: &str, phase: Phase) -> GameMessage {
        GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            phase,
            1,
            0,
            "subject".into(),
            "content".into(),
            MessagePayload::TributeKilled {
                victim: TributeRef {
                    identifier: victim_id.into(),
                    name: victim_name.into(),
                },
                killer: None,
                cause: "combat".into(),
            },
        )
    }

    #[test]
    fn trauma_disabled_skips_all_producers() {
        let mut game = Game::default();
        game.config.trauma_enabled = false;
        game.current_phase = Phase::Day;

        let mut t = make_tribute("Test");
        let ally_id = Uuid::new_v4();
        t.allies.push(ally_id);
        game.tributes.push(t);
        game.messages.push(make_killed_msg("x", "X", Phase::Day));

        run_trauma_producers(&mut game);
        assert!(game.messages.iter().all(|m| !matches!(
            &m.payload,
            MessagePayload::TraumaAcquired { .. } | MessagePayload::TraumaReinforced { .. }
        )));
    }

    #[test]
    fn witness_ally_death_acquires_mild_trauma() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let mut victim = make_tribute("Victim");
        let victim_id = victim.id;
        victim.area = Area::Sector1;
        victim.attributes.health = 0;
        game.tributes.push(victim);

        let mut witness = make_tribute("Witness");
        witness.area = Area::Sector1;
        witness.allies.push(victim_id);
        game.tributes.push(witness);

        game.messages.push(make_killed_msg(
            &game.tributes[0].identifier,
            "Victim",
            Phase::Day,
        ));

        run_trauma_producers(&mut game);

        let trauma_msg = game
            .messages
            .iter()
            .find(|m| matches!(&m.payload, MessagePayload::TraumaAcquired { .. }));
        assert!(trauma_msg.is_some(), "should emit TraumaAcquired message");

        let witness = game.tributes.iter().find(|t| t.name == "Witness").unwrap();
        assert!(
            witness
                .afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "witness should have trauma affliction"
        );
    }

    #[test]
    fn witness_different_area_no_trauma() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let mut victim = make_tribute("Victim");
        let victim_id = victim.id;
        victim.area = Area::Sector1;
        game.tributes.push(victim);

        let mut witness = make_tribute("Witness");
        witness.area = Area::Sector2; // different area
        witness.allies.push(victim_id);
        game.tributes.push(witness);

        game.messages.push(make_killed_msg(
            &game.tributes[0].identifier,
            "Victim",
            Phase::Day,
        ));

        run_trauma_producers(&mut game);

        let witness = game.tributes.iter().find(|t| t.name == "Witness").unwrap();
        assert!(
            !witness
                .afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "witness in different area should NOT have trauma"
        );
    }

    #[test]
    fn survive_near_death_acquires_moderate_trauma() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let mut t = make_tribute("Survivor");
        t.attributes.health = 8; // 8% HP, below 10% threshold
        game.tributes.push(t);

        game.messages.push(GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            Phase::Day,
            1,
            0,
            "subject".into(),
            "content".into(),
            MessagePayload::TributeWounded {
                victim: TributeRef {
                    identifier: game.tributes[0].identifier.clone(),
                    name: "Survivor".into(),
                },
                attacker: None,
                hp_lost: 92,
            },
        ));

        run_trauma_producers(&mut game);

        let trauma_msg = game
            .messages
            .iter()
            .find(|m| matches!(&m.payload, MessagePayload::TraumaAcquired { .. }));
        assert!(
            trauma_msg.is_some(),
            "should emit TraumaAcquired for near-death"
        );

        let t = game.tributes.iter().find(|t| t.name == "Survivor").unwrap();
        assert!(
            t.afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "survivor should have trauma affliction"
        );
    }

    #[test]
    fn survive_near_death_above_threshold_no_trauma() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let mut t = make_tribute("Lucky");
        t.attributes.health = 15; // 15% HP, above 10% threshold
        game.tributes.push(t);

        game.messages.push(GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            Phase::Day,
            1,
            0,
            "subject".into(),
            "content".into(),
            MessagePayload::TributeWounded {
                victim: TributeRef {
                    identifier: game.tributes[0].identifier.clone(),
                    name: "Lucky".into(),
                },
                attacker: None,
                hp_lost: 85,
            },
        ));

        run_trauma_producers(&mut game);

        let t = game.tributes.iter().find(|t| t.name == "Lucky").unwrap();
        assert!(
            !t.afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "tribute above 10% threshold should NOT have trauma"
        );
    }

    #[test]
    fn survive_betrayal_acquires_moderate_trauma() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let victim = make_tribute("Betrayed");
        game.tributes.push(victim);

        let betrayer = make_tribute("Traitor");
        game.tributes.push(betrayer);

        game.messages.push(GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            Phase::Day,
            1,
            0,
            "subject".into(),
            "content".into(),
            MessagePayload::BetrayalTriggered {
                betrayer: TributeRef {
                    identifier: game.tributes[1].identifier.clone(),
                    name: "Traitor".into(),
                },
                victim: TributeRef {
                    identifier: game.tributes[0].identifier.clone(),
                    name: "Betrayed".into(),
                },
            },
        ));

        run_trauma_producers(&mut game);

        let trauma_msg = game
            .messages
            .iter()
            .find(|m| matches!(&m.payload, MessagePayload::TraumaAcquired { .. }));
        assert!(
            trauma_msg.is_some(),
            "should emit TraumaAcquired for betrayal"
        );

        let victim = game.tributes.iter().find(|t| t.name == "Betrayed").unwrap();
        assert!(
            victim
                .afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "betrayal victim should have trauma"
        );
    }

    #[test]
    fn mass_casualty_three_deaths_moderate() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        // Three victims in Sector1
        for name in ["V1", "V2", "V3"] {
            let mut v = make_tribute(name);
            v.area = Area::Sector1;
            v.attributes.health = 0;
            game.tributes.push(v);
        }

        // Witness in same area
        let mut w = make_tribute("Witness");
        w.area = Area::Sector1;
        game.tributes.push(w);

        for v in &game.tributes[..3] {
            game.messages
                .push(make_killed_msg(&v.identifier, &v.name, Phase::Day));
        }

        run_trauma_producers(&mut game);

        let trauma_msgs: Vec<_> = game
            .messages
            .iter()
            .filter(|m| matches!(&m.payload, MessagePayload::TraumaAcquired { .. }))
            .collect();
        assert_eq!(
            trauma_msgs.len(),
            1,
            "witness should get one trauma from mass casualty"
        );
    }

    #[test]
    fn mass_casualty_five_deaths_severe() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        for name in ["V1", "V2", "V3", "V4", "V5"] {
            let mut v = make_tribute(name);
            v.area = Area::Sector1;
            v.attributes.health = 0;
            game.tributes.push(v);
        }

        let mut w = make_tribute("Witness");
        w.area = Area::Sector1;
        game.tributes.push(w);

        for v in &game.tributes[..5] {
            game.messages
                .push(make_killed_msg(&v.identifier, &v.name, Phase::Day));
        }

        run_trauma_producers(&mut game);

        let trauma_msgs: Vec<_> = game
            .messages
            .iter()
            .filter(|m| matches!(&m.payload, MessagePayload::TraumaAcquired { .. }))
            .collect();
        assert_eq!(trauma_msgs.len(), 1);

        // Check severity is Severe
        if let MessagePayload::TraumaAcquired { severity, .. } = &trauma_msgs[0].payload {
            assert_eq!(severity, "severe");
        } else {
            panic!("expected TraumaAcquired payload");
        }
    }

    #[test]
    fn different_phase_messages_ignored() {
        let mut game = Game::default();
        game.config.trauma_enabled = true;
        game.current_phase = Phase::Day;

        let mut victim = make_tribute("Victim");
        let victim_id = victim.id;
        victim.area = Area::Sector1;
        game.tributes.push(victim);

        let mut witness = make_tribute("Witness");
        witness.area = Area::Sector1;
        witness.allies.push(victim_id);
        game.tributes.push(witness);

        // Message from different phase
        game.messages.push(make_killed_msg(
            &game.tributes[0].identifier,
            "Victim",
            Phase::Night,
        ));

        run_trauma_producers(&mut game);

        let witness = game.tributes.iter().find(|t| t.name == "Witness").unwrap();
        assert!(
            !witness
                .afflictions
                .contains_key(&(shared::afflictions::AfflictionKind::Trauma, None)),
            "messages from different phase should be ignored"
        );
    }

    #[test]
    fn map_cause_to_death_cause_killer_takes_priority() {
        let killer = TributeRef {
            identifier: "tributes:killer".into(),
            name: "Killer".into(),
        };
        let cause = map_cause_to_death_cause(Some(&killer), "fire");
        assert!(matches!(cause, DeathCause::Tribute(id) if id == "tributes:killer"));
    }

    #[test]
    fn map_cause_to_death_cause_string_fallback() {
        assert!(matches!(
            map_cause_to_death_cause(None, "fire"),
            DeathCause::Fire
        ));
        assert!(matches!(
            map_cause_to_death_cause(None, "drowning"),
            DeathCause::Drowning
        ));
        assert!(matches!(
            map_cause_to_death_cause(None, "starvation"),
            DeathCause::Starvation
        ));
        assert!(matches!(
            map_cause_to_death_cause(None, "unknown_cause"),
            DeathCause::Unknown
        ));
    }
}
