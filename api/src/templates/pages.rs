use shared::{GameStatus, ListDisplayGame};

/// Aggregated game statistics for the dashboard.
pub struct GameStats {
    pub running: u32,
    pub waiting: u32,
    pub finished: u32,
    pub total: u32,
}

impl GameStats {
    pub fn from_games(games: &[ListDisplayGame]) -> Self {
        let mut running = 0;
        let mut waiting = 0;
        let mut finished = 0;
        for g in games {
            match g.status {
                GameStatus::InProgress => running += 1,
                GameStatus::NotStarted => waiting += 1,
                GameStatus::Finished => finished += 1,
            }
        }
        Self {
            running,
            waiting,
            finished,
            total: games.len() as u32,
        }
    }
}
