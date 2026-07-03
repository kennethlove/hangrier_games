use super::*;
use crate::tributes::Attributes;

pub(crate) fn create_test_game_with_tributes(tributes: Vec<Tribute>) -> Game {
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

pub(crate) fn create_tribute(name: &str, is_alive: bool) -> Tribute {
    let mut tribute = Tribute::new(name.to_string(), None, None);
    if is_alive {
        tribute.attributes.set_health(100);
        tribute.status = TributeStatus::Healthy;
    } else {
        tribute.attributes.set_health(0);
        tribute.status = TributeStatus::Dead;
    }
    tribute
}

mod alliances;
mod messaging;
mod survival;
