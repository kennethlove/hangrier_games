use game::tributes::Tribute;
use game::tributes::afflictions::trauma::TraumaAcquisition;
use shared::afflictions::*;

fn tribute_at_cycle(cycle: u32) -> Tribute {
    Tribute {
        identifier: "tributes:integration".into(),
        game_day: Some(cycle as i64),
        ..Default::default()
    }
}

#[test]
fn trauma_survives_json_round_trip() {
    let mut t = tribute_at_cycle(3);
    t.try_acquire_trauma(
        TraumaSource::WitnessedAllyDeath {
            ally: "tributes:glimmer".into(),
            cause: Some(DeathCause::Fire),
        },
        Severity::Mild,
    );

    // Serialize affliction values as Vec (tuple keys can't be JSON map keys)
    let afflictions_vec: Vec<_> = t.afflictions.values().cloned().collect();
    let json = serde_json::to_string(&afflictions_vec).unwrap();

    // Deserialize
    let restored: Vec<shared::afflictions::Affliction> = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.len(), 1);
    let trauma = &restored[0];
    assert_eq!(trauma.kind, AfflictionKind::Trauma);
    assert_eq!(trauma.severity, Severity::Mild);
    assert_eq!(trauma.acquired_cycle, 3);
    assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 1);
}

#[test]
fn full_lifecycle_acquire_then_reinforce_via_floor_bump() {
    let mut t = tribute_at_cycle(1);

    let r1 = t.try_acquire_trauma(
        TraumaSource::WitnessedAllyDeath {
            ally: "tributes:glimmer".into(),
            cause: Some(DeathCause::Fire),
        },
        Severity::Mild,
    );
    assert!(matches!(r1, TraumaAcquisition::Acquired { .. }));

    // Simulate cycles passing.
    t.game_day = Some(7);
    if let Some(trauma) = t.afflictions.get_mut(&(AfflictionKind::Trauma, None)) {
        trauma
            .trauma_metadata
            .as_mut()
            .unwrap()
            .cycles_since_last_event = 6;
    }

    // Mass casualty event (Severe floor) at cycle 7.
    let r2 = t.try_acquire_trauma(
        TraumaSource::MassCasualty {
            cause_class: CauseClass::Combat,
            deaths_this_cycle: 5,
        },
        Severity::Severe,
    );
    match r2 {
        TraumaAcquisition::Reinforced {
            from_severity,
            to_severity,
            floor_bumped,
        } => {
            assert_eq!(from_severity, Severity::Mild);
            assert_eq!(to_severity, Severity::Severe);
            assert!(floor_bumped);
        }
        other => panic!("expected Reinforced with floor bump, got {:?}", other),
    }

    let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
    assert_eq!(trauma.severity, Severity::Severe);
    assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 2);
    assert_eq!(
        trauma
            .trauma_metadata
            .as_ref()
            .unwrap()
            .cycles_since_last_event,
        0
    );
    assert_eq!(trauma.last_progressed_cycle, 7);
}

#[test]
fn three_distinct_producers_merge_into_one_trauma() {
    let mut t = tribute_at_cycle(1);
    t.try_acquire_trauma(
        TraumaSource::WitnessedAllyDeath {
            ally: "tributes:a".into(),
            cause: Some(DeathCause::Fire),
        },
        Severity::Mild,
    );
    t.try_acquire_trauma(
        TraumaSource::Betrayal {
            by: "tributes:b".into(),
        },
        Severity::Moderate,
    );
    t.try_acquire_trauma(
        TraumaSource::NearDeath {
            cause: DeathCause::Drowning,
        },
        Severity::Moderate,
    );

    // Single Trauma slot, three sources merged, severity = floor of strongest = Moderate.
    assert_eq!(
        t.afflictions
            .keys()
            .filter(|(k, _)| *k == AfflictionKind::Trauma)
            .count(),
        1
    );
    let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
    assert_eq!(trauma.severity, Severity::Moderate);
    assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 3);
}
