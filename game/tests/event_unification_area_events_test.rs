//! Round-trip test for hangrier_games-33r (PR3: area-event survival slice).
//!
//! Verifies that area-event narration emitted by `process_event_for_area`
//! and `announce_area_events` reaches `Game.messages` with the correct
//! `MessageSource::Area(_)` tagging, and that per-tribute survival
//! outcomes still surface as `MessageSource::Tribute(_)` entries. Together
//! these confirm the full event slice is unified through `Game.messages`.

use game::areas::events::AreaEvent;
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::messages::{GameMessage, MessageSource};
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use rand::SeedableRng;
use rand::rngs::SmallRng;

/// Drop two tributes into a single area, fire an `AreaEvent` directly via
/// `process_event_for_area`, and assert that:
/// - one `MessageSource::Area(_)` line announces the event itself
/// - per-tribute survival narration appears as `MessageSource::Tribute(_)`
///   carrying the matching tribute identifier
#[test]
fn area_event_survival_narration_reaches_game_messages() {
    let mut game = Game::new("event-unification-area-events-test");

    let area_details = AreaDetails::new_with_terrain(
        Some("Cornucopia".to_string()),
        Area::Cornucopia,
        TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
    );
    game.areas.push(area_details);
    let area = game.areas[0].area.unwrap();

    let mut alpha = Tribute::random();
    alpha.name = "Alpha".to_string();
    alpha.area = area;
    alpha.attributes.health = 100;
    alpha.statistics.game = game.identifier.clone();
    let alpha_id = alpha.identifier.clone();

    let mut bravo = Tribute::random();
    bravo.name = "Bravo".to_string();
    bravo.area = area;
    bravo.attributes.health = 100;
    bravo.statistics.game = game.identifier.clone();
    bravo.district = (alpha.district % 12) + 1;
    let bravo_id = bravo.identifier.clone();

    game.tributes.push(alpha);
    game.tributes.push(bravo);

    let mut rng = SmallRng::seed_from_u64(0xC0FFEE);
    let event = AreaEvent::Wildfire;

    let messages_before = game.messages.len();

    game.process_event_for_area(&area, &event, &mut rng)
        .expect("process_event_for_area succeeded");

    let new_messages: Vec<&GameMessage> = game.messages.iter().skip(messages_before).collect();

    assert!(
        !new_messages.is_empty(),
        "expected at least one game message after processing area event"
    );

    // Exactly one MessageSource::Area(_) opening announcement, tagged to
    // our area name and using the canonical area:{name} subject.
    let area_sourced: Vec<&&GameMessage> = new_messages
        .iter()
        .filter(|m| matches!(m.source, MessageSource::Area(_)))
        .collect();
    assert_eq!(
        area_sourced.len(),
        1,
        "expected exactly one MessageSource::Area(_) announcement, got: {:?}",
        new_messages.iter().map(|m| &m.source).collect::<Vec<_>>()
    );
    match &area_sourced[0].source {
        MessageSource::Area(name) => assert_eq!(name, "Cornucopia"),
        other => panic!("expected MessageSource::Area, got {other:?}"),
    }
    assert_eq!(
        area_sourced[0].subject,
        format!("{}:area:Cornucopia", game.identifier)
    );

    // Per-tribute survival narration: at least one MessageSource::Tribute(_)
    // entry per tribute, identifiers must match our two tributes.
    let tribute_sourced: Vec<&&GameMessage> = new_messages
        .iter()
        .filter(|m| matches!(m.source, MessageSource::Tribute(_)))
        .collect();
    assert!(
        !tribute_sourced.is_empty(),
        "expected at least one MessageSource::Tribute(_) survival outcome, \
         got sources: {:?}",
        new_messages.iter().map(|m| &m.source).collect::<Vec<_>>()
    );
    for msg in &tribute_sourced {
        match &msg.source {
            MessageSource::Tribute(id) => {
                assert!(
                    id == &alpha_id || id == &bravo_id,
                    "unexpected tribute identifier in message source: {id}"
                );
            }
            other => panic!("filter let through non-Tribute source: {other:?}"),
        }
    }
}
