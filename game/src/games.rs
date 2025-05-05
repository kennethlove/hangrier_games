use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::items::OwnsItems;
use crate::messages::{add_area_message, add_game_message, clear_messages};
use crate::output::GameOutput;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use shared::GameStatus;
use std::fmt::Display;
use std::ops::Index;
use std::str::FromStr;
use uuid::Uuid;

/// Represents the current state of the game.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Game {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    #[serde(default)]
    pub areas: Vec<AreaDetails>,
    #[serde(default)]
    pub tributes: Vec<Tribute>,
}

impl Default for Game {
    /// Creates a new game with a random name.
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
            tributes: vec![],
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Game {
    /// Create a new game with a given name.
    pub fn new(name: &str) -> Self {
        Game {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Runs at the end of the game.
    pub fn end(&mut self) {
        self.status = GameStatus::Finished
    }

    /// Runs at the start of the game.
    pub fn start(&mut self) {
        self.status = GameStatus::InProgress;
        clear_messages().expect("Failed to clear messages");
    }

    /// Returns the tributes that are alive.
    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    /// Returns the tributes that are recently dead, i.e., died in the current round.
    fn recently_dead_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.status == TributeStatus::RecentlyDead)
            .cloned()
            .collect()
    }

    /// Returns the tribute that is the winner of the game if there is one.
    pub fn winner(&self) -> Option<Tribute> {
        match self.living_tributes().len() {
            1 => Some(self.living_tributes().index(0).clone()),
            _ => None,
        }
    }

    /// Returns a random area from the game.
    fn random_area(&self) -> Option<AreaDetails> {
        self.areas.choose(&mut rand::thread_rng()).cloned()
    }

    /// Returns a random open area from the game.
    fn random_open_area(&self) -> Option<AreaDetails> {
        self.areas.iter()
            .filter(|a| a.is_open())
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    /// Checks if the game has concluded (i.e., if there is a winner or if all tributes are dead).
    /// If concluded, it updates the game status, posts the final messages, and returns the game.
    fn check_game_state(&mut self) {
        if let Some(winner) = self.winner() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::TributeWins(winner.clone()))
            ).expect("Failed to add winner message");
            self.end();
        } else if self.living_tributes().is_empty() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::NoOneWins)
            ).expect("Failed to add no winner message");
            self.end();
        }
    }

    /// Prepares the game state for a new cycle.
    /// Clears old messages and area events.
    /// Increments day count by 1 if it's a day cycle.
    fn prepare_cycle(&mut self, day: bool) {
        clear_messages().expect("Failed to clear messages for day");

        // Clear all events from the previous cycle
        for area in self.areas.iter_mut() {
            area.events.clear();
        }

        if day {
            self.day = Some(self.day.unwrap_or(0) + 1);
        }
    }

    /// Announces the start of the cycle.
    fn announce_cycle_start(&self, day: bool) {
        let current_day = self.day.unwrap_or(1);

        if day {
            // Make any announcements for the day
            match current_day {
                1 => {
                    add_game_message(
                        self.identifier.as_str(),
                        format!("{}", GameOutput::FirstDayStart),
                    ).expect("Failed to add first day message");
                }
                3 => {
                    add_game_message(
                        self.identifier.as_str(),
                        format!("{}", GameOutput::FeastDayStart),
                    ).expect("Failed to add feast day message");
                },
                _ => {}
            }
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameDayStart(self.clone().day.unwrap())),
            ).expect("Failed to add day start message");
        } else {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameNightStart(self.day.unwrap())),
            ).expect("Failed to add night start message");
        }

        add_game_message(
            self.identifier.as_str(),
            format!("{}", GameOutput::TributesLeft(self.living_tributes().len() as u32))
        ).expect("");

    }

    /// Announces the end of a cycle
    fn announce_cycle_end(&self, day: bool) {
        add_game_message(
            self.identifier.as_str(),
            format!("{}", GameOutput::TributesLeft(self.living_tributes().len() as u32)),
        ).expect("Failed to add tributes left message");

        // Announce tribute deaths
        for tribute in self.recently_dead_tributes() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::TributeDeath(tribute.clone())),
            ).expect("Failed to add tribute death message");
        }

        if day {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameDayEnd(self.clone().day.unwrap())),
            ).expect("Failed to add day end message");
        } else {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::GameNightEnd(self.clone().day.unwrap())),
            ).expect("Failed to add night end message");
        }
    }

    /// Runs the day and night cycles of one game round.
    pub async fn run_day_night_cycle(&mut self, day: bool) {
        // Check if the game is over, and if so, end it.
        // This will also post the final messages.
        self.check_game_state();

        // Prepare the game for a new cycle
        self.prepare_cycle(day);

        // Announce the start of the cycle
        self.announce_cycle_start(day);

        // Run the day
        self.do_a_cycle(day).await;

        // Announce the end of the cycle
        self.announce_cycle_end(day);

        // Clean up any deaths
        self.clean_up_recent_deaths().await;
    }

    /// Runs a cycle of the game, either day or night.
    /// 1. Announce area events.
    /// 2. Open an area if there are no open areas.
    /// 3. Trigger any events for this cycle if we're past the first three days.
    /// 4. Trigger Feast Day events.
    /// 5. Close more areas by spawning more events if the tributes are getting low.
    /// 6. Shuffle the tributes.
    /// 6a. If the tribute is unlucky, they get a random event.
    /// 6b. Trigger day or night cycles for the tribute.
    /// 7. Update the tributes in the game.
    async fn do_a_cycle(&mut self, day: bool) {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let day_event_frequency = 1.0 / 4.0;
        let night_event_frequency = 1.0 / 8.0;

        // Announce area events
        for area in &self.areas {
            if !area.is_open() {
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

        // If there are no open areas, we need to open one.
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

        // Day 3 is Feast Day, refill the Cornucopia
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
        // more events. This will also constrain the tributes to fewer areas.
        if self.living_tributes().len() > 1 && self.living_tributes().len() < 8 {
            if let Some(mut area) = self.random_open_area() {
                // TODO: Announce area event?
                let event = AreaEvent::random();
                area.events.push(event);
            }

            // If the tributes are really unlucky, they get a second event.
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

        // Shuffle the tributes
        self.tributes.shuffle(&mut rng);
        let mut updated_tributes: Vec<Tribute> = vec![];

        for mut tribute in self.tributes.clone() {
            // Non-alive tributes should be skipped.
            if !tribute.is_alive() {
                tribute.status = TributeStatus::Dead;
                updated_tributes.push(tribute);
                continue;
            }

            // If the tribute is unlucky, they get a random event.
            if !rng.gen_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.events.push(TributeEvent::random());
            }

            // Trigger day or night cycles for the tribute
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

        // Update the tributes in the game
        self.tributes = updated_tributes;
    }

    /// Any tributes who have died in the current cycle will be moved to the "dead" list,
    /// and their items will be added to the area they died in.
    async fn clean_up_recent_deaths(&mut self) {
        let tribute_count = self.tributes.len();

        for i in 0..tribute_count {  // Using a for loop to avoid mutable borrow issues
            if self.tributes[i].is_alive() { continue }
            let tribute_items: Vec<Item> = self.tributes[i].items.clone();

            if self.tributes[i].status == TributeStatus::RecentlyDead {
                let tribute_area = self.tributes[i].area.clone();

                if let Some(area) = self.get_area_details_mut(tribute_area) {
                    for item in tribute_items {
                        area.add_item(item.clone());
                    }
                }
            }

            self.tributes[i].dies();
        }
    }

    /// Get a mutable reference to the area details for a given area.
    fn get_area_details_mut(&mut self, area: Area) -> Option<&mut AreaDetails> {
        self.areas.iter_mut().find(|a| a.area == area.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_game_with_tributes(tributes: Vec<Tribute>) -> Game {
        Game {
            identifier: "test-game".to_string(),
            name: "Test Game".to_string(),
            status: GameStatus::InProgress,
            day: Some(1),
            areas: vec![],
            tributes,
        }
    }

    fn create_tribute(name: &str, is_alive: bool) -> Tribute {
        let mut tribute = Tribute::new(name.to_string(), None, None);
        if is_alive {
            tribute.attributes.health = 100;
            tribute.status = TributeStatus::Healthy;
        } else {
            tribute.attributes.health = 0;
            tribute.status = TributeStatus::Dead;
        }
        tribute
    }

    #[test]
    fn test_game_new() {
        let game = Game::new("Test Game");
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.status, GameStatus::NotStarted);
        assert_eq!(game.day, None);
        assert_eq!(game.tributes.len(), 0);
    }

    #[test]
    fn test_game_start() {
        let mut game = Game::new("Test Game");
        game.start();
        assert_eq!(game.status, GameStatus::InProgress);
        assert_eq!(game.day, None);
    }

    #[test]
    fn test_game_end() {
        let mut game = Game::new("Test Game");
        game.start();
        game.end();
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_living_and_recently_dead_tributes() {
        let mut game = Game::new("Test Game");
        let t1 = Tribute::default();
        let t2 = Tribute::default();
        game.tributes.push(t1);
        game.tributes.push(t2);
        assert_eq!(game.living_tributes().len(), 2);
        assert_eq!(game.recently_dead_tributes().len(), 0);
        game.tributes[0].status = TributeStatus::RecentlyDead;
        assert_eq!(game.living_tributes().len(), 1);
        assert_eq!(game.recently_dead_tributes().len(), 1);
    }

    #[test]
    fn test_game_winner() {
        let mut game = Game::new("Test Game");
        let t1 = Tribute::default();
        let t2 = Tribute::default();
        game.tributes.push(t1);
        game.tributes.push(t2.clone());
        game.start();
        assert_eq!(game.winner(), None);
        game.tributes[0].status = TributeStatus::Dead;
        assert_eq!(game.winner().unwrap().name, t2.name);
    }

    #[test]
    fn test_random_open_area() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::North);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::South);
        game.areas.push(area1);
        game.areas.push(area2.clone());
        assert!(game.random_area().is_some());
        let event = AreaEvent::random();
        game.areas[0].events.push(event.clone());
        assert_eq!(game.random_open_area().unwrap(), area2);
    }

    #[tokio::test]
    async fn test_clean_up_recent_deaths() {
        let mut game = Game::new("Test Game");

        let mut tribute = Tribute::default();
        tribute.set_status(TributeStatus::RecentlyDead);
        game.tributes.push(tribute.clone());

        assert_eq!(game.recently_dead_tributes().len(), 1);
        assert_eq!(game.recently_dead_tributes()[0], tribute);

        game.clean_up_recent_deaths().await;
        assert_eq!(game.tributes[0].status, TributeStatus::Dead);
    }

    #[test]
    fn test_check_game_state_winner_exists() {
        let winner_tribute = create_tribute("Winner", true);
        let loser_tribute = create_tribute("Loser", false);
        let mut game = create_test_game_with_tributes(vec![winner_tribute.clone(), loser_tribute.clone()]);

        // Game should have only one living tribute and they should be the winner
        assert_eq!(game.living_tributes().len(), 1);
        assert_eq!(game.winner(), Some(winner_tribute.clone()));

        game.check_game_state();

        // Game should be finished
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_check_game_state_no_survivors() {
        let loser_tribute = create_tribute("Loser", false);
        let loser2_tribute = create_tribute("Loser 2", false);
        let mut game = create_test_game_with_tributes(vec![loser_tribute.clone(), loser2_tribute.clone()]);

        // Game should have only no living tributes and no winner
        assert!(game.living_tributes().is_empty());
        assert!(game.winner().is_none());

        game.check_game_state();

        // Game should be finished
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_check_game_state_continues() {
        let living_tribute1 = create_tribute("Living1", true);
        let living_tribute2 = create_tribute("Living2", true);
        let mut game = create_test_game_with_tributes(vec![living_tribute1.clone(), living_tribute2.clone()]);
        let starting_state = game.status.clone();

        // Game should have only one living tribute and they should be the winner
        assert_eq!(game.living_tributes().len(), 2);
        assert!(game.winner().is_none());

        game.check_game_state();

        // Game should be finished
        assert_eq!(game.status, starting_state);
    }
}
