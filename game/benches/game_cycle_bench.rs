use criterion::{Criterion, criterion_group, criterion_main};
use game::games::Game;
use game::tributes::Tribute;
use game::tributes::statuses::TributeStatus;
use std::hint::black_box;

fn create_test_game(tribute_count: usize) -> Game {
    let mut game = Game::new("bench-game");
    let _ = game.start();

    for i in 0..tribute_count {
        let mut tribute = Tribute::new(format!("Tribute {}", i), None, None);
        tribute.attributes.health = 100;
        tribute.status = TributeStatus::Healthy;
        game.tributes.push(tribute);
    }

    game
}

fn bench_living_tributes_full(c: &mut Criterion) {
    let game = create_test_game(24);

    c.bench_function("living_tributes_count (24 alive)", |b| {
        b.iter(|| black_box(game.living_tributes_count()))
    });
}

fn bench_living_tributes_half(c: &mut Criterion) {
    let mut game = create_test_game(24);

    // Kill half the tributes
    for i in 0..12 {
        game.tributes[i].attributes.health = 0;
        game.tributes[i].status = TributeStatus::Dead;
    }

    c.bench_function("living_tributes_count (12 alive)", |b| {
        b.iter(|| black_box(game.living_tributes_count()))
    });
}

fn bench_living_tributes_few(c: &mut Criterion) {
    let mut game = create_test_game(24);

    // Kill all but 2 tributes
    for i in 0..22 {
        game.tributes[i].attributes.health = 0;
        game.tributes[i].status = TributeStatus::Dead;
    }

    c.bench_function("living_tributes_count (2 alive)", |b| {
        b.iter(|| black_box(game.living_tributes_count()))
    });
}

criterion_group!(
    benches,
    bench_living_tributes_full,
    bench_living_tributes_half,
    bench_living_tributes_few
);
criterion_main!(benches);
