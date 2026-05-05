//! Round-trip test for hangrier_games-33r (PR1: combat slice).
//!
//! Verifies that combat events emitted by `Tribute::attacks`,
//! `attack_contest`, and `apply_combat_results` are now collected into
//! `Game.messages` (rather than silently dropped by the old
//! `try_log_action` no-op), with `MessageSource::Tribute(identifier)`.

use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::messages::MessageSource;
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;

/// Pin two tributes in the same area, force one to be obviously stronger,
/// run a single day cycle, and assert combat narration reaches `game.messages`
/// tagged to the attacker via `MessageSource::Tribute(identifier)`.
#[test]
fn combat_events_reach_game_messages_with_tribute_source() {
    let mut game = Game::new("event-unification-combat-test");

    // Single-area arena keeps both tributes guaranteed-adjacent.
    let area_details = AreaDetails::new_with_terrain(
        Some("Cornucopia".to_string()),
        Area::Cornucopia,
        TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
    );
    game.areas.push(area_details);
    let area = game.areas[0].area.unwrap();

    // Strong attacker, frail defender — combat is overwhelmingly likely.
    let mut attacker = Tribute::random();
    attacker.name = "Attacker".to_string();
    attacker.area = area;
    attacker.attributes.health = 100;
    attacker.attributes.strength = 60;
    attacker.attributes.sanity = 80;
    attacker.statistics.game = game.identifier.clone();

    let mut defender = Tribute::random();
    defender.name = "Defender".to_string();
    defender.area = area;
    defender.attributes.health = 5;
    defender.attributes.defense = 0;
    defender.statistics.game = game.identifier.clone();
    // Different district so attacker classifies them as an enemy.
    defender.district = (attacker.district % 12) + 1;

    let attacker_id = attacker.identifier.clone();
    let defender_id = defender.identifier.clone();

    game.tributes.push(attacker);
    game.tributes.push(defender);

    let messages_before = game.messages.len();

    // Run several cycles to give combat plenty of opportunity to fire,
    // even if the brain occasionally picks Rest/Hide/Move.
    for _ in 0..6 {
        game.run_phase(shared::messages::Phase::Day)
            .expect("day cycle ran");
        if game.messages.iter().any(is_tribute_sourced) {
            break;
        }
    }

    let new_messages: Vec<_> = game.messages.iter().skip(messages_before).collect();

    assert!(
        !new_messages.is_empty(),
        "expected at least one game message after running cycles"
    );

    let tribute_sourced: Vec<_> = new_messages
        .iter()
        .filter(|m| is_tribute_sourced(m))
        .collect();

    assert!(
        !tribute_sourced.is_empty(),
        "expected at least one MessageSource::Tribute(_) entry, got sources: {:?}",
        new_messages.iter().map(|m| &m.source).collect::<Vec<_>>()
    );

    // Every tribute-sourced message should carry an identifier matching
    // one of our two tributes (no stray identifiers).
    for msg in &tribute_sourced {
        match &msg.source {
            MessageSource::Tribute(id) => {
                assert!(
                    id == &attacker_id || id == &defender_id,
                    "unexpected tribute identifier in message source: {id}"
                );
            }
            other => panic!("filter let through non-Tribute source: {other:?}"),
        }
    }
}

fn is_tribute_sourced(m: &game::messages::GameMessage) -> bool {
    matches!(m.source, MessageSource::Tribute(_))
}
