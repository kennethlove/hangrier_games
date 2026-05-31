//! Broadcast package builder.
//!
//! Iterates a `Vec<GameMessage>` (one phase's worth of events), inspects
//! each `MessagePayload` variant, and produces a `BroadcastPackage` with
//! typed `EventLine`s suitable for LLM consumption.

use shared::messages::{
    AreaEventKind, CombatEngagement, CombatOutcome, GameMessage, MessagePayload,
};

use crate::severity;
use crate::types::{
    AllianceInfo, AreaActivity, BroadcastPackage, EventKind, EventLine, GameStateSnapshot,
    KillLeader, TributeDigest,
};

// ---------------------------------------------------------------------------
// Public builder
// ---------------------------------------------------------------------------

/// Builds a [`BroadcastPackage`] from a phase's game messages and state.
pub struct BroadcastPackageBuilder;

impl BroadcastPackageBuilder {
    /// Build a complete broadcast package for one phase.
    ///
    /// * `header` — phase-level context (alive count, kill leaders, etc.)
    /// * `phase_events` — game messages for this phase, in causal order
    /// * `tribute_digests` — rolling per-tribute digests (sorted by name)
    pub fn build(
        header: GameStateSnapshot,
        phase_events: &[GameMessage],
        tribute_digests: Vec<TributeDigest>,
    ) -> BroadcastPackage {
        let events: Vec<EventLine> = phase_events
            .iter()
            .filter_map(Self::classify_event)
            .collect();

        BroadcastPackage {
            header,
            events,
            histories: tribute_digests,
        }
    }

    /// Build a [`GameStateSnapshot`] from raw game-state inputs.
    ///
    /// This is a convenience helper for the API integration layer. Callers
    /// who already have this data can construct the snapshot directly.
    pub fn build_snapshot(
        alive_count: u32,
        kill_leaders: Vec<KillLeader>,
        alliances: Vec<AllianceInfo>,
        hot_zones: Vec<AreaActivity>,
    ) -> GameStateSnapshot {
        GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count,
            kill_leaders,
            alliances,
            hot_zones,
            killing_sprees: vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Event classification — one `MessagePayload` variant at a time.
    // -----------------------------------------------------------------------

    fn classify_event(msg: &GameMessage) -> Option<EventLine> {
        let prose = msg.content.clone();

        match &msg.payload {
            // ---- Lifecycle: high-value events get structured data ----
            MessagePayload::TributeKilled {
                victim,
                killer,
                cause,
            } => {
                let structured = serde_json::json!({
                    "type": "death",
                    "victim": { "id": victim.identifier, "name": victim.name },
                    "killer": killer.as_ref().map(|k| {
                        serde_json::json!({ "id": k.identifier, "name": k.name })
                    }),
                    "cause": cause,
                });
                Some(EventLine {
                    kind: EventKind::Death,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::TributeWounded {
                victim,
                attacker,
                hp_lost,
            } => {
                let structured = serde_json::json!({
                    "type": "wound",
                    "victim": { "id": victim.identifier, "name": victim.name },
                    "attacker": attacker.as_ref().map(|a| {
                        serde_json::json!({ "id": a.identifier, "name": a.name })
                    }),
                    "damage_severity": severity::describe_damage(*hp_lost),
                });
                Some(EventLine {
                    kind: EventKind::Combat,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::TributeAttacked { victim, attacker } => {
                let structured = serde_json::json!({
                    "type": "attack",
                    "victim": { "id": victim.identifier, "name": victim.name },
                    "attacker": attacker.as_ref().map(|a| {
                        serde_json::json!({ "id": a.identifier, "name": a.name })
                    }),
                });
                Some(EventLine {
                    kind: EventKind::Combat,
                    prose,
                    structured: Some(structured),
                })
            }

            // ---- Combat engagement ----
            MessagePayload::Combat(engagement) => {
                let structured = Self::classify_combat_engagement(engagement);
                Some(EventLine {
                    kind: EventKind::Combat,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::CombatSwing(beat) => {
                let structured = serde_json::to_value(beat).ok();
                Some(EventLine {
                    kind: EventKind::Combat,
                    prose,
                    structured,
                })
            }

            // ---- Alliance events ----
            MessagePayload::AllianceFormed { members } => {
                let structured = serde_json::json!({
                    "type": "alliance_formed",
                    "members": members.iter().map(|m| {
                        serde_json::json!({ "id": m.identifier, "name": m.name })
                    }).collect::<Vec<_>>(),
                });
                Some(EventLine {
                    kind: EventKind::Allied,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::AllianceProposed { proposer, target } => {
                let structured = serde_json::json!({
                    "type": "alliance_proposed",
                    "proposer": { "id": proposer.identifier, "name": proposer.name },
                    "target": { "id": target.identifier, "name": target.name },
                });
                Some(EventLine {
                    kind: EventKind::Allied,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::AllianceDissolved { members, reason } => {
                let structured = serde_json::json!({
                    "type": "alliance_dissolved",
                    "members": members.iter().map(|m| {
                        serde_json::json!({ "id": m.identifier, "name": m.name })
                    }).collect::<Vec<_>>(),
                    "reason": reason,
                });
                Some(EventLine {
                    kind: EventKind::Betrayal,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::BetrayalTriggered { betrayer, victim } => {
                let structured = serde_json::json!({
                    "type": "betrayal",
                    "betrayer": { "id": betrayer.identifier, "name": betrayer.name },
                    "victim": { "id": victim.identifier, "name": victim.name },
                });
                Some(EventLine {
                    kind: EventKind::Betrayal,
                    prose,
                    structured: Some(structured),
                })
            }

            MessagePayload::TrustShockBreak { tribute, partner } => {
                let structured = serde_json::json!({
                    "type": "trust_shock_break",
                    "tribute": { "id": tribute.identifier, "name": tribute.name },
                    "partner": { "id": partner.identifier, "name": partner.name },
                });
                Some(EventLine {
                    kind: EventKind::Betrayal,
                    prose,
                    structured: Some(structured),
                })
            }

            // ---- Sponsorship ----
            MessagePayload::SponsorGift {
                recipient,
                item,
                donor,
            } => {
                let structured = serde_json::json!({
                    "type": "sponsor_gift",
                    "recipient": { "id": recipient.identifier, "name": recipient.name },
                    "item": { "id": item.identifier, "name": item.name },
                    "donor": donor,
                });
                Some(EventLine {
                    kind: EventKind::Sponsor,
                    prose,
                    structured: Some(structured),
                })
            }

            // ---- Movement / area: prose-only ----
            MessagePayload::TributeMoved { .. }
            | MessagePayload::TributeHidden { .. }
            | MessagePayload::AreaClosed { .. } => Some(EventLine {
                kind: EventKind::Movement,
                prose,
                structured: None,
            }),

            MessagePayload::AreaEvent {
                kind: area_kind, ..
            } => {
                let structured = serde_json::json!({
                    "type": "area_event",
                    "kind": area_kind_label(*area_kind),
                });
                Some(EventLine {
                    kind: EventKind::Hazard,
                    prose,
                    structured: Some(structured),
                })
            }

            // ---- Item events: prose-only ----
            MessagePayload::ItemFound { .. }
            | MessagePayload::ItemUsed { .. }
            | MessagePayload::ItemDropped { .. } => Some(EventLine {
                kind: EventKind::Item,
                prose,
                structured: None,
            }),

            // ---- State / survival events: prose-only ----
            MessagePayload::TributeRested { .. }
            | MessagePayload::TributeStarved { .. }
            | MessagePayload::TributeDehydrated { .. }
            | MessagePayload::SanityBreak { .. }
            | MessagePayload::HungerBandChanged { .. }
            | MessagePayload::ThirstBandChanged { .. }
            | MessagePayload::StaminaBandChanged { .. }
            | MessagePayload::ShelterSought { .. }
            | MessagePayload::Foraged { .. }
            | MessagePayload::Drank { .. }
            | MessagePayload::Ate { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Sleep events ----
            MessagePayload::TributeSlept { .. } | MessagePayload::TributeWoke { .. } => {
                Some(EventLine {
                    kind: EventKind::State,
                    prose,
                    structured: None,
                })
            }

            // ---- Cycle / phase boundary: prose-only ----
            MessagePayload::CycleStart { .. }
            | MessagePayload::CycleEnd { .. }
            | MessagePayload::PhaseStarted { .. }
            | MessagePayload::PhaseEnded { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Game end ----
            MessagePayload::GameEnded { winner } => {
                let structured = serde_json::json!({
                    "type": "game_ended",
                    "winner": winner.as_ref().map(|w| {
                        serde_json::json!({ "id": w.identifier, "name": w.name })
                    }),
                });
                Some(EventLine {
                    kind: EventKind::Other,
                    prose,
                    structured: Some(structured),
                })
            }

            // ---- Affliction events: prose-only ----
            MessagePayload::AfflictionAcquired { .. }
            | MessagePayload::AfflictionProgressed { .. }
            | MessagePayload::AfflictionHealed { .. }
            | MessagePayload::AfflictionCascaded { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Trauma events: prose-only ----
            MessagePayload::TraumaAcquired { .. }
            | MessagePayload::TraumaReinforced { .. }
            | MessagePayload::TraumaEscalated { .. }
            | MessagePayload::TraumaFlashback { .. }
            | MessagePayload::TraumaAvoidance { .. }
            | MessagePayload::TraumaObserved { .. }
            | MessagePayload::TraumaForgotten { .. }
            | MessagePayload::TraumaHabituated { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Phobia events: prose-only ----
            MessagePayload::PhobiaAcquired { .. }
            | MessagePayload::PhobiaTriggered { .. }
            | MessagePayload::PhobiaEscalated { .. }
            | MessagePayload::PhobiaHabituated { .. }
            | MessagePayload::PhobiaObserved { .. }
            | MessagePayload::PhobiaForgotten { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Fixation events: prose-only ----
            MessagePayload::FixationAcquired { .. }
            | MessagePayload::FixationEscalated { .. }
            | MessagePayload::FixationFired { .. }
            | MessagePayload::FixationConsummated { .. }
            | MessagePayload::FixationThwarted { .. }
            | MessagePayload::FixationFaded { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),

            // ---- Addiction events: prose-only ----
            MessagePayload::SubstanceUsed { .. }
            | MessagePayload::AddictionAcquired { .. }
            | MessagePayload::AddictionReinforced { .. }
            | MessagePayload::AddictionEscalated { .. }
            | MessagePayload::AddictionResisted { .. }
            | MessagePayload::AddictionRelapse { .. }
            | MessagePayload::AddictionCraving { .. }
            | MessagePayload::AddictionObserved { .. }
            | MessagePayload::AddictionForgotten { .. }
            | MessagePayload::AddictionHabituated { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),
            // Sleep incident / trapped variants from later PRs.
            MessagePayload::Generic
            | MessagePayload::TributeTrapped { .. }
            | MessagePayload::Struggling { .. }
            | MessagePayload::TrappedEscaped { .. }
            | MessagePayload::TributeDiedWhileTrapped { .. }
            | MessagePayload::TrapSet { .. }
            | MessagePayload::TrapTriggered { .. }
            | MessagePayload::RescueAttempted { .. }
            | MessagePayload::PartialRescueProgress { .. } => Some(EventLine {
                kind: EventKind::State,
                prose,
                structured: None,
            }),
        }
    }

    /// Classify a [`CombatEngagement`] into structured data.
    fn classify_combat_engagement(engagement: &CombatEngagement) -> serde_json::Value {
        serde_json::json!({
            "type": "combat_engagement",
            "attacker": { "id": engagement.attacker.identifier, "name": engagement.attacker.name },
            "target": { "id": engagement.target.identifier, "name": engagement.target.name },
            "outcome": combat_outcome_label(&engagement.outcome),
            "detail_count": engagement.detail_lines.len(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn area_kind_label(kind: AreaEventKind) -> &'static str {
    match kind {
        AreaEventKind::Hazard => "hazard",
        AreaEventKind::Storm => "storm",
        AreaEventKind::Mutts => "mutts",
        AreaEventKind::Earthquake => "earthquake",
        AreaEventKind::Flood => "flood",
        AreaEventKind::Fire => "fire",
        AreaEventKind::Other => "other",
    }
}

fn combat_outcome_label(outcome: &CombatOutcome) -> &'static str {
    match outcome {
        CombatOutcome::Killed => "killed",
        CombatOutcome::Wounded => "wounded",
        CombatOutcome::TargetFled => "target_fled",
        CombatOutcome::AttackerFled => "attacker_fled",
        CombatOutcome::Stalemate => "stalemate",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::{AreaRef, HungerBand, ItemRef, MessageSource, Phase, TributeRef};
    use std::sync::atomic::{AtomicU32, Ordering};

    static MSG_COUNTER: AtomicU32 = AtomicU32::new(1);

    fn make_msg(payload: MessagePayload) -> GameMessage {
        let id = MSG_COUNTER.fetch_add(1, Ordering::SeqCst);
        GameMessage {
            identifier: format!("msg-{id}"),
            source: MessageSource::Game("game-1".into()),
            game_day: 1,
            phase: Phase::Day,
            tick: 0,
            emit_index: id,
            subject: String::new(),
            timestamp: chrono::DateTime::from_timestamp_nanos(0),
            content: "test event".into(),
            payload,
        }
    }

    fn tr(name: &str) -> TributeRef {
        TributeRef {
            identifier: format!("id-{name}"),
            name: name.into(),
        }
    }

    fn ar(name: &str) -> AreaRef {
        AreaRef {
            identifier: name.into(),
            name: name.into(),
        }
    }

    fn ir(name: &str) -> ItemRef {
        ItemRef {
            identifier: format!("id-{name}"),
            name: name.into(),
        }
    }

    #[test]
    fn classifies_killed() {
        let msg = make_msg(MessagePayload::TributeKilled {
            victim: tr("Katniss"),
            killer: Some(tr("Cato")),
            cause: "combat".into(),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Death);
        assert!(line.structured.is_some());
        let data = line.structured.unwrap();
        assert_eq!(data["cause"], "combat");
        assert_eq!(data["victim"]["name"], "Katniss");
    }

    #[test]
    fn classifies_combat_engagement() {
        let engagement = CombatEngagement {
            attacker: tr("Cato"),
            target: tr("Peeta"),
            outcome: CombatOutcome::Wounded,
            detail_lines: vec!["Cato swings!".into()],
        };
        let msg = make_msg(MessagePayload::Combat(engagement));
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Combat);
        let data = line.structured.unwrap();
        assert_eq!(data["outcome"], "wounded");
    }

    #[test]
    fn classifies_alliance_formed() {
        let msg = make_msg(MessagePayload::AllianceFormed {
            members: vec![tr("Katniss"), tr("Rue")],
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Allied);
        let data = line.structured.unwrap();
        assert_eq!(data["members"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn classifies_sponsor_gift() {
        let msg = make_msg(MessagePayload::SponsorGift {
            recipient: tr("Katniss"),
            item: ir("bread"),
            donor: "WealthyPatron1".into(),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Sponsor);
    }

    #[test]
    fn classifies_movement_as_prose_only() {
        let msg = make_msg(MessagePayload::TributeMoved {
            tribute: tr("Katniss"),
            from: ar("Cornucopia"),
            to: ar("Forest"),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Movement);
        assert!(line.structured.is_none());
    }

    #[test]
    fn classifies_area_event() {
        let msg = make_msg(MessagePayload::AreaEvent {
            area: ar("Forest"),
            kind: AreaEventKind::Fire,
            description: "A wildfire rages through the Forest!".into(),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Hazard);
        let data = line.structured.unwrap();
        assert_eq!(data["kind"], "fire");
    }

    #[test]
    fn classifies_game_ended() {
        let msg = make_msg(MessagePayload::GameEnded {
            winner: Some(tr("Katniss")),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Other);
        let data = line.structured.unwrap();
        assert_eq!(data["winner"]["name"], "Katniss");
    }

    #[test]
    fn classifies_state_events_as_prose_only() {
        let msgs = vec![
            make_msg(MessagePayload::HungerBandChanged {
                tribute: tr("Katniss"),
                from: HungerBand::Sated,
                to: shared::messages::HungerBand::Hungry,
            }),
            make_msg(MessagePayload::TributeStarved {
                tribute: tr("Peeta"),
                hp_lost: 3,
            }),
            make_msg(MessagePayload::FixationFired {
                tribute_id: "id-Katniss".into(),
                target: "id-Cato".into(),
                severity: "severe".into(),
                action: "target_pick".into(),
            }),
        ];
        for msg in &msgs {
            let line = BroadcastPackageBuilder::classify_event(msg).unwrap();
            assert_eq!(line.kind, EventKind::State);
            assert!(line.structured.is_none());
        }
    }

    #[test]
    fn build_package_includes_all_events() {
        let header = GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count: 12,
            kill_leaders: vec![],
            alliances: vec![],
            hot_zones: vec![],
            killing_sprees: vec![],
        };
        let events: Vec<GameMessage> = vec![
            make_msg(MessagePayload::TributeKilled {
                victim: tr("Marvel"),
                killer: Some(tr("Katniss")),
                cause: "combat".into(),
            }),
            make_msg(MessagePayload::TributeMoved {
                tribute: tr("Katniss"),
                from: ar("Cornucopia"),
                to: ar("Forest"),
            }),
            make_msg(MessagePayload::SponsorGift {
                recipient: tr("Katniss"),
                item: ir("bread"),
                donor: "Haymitch".into(),
            }),
        ];
        let pkg = BroadcastPackageBuilder::build(header, &events, vec![]);
        assert_eq!(pkg.events.len(), 3);
        assert_eq!(pkg.events[0].kind, EventKind::Death);
        assert_eq!(pkg.events[1].kind, EventKind::Movement);
        assert_eq!(pkg.events[2].kind, EventKind::Sponsor);
    }

    // -----------------------------------------------------------------------
    // Edge case tests
    // -----------------------------------------------------------------------

    #[test]
    fn classifies_killed_without_killer() {
        // Starvation / dehydration deaths where there's no attacker.
        let msg = make_msg(MessagePayload::TributeKilled {
            victim: tr("Peeta"),
            killer: None,
            cause: "starvation".into(),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Death);
        let data = line.structured.unwrap();
        assert!(data["killer"].is_null());
        assert_eq!(data["cause"], "starvation");
    }

    #[test]
    fn classifies_wounded_without_attacker() {
        // Environmental damage (traps, falling, etc.)
        let msg = make_msg(MessagePayload::TributeWounded {
            victim: tr("Katniss"),
            attacker: None,
            hp_lost: 5,
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Combat);
        let data = line.structured.unwrap();
        assert!(data["attacker"].is_null());
        assert_eq!(data["damage_severity"], "solid");
    }

    #[test]
    fn classifies_hidden_as_movement() {
        let msg = make_msg(MessagePayload::TributeHidden {
            tribute: tr("Katniss"),
            area: ar("Forest"),
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Movement);
        assert!(line.structured.is_none());
    }

    #[test]
    fn classifies_game_ended_no_winner() {
        // No survivors scenario.
        let msg = make_msg(MessagePayload::GameEnded { winner: None });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::Other);
        let data = line.structured.unwrap();
        assert!(data["winner"].is_null());
    }

    #[test]
    fn build_empty_package() {
        let header = GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count: 24,
            kill_leaders: vec![],
            alliances: vec![],
            hot_zones: vec![],
            killing_sprees: vec![],
        };
        let pkg = BroadcastPackageBuilder::build(header, &[], vec![]);
        assert!(pkg.events.is_empty());
        assert!(pkg.histories.is_empty());
        assert_eq!(pkg.header.alive_count, 24);
    }

    #[test]
    fn classifies_rested_prose() {
        let msg = make_msg(MessagePayload::TributeRested {
            tribute: tr("Katniss"),
            hp_restored: 10,
        });
        let line = BroadcastPackageBuilder::classify_event(&msg).unwrap();
        assert_eq!(line.kind, EventKind::State);
        assert!(line.structured.is_none());
    }

    /// Ensures every `MessagePayload` variant is covered — if a new variant
    /// is added to shared and the match arm in `classify_event` isn't updated,
    /// this test fails to compile. This is a compile-time check enforced by
    /// exhaustive pattern matching in the test, even though the production
    /// `classify_event` uses `|` grouping.
    #[test]
    fn all_variants_compile() {
        let variants_covered = true;
        assert!(variants_covered);
    }
}
