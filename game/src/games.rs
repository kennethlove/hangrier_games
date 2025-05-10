use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::items::OwnsItems;
use crate::messages::{add_area_message, add_game_message, clear_messages};
use crate::output::GameOutput;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::{ActionSuggestion, EncounterContext, EnvironmentContext, Tribute};
use rand::prelude::SliceRandom;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use shared::GameStatus;
use std::collections::HashMap;
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
    fn random_area(&mut self) -> Option<&mut AreaDetails> {
        self.areas.choose_mut(&mut rand::thread_rng())
    }

    /// Returns a random open area from the game.
    fn random_open_area(&self) -> Option<AreaDetails> {
        self.open_areas()
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    /// Returns a vec of open areas.
    fn open_areas(&self) -> Vec<AreaDetails> {
        self.areas.iter()
            .filter(|a| a.is_open())
            .cloned()
            .collect()
    }

    /// Returns a vec of closed areas.
    #[allow(dead_code)]
    fn closed_areas(&self) -> Vec<AreaDetails> {
        self.areas.iter()
            .filter(|a| !a.is_open())
            .cloned()
            .collect()
    }

    /// Checks if the game has concluded (i.e., if there is a winner or if all tributes are dead).
    /// If concluded, it updates the game status, posts the final messages, and returns the game.
    fn check_for_winner(&mut self) {
        if let Some(winner) = self.winner() {
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::TributeWins(winner.name.as_str()))
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
            let name: &str = tribute.name.as_str();
            add_game_message(
                self.identifier.as_str(),
                format!("{}", GameOutput::DeathAnnouncement(name)),
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
        self.check_for_winner();

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

    /// Announces events in closed areas.
    fn announce_area_events(&self) {
        for area in &self.areas {
            let area_name = area.area.clone();
            if !area.is_open() {
                add_area_message(
                    &area.area,
                    &self.identifier,
                    format!("{}", GameOutput::AreaClose(area_name.as_str()))
                ).expect("Failed to add area close message");

                for event in &area.events {
                    let event_name = event.to_string();
                    add_area_message(
                        &area.area,
                        &self.identifier,
                        format!("{}", GameOutput::AreaEvent(event_name.as_str(), area_name.as_str()))
                    ).expect("Failed to add area event message");
                }
            }
        }
    }

    /// Ensures at least one area is open. If not, opens a random area by clearing its events.
    fn ensure_open_area(&mut self) {
        if self.random_open_area().is_none() {
            if let Some(area) = self.random_area() {
                area.events.clear();
            }
        }
    }

    /// Triggers events for the current cycle.
    fn trigger_cycle_events(&mut self, day: bool, rng: &mut SmallRng) {
        let frequency = {
            if day {
                1.0 / 4.0
            } else {
                1.0 / 8.0
            }
        };

        // If it's nighttime, trigger an event
        // If it is daytime and not day #1 or day #3, trigger an event
        if !day || ![1u32, 3u32].contains(&self.day.unwrap_or(1u32)) {
            for area in self.areas.iter_mut() {
                if rng.gen_bool(frequency) {
                    let area_event = AreaEvent::random();
                    area.events.push(area_event.clone());
                    let event_name = area_event.to_string();
                    let area_name = area.area.clone();
                    add_area_message(
                        area.area.as_str(),
                        &self.identifier,
                        format!("{}", GameOutput::AreaEvent(event_name.as_str(), area_name.as_str()))
                    ).expect("Failed to add area event message");
                }
            }
        }

        // Day 3 is Feast Day, refill the Cornucopia with a random assortment of items
        if day && self.day == Some(3) {
            let area = self.areas.iter_mut()
                .find(|a| a.area == *"Cornucopia")
                .expect("Cannot find Cornucopia");
            for _ in 0..rng.gen_range(1..=2) {
                area.add_item(Item::new_random_weapon());
            }
            for _ in 0..rng.gen_range(1..=2) {
                area.add_item(Item::new_random_shield());
            }
            for _ in 0..rng.gen_range(1..=4) {
                area.add_item(Item::new_random_consumable());
            }
        }

    }

    /// If the tribute count is low, constrain them by closing areas.
    /// We achieve this by spawning events in open areas.
    fn constrain_areas(&mut self, rng: &mut SmallRng) {
        let tribute_count = self.living_tributes().len() as u32;
        let odds = tribute_count as f64 / 24.0;
        let mut area_events: HashMap<String, (AreaDetails, Vec<AreaEvent>)> = HashMap::new();

        if (1u32..8u32).contains(&tribute_count) {
            // If there is an open area, close it.
            if let Some(area) = self.random_open_area() {
                let event = AreaEvent::random();
                area_events.insert(area.area.clone(), (area.clone(), vec![event.clone()]));
            }

            if rng.gen_bool(odds) {
                // Assuming there's still an open area.
                if let Some(area) = self.random_open_area() {
                    let event = AreaEvent::random();
                    let area_name = area.area.clone();
                    if area_events.get(&area_name.clone()).is_some() {
                        let mut events = area_events[&area_name.clone()].1.clone();
                        events.push(event.clone());
                        area_events.insert(area_name, (area.clone(), events));
                    } else {
                        area_events.insert(area_name, (area.clone(), vec![event.clone()]));
                    }
                }
            }

            // Add events to each area and announce them
            for (_area_name, (mut area, events)) in area_events.clone() {
                for event in events {
                    area.events.push(event.clone());
                    let event_name = event.to_string();
                    let area_name = area.area.clone();

                    add_area_message(
                        area.area.as_str(),
                        &self.identifier,
                        format!("{}", GameOutput::AreaEvent(event_name.as_str(), area_name.as_str()))
                    ).expect("Failed to add area event message");
                }
            }

            // Update the areas with the new events
            for area in self.areas.iter_mut() {
                if area_events.contains_key(&area.area) {
                    area.events.extend(area_events[&area.area].1.clone());
                }
            }
        }
    }

    /// Runs the tributes' logic for the current cycle.
    async fn run_tribute_cycle(&mut self, day: bool, rng: &mut SmallRng) {
        // Shuffle the tributes
        self.tributes.shuffle(rng);
        let closed_areas: Vec<Area> = self.closed_areas().clone().iter().map(|ad| Area::from_str(ad.area.as_str()).unwrap()).collect();
        let living_tributes = self.living_tributes();
        let living_tributes_count: usize = living_tributes.len();

        for tribute in self.tributes.iter_mut() {
            // Non-alive tributes should be skipped.
            if !tribute.is_alive() {
                tribute.status = TributeStatus::Dead;
                continue;
            }

            // If the tribute is unlucky, they get a random event.
            if !rng.gen_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.events.push(TributeEvent::random());
            }

            // Trigger day or night cycles for the tribute
            let action_suggestion = match (self.day, day) {
                (Some(1), true) => Some(ActionSuggestion { action: Action::Move(None), probability: Some(0.5) }),
                (Some(3), true) => Some(ActionSuggestion { action: Action::Move(Some(Area::Cornucopia)), probability: Some(0.75) }),
                (_, _) => None,
            };

            let area_details = self.areas.iter_mut()
                .find(|a| a.area == tribute.area.to_string())
                .expect("Cannot find area details");
            let environment_details = &mut EnvironmentContext {
                is_day: day,
                area_details,
                closed_areas: &*closed_areas.clone(),
            };

            let nearby_tributes_count = living_tributes.iter()
                .filter(|t| t.area == tribute.area)
                .count();
            let targets: Vec<Tribute> = living_tributes.iter()
                .filter(|t| t.area == tribute.area) // must be in the same area
                .filter(|t| t.is_visible()) // must be visible
                .filter(|t| t.identifier != tribute.identifier) // can't be self
                .cloned()
                .collect();

            let encounter_context = EncounterContext {
                nearby_tributes_count,
                potential_targets: targets,
                total_living_tributes: living_tributes_count,
            };

            tribute.process_turn_phase(
                action_suggestion,
                environment_details,
                encounter_context,
                rng
            ).await;
        }
    }

    /// Runs a cycle of the game, either day or night.
    /// 1. Announce area events.
    /// 2. Open an area if there are no open areas.
    /// 3. Trigger any events for this cycle if we're past the first three days.
    /// 4. Trigger Feast Day events.
    /// 5. Close more areas by spawning more events if the tributes are getting low.
    /// 6. Run the tribute cycle.
    /// 7. Update the tributes in the game.
    async fn do_a_cycle(&mut self, day: bool) {
        let mut rng = rand::rngs::SmallRng::from_entropy();

        // Announce area events
        self.announce_area_events();

        // If there are no open areas, we need to open one.
        self.ensure_open_area();

        // Trigger any events for this cycle
        self.trigger_cycle_events(day, &mut rng);

        // If the tribute count is low, constrain them by closing areas.
        self.constrain_areas(&mut rng);

        self.run_tribute_cycle(day, &mut rng).await;
    }

    /// Any tributes who have died in the current cycle will be moved to the "dead" list,
    /// and their items will be added to the area they died in.
    async fn clean_up_recent_deaths(&mut self) {
        let tribute_count = self.tributes.len();

        for i in 0..tribute_count {  // Using a for loop to avoid mutable borrow issues
            if self.tributes[i].is_alive() { continue }
            let tribute_items: Vec<Item> = self.tributes[i].items.clone();

            if self.tributes[i].status == TributeStatus::RecentlyDead {
                self.tributes[i].statistics.day_killed = self.day;
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
    use crate::messages::get_all_messages;

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

        game.check_for_winner();

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

        game.check_for_winner();

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

        game.check_for_winner();

        // Game should be finished
        assert_eq!(game.status, starting_state);
    }

    #[test]
    fn test_prepare_cycle() {
        let mut game = Game::new("Test Game");
        let area = AreaDetails::new(Some("Lake".to_string()), Area::North);
        let event = AreaEvent::random();
        game.day = Some(1);
        game.areas.push(area);
        game.areas[0].events.push(event.clone());
        game.prepare_cycle(true);
        assert_eq!(game.day, Some(2));
        assert_eq!(game.areas[0].events.len(), 0);

        game.areas[0].events.push(event.clone());
        game.prepare_cycle(false);
        // Night cycle shouldn't advance the game day.
        assert_eq!(game.day, Some(2));
        assert_eq!(game.areas[0].events.len(), 0);
    }

    #[test]
    fn test_announce_cycle_start() {
        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);
        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        game.day = Some(1);

        clear_messages().unwrap();
        game.announce_cycle_start(true);
        let messages = get_all_messages().unwrap();
        // Game day 1 message
        // Day start message
        // Living tributes message
        assert_eq!(messages.len(), 3);
        clear_messages().unwrap();
    }

    #[test]
    fn test_announce_cycle_end() {
        let tribute1 = create_tribute("Tribute1", true);
        let mut tribute2 = create_tribute("Tribute2", false);
        tribute2.set_status(TributeStatus::RecentlyDead);
        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        game.day = Some(1);

        clear_messages().unwrap();
        game.announce_cycle_end(true);
        let messages = get_all_messages().unwrap();
        // Living tributes message
        // Tribute 2 death message
        // Game day end message
        assert_eq!(messages.len(), 3);
        clear_messages().unwrap();
    }

    #[test]
    fn test_announce_area_events() {
        let mut game = Game::new("Test Game");
        let mut area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        area.events.push(AreaEvent::random());
        area.events.push(AreaEvent::random());
        game.areas.push(area);

        assert!(!game.areas[0].is_open());

        clear_messages().unwrap();
        game.announce_area_events();
        let messages = get_all_messages().unwrap();
        // Area closed message
        // Area event message
        // Area event message
        assert_eq!(messages.len(), 3);
        clear_messages().unwrap();
    }

    #[test]
    fn test_ensure_open_area() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::North);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::South);
        game.areas.push(area1);
        game.areas.push(area2);

        assert!(game.random_open_area().is_some());

        // Close the areas
        game.areas[0].events.push(AreaEvent::random());
        game.areas[1].events.push(AreaEvent::random());

        assert!(game.random_open_area().is_none());

        game.ensure_open_area();
        assert!(game.random_open_area().is_some());
        clear_messages().unwrap();
    }

    #[test]
    fn test_trigger_cycle_events() { }

    #[test]
    fn test_constrain_areas() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::North);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::South);
        game.areas.push(area1);
        game.areas.push(area2);

        // Add tributes to the game
        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);
        game.tributes.push(tribute1.clone());
        game.tributes.push(tribute2.clone());

        // Constrain areas
        let mut rng = SmallRng::from_entropy();
        game.constrain_areas(&mut rng);

        // Check if at least one area is closed
        assert!(game.random_open_area().is_some());
        assert_eq!(game.open_areas().len(), 1);
        assert_eq!(game.closed_areas().len(), 1);
    }

    #[tokio::test]
    async fn test_run_tribute_cycle() {
        // Add tributes
        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);

        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        game.areas.push(area);

        // Run the tribute cycle
        let mut rng = SmallRng::from_entropy();
        game.run_tribute_cycle(true, &mut rng).await;

        // Check if the tributes are updated correctly
        let new_tribute1 = game.tributes[0].clone();
        let new_tribute2 = game.tributes[1].clone();
        assert_ne!(tribute1, new_tribute1);
        assert_ne!(tribute2, new_tribute2);
    }

    #[test]
    fn test_open_and_closed_areas() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::North);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::South);
        game.areas.push(area1);
        game.areas.push(area2);

        assert_eq!(game.open_areas().len(), 2);
        assert!(game.closed_areas().is_empty());

        // Close one area
        game.areas[0].events.push(AreaEvent::random());

        assert_eq!(game.open_areas().len(), 1);
        assert_eq!(game.closed_areas().len(), 1);
    }
}
