use rand::prelude::SliceRandom;
use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;
use crate::areas::Area;
use crate::items::Item;
use crate::tributes::events::TributeEvent;
use crate::tributes::actions::TributeAction;
use crate::tributes::Tribute;
use crate::tributes::statuses::TributeStatus;
use crate::messages::GameMessage;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    pub id: Option<i32>,
    pub name: String,
    pub day: Option<i32>,
    pub closed_areas: Option<Vec<Area>>,
    pub status: GameStatus,
}

impl Game {
    pub fn new(game_name: &str) -> Game {
        todo!();
        Game::from(create_game(Some(game_name)))
    }

    pub fn delete(game_id: i32) {
        todo!();
        delete_game_logs(game_id);
        delete_game_area_events(game_id);
        delete_game_items(game_id);
        delete_game_tribute_actions(game_id);
        delete_game_tributes(game_id);
        delete_game(game_id);
    }

    pub fn as_str(&self) -> &str {
        todo!();
        self.name.as_str()
    }

    pub fn default() -> Game {
        todo!();
        Game {
            id: None,
            name: "".to_string(),
            day: Some(0),
            closed_areas: None,
            status: GameStatus::NotStarted,
        }
    }

    pub fn end(&self) {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        game.end();
    }

    // Runs at the start of the game
    pub fn start(&self) {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        let the_cornucopia = Area::from_str("cornucopia").expect("Error loading area");
        for _ in 0..10 {
            Item::new_random_weapon(
                Some(game.id),
                Some(the_cornucopia.id()),
                None
            );
            Item::new_generic_consumable(
                Some(game.id),
                Some(the_cornucopia.id()),
                None
            );
        }
    }

    pub fn tributes(&self) -> Vec<Tribute> {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        game.tributes().iter().map(|t| Tribute::from(t.clone())).collect()
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        get_all_living_tributes(&game).iter().map(|t| Tribute::from(t.clone())).collect()
    }

    pub fn dead_tributes(&self) -> Vec<Tribute> {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        get_dead_tributes(&game).iter().map(|t| Tribute::from(t.clone())).collect()
    }

    pub fn winner(&self) -> Option<Tribute> {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        let winner = get_all_living_tributes(&game);
        if winner.len() == 1 {
            Some(Tribute::from(winner[0].clone()))
        } else {
            None
        }
    }

    pub fn add_tribute(&self, name: String, avatar: Option<String>) -> Result<Tribute, ()> {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        let mut tribute = create_tribute(name.as_str(), avatar);
        tribute.set_game(&game);
        tribute.game_id = Some(game.id);

        Ok(Tribute::from(tribute))
    }

    pub fn run_day_night_cycle(&mut self) {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        self.day = Some(self.day.unwrap_or(0) + 1);
        game.set_day(self.day.unwrap());

        // Get all the living tributes
        let living_tributes = get_all_living_tributes(&game);

        // See if we have a winner or a dud game
        match living_tributes.len() {
            0 => {
                let message = GameMessage::NoOneWins;
                create_full_log(game.id, message.to_string(), None, None, None, None);
                game.end();
                return;
            }
            1 => {
                let winner = living_tributes[0].clone();
                let message = GameMessage::TributeWins(Tribute::from(winner.clone()));
                create_full_log(game.id, message.to_string(), None, Some(winner.id), None, None);
                game.end();
                return;
            }
            _ => {}
        }

        // Make any announcements for the day
        match self.day {
            Some(1) => {
                create_full_log(game.id, GameMessage::FirstDayStart.to_string(), None, None, None, None);
            }
            Some(3) => {
                create_full_log(game.id, GameMessage::FeastDayStart.to_string(), None, None, None, None);
            }
            _ => {
                create_full_log(game.id, GameMessage::GameDayStart(self.day.unwrap()).to_string(), None, None, None, None);
            }
        }

        create_full_log(game.id, GameMessage::TributesLeft(living_tributes.len() as i32).to_string(), None, None, None, None);

        // Run the day
        self.do_day_night_cycle(true);

        // Clean up any deaths
        self.clean_up_recent_deaths();

        create_full_log(game.id, GameMessage::GameNightStart(self.day.unwrap()).to_string(), None, None, None, None);

        // Run the night
        self.do_day_night_cycle(false);

        // Clean up any deaths
        self.clean_up_recent_deaths();
    }

    pub fn do_day_night_cycle(&mut self, day: bool) {
        todo!();
        let mut rng = rand::thread_rng();
        let day_event_frequency = 1.0 / 4.0;
        let night_event_frequency = 1.0 / 8.0;
        let game = get_game(self.name.as_str()).expect("Error loading game");

        // Clean up any deaths from the previous cycle's events
        Area::clean_up_area_events(self.id.unwrap());

        // Trigger any events for this cycle
        if self.day > Some(3) || !day {
            if rng.gen_bool(if day { day_event_frequency } else { night_event_frequency }) {
                Area::do_area_event(self.id.unwrap());
            }
        }

        if self.day == Some(3) && day {
            // Add goodies to the Cornucopia
            let cornucopia = Area::from_str("cornucopia").expect("Error loading area");
            let items = cornucopia.available_items(game.id);
            if items.len() <= 12 {
                let count = (12 - items.len()) / 3;
                for _ in 0..count {
                    Item::new_generic_consumable(
                        self.id,
                        Some(cornucopia.id()),
                        None
                    );
                    Item::new_random_weapon(
                        self.id,
                        Some(cornucopia.id()),
                        None
                    );
                    Item::new_random_shield(
                        self.id,
                        Some(cornucopia.id()),
                        None
                    );
                }
            }
        }

        // Get all the remaining tributes to run their appropriate actions
        let mut living_tributes = get_all_living_tributes(&game);

        // If there are too few, but not just one, tribute left, close an area or two
        if living_tributes.len() > 1 && living_tributes.len() < 7 {
            Area::do_area_event(self.id.unwrap());

            if rng.gen_bool(living_tributes.len() as f64 / 24.0) {
                Area::do_area_event(self.id.unwrap());
            }
        }

        living_tributes.shuffle(&mut rng);
        for tribute in living_tributes {
            let mut tribute = Tribute::from(tribute.clone());

            // Use luck to decide if the tribute is caught by an event
            if !rng.gen_bool(tribute.luck.unwrap_or(0) as f64 / 100.0) {
                let event = TributeEvent::random();
                tribute.handle_event(event);
            }

            // If the event killed the tribute, move on
            if !tribute.is_alive() {
                tribute.status = TributeStatus::RecentlyDead;
                continue;
            };

            match (self.day, day) {
                (Some(1), true) => {
                    tribute = tribute.do_day_night(
                        Some(TributeAction::Move(None)),
                        Some(0.5),
                        day
                    );
                }
                (Some(3), true) => {
                    // Feast day

                    // Encourage tributes to move to the Cornucopia
                    tribute = tribute.do_day_night(
                        Some(TributeAction::Move(Some(Area::Cornucopia.to_string()))),
                        Some(0.75),
                        day,
                    );
                }
                (_, _) => {
                    tribute = tribute.do_day_night(None, None, day);
                }
            };
            update_tribute(tribute.id.unwrap(), tribute.into());
        }
    }
    pub fn clean_up_recent_deaths(&self) {
        todo!();
        let game = get_game(self.name.as_str()).expect("Error loading game");
        let dead_tributes = get_recently_dead_tributes(&game);

        create_full_log(game.id, GameMessage::DailyDeathAnnouncement(dead_tributes.len() as i32).to_string(), None, None, None, None);

        for tribute in dead_tributes {
            create_full_log(game.id, GameMessage::DeathAnnouncement(Tribute::from(tribute.clone())).to_string(), None, Some(tribute.id), None, None);
            tribute.dies();
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for Game {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!();
        let game = get_game(s).expect("Error loading game");
        Ok(Game::from(game))
    }
}

impl From<String> for Game {
    fn from(s: String) -> Self {
        Self::from_str(s.as_str()).expect("Couldn't match that game")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum GameStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::NotStarted => write!(f, "Not Started"),
            GameStatus::InProgress => write!(f, "In Progress"),
            GameStatus::Finished => write!(f, "Finished"),
        }
    }
}

impl FromStr for GameStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "not started" => Ok(GameStatus::NotStarted),
            "in progress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            _ => Err(()),
        }
    }
}
