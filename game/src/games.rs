use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::items::OwnsItems;
use crate::items::Item;
use crate::messages::GameMessage;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Index;
use std::str::FromStr;
use uuid::Uuid;

thread_local!(pub static GAME: RefCell<Game> = RefCell::new(Game::default()));

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Game {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    #[serde(default)]
    pub areas: Vec<AreaDetails>,
    #[serde(default)]
    pub tribute_count: u32,
    #[serde(default)]
    pub tributes: Vec<Tribute>,
    #[serde(default)]
    pub ready: bool
}

impl Default for Game {
    fn default() -> Game {
        let wp_gen = witty_phrase_generator::WPGen::new();
        let mut name = String::new();
        if let Some(words) = wp_gen.with_words(3) {
            name = words.join("-").to_string();
        };

        Game {
            identifier: Uuid::new_v4().to_string(),
            name,
            status: Default::default(),
            day: None,
            areas: Vec::new(),
            tribute_count: 0,
            tributes: Vec::new(),
            ready: false
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Game {
    pub fn new(name: &str) -> Self {
        Game {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn end(&mut self) {
        self.status = GameStatus::Finished
    }

    // Runs at the start of the game
    pub fn start(&mut self) {
        self.status = GameStatus::InProgress;
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    pub fn dead_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| !t.is_alive())
            .cloned()
            .collect()
    }

    pub fn recently_dead_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.status == TributeStatus::RecentlyDead)
            .cloned()
            .collect()
    }

    pub fn winner(&self) -> Option<Tribute> {
        match self.living_tributes().len() {
            1 => Some(self.living_tributes().index(0).clone()),
            _ => None,
        }
    }

    pub fn random_area(&self) -> Option<AreaDetails> {
        self.areas.choose(&mut rand::thread_rng()).cloned()
    }
    
    pub fn random_open_area(&self) -> Option<AreaDetails> {
        self.areas.iter()
            .filter(|a| a.open)
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    pub fn run_day_night_cycle(&mut self) -> Game {
        self.day = Some(self.day.unwrap_or(0) + 1);
        let living_tributes = self.living_tributes();

        if let Some(winner) = self.winner() {
            println!("{}", GameMessage::TributeWins(winner));
            self.end();
            return self.clone();
        } else if living_tributes.is_empty() {
            println!("{}", GameMessage::NoOneWins);
            self.end();
            return self.clone();
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

        for area in self.areas.iter_mut() {
            area.events.clear();
        }

        // Run the day
        self.do_day_night_cycle(true);

        // Clean up any deaths
        self.clean_up_recent_deaths();

        println!("{}", GameMessage::GameNightStart(self.day.unwrap()));

        // Run the night
        self.do_day_night_cycle(false);

        // Clean up any deaths
        self.clean_up_recent_deaths();

        self.clone()
    }

    pub fn do_day_night_cycle(&mut self, day: bool) {
        let mut rng = rand::thread_rng();
        let day_event_frequency = 1.0 / 4.0;
        let night_event_frequency = 1.0 / 8.0;

        // TODO: Remove this
        let area = self.random_open_area();
        area.unwrap().events.push(AreaEvent::random());

        // Trigger any events for this cycle if we're past the first three days
        if self.day > Some(3) || !day {
            for area in self.areas.iter_mut() {
                if rng.gen_bool(if day {
                    day_event_frequency
                } else {
                    night_event_frequency
                }) {
                    let area_event = AreaEvent::random();
                    area.events.push(area_event);
                }
            }
        }

        if self.day == Some(3) && day {
            // TODO: add goodies to the cornucopia
            let mut area = self.areas.iter_mut()
                .find(|a| a.area == "Cornucopia".to_string())
                .expect("Cannot find Cornucopia");
            for _ in 0..=5 {
                area.add_item(Item::new_random_weapon());
                area.add_item(Item::new_random_consumable());
                area.add_item(Item::new_random_shield());
            }
        }

        if self.living_tributes().len() > 1 && self.living_tributes().len() < 6 {
            if let Some(mut area) = self.random_open_area() {
                let event = AreaEvent::random();
                area.events.push(event);
            }

            if rng.gen_bool(self.living_tributes().len() as f64 / 24.0) {
                if let Some(mut area) = self.random_open_area() {
                    let event = AreaEvent::random();
                    area.events.push(event);
                }
            }
        }

        self.tributes.shuffle(&mut rng);
        let mut updated_tributes: Vec<Tribute> = vec![];

        for mut tribute in self.tributes.clone() {
            if !rng.gen_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.events.push(TributeEvent::random());
            }

            if !tribute.is_alive() {
                tribute.status = TributeStatus::Dead;
                updated_tributes.push(tribute);
                continue;
            }

            match (self.day, day) {
                (Some(1), true) => {
                    tribute = tribute.do_day_night(Some(Action::Move(None)), Some(0.5), day, self);
                }
                (Some(3), true) => {
                    tribute.do_day_night(Some(Action::Move(Some(Area::Cornucopia))), Some(0.75), day, self);
                }
                (_, _) => {
                    tribute = tribute.do_day_night(None, None, day, self);
                }
            }
            updated_tributes.push(tribute);
        }
        
        self.tributes = updated_tributes;
    }

    pub fn clean_up_recent_deaths(&mut self) {
        for mut tribute in self.recently_dead_tributes() {
            let area = self.get_area_details_mut(tribute.area.clone());
            if let Some(mut area) = area {
                for item in tribute.items.iter() {
                    area.add_item(item.clone());
                }
            }

            tribute.dies();
        }
    }

    pub fn move_tribute(&self, tribute: &mut Tribute, area: Area) {
        tribute.area = area;
    }

    fn get_area_details(&self, area: Area) -> Option<AreaDetails> {
        self.areas.iter().cloned().find(|a| a.area == area.to_string())
    }

    fn get_area_details_mut(&mut self, area: Area) -> Option<&mut AreaDetails> {
        self.areas.iter_mut().find(|a| a.area == area.to_string())
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
