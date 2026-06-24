//! Snapshots sponsor affinity evolution given a deterministic set of
//! audience events. Regenerate with `cargo insta accept` after intentional
//! rebalances.

use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::sponsors::{SponsorContext, translate, update_affinities};
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use rand::SeedableRng;
use shared::messages::{MessagePayload, TributeRef};

fn build_test_game() -> Game {
    let mut game = Game::new("snapshot-test");
    let _ = game.start();

    let area = AreaDetails::new_with_terrain(
        Some("Arena".to_string()),
        Area::Sector1,
        TerrainType::new(BaseTerrain::Clearing, vec![]).unwrap(),
    );
    game.areas.push(area);

    for i in 0..6 {
        let mut tribute = Tribute::new(format!("Tribute{}", i), Some((i % 12) + 1), None);
        tribute.area = Area::Sector1;
        tribute.attributes.health = 100;
        tribute.statistics.game = game.identifier.clone();
        game.tributes.push(tribute);
    }

    game
}

fn tref(name: &str) -> TributeRef {
    TributeRef {
        identifier: name.into(),
        name: name.into(),
    }
}

#[test]
fn three_cycle_affinity() {
    let mut game = build_test_game();

    // Seed sponsors deterministically.
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0xDEAD_BEEF);
    game.spawn_sponsors(&mut rng);

    // Simulate three cycles worth of events deterministically.
    // Cycle 1: a kill + an alliance
    let cycle1_payloads = vec![
        MessagePayload::TributeKilled {
            victim: tref("Tribute0"),
            killer: Some(tref("Tribute1")),
            cause: shared::afflictions::DeathCause::Tribute("spear".into()),
        },
        MessagePayload::AllianceFormed {
            members: vec![tref("Tribute2"), tref("Tribute3")],
        },
    ];

    // Cycle 2: a betrayal
    let cycle2_payloads = vec![MessagePayload::BetrayalTriggered {
        betrayer: tref("Tribute2"),
        victim: tref("Tribute3"),
    }];

    // Cycle 3: another kill (environmental)
    let cycle3_payloads = vec![MessagePayload::TributeKilled {
        victim: tref("Tribute4"),
        killer: None,
        cause: shared::afflictions::DeathCause::Hazard(
            shared::afflictions::HazardKind::FallingDebris,
        ),
    }];

    for payloads in [cycle1_payloads, cycle2_payloads, cycle3_payloads] {
        let ctx = SponsorContext::new(&game);
        let mut events = Vec::new();
        for p in &payloads {
            events.extend(translate(p, &ctx));
        }
        update_affinities(&mut game, &events);
    }

    let snapshot = game.sponsor_affinity_snapshot();
    insta::assert_yaml_snapshot!(snapshot);
}
