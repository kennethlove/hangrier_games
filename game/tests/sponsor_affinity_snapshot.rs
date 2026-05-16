//! Snapshots a 3-cycle simulation with a fixed seed and asserts the resulting
//! per-(sponsor, tribute) affinity table is stable. Regenerate with
//! `cargo insta accept` after intentional rebalances.

use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::terrain::{BaseTerrain, TerrainType};
use game::tributes::Tribute;
use shared::messages::Phase;

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

#[test]
fn three_cycle_affinity() {
    let mut game = build_test_game();

    // Run 3 phases (Day → Dusk → Night) to simulate a partial day.
    for phase in [Phase::Day, Phase::Dusk, Phase::Night] {
        game.run_phase(phase).expect("phase should succeed");
    }

    let snapshot = game.sponsor_affinity_snapshot();
    insta::assert_yaml_snapshot!(snapshot);
}
