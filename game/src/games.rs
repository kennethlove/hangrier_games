use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::items::OwnsItems;
use crate::messages::{add_area_message, add_game_message, clear_messages};
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Index;
use std::str::FromStr;
use uuid::Uuid;

use crate::output::GameOutput;

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
    pub ready: bool,
    #[serde(default)]
    pub private: bool,
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
            areas: vec![],
            tribute_count: 0,
            tributes: vec![],
            ready: false,
            private: true,
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
        clear_messages().expect("Failed to clear messages");
    }

    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    fn recently_dead_tributes(&self) -> Vec<Tribute> {
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

    fn random_area(&self) -> Option<AreaDetails> {
        self.areas.choose(&mut rand::thread_rng()).cloned()
    }

    fn random_open_area(&self) -> Option<AreaDetails> {
        self.areas.iter()
            .filter(|a| a.open())
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    pub async fn run_day_night_cycle(&mut self, day: bool) -> Game {
        clear_messages().expect("Failed to clear messages for day");
        let living_tributes = self.living_tributes();

        if let Some(winner) = self.winner() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::TributeWins(winner.clone()))
            ).expect("");
            self.end();
            return self.clone();
        } else if living_tributes.is_empty() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::NoOneWins)
            ).expect("");
            self.end();
            return self.clone();
        }

        // Clear all events from the previous cycle
        for area in self.areas.iter_mut() {
            area.events.clear();
        }

        if day {
            self.day = Some(self.day.unwrap_or(0) + 1);

            // Make any announcements for the day
            match self.day {
                Some(1) => {
                    add_game_message(
                        self.identifier.as_str(),
                        format!("{}", GameOutput::FirstDayStart),
                    ).expect("");
                }
                Some(3) => {
                    add_game_message(
                        self.identifier.as_str(),
                        format!("{}", GameOutput::FeastDayStart),
                    ).expect("");
                },
                _ => {}
            }
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameDayStart(self.clone().day.unwrap())),
            ).expect("");
        } else {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameNightStart(self.day.unwrap())),
            ).expect("");
        }

        add_game_message(
            self.identifier.as_str(),
            format!("{}", GameOutput::TributesLeft(living_tributes.len() as u32))
        ).expect("");

        // Run the day
        self.do_a_cycle(day).await;

        // Clean up any deaths
        self.clean_up_recent_deaths().await;

        // match self.living_tributes().len() {
        //     0 | 1 => self.status = GameStatus::Finished,
        //     _ => self.status = GameStatus::InProgress,
        // }

        add_game_message(
            self.identifier.as_str(),
            format!("{}", GameOutput::TributesLeft(self.living_tributes().len() as u32)),
        ).expect("");

        if day {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameDayEnd(self.clone().day.unwrap())),
            ).expect("");
        } else {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameNightEnd(self.clone().day.unwrap())),
            ).expect("");
        }

        self.clone()
    }

    async fn do_a_cycle(&mut self, day: bool) {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let day_event_frequency = 1.0 / 4.0;
        let night_event_frequency = 1.0 / 8.0;

        for area in &self.areas {
            if !area.open() {
                add_area_message(
                    &area.area,
                    &self.identifier,
                    format!("{}", GameOutput::AreaClose(Area::from_str(&area.area).unwrap()))
                ).expect("");

                for event in &area.events {
                    add_area_message(
                        &area.area,
                        &self.identifier,
                        format!("{}", GameOutput::AreaEvent(event.clone(), Area::from_str(&area.area).unwrap()))
                    ).expect("");
                }
            }
        }

        if self.random_open_area().is_none() {
            let mut area = self.random_area().expect("No areas?");
            area.events.clear();
        }

        // Trigger any events for this cycle if we're past the first three days
        if self.day > Some(3) || !day {
            for area in self.areas.iter_mut() {
                if rng.gen_bool(if day {
                    day_event_frequency
                } else {
                    night_event_frequency
                }) {
                    // TODO: Announce area event?
                    let area_event = AreaEvent::random();
                    area.events.push(area_event);
                }
            }
        }

        // Feast day, refill the Cornucopia
        if self.day == Some(3) && day {
            let area = self.areas.iter_mut()
                .find(|a| a.area == *"Cornucopia")
                .expect("Cannot find Cornucopia");
            for _ in 0..=3 {
                area.add_item(Item::new_random_weapon());
                area.add_item(Item::new_random_shield());
                area.add_item(Item::new_random_consumable());
                area.add_item(Item::new_random_consumable());
            }
        }

        // When we're getting low on tributes, close more areas by spawning
        // more events.
        if self.living_tributes().len() > 1 && self.living_tributes().len() < 8 {
            if let Some(mut area) = self.random_open_area() {
                // TODO: Announce area event?
                let event = AreaEvent::random();
                area.events.push(event);
            }

            // If the tributes are really unlucky, they get two events.
            if rng.gen_bool(self.living_tributes().len() as f64 / 24.0) {
                if let Some(mut area) = self.random_open_area() {
                    // TODO: Announce area event?
                    let event = AreaEvent::random();
                    area.events.push(event.clone());
                    add_area_message(
                        area.area.as_str(),
                        &self.identifier,
                    format!("{}", GameOutput::AreaEvent(event.clone(), Area::from_str(&area.area).unwrap()))
                    ).expect("");
                }
            }
        }

        self.tributes.shuffle(&mut rng);
        let mut updated_tributes: Vec<Tribute> = vec![];

        for mut tribute in self.tributes.clone() {
            if !tribute.is_alive() {
                tribute.status = TributeStatus::Dead;
                updated_tributes.push(tribute);
                continue;
            }

            if !rng.gen_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.events.push(TributeEvent::random());
            }

            match (self.day, day) {
                (Some(1), true) => {
                    tribute = tribute.do_day_night(Some(Action::Move(None)), Some(0.5), day, self).await;
                }
                (Some(3), true) => {
                    tribute.do_day_night(Some(Action::Move(Some(Area::Cornucopia))), Some(0.75), day, self).await;
                }
                (_, _) => {
                    tribute = tribute.do_day_night(None, None, day, self).await;
                }
            }
            updated_tributes.push(tribute);
        }
        
        self.tributes = updated_tributes;
    }

    async fn clean_up_recent_deaths(&mut self) {
        for mut tribute in self.recently_dead_tributes() {
            let area = self.get_area_details_mut(tribute.area.clone());
            if let Some(area) = area {
                for item in tribute.items.iter() {
                    area.add_item(item.clone());
                }
            }

            tribute.dies();
        }
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
            "not started" => Ok(GameStatus::NotStarted),
            "notstarted" => Ok(GameStatus::NotStarted),
            "in progress" => Ok(GameStatus::InProgress),
            "inprogress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DisplayGame {
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
    pub ready: bool,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub is_mine: bool,
    pub created_by: CreatedBy
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreatedBy {
    pub username: String,
}

impl From<DisplayGame> for Game {
    fn from(display_game: DisplayGame) -> Self {
        Game {
            identifier: display_game.identifier,
            name: display_game.name,
            status: display_game.status,
            day: display_game.day,
            areas: display_game.areas,
            tribute_count: display_game.tribute_count,
            tributes: display_game.tributes,
            ready: display_game.ready,
            private: display_game.private,
        }
    }
}
