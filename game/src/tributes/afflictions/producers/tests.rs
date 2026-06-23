use super::run_trauma_producers;
use super::shared::map_cause_to_death_cause;
use crate::areas::Area;
use crate::games::Game;
use crate::messages::{GameMessage, MessagePayload, MessageSource, TributeRef};
use crate::tributes::Tribute;
use shared::afflictions::DeathCause;
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
            cause: shared::afflictions::DeathCause::Combat,
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
    let cause = map_cause_to_death_cause(Some(&killer), &DeathCause::Fire);
    assert!(matches!(cause, DeathCause::Tribute(id) if id == "tributes:killer"));
}

#[test]
fn map_cause_to_death_cause_passthrough() {
    assert!(matches!(
        map_cause_to_death_cause(None, &DeathCause::Fire),
        DeathCause::Fire
    ));
    assert!(matches!(
        map_cause_to_death_cause(None, &DeathCause::Drowning),
        DeathCause::Drowning
    ));
    assert!(matches!(
        map_cause_to_death_cause(None, &DeathCause::Starvation),
        DeathCause::Starvation
    ));
    assert!(matches!(
        map_cause_to_death_cause(None, &DeathCause::Unknown),
        DeathCause::Unknown
    ));
}
