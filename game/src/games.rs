use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::messages::GameMessage;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use rand::prelude::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use strum::IntoEnumIterator;

thread_local!(pub static GAME: RefCell<Game> = RefCell::new(Game::default()));

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Game {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    pub areas: BTreeMap<String, AreaDetails>,
    #[serde(default)]
    pub tribute_count: u32,
    // #[serde(skip_serializing, skip_deserializing)]
    #[serde(default)]
    pub tributes: Vec<Tribute>,
}

impl Default for Game {
    fn default() -> Game {
        let wpgen = witty_phrase_generator::WPGen::new();
        let mut name = String::new();
        if let Some(words) = wpgen.with_words(3) {
            name = words.join("-").to_string();
        };

        let mut game = Game {
            identifier: name.clone(),
            name,
            status: Default::default(),
            day: None,
            areas: BTreeMap::new(),
            tribute_count: 0,
            tributes: Vec::new(),
        };

        for area in Area::iter() {
            game.areas.insert(
                area.to_string(),
                AreaDetails::new(true, vec![])
            );
        }

        game
    }
}

impl Game {
    pub fn new(name: &str) -> Self {
        let mut game = Game::default();
        game.name = name.to_string();
        game.identifier = name.to_string();
        game
    }

    pub fn where_am_i(&self, tribute: &Tribute) -> Option<Area> {
        todo!();
        // if let Some(tribute) = self.tributes.iter().find(|t| *t == tribute) {
        //     Some(tribute.area.clone())
        // } else { None }
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

    pub fn add_tribute(&mut self, tribute: Tribute) {
        todo!();
        // self.tributes.push(tribute);
    }

    pub fn remove_tribute(&mut self, tribute: &Tribute) {
        todo!();
        // self.tributes.retain(|item| item != tribute);
    }

    pub fn add_random_tribute(&mut self) {
        todo!();
        let tribute = Tribute::random();
        self.add_tribute(tribute.clone());
    }

    pub fn shuffle_tributes(&mut self) {
        todo!();
        // let mut rng = rand::thread_rng();
        // let mut tributes = self.tributes.clone();
        // tributes.shuffle(&mut rng);
        // self.tributes = tributes;
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        todo!();
        // self.tributes
        //     .iter()
        //     .filter(|t| t.is_alive())
        //     .cloned()
        //     .collect()
    }

    pub fn dead_tributes(&self) -> Vec<Tribute> {
        todo!();
        // self.tributes
        //     .iter()
        //     .filter(|t| !t.is_alive())
        //     .cloned()
        //     .collect()
    }

    pub fn recently_dead_tributes(&self) -> Vec<Tribute> {
        todo!();
        // self.tributes
        //     .iter()
        //     .filter(|t| t.status == TributeStatus::RecentlyDead)
        //     .cloned()
        //     .collect()
    }

    pub fn winner(&self) -> Option<Tribute> {
        match self.living_tributes().len() {
            1 => Some(self.living_tributes()[0].clone()),
            _ => None,
        }
    }

    pub fn get_area(&self, name: &str) -> Option<&Area> {
        todo!();
        // self.areas.iter().find(|a| a.name() == name)
    }

    pub fn get_or_create_area(&mut self, _name: &str) -> &Area {
        todo!();
        // match self.get_area(name) {
        //     Some(area) => area,
        //     None => {
        //         let area = Area::new(name);
        //         self.areas.push(area);
        //         self.areas.last().unwrap()
        //     }
        // }
    }

    pub fn get_area_mut(&mut self, name: &str) -> Option<&mut Area> {
        todo!();
        // self.areas.iter_mut().find(|a| a.name() == name)
    }

    pub fn random_area(&self) -> Option<Area> {
        todo!();
        // self.areas.choose(&mut rand::thread_rng()).cloned()
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
            // for _ in &self.areas {
            //     if rng.gen_bool(if day {
            //         day_event_frequency
            //     } else {
            //         night_event_frequency
            //     }) {
            //         AreaEvent::random();
            //     }
            // }
        }

        if self.day == Some(3) && day {
            // TODO: add goodies to the cornucopia
            todo!();
            // let area = self.areas.iter_mut().find(|a| a.name() == "The Cornucopia").unwrap();
            // for _ in 0..=5 {
            //     area.add_item(Item::new_random_weapon());
            //     area.add_item(Item::new_random_consumable());
            //     area.add_item(Item::new_random_shield());
            // }
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
                    // let cornucopia: Option<Area> = self
                    //     .areas
                    //     .iter()
                    //     .filter(|a| a.name() == "cornucopia")
                    //     .cloned()
                    //     .collect::<Vec<Area>>()
                    //     .first()
                    //     .cloned();
                    // tribute.do_day_night(Some(Action::Move(cornucopia)), Some(0.75), day);
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

    pub fn move_tribute(&self, tribute: &mut Tribute, area: Area) {
        tribute.area = area;
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
            GameStatus::NotStarted => write!(f, "NotStarted"),
            GameStatus::InProgress => write!(f, "InProgress"),
            GameStatus::Finished => write!(f, "Finished"),
        }
    }
}

impl FromStr for GameStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "notstarted" => Ok(GameStatus::NotStarted),
            "inprogress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            _ => Err(()),
        }
    }
}
