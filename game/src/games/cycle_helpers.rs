use super::*;
use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use rand::rngs::SmallRng;
use std::collections::HashMap;

impl Game {
    pub(super) fn run_trauma_producers(&mut self, _phase: crate::messages::Phase) {
        crate::tributes::afflictions::producers::run_trauma_producers(self);
    }

    /// Announce events in closed areas.
    ///
    /// Emits one `MessageSource::Area` line per active event (using
    /// `GameOutput::AreaEvent`) plus a closing `GameOutput::AreaClose`
    /// summary so consumers know the area is currently uninhabitable.
    pub(super) fn announce_area_events(&mut self) -> Result<(), GameError> {
        // Snapshot to avoid borrow conflicts with self.log_output below.
        let snapshots: Vec<(String, Vec<String>)> = self
            .areas
            .iter()
            .filter(|a| !a.is_open())
            .filter_map(|a| {
                a.area.map(|area| {
                    (
                        area.to_string(),
                        a.events.iter().map(|e| e.to_string()).collect(),
                    )
                })
            })
            .collect();

        for (area_name, event_names) in snapshots {
            let subject = format!("area:{}", area_name);
            for event_name in &event_names {
                self.log_event(
                    crate::messages::MessageSource::Area(area_name.clone()),
                    subject.clone(),
                    crate::events::GameEvent::AreaEvent {
                        area_event: event_name.clone(),
                        area_name: area_name.clone(),
                    },
                );
            }
            self.log_event(
                crate::messages::MessageSource::Area(area_name.clone()),
                subject,
                crate::events::GameEvent::AreaClose {
                    area_name: area_name.clone(),
                },
            );
        }
        Ok(())
    }

    /// Ensures at least one area is open. If not, opens a random area by clearing its events.
    pub(super) fn ensure_open_area(&mut self) {
        if self.random_open_area().is_none()
            && let Some(area) = self.random_area()
        {
            area.events.clear();
        }
    }

    /// Triggers events for the current cycle.
    pub(super) fn trigger_cycle_events(
        &mut self,
        phase: crate::messages::Phase,
        rng: &mut SmallRng,
    ) -> Result<(), GameError> {
        use crate::messages::Phase;
        let frequency = match phase {
            Phase::Day => DAY_EVENT_FREQUENCY,
            Phase::Night => NIGHT_EVENT_FREQUENCY,
            // Substrate-only: Dawn/Dusk are silent in PR1. PR2 redistributes.
            Phase::Dawn | Phase::Dusk => return Ok(()),
        };
        let day = phase == Phase::Day;

        // Collect events to trigger (avoid borrow conflicts)
        let mut events_to_process: Vec<(Area, AreaEvent)> = Vec::new();

        // If it's nighttime, trigger an event
        // If it is daytime and not day #1 or day #3, trigger an event
        if !day || ![1, 3].contains(&self.day.unwrap_or(1)) {
            for area_details in self.areas.iter_mut() {
                if rng.random_bool(frequency) {
                    // Generate terrain-appropriate event
                    let area_event = AreaEvent::random_for_terrain(&area_details.terrain.base, rng);
                    let area = area_details.area.unwrap();

                    // Add event to area
                    area_details.events.push(area_event.clone());

                    // Announce event
                    let _event_name = area_event.to_string();
                    let _area_name = area.to_string();

                    // Collect for processing
                    events_to_process.push((area, area_event));
                }
            }
        }

        // Process survival checks for all triggered events
        for (area, event) in events_to_process {
            self.process_event_for_area(&area, &event, rng)?;
        }

        // Day 3 is Feast Day, refill the Cornucopia with a random assortment of items
        if day
            && self.day == Some(3)
            && let Some(area_details) = self
                .areas
                .iter_mut()
                .find(|ad| ad.area == Some(Area::Cornucopia))
        {
            for _ in 0..rng.random_range(1..=FEAST_WEAPON_COUNT) {
                area_details.add_item(Item::new_random_weapon());
            }
            for _ in 0..rng.random_range(1..=FEAST_SHIELD_COUNT) {
                area_details.add_item(Item::new_random_shield());
            }
            for _ in 0..rng.random_range(1..=FEAST_CONSUMABLE_COUNT) {
                area_details.add_item(Item::new_random_consumable());
            }
        }
        Ok(())
    }

    /// If the tribute count is low, constrain them by closing areas.
    /// We achieve this by spawning events in open areas.
    pub(super) fn constrain_areas(&mut self, rng: &mut SmallRng) -> Result<(), GameError> {
        let tribute_count = self.living_tributes_count() as u32;
        let odds = tribute_count as f64 / 24.0;
        let mut area_events: HashMap<String, (AreaDetails, Vec<AreaEvent>)> = HashMap::new();

        if (1..LOW_TRIBUTE_THRESHOLD).contains(&tribute_count) {
            // If there is an open area, close it.
            if let Some(area_details) = self.random_open_area() {
                let event = AreaEvent::random(rng);
                let area_name = area_details.area.unwrap().to_string();
                area_events.insert(area_name, (area_details.clone(), vec![event.clone()]));
            }

            if rng.random_bool(odds) {
                // Assuming there's still an open area.
                if let Some(area_details) = self.random_open_area() {
                    let event = AreaEvent::random(rng);
                    let area_name = area_details.area.unwrap().to_string();
                    if area_events.contains_key(&area_name) {
                        let mut events = area_events[&area_name].1.clone();
                        events.push(event.clone());
                        area_events.insert(area_name, (area_details.clone(), events));
                    } else {
                        area_events.insert(area_name, (area_details.clone(), vec![event.clone()]));
                    }
                }
            }

            // Add events to each area and announce them
            for (area_name, (mut area_details, events)) in area_events.drain() {
                for event in events {
                    area_details.events.push(event.clone());
                    let _event_name = event.to_string();
                    // let area_name = area_details.area.clone().unwrap().to_string();
                }

                // Update the corresponding area with the new events
                for area in self.areas.iter_mut() {
                    let key = area.area.unwrap().to_string();
                    if key == area_name {
                        area.events = area_details.events;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
