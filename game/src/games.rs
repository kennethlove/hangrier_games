use crate::areas::Area;
use crate::areas::events::AreaEvent;
use crate::messages::GameMessage;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use rand::Rng;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;
use crate::items::Item;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Game {
    pub id: Uuid,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    pub areas: Vec<Area>,
}

impl Default for Game {
    fn default() -> Game {
        let wpgen = witty_phrase_generator::WPGen::new();
        let mut name = String::new();
        if let Some(words) = wpgen.with_words(3) {
            name = words.join("-").to_string();
        };

        let mut cornucopia = Area::new("The Cornucopia");
        let mut nw = Area::new("Northwest");
        let mut ne = Area::new("Northeast");
        let mut sw = Area::new("Southwest");
        let mut se = Area::new("Southeast");

        cornucopia.add_neighbors(vec![&nw, &ne, &sw, &se]);
        nw.add_neighbors(vec![&ne, &sw, &cornucopia]);
        ne.add_neighbors(vec![&nw, &se, &cornucopia]);
        sw.add_neighbors(vec![&nw, &se, &cornucopia]);
        se.add_neighbors(vec![&ne, &sw, &cornucopia]);

        let areas = vec![cornucopia, nw, ne, sw, se];

        Game {
            id: Uuid::new_v4(),
            name,
            status: Default::default(),
            day: None,
            areas,
        }
    }
}

impl Game {
    pub fn new(name: &str) -> Self {
        let mut game = Game::default();
        game.name = name.to_string();
        game
    }

    pub fn where_am_i(&self, tribute: &Tribute) -> Option<Area> {
        for area in self.areas.iter() {
            if area.tributes.contains(tribute) {
                return Some(area.clone());
            }
        }
        None
    }

    pub fn end(&mut self) {
        self.status = GameStatus::Finished
    }

    // Runs at the start of the game
    pub fn start(&mut self) {
        // TODO: add items to the cornucopia

        // TODO: add tributes

        self.status = GameStatus::InProgress;
    }

    pub fn add_tribute(&self, mut tribute: Tribute) {
        let cornucopias = self
            .areas
            .iter()
            .filter(|a| a.name == "The Cornucopia")
            .collect::<Vec<&Area>>();
        let mut cornucopia = cornucopias.first().cloned().unwrap().clone();
        cornucopia.add_tribute(tribute.clone());
        tribute.game = Some(self.clone());
    }

    pub fn tributes(&self) -> Vec<Tribute> {
        let mut tributes: Vec<Tribute> = vec![];
        for area in &self.areas {
            let tribs = area.tributes.clone();
            tributes.extend_from_slice(tribs.as_slice());
        }
        tributes
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes()
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    pub fn dead_tributes(&self) -> Vec<Tribute> {
        self.tributes()
            .iter()
            .filter(|t| !t.is_alive())
            .cloned()
            .collect()
    }

    pub fn recently_dead_tributes(&self) -> Vec<Tribute> {
        self.tributes()
            .iter()
            .filter(|t| t.status == TributeStatus::RecentlyDead)
            .cloned()
            .collect()
    }

    pub fn winner(&self) -> Option<Tribute> {
        match self.living_tributes().len() {
            1 => Some(self.living_tributes()[0].clone()),
            _ => None,
        }
    }

    pub fn run_day_night_cycle(&mut self) {
        self.day = Some(self.day.unwrap_or(0) + 1);
        let living_tributes = self.living_tributes();

        if let Some(winner) = self.winner() {
            println!("{}", GameMessage::TributeWins(winner));
            self.end();
            return;
        } else if living_tributes.len() == 0 {
            println!("{}", GameMessage::NoOneWins);
            self.end();
            return;
        }

        // Make any announcements for the day
        match self.day {
            Some(1) => {
                println!("{}", GameMessage::FirstDayStart);
            }
            Some(3) => {
                println!("{}", GameMessage::FeastDayStart);
            }
            _ => {
                println!("{}", GameMessage::GameDayStart(self.day.unwrap()));
            }
        }

        println!(
            "{}",
            GameMessage::TributesLeft(living_tributes.len() as u32)
        );

        // Run the day
        self.do_day_night_cycle(true);

        // Clean up any deaths
        self.clean_up_recent_deaths();

        println!("{}", GameMessage::GameNightStart(self.day.unwrap()));

        // Run the night
        self.do_day_night_cycle(false);

        // Clean up any deaths
        self.clean_up_recent_deaths();
    }

    pub fn do_day_night_cycle(&mut self, day: bool) {
        let mut rng = rand::thread_rng();
        let day_event_frequency = 1.0 / 4.0;
        let night_event_frequency = 1.0 / 8.0;

        // Trigger any events for this cycle if we're past the first three days
        if self.day > Some(3) || !day {
            for _ in &self.areas {
                if rng.gen_bool(if day {
                    day_event_frequency
                } else {
                    night_event_frequency
                }) {
                    AreaEvent::random();
                }
            }
        }

        if self.day == Some(3) && day {
            // TODO: add goodies to the cornucopia
            let area = self.areas.iter_mut().find(|a| a.name == "The Cornucopia").unwrap();
            for _ in 0..=5 {
                area.add_item(Item::new_random_weapon());
                area.add_item(Item::new_random_consumable());
                area.add_item(Item::new_random_shield());
            }
        }

        if self.living_tributes().len() > 1 && self.living_tributes().len() < 6 {
            AreaEvent::random();

            if rng.gen_bool(self.living_tributes().len() as f64 / 24.0) {
                AreaEvent::random();
            }
        }

        self.living_tributes().shuffle(&mut rng);

        for mut tribute in self.living_tributes() {
            if !rng.gen_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.handle_event(TributeEvent::random());
            }

            if !tribute.is_alive() {
                tribute.status = TributeStatus::RecentlyDead;
                continue;
            }

            match (self.day, day) {
                (Some(1), true) => {
                    tribute.do_day_night(Some(Action::Move(None)), Some(0.5), day);
                }
                (Some(3), true) => {
                    let cornucopia: Option<Area> = self
                        .areas
                        .iter()
                        .filter(|a| a.name == "cornucopia")
                        .cloned()
                        .collect::<Vec<Area>>()
                        .first()
                        .cloned();
                    tribute.do_day_night(Some(Action::Move(cornucopia)), Some(0.75), day);
                }
                (_, _) => {
                    tribute.do_day_night(None, None, day);
                }
            }
        }
    }

    pub fn clean_up_recent_deaths(&self) {
        for mut tribute in self.recently_dead_tributes() {
            tribute.dies();
        }
    }

    pub fn move_tribute(&self, tribute: &Tribute, mut area: Area) {
        let mut current_area = self.where_am_i(tribute).unwrap().clone();
        current_area.remove_tribute(tribute);
        area.add_tribute(tribute.clone());
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum GameStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::NotStarted => write!(f, "not started"),
            GameStatus::InProgress => write!(f, "in progress"),
            GameStatus::Finished => write!(f, "finished"),
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
