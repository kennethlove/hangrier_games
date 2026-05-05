//! Round-trip test for hangrier_games-33r (PR2: movement + turn-phase slice).
//!
//! Verifies that movement and turn-phase narration emitted by
//! `Tribute::travels`, `process_turn_phase` (rest/hide/take-item/sponsor-gift,
//! exhausted-travel, suicide), now reach `Game.messages` rather than being
//! silently dropped by the old `try_log_action` no-op.

use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::messages::{GameMessage, MessageSource};
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;

/// Drop a single tribute alone in an arena (no possible combat) and run
/// several day cycles. The tribute will rest, hide, move, etc. Each of
/// these used to vanish via `try_log_action`; they should now surface as
/// `MessageSource::Tribute(_)` entries in `game.messages`.
#[test]
fn movement_and_turn_phase_events_reach_game_messages() {
    let mut game = Game::new("event-unification-movement-test");

    // Two areas so movement is actually possible.
    for (name, area) in [
        ("Cornucopia", Area::Cornucopia),
        ("North Field", Area::Sector1),
    ] {
        let details = AreaDetails::new_with_terrain(
            Some(name.to_string()),
            area,
            TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
        );
        game.areas.push(details);
    }

    // Single tribute: no targets means combat code paths can't fire and
    // any tribute-sourced messages must come from movement / rest / hide /
    // take-item / use-item / sponsor-gift / exhausted-travel paths.
    let mut lone = Tribute::random();
    lone.name = "Lone".to_string();
    lone.area = Area::Cornucopia;
    lone.attributes.health = 100;
    lone.attributes.movement = 80; // high movement → travel branch
    lone.attributes.sanity = 80;
    lone.statistics.game = game.identifier.clone();

    let lone_id = lone.identifier.clone();
    game.tributes.push(lone);

    let messages_before = game.messages.len();

    // Run several cycles to give the brain plenty of opportunities to
    // pick non-attack actions.
    for _ in 0..8 {
        game.run_phase(shared::messages::Phase::Day)
            .expect("day cycle ran");
    }

    let new_messages: Vec<&GameMessage> = game.messages.iter().skip(messages_before).collect();

    let tribute_sourced: Vec<&&GameMessage> = new_messages
        .iter()
        .filter(|m| matches!(m.source, MessageSource::Tribute(_)))
        .collect();

    assert!(
        !tribute_sourced.is_empty(),
        "expected at least one MessageSource::Tribute(_) entry from \
         movement/turn-phase narration; sources observed: {:?}",
        new_messages.iter().map(|m| &m.source).collect::<Vec<_>>()
    );

    // Every tribute-sourced message must carry our lone tribute's identifier.
    for msg in &tribute_sourced {
        match &msg.source {
            MessageSource::Tribute(id) => {
                assert_eq!(
                    id, &lone_id,
                    "unexpected tribute identifier in message source: {id}"
                );
            }
            other => panic!("filter let through non-Tribute source: {other:?}"),
        }
    }
}
