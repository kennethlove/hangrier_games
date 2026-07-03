use super::*;
use serial_test::serial;

#[test]
fn test_game_new() {
    let game = Game::new("Test Game");
    assert_eq!(game.name, "Test Game");
    assert_eq!(game.status, GameStatus::NotStarted);
    assert_eq!(game.day, None);
    assert_eq!(game.tributes.len(), 0);
}

#[test]
fn game_has_empty_alliance_event_queue_on_new() {
    let g = Game::default();
    assert!(g.alliance_events.is_empty());
}

#[test]
fn test_game_start() {
    let mut game = Game::new("Test Game");
    game.start().expect("Failed to start game");
    assert_eq!(game.status, GameStatus::InProgress);
    assert_eq!(game.day, None);
}

#[test]
fn test_game_end() {
    let mut game = Game::new("Test Game");
    game.start().expect("Failed to start game");
    game.end();
    assert_eq!(game.status, GameStatus::Finished);
}

#[test]
fn test_living_and_recently_dead_tributes() {
    let mut game = Game::new("Test Game");
    let t1 = Tribute::default();
    let t2 = Tribute::default();
    game.tributes.push(t1);
    game.tributes.push(t2);
    assert_eq!(game.living_tributes().len(), 2);
    assert_eq!(game.recently_dead_tributes().len(), 0);
    game.tributes[0].status = TributeStatus::RecentlyDead;
    assert_eq!(game.living_tributes().len(), 1);
    assert_eq!(game.recently_dead_tributes().len(), 1);
}

#[test]
fn test_game_winner() {
    let mut game = Game::new("Test Game");
    let t1 = Tribute::default();
    let t2 = Tribute::default();
    game.tributes.push(t1);
    game.tributes.push(t2.clone());
    game.start().expect("Failed to start game");
    assert_eq!(game.winner(), None);
    game.tributes[0].status = TributeStatus::Dead;
    assert_eq!(game.winner().unwrap().name, t2.name);
}

#[test]
fn initiative_order_prefers_higher_agility() {
    let mut rng = SmallRng::seed_from_u64(42);
    let s1 = initiative_score(100, &mut rng);
    let mut rng = SmallRng::seed_from_u64(42);
    let s2 = initiative_score(1, &mut rng);
    assert!(s1 > s2, "agi 100 should beat agi 1 with same seed");
}

#[test]
fn initiative_fuzz_can_flip_close_scores() {
    let mut lower_won = false;
    for seed in 0..1000 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let s1 = initiative_score(50, &mut rng);
        let s2 = initiative_score(55, &mut rng);
        if s1 > s2 {
            lower_won = true;
            break;
        }
    }
    assert!(
        lower_won,
        "fuzz should sometimes overcome 5-point agility gap"
    );
}

#[test]
fn initiative_liveness_gate_still_works() {
    let mut game = Game::new("Test Game");
    game.start().expect("Failed to start game");

    let mut killer = Tribute::new("Killer".to_string(), None, None);
    killer.attributes.set_health(100);
    killer.attributes.strength = 50;
    killer.attributes.agility = 100;
    killer.area = Area::Cornucopia;

    let mut victim = Tribute::new("Victim".to_string(), None, None);
    victim.attributes.set_health(1);
    victim.attributes.strength = 1;
    victim.attributes.defense = 1;
    victim.attributes.agility = 1;
    victim.area = Area::Cornucopia;

    game.tributes.clear();
    game.tributes.push(killer);
    game.tributes.push(victim);

    let mut rng = SmallRng::seed_from_u64(42);
    let agility_0 = game.tributes[0].attributes.agility;
    let agility_1 = game.tributes[1].attributes.agility;
    let s0 = initiative_score(agility_0, &mut rng);
    let mut rng = SmallRng::seed_from_u64(42);
    let s1 = initiative_score(agility_1, &mut rng);
    assert!(
        s0 >= s1,
        "high-agility tribute should have >= initiative of low-agility"
    );
}

#[test]
fn attributes_new_includes_agility() {
    let attrs = Attributes::new();
    assert!(
        attrs.agility >= 1 && attrs.agility <= 100,
        "agility should be in 1..=100 range, got {}",
        attrs.agility
    );
}

#[test]
fn test_random_open_area() {
    let mut game = Game::new("Test Game");
    let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
    game.areas.push(area1);
    game.areas.push(area2.clone());
    assert!(game.random_area().is_some());
    let mut rng = rand::rng();
    let event = AreaEvent::random(&mut rng);
    game.areas[0].events.push(event.clone());
    assert_eq!(game.random_open_area().unwrap(), area2);
}

#[test]
fn test_clean_up_recent_deaths() {
    let mut game = Game::new("Test Game");

    let mut tribute = Tribute::default();
    tribute.set_status(TributeStatus::RecentlyDead);
    game.tributes.push(tribute.clone());

    assert_eq!(game.recently_dead_tributes().len(), 1);
    assert_eq!(game.recently_dead_tributes()[0], tribute);

    game.clean_up_recent_deaths();
    assert_eq!(game.tributes[0].status, TributeStatus::Dead);
}

#[test]
fn test_check_game_state_winner_exists() {
    let winner_tribute = create_tribute("Winner", true);
    let loser_tribute = create_tribute("Loser", false);
    let mut game =
        create_test_game_with_tributes(vec![winner_tribute.clone(), loser_tribute.clone()]);

    assert_eq!(game.living_tributes().len(), 1);
    assert_eq!(game.winner(), Some(winner_tribute.clone()));

    let _ = game.check_for_winner();

    assert_eq!(game.status, GameStatus::Finished);
}

#[test]
fn test_check_game_state_no_survivors() {
    let loser_tribute = create_tribute("Loser", false);
    let loser2_tribute = create_tribute("Loser 2", false);
    let mut game =
        create_test_game_with_tributes(vec![loser_tribute.clone(), loser2_tribute.clone()]);

    assert!(game.living_tributes().is_empty());
    assert!(game.winner().is_none());

    let _ = game.check_for_winner();

    assert_eq!(game.status, GameStatus::Finished);
}

#[test]
fn test_check_game_state_continues() {
    let living_tribute1 = create_tribute("Living1", true);
    let living_tribute2 = create_tribute("Living2", true);
    let mut game =
        create_test_game_with_tributes(vec![living_tribute1.clone(), living_tribute2.clone()]);
    let starting_state = game.status.clone();

    assert_eq!(game.living_tributes().len(), 2);
    assert!(game.winner().is_none());

    let _ = game.check_for_winner();

    assert_eq!(game.status, starting_state);
}

#[test]
fn test_prepare_cycle() {
    use crate::messages::Phase;
    let mut game = Game::new("Test Game");
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let mut rng = rand::rng();
    let event = AreaEvent::random(&mut rng);
    game.day = Some(1);
    game.areas.push(area);
    game.areas[0].events.push(event.clone());
    let _ = game.prepare_cycle(Phase::Dawn);
    assert_eq!(game.day, Some(2));
    assert_eq!(game.areas[0].events.len(), 0);

    game.areas[0].events.push(event.clone());
    let _ = game.prepare_cycle(Phase::Night);
    assert_eq!(game.day, Some(2));
    assert_eq!(game.areas[0].events.len(), 0);
}

#[test]
fn test_trigger_cycle_events() {}

#[test]
fn test_constrain_areas() {
    let mut game = Game::new("Test Game");
    let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
    game.areas.push(area1);
    game.areas.push(area2);

    let tribute1 = create_tribute("Tribute1", true);
    let tribute2 = create_tribute("Tribute2", true);
    game.tributes.push(tribute1.clone());
    game.tributes.push(tribute2.clone());

    let mut rng = SmallRng::seed_from_u64(0);
    let _ = game.constrain_areas(&mut rng);

    assert!(game.random_open_area().is_some());
    assert_eq!(game.open_areas().len(), 1);
    assert_eq!(game.closed_areas().len(), 1);
}

#[test]
fn test_run_tribute_cycle() {
    let tribute1 = create_tribute("Tribute1", true);
    let tribute2 = create_tribute("Tribute2", true);

    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let closed_areas = game
        .areas
        .iter()
        .filter(|ad| ad.area.is_some() & !ad.is_open())
        .map(|ad| ad.area.unwrap())
        .collect::<Vec<Area>>();

    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![tribute1.clone(), tribute2.clone()],
        2,
    );

    let new_tribute1 = game.tributes[0].clone();
    let new_tribute2 = game.tributes[1].clone();
    assert_ne!(tribute1, new_tribute1);
    assert_ne!(tribute2, new_tribute2);
}

#[test]
fn test_open_and_closed_areas() {
    let mut game = Game::new("Test Game");
    let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
    game.areas.push(area1);
    game.areas.push(area2);

    assert_eq!(game.open_areas().len(), 2);
    assert!(game.closed_areas().is_empty());

    let mut rng = rand::rng();
    game.areas[0].events.push(AreaEvent::random(&mut rng));

    assert_eq!(game.open_areas().len(), 1);
    assert_eq!(game.closed_areas().len(), 1);
}

#[test]
fn test_ensure_open_area() {
    let mut game = Game::new("Test Game");
    let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
    game.areas.push(area1);
    game.areas.push(area2);

    assert!(game.random_open_area().is_some());

    let mut rng = rand::rng();
    game.areas[0].events.push(AreaEvent::random(&mut rng));
    game.areas[1].events.push(AreaEvent::random(&mut rng));

    assert!(game.random_open_area().is_none());

    game.ensure_open_area();
    assert!(game.random_open_area().is_some());
}

#[test]
fn survival_tick_increments_hunger_and_thirst_per_phase() {
    let mut a = Tribute::new("A".to_string(), Some(1), None);
    let mut b = Tribute::new("B".to_string(), Some(2), None);
    for t in [&mut a, &mut b] {
        t.attributes.strength = 30;
        t.stamina = t.max_stamina / 2;
    }
    let mut game = create_test_game_with_tributes(vec![a, b]);
    game.day = Some(1);
    let _ = game.run_phase(crate::messages::Phase::Day);
    for t in &game.tributes {
        assert_eq!(t.hunger, 1, "{} hunger should be 1 after one tick", t.name);
        assert_eq!(t.thirst, 1, "{} thirst should be 1 after one tick", t.name);
    }
}

#[test]
fn survival_tick_routes_dehydration_death_through_tribute_killed() {
    use crate::messages::MessagePayload;
    use shared::afflictions::DeathCause;
    let mut a = Tribute::new("Doomed".to_string(), Some(1), None);
    a.thirst = 4;
    a.dehydration_drain_step = 5;
    a.attributes.set_health(1);
    let mut game = create_test_game_with_tributes(vec![a]);
    game.day = Some(1);
    let _ = game.run_phase(crate::messages::Phase::Day);
    let killed = game.messages.iter().any(|m| {
        matches!(&m.payload,
            MessagePayload::TributeKilled { cause, .. } if *cause == DeathCause::Dehydration)
    });
    assert!(killed, "expected a TributeKilled with cause=dehydration");
}

#[test]
#[serial]
fn sleeping_tribute_naturally_wakes_after_duration_emits_tribute_woke() {
    use crate::messages::{MessagePayload, Phase};
    let mut t = create_tribute("Sleeper", true);
    t.sleeping = true;
    t.sleep_remaining = 1;
    t.cycles_awake = 9;
    t.stamina = 50;
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);

    let woken = &game.tributes[0];
    assert!(!woken.sleeping, "tribute should be awake");
    assert_eq!(woken.sleep_remaining, 0);
    assert_eq!(woken.cycles_awake, 0, "natural wake resets cycles_awake");

    let woke = game.messages.iter().any(|m| {
        matches!(
            &m.payload,
            MessagePayload::TributeWoke {
                reason: shared::messages::WakeReason::Rested,
                ..
            }
        )
    });
    assert!(woke, "expected a TributeWoke{{Rested}} message");
}

#[test]
#[serial]
fn sleeping_tribute_regenerates_stamina_each_phase() {
    use crate::messages::Phase;
    let mut t = create_tribute("Sleeper", true);
    t.sleeping = true;
    t.sleep_remaining = 3;
    t.stamina = 10;
    t.max_stamina = 100;
    let prior = t.stamina;
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);

    let after = &game.tributes[0];
    assert!(after.sleeping, "still mid-sleep");
    assert_eq!(after.sleep_remaining, 2);
    assert!(after.stamina > prior, "stamina should regen");
}

#[test]
#[serial]
fn sleeping_wounded_tribute_does_not_regen_hp() {
    use crate::messages::Phase;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, Severity};
    let mut t = create_tribute("Sleeper", true);
    t.sleeping = true;
    t.sleep_remaining = 3;
    t.attributes.set_health(40);
    t.afflictions.insert(
        (AfflictionKind::Wounded, None),
        Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        },
    );
    let prior_hp = t.attributes.health();
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);
    assert_eq!(
        game.tributes[0].attributes.health(),
        prior_hp,
        "wounded tributes do not heal while sleeping"
    );
}

#[test]
#[serial]
fn cycles_awake_does_not_increment_while_sleeping() {
    use crate::messages::Phase;
    let mut t = create_tribute("Sleeper", true);
    t.sleeping = true;
    t.sleep_remaining = 3;
    t.cycles_awake = 4;
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);
    assert_eq!(game.tributes[0].cycles_awake, 4);
}

#[test]
fn area_event_interrupts_sleeping_tribute() {
    use crate::areas::events::AreaEvent;
    use crate::messages::{MessagePayload, Phase};
    let mut t = create_tribute("Sleeper", true);
    t.sleeping = true;
    t.sleep_remaining = 5;
    t.cycles_awake = 9;
    t.stamina = 10;
    let prior_stamina = t.stamina;
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let mut area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    area.events.push(AreaEvent::Wildfire);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);

    let woken = &game.tributes[0];
    assert!(!woken.sleeping, "area-event should wake sleeper");
    assert_eq!(woken.sleep_remaining, 0);
    assert_eq!(woken.cycles_awake, 0);
    assert!(
        woken.stamina < prior_stamina + SLEEP_STAMINA_PER_PHASE,
        "sleep-tick regen must be skipped on interruption (stamina={})",
        woken.stamina
    );

    let woke = game.messages.iter().any(|m| {
        matches!(
            &m.payload,
            MessagePayload::TributeWoke {
                reason: shared::messages::WakeReason::Interrupted {
                    event: shared::messages::InterruptionKind::AreaEvent {
                        kind: shared::messages::AreaEventKind::Fire,
                    },
                },
                ..
            }
        )
    });
    assert!(woke, "expected TributeWoke{{Interrupted/AreaEvent/Fire}}");
}

#[test]
fn alliance_summons_wakes_sleeping_target() {
    use crate::messages::MessagePayload;
    let mut summoner = create_tribute("Cinna", true);
    let mut target = create_tribute("Katniss", true);
    target.sleeping = true;
    target.sleep_remaining = 3;
    target.cycles_awake = 6;
    let summoner_id = summoner.id;
    let target_id = target.id;
    summoner.allies.push(target_id);
    target.allies.push(summoner_id);

    let mut game = create_test_game_with_tributes(vec![summoner, target]);
    game.alliance_events
        .push(crate::tributes::alliances::AllianceEvent::AllianceSummons {
            summoner: summoner_id,
            target: target_id,
        });
    let mut rng = SmallRng::seed_from_u64(42);
    game.process_alliance_events(&mut rng);

    let woken = &game.tributes[1];
    assert!(!woken.sleeping, "summons should wake sleeping ally");
    assert_eq!(woken.sleep_remaining, 0);
    assert_eq!(woken.cycles_awake, 0);

    let woke = game.messages.iter().any(|m| {
        matches!(
            &m.payload,
            MessagePayload::TributeWoke {
                reason: shared::messages::WakeReason::Interrupted {
                    event: shared::messages::InterruptionKind::AllianceSummons { .. },
                },
                ..
            }
        )
    });
    assert!(woke, "expected TributeWoke{{Interrupted/AllianceSummons}}");
}
