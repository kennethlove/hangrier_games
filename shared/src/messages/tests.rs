use super::*;
fn t(name: &str) -> TributeRef {
    TributeRef {
        identifier: format!("id-{name}"),
        name: name.into(),
    }
}

#[test]
fn phase_display_roundtrip() {
    for p in Phase::all() {
        let s = p.to_string();
        assert_eq!(s.parse::<Phase>().unwrap(), p);
    }
    assert_eq!(Phase::Dawn.to_string(), "dawn");
    assert_eq!(Phase::Day.to_string(), "day");
    assert_eq!(Phase::Dusk.to_string(), "dusk");
    assert_eq!(Phase::Night.to_string(), "night");
    assert!("noon".parse::<Phase>().is_err());
}

#[test]
fn phase_serde_lowercase() {
    assert_eq!(serde_json::to_string(&Phase::Dawn).unwrap(), "\"dawn\"");
    assert_eq!(serde_json::to_string(&Phase::Day).unwrap(), "\"day\"");
    assert_eq!(serde_json::to_string(&Phase::Dusk).unwrap(), "\"dusk\"");
    assert_eq!(serde_json::to_string(&Phase::Night).unwrap(), "\"night\"");
    let p: Phase = serde_json::from_str("\"night\"").unwrap();
    assert_eq!(p, Phase::Night);
}

#[test]
fn phase_ord_and_next_canonical_cycle() {
    assert_eq!(Phase::Dawn.ord(), 0);
    assert_eq!(Phase::Day.ord(), 1);
    assert_eq!(Phase::Dusk.ord(), 2);
    assert_eq!(Phase::Night.ord(), 3);
    // Canonical cycle wraps Night -> Dawn so the engine can advance the
    // game-day at the boundary without special-casing the wire format.
    assert_eq!(Phase::Dawn.next(), Phase::Day);
    assert_eq!(Phase::Day.next(), Phase::Dusk);
    assert_eq!(Phase::Dusk.next(), Phase::Night);
    assert_eq!(Phase::Night.next(), Phase::Dawn);
    assert_eq!(
        Phase::all(),
        [Phase::Dawn, Phase::Day, Phase::Dusk, Phase::Night]
    );
}

#[test]
fn message_kind_serde_roundtrip() {
    for kind in [
        MessageKind::Death,
        MessageKind::Combat,
        MessageKind::Alliance,
        MessageKind::Movement,
        MessageKind::Item,
        MessageKind::State,
        MessageKind::CombatSwing,
    ] {
        let s = serde_json::to_string(&kind).unwrap();
        let back: MessageKind = serde_json::from_str(&s).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn kind_lifecycle_variants_map_correctly() {
    let p = MessagePayload::TributeKilled {
        victim: t("v"),
        killer: None,
        cause: "fall".into(),
    };
    assert_eq!(p.kind(), MessageKind::Death);

    let p = MessagePayload::TributeWounded {
        victim: t("v"),
        attacker: None,
        hp_lost: 5,
    };
    assert_eq!(p.kind(), MessageKind::State);
}

#[test]
fn kind_combat_maps_to_combat() {
    let p = MessagePayload::Combat(CombatEngagement {
        attacker: t("a"),
        target: t("b"),
        outcome: CombatOutcome::Killed,
        detail_lines: vec![],
    });
    assert_eq!(p.kind(), MessageKind::Combat);
}

#[test]
fn kind_alliance_variants_map_correctly() {
    for p in [
        MessagePayload::AllianceFormed {
            members: vec![t("a"), t("b")],
        },
        MessagePayload::AllianceProposed {
            proposer: t("a"),
            target: t("b"),
        },
        MessagePayload::AllianceDissolved {
            members: vec![t("a")],
            reason: "x".into(),
        },
        MessagePayload::BetrayalTriggered {
            betrayer: t("a"),
            victim: t("b"),
        },
        MessagePayload::TrustShockBreak {
            tribute: t("a"),
            partner: t("b"),
        },
    ] {
        assert_eq!(p.kind(), MessageKind::Alliance);
    }
}

#[test]
fn kind_movement_variants_map_correctly() {
    let area = AreaRef {
        identifier: "a1".into(),
        name: "A".into(),
    };
    for p in [
        MessagePayload::TributeMoved {
            tribute: t("a"),
            from: area.clone(),
            to: area.clone(),
        },
        MessagePayload::TributeHidden {
            tribute: t("a"),
            area: area.clone(),
        },
        MessagePayload::AreaClosed { area: area.clone() },
        MessagePayload::AreaEvent {
            area: area.clone(),
            kind: AreaEventKind::Storm,
            description: "x".into(),
        },
    ] {
        assert_eq!(p.kind(), MessageKind::Movement);
    }
}

#[test]
fn kind_item_variants_map_correctly() {
    let area = AreaRef {
        identifier: "a1".into(),
        name: "A".into(),
    };
    let item = ItemRef {
        identifier: "i1".into(),
        name: "I".into(),
    };
    for p in [
        MessagePayload::ItemFound {
            tribute: t("a"),
            item: item.clone(),
            area: area.clone(),
        },
        MessagePayload::ItemUsed {
            tribute: t("a"),
            item: item.clone(),
        },
        MessagePayload::ItemDropped {
            tribute: t("a"),
            item: item.clone(),
            area: area.clone(),
        },
    ] {
        assert_eq!(p.kind(), MessageKind::Item);
    }
    let sponsor = MessagePayload::SponsorGift {
        recipient: t("a"),
        item: item.clone(),
        donor: "Capitol".into(),
    };
    assert_eq!(sponsor.kind(), MessageKind::SponsorGift);
}

#[test]
fn kind_state_variants_map_correctly() {
    for p in [
        MessagePayload::TributeRested {
            tribute: t("a"),
            hp_restored: 3,
        },
        MessagePayload::TributeStarved {
            tribute: t("a"),
            hp_lost: 1,
        },
        MessagePayload::TributeDehydrated {
            tribute: t("a"),
            hp_lost: 2,
        },
        MessagePayload::SanityBreak { tribute: t("a") },
    ] {
        assert_eq!(p.kind(), MessageKind::State);
    }
}

#[test]
fn unknown_payload_tag_hard_errors() {
    let raw = serde_json::json!({ "type": "DefinitelyNotAVariant" });
    let result: Result<MessagePayload, _> = serde_json::from_value(raw);
    assert!(result.is_err());
}

#[test]
fn game_message_new_populates_required_fields() {
    let msg = GameMessage::new(
        MessageSource::Game("g".into()),
        2,
        Phase::Night,
        3,
        0,
        "subj".into(),
        "content".into(),
        MessagePayload::SanityBreak { tribute: t("a") },
    );
    assert_eq!(msg.game_day, 2);
    assert_eq!(msg.phase, Phase::Night);
    assert_eq!(msg.tick, 3);
    assert_eq!(msg.emit_index, 0);
    assert_eq!(msg.payload.kind(), MessageKind::State);
}

fn make_msg(day: u32, phase: Phase, payload: MessagePayload) -> GameMessage {
    GameMessage::new(
        MessageSource::Game("g".into()),
        day,
        phase,
        1,
        0,
        "subject".into(),
        "content".into(),
        payload,
    )
}

#[test]
fn summarize_empty_input_with_current_day_zero() {
    let result = summarize_periods(&[], (0, Phase::Day));
    assert_eq!(
        result.len(),
        1,
        "current period (day 0, Day) should always be seeded"
    );
    assert_eq!(result[0].day, 0);
    assert_eq!(result[0].phase, Phase::Day);
    assert!(result[0].is_current);
    assert_eq!(result[0].event_count, 0);
    assert_eq!(result[0].deaths, 0);
}

#[test]
fn summarize_groups_by_day_and_phase() {
    let tref = TributeRef {
        identifier: "t".into(),
        name: "T".into(),
    };
    let killed = MessagePayload::TributeKilled {
        victim: tref.clone(),
        killer: None,
        cause: "x".into(),
    };
    let moved = MessagePayload::TributeHidden {
        tribute: tref.clone(),
        area: AreaRef {
            identifier: "a".into(),
            name: "A".into(),
        },
    };

    let msgs = vec![
        make_msg(1, Phase::Day, killed.clone()),
        make_msg(1, Phase::Day, moved.clone()),
        make_msg(1, Phase::Night, moved.clone()),
        make_msg(2, Phase::Day, killed.clone()),
    ];
    let result = summarize_periods(&msgs, (2, Phase::Day));
    // Day 1: Day/Dusk/Night (Dawn1 skipped per spec §3) + Day 2: Dawn/Day.
    assert_eq!(result.len(), 5);
    assert_eq!(
        result[0],
        PeriodSummary {
            day: 1,
            phase: Phase::Day,
            deaths: 1,
            event_count: 2,
            is_current: false
        }
    );
    assert_eq!(
        result[1],
        PeriodSummary {
            day: 1,
            phase: Phase::Dusk,
            deaths: 0,
            event_count: 0,
            is_current: false
        }
    );
    assert_eq!(
        result[2],
        PeriodSummary {
            day: 1,
            phase: Phase::Night,
            deaths: 0,
            event_count: 1,
            is_current: false
        }
    );
    assert_eq!(
        result[3],
        PeriodSummary {
            day: 2,
            phase: Phase::Dawn,
            deaths: 0,
            event_count: 0,
            is_current: false
        }
    );
    assert_eq!(
        result[4],
        PeriodSummary {
            day: 2,
            phase: Phase::Day,
            deaths: 1,
            event_count: 1,
            is_current: true
        }
    );
}

#[test]
fn summarize_includes_empty_reached_periods() {
    let result = summarize_periods(&[], (2, Phase::Day));
    // Day 1: Day/Dusk/Night (Dawn1 skipped) + Day 2: Dawn/Day.
    assert_eq!(result.len(), 5);
    assert_eq!(result[0].day, 1);
    assert_eq!(result[0].phase, Phase::Day);
    assert_eq!(result[1].phase, Phase::Dusk);
    assert_eq!(result[2].phase, Phase::Night);
    assert_eq!(result[3].day, 2);
    assert_eq!(result[3].phase, Phase::Dawn);
    assert_eq!(result[4].phase, Phase::Day);
    assert!(result[4].is_current);
}

#[test]
fn summarize_counts_combat_kills_as_deaths() {
    let combat_kill = MessagePayload::Combat(CombatEngagement {
        attacker: t("a"),
        target: t("b"),
        outcome: CombatOutcome::Killed,
        detail_lines: vec![],
    });
    let combat_wound = MessagePayload::Combat(CombatEngagement {
        attacker: t("a"),
        target: t("b"),
        outcome: CombatOutcome::Wounded,
        detail_lines: vec![],
    });
    let msgs = vec![
        make_msg(1, Phase::Day, combat_kill.clone()),
        make_msg(1, Phase::Day, combat_wound),
        make_msg(1, Phase::Day, combat_kill),
    ];
    let result = summarize_periods(&msgs, (1, Phase::Day));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].deaths, 2);
    assert_eq!(result[0].event_count, 3);
}

#[test]
fn summarize_is_current_flag_set_correctly() {
    let tref = TributeRef {
        identifier: "t".into(),
        name: "T".into(),
    };
    let p = MessagePayload::TributeRested {
        tribute: tref,
        hp_restored: 1,
    };
    let msgs = vec![make_msg(2, Phase::Night, p.clone())];
    let result = summarize_periods(&msgs, (2, Phase::Night));
    let current: Vec<_> = result.iter().filter(|s| s.is_current).collect();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].day, 2);
    assert_eq!(current[0].phase, Phase::Night);
}
