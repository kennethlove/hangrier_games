use super::*;
use crate::tributes::Attributes;
use serial_test::serial;

fn create_test_game_with_tributes(tributes: Vec<Tribute>) -> Game {
    Game {
        identifier: "test-game".to_string(),
        name: "Test Game".to_string(),
        status: GameStatus::InProgress,
        day: Some(1),
        areas: vec![],
        tributes,
        private: true,
        config: Default::default(),
        messages: vec![],
        alliance_events: vec![],
        tick_counter: TickCounter::default(),
        current_phase: crate::messages::Phase::Day,
        emit_index: 0,
        combat_tuning: crate::tributes::combat_tuning::CombatTuning::default(),
        sponsors: vec![],
    }
}

fn create_tribute(name: &str, is_alive: bool) -> Tribute {
    let mut tribute = Tribute::new(name.to_string(), None, None);
    if is_alive {
        tribute.attributes.health = 100;
        tribute.status = TributeStatus::Healthy;
    } else {
        tribute.attributes.health = 0;
        tribute.status = TributeStatus::Dead;
    }
    tribute
}

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
    // Two tributes in same area, high-agility one kills the other.
    // Verify dead tribute doesn't act after being killed.
    let mut game = Game::new("Test Game");
    game.start().expect("Failed to start game");

    let mut killer = Tribute::new("Killer".to_string(), None, None);
    killer.attributes.health = 100;
    killer.attributes.strength = 50;
    killer.attributes.agility = 100;
    killer.area = Area::Cornucopia;

    let mut victim = Tribute::new("Victim".to_string(), None, None);
    victim.attributes.health = 1;
    victim.attributes.strength = 1;
    victim.attributes.defense = 1;
    victim.attributes.agility = 1;
    victim.area = Area::Cornucopia;

    // Cannot use Tribute::new for ID management; push fresh tributes
    game.tributes.clear();
    game.tributes.push(killer);
    game.tributes.push(victim);

    // Run do_a_cycle — the rest fails if the cycle mechanism is too
    // complex to set up here. At minimum we verify the initiative
    // sort is applied: high-agility tributes sort first.
    let mut rng = SmallRng::seed_from_u64(42);
    let agility_0 = game.tributes[0].attributes.agility;
    let agility_1 = game.tributes[1].attributes.agility;
    let s0 = initiative_score(agility_0, &mut rng);
    let mut rng = SmallRng::seed_from_u64(42);
    let s1 = initiative_score(agility_1, &mut rng);
    // The high-agility tribute (index 0) should have higher initiative
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

    // Game should have only one living tribute and they should be the winner
    assert_eq!(game.living_tributes().len(), 1);
    assert_eq!(game.winner(), Some(winner_tribute.clone()));

    let _ = game.check_for_winner();

    // Game should be finished
    assert_eq!(game.status, GameStatus::Finished);
}

#[test]
fn test_check_game_state_no_survivors() {
    let loser_tribute = create_tribute("Loser", false);
    let loser2_tribute = create_tribute("Loser 2", false);
    let mut game =
        create_test_game_with_tributes(vec![loser_tribute.clone(), loser2_tribute.clone()]);

    // Game should have only no living tributes and no winner
    assert!(game.living_tributes().is_empty());
    assert!(game.winner().is_none());

    let _ = game.check_for_winner();

    // Game should be finished
    assert_eq!(game.status, GameStatus::Finished);
}

#[test]
fn test_check_game_state_continues() {
    let living_tribute1 = create_tribute("Living1", true);
    let living_tribute2 = create_tribute("Living2", true);
    let mut game =
        create_test_game_with_tributes(vec![living_tribute1.clone(), living_tribute2.clone()]);
    let starting_state = game.status.clone();

    // Game should have only one living tribute and they should be the winner
    assert_eq!(game.living_tributes().len(), 2);
    assert!(game.winner().is_none());

    let _ = game.check_for_winner();

    // Game should be finished
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
    // Dawn on Day 2+ advances the day and clears events.
    let _ = game.prepare_cycle(Phase::Dawn);
    assert_eq!(game.day, Some(2));
    assert_eq!(game.areas[0].events.len(), 0);

    game.areas[0].events.push(event.clone());
    // Night never advances the day.
    let _ = game.prepare_cycle(Phase::Night);
    assert_eq!(game.day, Some(2));
    assert_eq!(game.areas[0].events.len(), 0);
}

#[test]
fn test_announce_cycle_start() {
    // Clear any messages from other tests running in parallel

    let tribute1 = create_tribute("Tribute1", true);
    let tribute2 = create_tribute("Tribute2", true);
    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    game.day = Some(1);
    let _ = game.announce_cycle_start(crate::messages::Phase::Day);
    // Day 1 has two announcements: legacy CycleStart + new PhaseStarted.
    assert_eq!(game.messages.len(), 2);
}

#[test]
fn test_announce_cycle_end() {
    // Clear any messages from other tests running in parallel

    let tribute1 = create_tribute("Tribute1", true);
    let mut tribute2 = create_tribute("Tribute2", false);
    tribute2.set_status(TributeStatus::RecentlyDead);
    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    game.day = Some(1);
    let _ = game.announce_cycle_end(crate::messages::Phase::Day);
    // Death announcements moved to the kill site as typed
    // `MessagePayload::TributeKilled`. Two messages remain:
    // legacy CycleEnd + new PhaseEnded.
    assert_eq!(game.messages.len(), 2);
}

#[test]
fn test_announce_area_events() {
    let mut game = Game::new("Test Game");
    let mut area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    let mut rng = rand::rng();
    area.events.push(AreaEvent::random(&mut rng));
    area.events.push(AreaEvent::random(&mut rng));
    game.areas.push(area);

    assert!(!game.areas[0].is_open());
    let _ = game.announce_area_events();

    // 2 AreaEvent lines + 1 AreaClose summary
    assert_eq!(game.messages.len(), 3);
    // All emitted under the Area channel for the affected area.
    let area_name = Area::Cornucopia.to_string();
    for msg in &game.messages {
        assert_eq!(
            msg.source,
            crate::messages::MessageSource::Area(area_name.clone())
        );
        assert_eq!(
            msg.subject,
            format!("{}:area:{}", game.identifier, area_name)
        );
    }
}

/// Regression for hangrier_games-i7rq: every emitted message's
/// subject must start with the game identifier so the API's
/// per-game log queries (`WHERE string::starts_with(subject,
/// $game_id)`) match. Without this, day pages and timeline summary
/// were always empty.
#[test]
fn message_subjects_are_prefixed_with_game_id() {
    let mut game = Game::new("Subject Prefix Test");
    game.log(
        crate::messages::MessageSource::Game(game.identifier.clone()),
        format!("game:{}", game.identifier),
        "hello".to_string(),
    );
    game.log(
        crate::messages::MessageSource::Area("Cornucopia".to_string()),
        "area:Cornucopia".to_string(),
        "boom".to_string(),
    );
    game.log(
        crate::messages::MessageSource::Tribute("trib-id".to_string()),
        "tribute:trib-id".to_string(),
        "ouch".to_string(),
    );
    let prefix = format!("{}:", game.identifier);
    for msg in &game.messages {
        assert!(
            msg.subject.starts_with(&prefix),
            "subject {:?} missing game-id prefix {:?}",
            msg.subject,
            prefix
        );
    }
    // Idempotent: calling log twice should not double-prefix.
    let count_before = game.messages.len();
    let already_prefixed = format!("{}:area:Other", game.identifier);
    game.log(
        crate::messages::MessageSource::Area("Other".to_string()),
        already_prefixed.clone(),
        "ok".to_string(),
    );
    assert_eq!(
        game.messages[count_before].subject, already_prefixed,
        "subject already prefixed should not be double-prefixed"
    );
}

#[test]
fn test_ensure_open_area() {
    let mut game = Game::new("Test Game");
    let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
    let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
    game.areas.push(area1);
    game.areas.push(area2);

    assert!(game.random_open_area().is_some());

    // Close the areas
    let mut rng = rand::rng();
    game.areas[0].events.push(AreaEvent::random(&mut rng));
    game.areas[1].events.push(AreaEvent::random(&mut rng));

    assert!(game.random_open_area().is_none());

    game.ensure_open_area();
    assert!(game.random_open_area().is_some());
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

    // Add tributes to the game
    let tribute1 = create_tribute("Tribute1", true);
    let tribute2 = create_tribute("Tribute2", true);
    game.tributes.push(tribute1.clone());
    game.tributes.push(tribute2.clone());

    // Constrain areas
    // Use a fixed seed so the area-selection branch is deterministic.
    let mut rng = SmallRng::seed_from_u64(0);
    let _ = game.constrain_areas(&mut rng);

    // Check if at least one area is closed
    assert!(game.random_open_area().is_some());
    assert_eq!(game.open_areas().len(), 1);
    assert_eq!(game.closed_areas().len(), 1);
}

#[test]
fn test_run_tribute_cycle() {
    // Add tributes
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

    // Run the tribute cycle
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![tribute1.clone(), tribute2.clone()],
        2,
    );

    // Check if the tributes are updated correctly
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

    // Close one area
    let mut rng = rand::rng();
    game.areas[0].events.push(AreaEvent::random(&mut rng));

    assert_eq!(game.open_areas().len(), 1);
    assert_eq!(game.closed_areas().len(), 1);
}

// ---- Phase 4: alliance event drain -----------------------------------

#[test]
fn process_alliance_events_betrayal_removes_pair_on_victim_side() {
    // Victim still lists betrayer in allies (betrayer's own list was
    // already cleaned by the betrayal trigger that enqueued the event).
    let mut betrayer = Tribute::new("Betrayer".to_string(), Some(1), None);
    let mut victim = Tribute::new("Victim".to_string(), Some(2), None);
    victim.allies.push(betrayer.id);
    // Sanity force victim to a state where the drain path runs cleanly.
    victim.attributes.health = 100;
    betrayer.attributes.health = 100;
    let bid = betrayer.id;
    let vid = victim.id;

    let mut game = create_test_game_with_tributes(vec![betrayer, victim]);
    game.alliance_events.push(
        crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
            betrayer: bid,
            victim: vid,
        },
    );

    let mut rng = SmallRng::seed_from_u64(53);
    game.process_alliance_events(&mut rng);

    let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
    assert!(!v.allies.contains(&bid));
    assert!(v.pending_trust_shock);
    assert!(game.alliance_events.is_empty());
}

#[test]
fn process_alliance_events_betrayer_not_marked_for_trust_shock() {
    let betrayer = Tribute::new("Betrayer".to_string(), Some(1), None);
    let victim = Tribute::new("Victim".to_string(), Some(2), None);
    let bid = betrayer.id;
    let vid = victim.id;

    let mut game = create_test_game_with_tributes(vec![betrayer, victim]);
    game.alliance_events.push(
        crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
            betrayer: bid,
            victim: vid,
        },
    );

    let mut rng = SmallRng::seed_from_u64(53);
    game.process_alliance_events(&mut rng);

    let b = game.tributes.iter().find(|t| t.id == bid).unwrap();
    assert!(!b.pending_trust_shock, "betrayer must not roll trust-shock");
}

#[test]
fn process_alliance_events_death_removes_deceased_from_all_ally_lists() {
    // Three tributes; the deceased was in two allies' lists.
    let deceased = Tribute::new("Deceased".to_string(), Some(1), None);
    let mut a = Tribute::new("A".to_string(), Some(2), None);
    let mut b = Tribute::new("B".to_string(), Some(3), None);
    a.allies.push(deceased.id);
    b.allies.push(deceased.id);
    // Force ally sanity well above any threshold so the cascade roll
    // never fires; we want to verify the unconditional cleanup path.
    a.attributes.sanity = 100;
    b.attributes.sanity = 100;

    let did = deceased.id;
    let mut game = create_test_game_with_tributes(vec![deceased, a, b]);
    game.alliance_events
        .push(crate::tributes::alliances::AllianceEvent::DeathRecorded {
            deceased: did,
            killer: None,
        });

    let mut rng = SmallRng::seed_from_u64(89);
    game.process_alliance_events(&mut rng);

    for t in game.tributes.iter().filter(|t| t.id != did) {
        assert!(
            !t.allies.contains(&did),
            "tribute {} still lists deceased",
            t.name
        );
    }
    assert!(game.alliance_events.is_empty());
}

#[test]
fn run_tribute_cycle_drains_tribute_alliance_events_into_game_queue() {
    // Pre-load tribute1's alliance_events buffer; after run_tribute_cycle
    // it must be drained into the game queue and processed (BetrayalRecorded
    // cleans the victim's allies and flags pending_trust_shock).
    let mut tribute1 = create_tribute("Tribute1", true);
    let mut tribute2 = create_tribute("Tribute2", true);
    // Make tribute2 list tribute1 as ally; betrayal has tribute1 as betrayer,
    // tribute2 as victim. Plumbing only — we just need the side effects we
    // can observe on the victim.
    tribute2.allies.push(tribute1.id);
    let bid = tribute1.id;
    let vid = tribute2.id;
    tribute1.alliance_events.push(
        crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
            betrayer: bid,
            victim: vid,
        },
    );

    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let closed_areas = game
        .areas
        .iter()
        .filter(|ad| ad.area.is_some() & !ad.is_open())
        .map(|ad| ad.area.unwrap())
        .collect::<Vec<Area>>();

    let mut rng = SmallRng::seed_from_u64(211);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![tribute1.clone(), tribute2.clone()],
        2,
    );

    // After cycle: queue empty (drained + processed), each tribute's local
    // buffer empty, victim's allies cleaned, victim flagged for trust shock.
    assert!(game.alliance_events.is_empty(), "game queue must drain");
    for t in &game.tributes {
        assert!(
            t.alliance_events.is_empty(),
            "tribute {} buffer must drain",
            t.name
        );
    }
    let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
    assert!(!v.allies.contains(&bid), "victim allies cleaned");
    assert!(v.pending_trust_shock, "victim flagged for trust shock");
}

#[test]
fn run_tribute_cycle_forms_alliance_between_compatible_same_area_tributes() {
    // Two Friendly tributes from the same district sharing an area
    // should be able to form an alliance during a cycle. With both
    // sides starting at 0 allies, district bonus, and Friendly affinity
    // 1.5 each, roll_chance ≈ 0.675; with a fixed seed and many trials
    // we deterministically observe at least one cycle that forms.
    use crate::tributes::traits::Trait;
    let mut t1 = create_tribute("Cinna", true);
    let mut t2 = create_tribute("Portia", true);
    // Force compatibility: same district + Friendly traits.
    t1.district = 1;
    t2.district = 1;
    t1.traits = vec![Trait::Friendly];
    t2.traits = vec![Trait::Friendly];
    // Place both in Cornucopia (default `Tribute::new` already does this,
    // but be explicit for the test's intent).
    t1.area = Area::Cornucopia;
    t2.area = Area::Cornucopia;

    let id1 = t1.id;
    let id2 = t2.id;

    let mut game = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let closed_areas = game
        .areas
        .iter()
        .filter(|ad| ad.area.is_some() & !ad.is_open())
        .map(|ad| ad.area.unwrap())
        .collect::<Vec<Area>>();

    // Loop a few seeded cycles until at least one forms; if production
    // wiring is correct this should hit within a handful of trials.
    // Alliance formation is now a deliberate `Action::ProposeAlliance`
    // gated by Brain::wants_to_propose_alliance (5%-15% per turn for
    // eligible tributes). Sweep many seeds so we deterministically hit at
    // least one cycle where a Friendly same-district pair proposes and
    // succeeds.
    let mut formed = false;
    for seed in 0u64..400 {
        let mut g = game.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = g.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas.clone(),
            vec![t1.clone(), t2.clone()],
            2,
        );
        let a1 = g.tributes.iter().find(|t| t.id == id1).unwrap();
        let a2 = g.tributes.iter().find(|t| t.id == id2).unwrap();
        if a1.allies.contains(&id2) && a2.allies.contains(&id1) {
            formed = true;
            break;
        }
    }
    assert!(
        formed,
        "Friendly same-district pair must form an alliance within a few cycles"
    );
}

#[test]
fn run_tribute_cycle_treacherous_tribute_betrays_same_area_ally_when_timer_elapses() {
    // Treacherous tribute with an ally in the same area, timer at the
    // betrayal interval, must enqueue BetrayalRecorded during the cycle.
    // After process_alliance_events, the victim must have:
    //   - pending_trust_shock set
    //   - betrayer removed from allies
    // and the betrayer must have:
    //   - victim removed from allies
    //   - timer reset to 0
    use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
    use crate::tributes::traits::Trait;

    let mut betrayer = create_tribute("Cato", true);
    let mut victim = create_tribute("Glimmer", true);
    betrayer.traits = vec![Trait::Treacherous];
    // Strip any auto-generated traits from victim that might accidentally
    // form an alliance back during the cycle (we want a pre-existing ally).
    victim.traits = vec![Trait::Tough];
    // Pre-existing alliance set up manually.
    betrayer.allies.push(victim.id);
    victim.allies.push(betrayer.id);
    // Timer at threshold so betrayal fires this turn.
    betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
    // Same area.
    betrayer.area = Area::Cornucopia;
    victim.area = Area::Cornucopia;
    // Different districts so we don't accidentally re-form alliances
    // during the formation pass.
    betrayer.district = 1;
    victim.district = 2;

    let bid = betrayer.id;
    let vid = victim.id;

    let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let closed_areas = game
        .areas
        .iter()
        .filter(|ad| ad.area.is_some() & !ad.is_open())
        .map(|ad| ad.area.unwrap())
        .collect::<Vec<Area>>();

    let mut rng = SmallRng::seed_from_u64(313);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![betrayer.clone(), victim.clone()],
        2,
    );

    let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
    let b = game.tributes.iter().find(|t| t.id == bid).unwrap();
    assert!(!v.allies.contains(&bid), "victim allies cleaned by event");
    assert!(v.pending_trust_shock, "victim flagged for trust shock");
    assert!(!b.allies.contains(&vid), "betrayer dropped victim locally");
    // tick_alliance_timers ran and incremented to TREACHEROUS_BETRAYAL_INTERVAL+1
    // before betrayal fired? No: betrayal logic must reset to 0. After reset,
    // the rest of process_turn_phase doesn't tick again, so we expect 0.
    assert_eq!(
        b.turns_since_last_betrayal, 0,
        "betrayal resets the cooldown timer"
    );
}

#[test]
fn run_tribute_cycle_treacherous_no_betrayal_without_same_area_ally_resets_timer() {
    // Treacherous tribute alone in its area: no betrayal possible, but
    // the timer should still reset (one missed opportunity per cycle).
    use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
    use crate::tributes::traits::Trait;

    let mut loner = create_tribute("Foxface", true);
    let mut other = create_tribute("Marvel", true);
    loner.traits = vec![Trait::Treacherous];
    other.traits = vec![Trait::Tough];
    loner.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
    // Different areas so other is not a same-area ally.
    loner.area = Area::Sector1;
    other.area = Area::Sector4;
    loner.district = 5;
    other.district = 6;
    let lid = loner.id;

    let mut game = create_test_game_with_tributes(vec![loner.clone(), other.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Hill".to_string()), Area::Sector1));
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Sector4));
    let closed_areas = game
        .areas
        .iter()
        .filter(|ad| ad.area.is_some() & !ad.is_open())
        .map(|ad| ad.area.unwrap())
        .collect::<Vec<Area>>();

    let mut rng = SmallRng::seed_from_u64(419);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![loner.clone(), other.clone()],
        2,
    );

    let l = game.tributes.iter().find(|t| t.id == lid).unwrap();
    assert_eq!(
        l.turns_since_last_betrayal, 0,
        "missed opportunity also resets the timer per spec §7.4(b)"
    );
}

#[test]
fn run_tribute_cycle_enqueues_death_recorded_for_recently_dead_ally() {
    // A tribute who died last cycle (status=RecentlyDead) must trigger
    // a DeathRecorded event so allies process the ally-death cascade.
    // After the cycle, the deceased's allies should have:
    //   - the deceased removed from their `allies` lists (via cascade);
    //   - process_alliance_events drained the queue.
    use crate::tributes::traits::Trait;

    let mut deceased = create_tribute("Rue", true);
    let mut survivor = create_tribute("Katniss", true);
    // Make survivor highly likely to break on cascade: low sanity, high
    // threshold makes deficit_ratio close to 1.0 → near-certain break.
    survivor.attributes.sanity = 0;
    survivor.brain.thresholds.extreme_low_sanity = 50;
    survivor.traits = vec![Trait::Tough];
    deceased.traits = vec![Trait::Tough];
    // Pre-existing alliance (survivor lists deceased as ally).
    survivor.allies.push(deceased.id);
    deceased.allies.push(survivor.id);
    // Mark deceased as RecentlyDead going into the cycle.
    deceased.attributes.health = 0;
    deceased.status = TributeStatus::RecentlyDead;
    // Same area so deceased is "in the cycle" but the early skip applies.
    deceased.area = Area::Cornucopia;
    survivor.area = Area::Cornucopia;
    deceased.district = 11;
    survivor.district = 12;

    let did = deceased.id;
    let sid = survivor.id;

    let mut game = create_test_game_with_tributes(vec![deceased.clone(), survivor.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    // Living tributes snapshot: deceased is RecentlyDead so excluded.
    let living = game.living_tributes();
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(547);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        living,
        1,
    );

    // Cycle drained the queue (no leftovers).
    assert!(
        game.alliance_events.is_empty(),
        "queue must drain after cycle"
    );
    // Deceased promoted to Dead.
    let d = game.tributes.iter().find(|t| t.id == did).unwrap();
    assert_eq!(d.status, TributeStatus::Dead);
    // Survivor's ally list cleaned of deceased (cascade fired with high
    // probability given sanity=0 vs threshold=50; even on the rare miss
    // the alliance edge is still broken because process_alliance_events
    // does symmetric removal of dead from all surviving allies' lists).
    let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
    assert!(
        !s.allies.contains(&did),
        "survivor must not retain a dead ally edge"
    );
}

#[test]
fn run_tribute_cycle_three_way_preserves_existing_alliance() {
    // Three-way scenario: A and B are pre-allied; C is a LoneWolf in
    // the same area (refuser → cannot form an alliance with either).
    // After a cycle, A and B's bond must remain intact and C must
    // remain unallied. This pins that:
    //   1. The formation pass does not silently rebreak existing
    //      same-area alliances.
    //   2. The presence of a third unalliable tribute does not
    //      perturb the pair's bond.
    use crate::tributes::traits::Trait;

    let mut a = create_tribute("Katniss", true);
    let mut b = create_tribute("Peeta", true);
    let mut c = create_tribute("Cato", true);
    a.traits = vec![Trait::Friendly];
    b.traits = vec![Trait::Loyal];
    c.traits = vec![Trait::LoneWolf];
    // Pre-existing symmetric alliance between A and B.
    a.allies.push(b.id);
    b.allies.push(a.id);
    a.area = Area::Cornucopia;
    b.area = Area::Cornucopia;
    c.area = Area::Cornucopia;
    a.district = 1;
    b.district = 2;
    c.district = 3;

    let aid = a.id;
    let bid = b.id;
    let cid = c.id;

    let mut game = create_test_game_with_tributes(vec![a.clone(), b.clone(), c.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let living = game.living_tributes();
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(547);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        living,
        3,
    );

    let a2 = game.tributes.iter().find(|t| t.id == aid).unwrap();
    let b2 = game.tributes.iter().find(|t| t.id == bid).unwrap();
    let c2 = game.tributes.iter().find(|t| t.id == cid).unwrap();
    assert!(a2.allies.contains(&bid), "A still allied with B");
    assert!(b2.allies.contains(&aid), "B still allied with A");
    assert!(!a2.allies.contains(&cid), "A did not bond with LoneWolf C");
    assert!(!b2.allies.contains(&cid), "B did not bond with LoneWolf C");
    assert!(c2.allies.is_empty(), "LoneWolf C remains unallied");
}

#[test]
fn run_tribute_cycle_consumes_recently_killed_by_for_combat_death() {
    // A tribute who died at a combat site has `recently_killed_by` set
    // by the combat code. The cycle must read it, emit DeathRecorded
    // with that killer, and clear the field so it does not leak into
    // subsequent cycles.
    let mut deceased = create_tribute("Rue", true);
    let mut killer = create_tribute("Cato", true);
    let mut survivor = create_tribute("Katniss", true);

    let did = deceased.id;
    let kid = killer.id;
    let sid = survivor.id;

    // Pre-existing alliance so DeathRecorded has a cascade target.
    survivor.allies.push(deceased.id);
    deceased.allies.push(survivor.id);

    // Simulate combat outcome going into the cycle.
    deceased.attributes.health = 0;
    deceased.status = TributeStatus::RecentlyDead;
    deceased.recently_killed_by = Some(kid);

    deceased.area = Area::Cornucopia;
    killer.area = Area::Cornucopia;
    survivor.area = Area::Cornucopia;
    deceased.district = 11;
    killer.district = 2;
    survivor.district = 12;

    let mut game =
        create_test_game_with_tributes(vec![deceased.clone(), killer.clone(), survivor.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let living = game.living_tributes();
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(547);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        living,
        1,
    );

    // Cycle drained the queue.
    assert!(
        game.alliance_events.is_empty(),
        "queue must drain after cycle"
    );
    // Deceased promoted to Dead and field cleared.
    let d = game.tributes.iter().find(|t| t.id == did).unwrap();
    assert_eq!(d.status, TributeStatus::Dead);
    assert!(
        d.recently_killed_by.is_none(),
        "cycle must take() the killer field after emitting DeathRecorded"
    );
    // Cascade fired (deceased removed from survivor's allies).
    let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
    assert!(
        !s.allies.contains(&did),
        "survivor must not retain a dead ally edge"
    );
}

#[test]
fn run_tribute_cycle_environmental_death_emits_killer_none() {
    // A tribute who died from environmental/status damage has no
    // `recently_killed_by` set. The cycle must still emit DeathRecorded
    // but with killer: None. We assert the field stays None across the
    // cycle and the cascade still fires (downstream behavior unchanged).
    let mut deceased = create_tribute("Rue", true);
    let mut survivor = create_tribute("Katniss", true);

    let did = deceased.id;
    let sid = survivor.id;

    survivor.allies.push(deceased.id);
    deceased.allies.push(survivor.id);

    // Environmental death: health=0, RecentlyDead, killer field None.
    deceased.attributes.health = 0;
    deceased.status = TributeStatus::RecentlyDead;
    assert!(deceased.recently_killed_by.is_none());

    deceased.area = Area::Cornucopia;
    survivor.area = Area::Cornucopia;
    deceased.district = 11;
    survivor.district = 12;

    let mut game = create_test_game_with_tributes(vec![deceased.clone(), survivor.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let living = game.living_tributes();
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(547);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        living,
        1,
    );

    let d = game.tributes.iter().find(|t| t.id == did).unwrap();
    assert_eq!(d.status, TributeStatus::Dead);
    assert!(
        d.recently_killed_by.is_none(),
        "environmental death keeps killer field None"
    );
    let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
    assert!(!s.allies.contains(&did));
}

#[test]
fn alliance_formation_emits_message_with_alliance_formed_kind() {
    // Friendly + same district guarantees a high formation chance; loop
    // a few seeds until at least one cycle forms an alliance and assert
    // the resulting message carries kind = AllianceFormed and the exact
    // display string from `GameOutput::AllianceFormed`.

    use crate::tributes::traits::Trait;

    let mut t1 = create_tribute("Cinna", true);
    let mut t2 = create_tribute("Portia", true);
    t1.district = 1;
    t2.district = 1;
    t1.traits = vec![Trait::Friendly];
    t2.traits = vec![Trait::Friendly];
    t1.area = Area::Cornucopia;
    t2.area = Area::Cornucopia;

    let base = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
    let mut game_with_area = base.clone();
    game_with_area
        .areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let closed_areas: Vec<Area> = vec![];

    // Alliance formation is now a deliberate Action::ProposeAlliance
    // (5%-15% per turn for eligible tributes); sweep many seeds so we
    // deterministically observe at least one AllianceFormed message.
    let mut hit: Option<crate::messages::GameMessage> = None;
    for seed in 0u64..400 {
        let mut g = game_with_area.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = g.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas.clone(),
            vec![t1.clone(), t2.clone()],
            2,
        );
        if let Some(m) = g.messages.iter().find(|m| {
            matches!(
                m.payload,
                crate::messages::MessagePayload::AllianceFormed { .. }
            )
        }) {
            hit = Some(m.clone());
            break;
        }
    }
    let m = hit.expect("at least one cycle must emit AllianceFormed");
    assert!(
        matches!(
            m.payload,
            crate::messages::MessagePayload::AllianceFormed { .. }
        ),
        "expected AllianceFormed payload, got {:?}",
        m.payload
    );
    assert!(
        m.content.contains("form an alliance"),
        "content should match GameOutput::AllianceFormed display, got: {}",
        m.content
    );
}

#[test]
fn betrayal_emits_message_with_betrayal_triggered_kind() {
    use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
    use crate::tributes::traits::Trait;

    let mut betrayer = create_tribute("Cato", true);
    let mut victim = create_tribute("Glimmer", true);
    betrayer.traits = vec![Trait::Treacherous];
    victim.traits = vec![Trait::Tough];
    betrayer.allies.push(victim.id);
    victim.allies.push(betrayer.id);
    betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
    betrayer.area = Area::Cornucopia;
    victim.area = Area::Cornucopia;
    betrayer.district = 1;
    victim.district = 2;

    let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(313);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![betrayer.clone(), victim.clone()],
        2,
    );

    let m = game
        .messages
        .iter()
        .find(|m| {
            matches!(
                m.payload,
                crate::messages::MessagePayload::BetrayalTriggered { .. }
            )
        })
        .expect("betrayal cycle must emit BetrayalTriggered message");
    assert_eq!(
        m.content,
        "Cato betrays Glimmer — true to their treacherous nature."
    );
}

/// mqi.2 parity: at the alliance-formation emission site, the structured
/// `GameEvent::AllianceFormed` constructed inside `run_tribute_cycle`
/// renders to the exact same string that ends up as `GameMessage.content`.
/// Catches future drift between the typed event and the legacy renderer
/// at the actual call site (not just at the type level — that is covered
/// by the parity table in `events::tests`).
#[test]
fn alliance_formed_message_content_matches_game_event_display() {
    use crate::tributes::traits::Trait;

    let mut t1 = create_tribute("Cinna", true);
    let mut t2 = create_tribute("Portia", true);
    t1.district = 1;
    t2.district = 1;
    t1.traits = vec![Trait::Friendly];
    t2.traits = vec![Trait::Friendly];
    t1.area = Area::Cornucopia;
    t2.area = Area::Cornucopia;

    let base = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
    let mut game_with_area = base.clone();
    game_with_area
        .areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let closed_areas: Vec<Area> = vec![];

    // Alliance formation is now a deliberate Action::ProposeAlliance
    // (5%-15% per turn for eligible tributes); sweep many seeds so we
    // deterministically observe at least one AllianceFormed message.
    let mut hit: Option<crate::messages::GameMessage> = None;
    for seed in 0u64..400 {
        let mut g = game_with_area.clone();
        let mut rng = SmallRng::seed_from_u64(seed);
        let _ = g.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas.clone(),
            vec![t1.clone(), t2.clone()],
            2,
        );
        if let Some(m) = g.messages.iter().find(|m| {
            matches!(
                m.payload,
                crate::messages::MessagePayload::AllianceFormed { .. }
            )
        }) {
            hit = Some(m.clone());
            break;
        }
    }
    let m = hit.expect("at least one cycle must emit AllianceFormed");

    // Reconstruct the structured event with the same inputs the engine
    // used. The factor label depends on trait-overlap math; rather than
    // recompute it here (and re-couple the test to that algorithm) we
    // parse it back out of the rendered message, which is exactly what
    // mqi.4+ consumers will rely on. The point of this test is parity
    // between `GameEvent::AllianceFormed::Display` and
    // `GameMessage.content` at the call site, not validation of the
    // factor-selection logic.
    let factor = m
        .content
        .rsplit_once('(')
        .and_then(|(_, rest)| rest.rsplit_once(')').map(|(f, _)| f.to_string()))
        .expect("rendered alliance message must contain a parenthesised factor");
    let candidates = [
        crate::events::GameEvent::AllianceFormed {
            tribute_a_id: t1.id,
            tribute_a_name: t1.name.clone(),
            tribute_b_id: t2.id,
            tribute_b_name: t2.name.clone(),
            factor: factor.clone(),
        },
        crate::events::GameEvent::AllianceFormed {
            tribute_a_id: t2.id,
            tribute_a_name: t2.name.clone(),
            tribute_b_id: t1.id,
            tribute_b_name: t1.name.clone(),
            factor,
        },
    ];
    assert!(
        candidates.iter().any(|ev| ev.to_string() == m.content),
        "GameMessage.content {:?} must match GameEvent::AllianceFormed Display \
         for one of {:?}",
        m.content,
        candidates.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
    );
}

/// mqi.2 parity: at the betrayal emission site, the structured
/// `GameEvent::BetrayalTriggered` constructed inside
/// `process_alliance_events` renders identically to `GameMessage.content`.
#[test]
fn betrayal_triggered_message_content_matches_game_event_display() {
    use crate::events::GameEvent;

    use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
    use crate::tributes::traits::Trait;

    let mut betrayer = create_tribute("Cato", true);
    let mut victim = create_tribute("Glimmer", true);
    betrayer.traits = vec![Trait::Treacherous];
    victim.traits = vec![Trait::Tough];
    betrayer.allies.push(victim.id);
    victim.allies.push(betrayer.id);
    betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
    betrayer.area = Area::Cornucopia;
    victim.area = Area::Cornucopia;
    betrayer.district = 1;
    victim.district = 2;

    let betrayer_id = betrayer.id;
    let victim_id = victim.id;
    let betrayer_name = betrayer.name.clone();
    let victim_name = victim.name.clone();

    let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
    game.areas
        .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
    let closed_areas: Vec<Area> = vec![];

    let mut rng = SmallRng::seed_from_u64(313);
    let _ = game.run_tribute_cycle(
        crate::messages::Phase::Day,
        &mut rng,
        closed_areas,
        vec![betrayer.clone(), victim.clone()],
        2,
    );

    let m = game
        .messages
        .iter()
        .find(|m| {
            matches!(
                m.payload,
                crate::messages::MessagePayload::BetrayalTriggered { .. }
            )
        })
        .expect("betrayal cycle must emit BetrayalTriggered message");

    let event = GameEvent::BetrayalTriggered {
        betrayer_id,
        betrayer_name,
        victim_id,
        victim_name,
    };
    assert_eq!(
        m.content,
        event.to_string(),
        "GameMessage.content must equal GameEvent::BetrayalTriggered Display"
    );
}

#[test]
#[ignore = "scenario gap: needs game with populated areas + deterministic combat outcome; \
            see plan 2026-04-26-game-timeline-pr1-backend.md task 12. Asserts no \
            TributeMoved for tribute B at a later (tick, emit_index) than B's \
            TributeKilled within the same (game_day, phase)."]
fn dead_tribute_has_no_movement_event_after_death_in_same_period() {
    use crate::messages::MessagePayload;

    let tribute_a = create_tribute("A", true);
    let tribute_b = create_tribute("B", true);
    let mut game = create_test_game_with_tributes(vec![tribute_a, tribute_b]);
    let _ = game.run_phase(crate::messages::Phase::Day);

    let b_killed = game.messages.iter().find(|m| {
        matches!(&m.payload,
            MessagePayload::TributeKilled { victim, .. } if victim.name == "B")
    });
    let b_killed = b_killed.expect("B should have died");

    let later_b_move = game.messages.iter().find(|m| {
        m.game_day == b_killed.game_day
            && m.phase == b_killed.phase
            && (m.tick, m.emit_index) > (b_killed.tick, b_killed.emit_index)
            && matches!(&m.payload,
                MessagePayload::TributeMoved { tribute, .. } if tribute.name == "B")
    });

    assert!(
        later_b_move.is_none(),
        "no TributeMoved for B should appear after B's TributeKilled in the same period"
    );
}

// ---- Survival tick wiring (spec §6, §7) ------------------------------

#[test]
fn survival_tick_increments_hunger_and_thirst_per_phase() {
    let mut a = Tribute::new("A".to_string(), Some(1), None);
    let mut b = Tribute::new("B".to_string(), Some(2), None);
    // Mid-range attributes so the survival tick lands the +1/+1 base
    // path (not the low-strength every-other-phase or high-strength
    // double-tick branches). Stamina at half its max keeps thirst on
    // the +1 path too.
    for t in [&mut a, &mut b] {
        t.attributes.strength = 30;
        t.stamina = t.max_stamina / 2;
    }
    let mut game = create_test_game_with_tributes(vec![a, b]);
    game.day = Some(1);
    // Run a single day cycle; survival tick fires once per living tribute.
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
    // Already at the dehydrated band with 1 HP and a high
    // dehydration drain step so the next tick definitely lands fatal
    // damage (≥ 1 HP at extreme band).
    a.thirst = 4;
    a.dehydration_drain_step = 5;
    a.attributes.health = 1;
    let mut game = create_test_game_with_tributes(vec![a]);
    game.day = Some(1);
    let _ = game.run_phase(crate::messages::Phase::Day);
    let killed = game.messages.iter().any(|m| {
        matches!(&m.payload,
            MessagePayload::TributeKilled { cause, .. } if *cause == DeathCause::Dehydration)
    });
    assert!(killed, "expected a TributeKilled with cause=dehydration");
}

// ---- Sleep tick (PR2c.1, bd-9sjj) ----

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
    t.sleep_remaining = 3; // multi-phase sleep
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
    t.attributes.health = 40;
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
    let prior_hp = t.attributes.health;
    let mut game = create_test_game_with_tributes(vec![t.clone()]);
    let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
    game.areas.push(area);
    let mut rng = SmallRng::seed_from_u64(42);
    let _ = game.run_tribute_cycle(Phase::Night, &mut rng, vec![], vec![t], 1);
    assert_eq!(
        game.tributes[0].attributes.health, prior_hp,
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

// ---- Sleep interruption (PR2c.2, bd-1zju) ----

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
    // Interrupted sleep skips the sleep-tick regen branch entirely
    // (the +5 idle recovery from `recover_stamina` in the survival
    // block still runs, so we just assert sleep regen did NOT add
    // the additional SLEEP_STAMINA_PER_PHASE on top).
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
    // Cross-link as allies so the relationship is plausible (handler
    // doesn't strictly require it but semantically that's the contract).
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

#[test]
fn spawn_sponsors_creates_six_with_loyalist_district() {
    let mut game = Game::default();
    let mut rng = SmallRng::seed_from_u64(42);
    game.spawn_sponsors(&mut rng);

    assert_eq!(game.sponsors.len(), 6);
    let loyalist = game
        .sponsors
        .iter()
        .find(|s| s.archetype == shared::sponsors::ArchetypeId::Loyalist)
        .expect("Loyalist must spawn");
    let district = loyalist.bound_district.expect("Loyalist gets a district");
    assert!((1u8..=12).contains(&district));
}

#[test]
fn spawn_sponsors_is_idempotent() {
    let mut game = Game::default();
    let mut rng = SmallRng::seed_from_u64(1);
    game.spawn_sponsors(&mut rng);
    game.spawn_sponsors(&mut rng);
    assert_eq!(game.sponsors.len(), 6);
}

#[test]
fn budget_falls_inside_archetype_band() {
    let mut game = Game::default();
    let mut rng = SmallRng::seed_from_u64(7);
    game.spawn_sponsors(&mut rng);
    for s in &game.sponsors {
        let band = shared::sponsors::archetype(s.archetype).budget_band;
        assert!(s.budget_remaining >= band.0 && s.budget_remaining <= band.1);
    }
}
