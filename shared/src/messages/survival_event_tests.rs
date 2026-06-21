use super::*;
use uuid::Uuid;

const TEST_TRIBUTE_ID: Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
const TEST_AREA_ID: Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9");
const TEST_ITEM_ID: Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0ca");

fn tref() -> TributeRef {
    TributeRef {
        identifier: TributeId(TEST_TRIBUTE_ID),
        name: "Cato".into(),
    }
}
fn aref() -> AreaRef {
    AreaRef {
        identifier: AreaId(TEST_AREA_ID),
        name: "Forest".into(),
    }
}
fn iref() -> ItemRef {
    ItemRef {
        identifier: ItemId(TEST_ITEM_ID),
        name: "Berries".into(),
    }
}

#[test]
fn shelter_sought_round_trip() {
    let p = MessagePayload::ShelterSought {
        tribute: tref(),
        area: aref(),
        success: true,
        roll: 2,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: MessagePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
    assert_eq!(p.kind(), MessageKind::State);
}

#[test]
fn band_change_payloads_round_trip() {
    let p = MessagePayload::HungerBandChanged {
        tribute: tref(),
        from: HungerBand::Sated,
        to: HungerBand::Hungry,
    };
    let back: MessagePayload = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
    let p = MessagePayload::ThirstBandChanged {
        tribute: tref(),
        from: ThirstBand::Sated,
        to: ThirstBand::Parched,
    };
    let back: MessagePayload = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
}

#[test]
fn stamina_band_change_round_trips_and_routes_to_state() {
    let p = MessagePayload::StaminaBandChanged {
        tribute: tref(),
        from: StaminaBand::Fresh,
        to: StaminaBand::Winded,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: MessagePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
    assert_eq!(p.kind(), MessageKind::State);
    assert!(p.involves(&tref().identifier.to_string()));
}

#[test]
fn stamina_band_enum_round_trips() {
    for band in [
        StaminaBand::Fresh,
        StaminaBand::Winded,
        StaminaBand::Exhausted,
    ] {
        let s = serde_json::to_string(&band).unwrap();
        let back: StaminaBand = serde_json::from_str(&s).unwrap();
        assert_eq!(band, back);
    }
}

#[test]
fn foraged_drank_ate_round_trip_and_kind() {
    let foraged = MessagePayload::Foraged {
        tribute: tref(),
        area: aref(),
        success: true,
        debt_recovered: 3,
    };
    let drank = MessagePayload::Drank {
        tribute: tref(),
        source: DrinkSource::Terrain { area: aref() },
        debt_recovered: 2,
    };
    let drank_item = MessagePayload::Drank {
        tribute: tref(),
        source: DrinkSource::Item { item: iref() },
        debt_recovered: 1,
    };
    let ate = MessagePayload::Ate {
        tribute: tref(),
        item: iref(),
        debt_recovered: 4,
    };
    for p in [foraged, drank, drank_item, ate] {
        let back: MessagePayload =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
        assert_eq!(p.kind(), MessageKind::State);
    }
}

#[test]
fn cause_constants_exist() {
    assert_eq!(CAUSE_STARVATION, "starvation");
    assert_eq!(CAUSE_DEHYDRATION, "dehydration");
}

#[test]
fn survival_payloads_involve_tribute() {
    let p = MessagePayload::Ate {
        tribute: tref(),
        item: iref(),
        debt_recovered: 1,
    };
    let id_str = TEST_TRIBUTE_ID.to_string();
    assert!(p.involves(&id_str));
    assert!(!p.involves("other"));
}

#[test]
fn wake_reason_serde_roundtrip_rested() {
    let r = WakeReason::Rested;
    let s = serde_json::to_string(&r).unwrap();
    let back: WakeReason = serde_json::from_str(&s).unwrap();
    assert_eq!(back, r);
}

#[test]
fn wake_reason_serde_roundtrip_interrupted_variants() {
    let cases = vec![
        WakeReason::Interrupted {
            event: InterruptionKind::Ambush {
                attacker: TributeRef {
                    identifier: TributeId(Uuid::new_v4()),
                    name: "A".into(),
                },
            },
        },
        WakeReason::Interrupted {
            event: InterruptionKind::AreaEvent {
                kind: AreaEventKind::Fire,
            },
        },
        WakeReason::Interrupted {
            event: InterruptionKind::AllianceSummons {
                ally: TributeRef {
                    identifier: TributeId(Uuid::new_v4()),
                    name: "B".into(),
                },
            },
        },
    ];
    for r in cases {
        let s = serde_json::to_string(&r).unwrap();
        let back: WakeReason = serde_json::from_str(&s).unwrap();
        assert_eq!(back, r);
    }
}

#[test]
fn tribute_slept_woke_payload_kind_is_state() {
    let slept = MessagePayload::TributeSlept {
        tribute: tref(),
        phase: Phase::Night,
        restored_stamina: 5,
        restored_hp: 2,
    };
    let woke = MessagePayload::TributeWoke {
        tribute: tref(),
        phase: Phase::Dawn,
        reason: WakeReason::Rested,
    };
    assert_eq!(slept.kind(), MessageKind::State);
    assert_eq!(woke.kind(), MessageKind::State);
}

#[test]
fn tribute_slept_woke_involves_tribute() {
    let slept = MessagePayload::TributeSlept {
        tribute: tref(),
        phase: Phase::Night,
        restored_stamina: 0,
        restored_hp: 0,
    };
    let woke = MessagePayload::TributeWoke {
        tribute: tref(),
        phase: Phase::Dawn,
        reason: WakeReason::Rested,
    };
    let id_str = TEST_TRIBUTE_ID.to_string();
    assert!(slept.involves(&id_str));
    assert!(!slept.involves("other"));
    assert!(woke.involves(&id_str));
    assert!(!woke.involves("other"));
}

#[test]
fn phobia_acquired_round_trips_and_kind() {
    let p = MessagePayload::PhobiaAcquired {
        tribute: "t1".into(),
        trigger: "fire".into(),
        severity: "mild".into(),
        origin: "innate".into(),
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: MessagePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
    assert_eq!(p.kind(), MessageKind::Phobia);
    assert!(p.involves("t1"));
    assert!(!p.involves("other"));
}

#[test]
fn phobia_triggered_round_trips_and_kind() {
    let p = MessagePayload::PhobiaTriggered {
        tribute: "t1".into(),
        trigger: "heights".into(),
        severity: "severe".into(),
        effect: PhobiaEffect::Freeze,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: MessagePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", p), format!("{:?}", back));
    assert_eq!(p.kind(), MessageKind::Phobia);
    assert!(p.involves("t1"));
    assert!(!p.involves("other"));
}

#[test]
fn phobia_effect_serde_roundtrip() {
    for effect in [
        PhobiaEffect::Penalty,
        PhobiaEffect::Flee,
        PhobiaEffect::Freeze,
    ] {
        let s = serde_json::to_string(&effect).unwrap();
        let back: PhobiaEffect = serde_json::from_str(&s).unwrap();
        assert_eq!(effect, back);
    }
}
