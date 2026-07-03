use super::*;

#[test]
fn test_announce_cycle_start() {
    let tribute1 = create_tribute("Tribute1", true);
    let tribute2 = create_tribute("Tribute2", true);
    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    game.day = Some(1);
    let _ = game.announce_cycle_start(crate::messages::Phase::Day);
    assert_eq!(game.messages.len(), 2);
}

#[test]
fn test_announce_cycle_end() {
    let tribute1 = create_tribute("Tribute1", true);
    let mut tribute2 = create_tribute("Tribute2", false);
    tribute2.set_status(TributeStatus::RecentlyDead);
    let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
    game.day = Some(1);
    let _ = game.announce_cycle_end(crate::messages::Phase::Day);
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

    assert_eq!(game.messages.len(), 3);
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
