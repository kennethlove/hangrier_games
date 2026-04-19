//! Lifecycle, health, and status management for tributes.
//!
//! This module handles:
//! - Life and death state
//! - Health and mental health (damage/healing)
//! - Attribute modifications
//! - Status effects and their processing
//! - Rest and recovery

use crate::areas::AreaDetails;
use crate::areas::events::AreaEvent;
use crate::output::GameOutput;
use crate::tributes::Tribute;
use crate::tributes::statuses::TributeStatus;
use rand::prelude::*;
use rand::rngs::SmallRng;

/// Attribute maximums
const MAX_HEALTH: u32 = 100;
const MAX_SANITY: u32 = 100;
const MAX_MOVEMENT: u32 = 100;
const MAX_STRENGTH: u32 = 50;
const MAX_BRAVERY: u32 = 100;

/// Default healing amounts
const DEFAULT_HEAL: u32 = 5;
const DEFAULT_MENTAL_HEAL: u32 = 5;

/// Status effect damage constants
const WOUNDED_DAMAGE: u32 = 1;
const SICK_STRENGTH_REDUCTION: u32 = 1;
const SICK_MOVEMENT_REDUCTION: u32 = 1;
const ELECTROCUTED_DAMAGE: u32 = 20;
const FROZEN_MOVEMENT_REDUCTION: u32 = 1;
const OVERHEATED_MOVEMENT_REDUCTION: u32 = 1;
const DEHYDRATED_STRENGTH_REDUCTION: u32 = 1;
const STARVING_STRENGTH_REDUCTION: u32 = 1;
const POISONED_MENTAL_DAMAGE: u32 = 5;
const BROKEN_BONE_LEG_MOVEMENT_REDUCTION: u32 = 10;
const BROKEN_BONE_ARM_STRENGTH_REDUCTION: u32 = 5;
const BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION: u32 = 5;
const BROKEN_BONE_RIB_DAMAGE: u32 = 5;
const INFECTED_DAMAGE: u32 = 2;
const INFECTED_MENTAL_DAMAGE: u32 = 5;
const DROWNED_DAMAGE: u32 = 2;
const DROWNED_MENTAL_DAMAGE: u32 = 2;
const BURNED_DAMAGE: u32 = 5;
const BURIED_DAMAGE: u32 = 5;

impl Tribute {
    /// Marks the tribute as dead and reveals them.
    pub fn dies(&mut self) {
        self.attributes.health = 0;
        self.set_status(TributeStatus::Dead);
        self.attributes.is_hidden = false;
        self.items.clear();
    }

    /// Does the tribute have health and an OK status?
    pub fn is_alive(&self) -> bool {
        self.attributes.health > 0
            && self.status != TributeStatus::Dead
            && self.status != TributeStatus::RecentlyDead
    }

    /// Hides the tribute from view.
    pub(crate) fn hides(&mut self) -> bool {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let hidden = rng.random_bool(self.attributes.intelligence as f64 / 100.0);
        self.attributes.is_hidden = hidden;
        hidden
    }

    /// Helper function to see if the tribute is hidden
    pub fn is_visible(&self) -> bool {
        !self.attributes.is_hidden
    }

    /// Tribute is lonely/homesick/etc., loses some sanity.
    pub(crate) fn misses_home(&mut self) {
        let loneliness = self.attributes.bravery as f64 / 100.0; // how lonely is the tribute?

        if loneliness.round() < 0.25 {
            if self.attributes.sanity < 25 {
                self.takes_mental_damage(self.attributes.bravery);
            }
            self.takes_mental_damage(self.attributes.bravery);
        }
    }

    /// Reduces physical health.
    pub(crate) fn takes_physical_damage(&mut self, damage: u32) {
        self.attributes.health = self.attributes.health.saturating_sub(damage);
    }

    /// Reduces mental health.
    pub(crate) fn takes_mental_damage(&mut self, damage: u32) {
        self.attributes.sanity = self.attributes.sanity.saturating_sub(damage);
    }

    /// Reduces attack strength.
    pub(crate) fn reduce_strength(&mut self, amount: u32) {
        self.attributes.strength = self.attributes.strength.saturating_sub(amount).max(1);
    }

    /// Increases attack strength.
    pub(crate) fn increase_strength(&mut self, amount: u32) {
        self.attributes.strength = self
            .attributes
            .strength
            .saturating_add(amount)
            .min(MAX_STRENGTH);
    }

    /// Reduces movement which limits travel and is used by AI for retreat decisions.
    pub(crate) fn reduce_movement(&mut self, amount: u32) {
        self.attributes.movement = self.attributes.movement.saturating_sub(amount).max(1);
    }

    /// Reduces intelligence which affects decision-making and hiding.
    pub(crate) fn reduce_intelligence(&mut self, amount: u32) {
        self.attributes.intelligence = self.attributes.intelligence.saturating_sub(amount).max(1);
    }

    /// Increases bravery which affects decision-making.
    pub(crate) fn increase_bravery(&mut self, amount: u32) {
        self.attributes.bravery = self
            .attributes
            .bravery
            .saturating_add(amount)
            .min(MAX_BRAVERY);
    }

    /// Increases movement which allows more travel
    pub(crate) fn increase_movement(&mut self, amount: u32) {
        self.attributes.movement = self
            .attributes
            .movement
            .saturating_add(amount)
            .min(MAX_MOVEMENT);
    }

    /// Restores health.
    pub(crate) fn heals(&mut self, health: u32) {
        if self.is_alive() {
            self.attributes.health = self
                .attributes
                .health
                .saturating_add(health)
                .min(MAX_HEALTH);
        }
    }

    /// Restores mental health.
    pub(crate) fn heals_mental_damage(&mut self, sanity: u32) {
        self.attributes.sanity = self
            .attributes
            .sanity
            .saturating_add(sanity)
            .min(MAX_SANITY);
    }

    /// Restores movement.
    pub(crate) fn short_rests(&mut self) {
        self.attributes.movement = MAX_MOVEMENT;
    }

    /// Restores movement, some health, and some sanity
    pub(crate) fn long_rests(&mut self) {
        self.short_rests();
        self.heals(DEFAULT_HEAL);
        self.heals_mental_damage(DEFAULT_MENTAL_HEAL);
    }

    /// Restores stamina to maximum capacity
    /// Should be called at the start of each turn
    pub fn restore_stamina(&mut self) {
        self.stamina = self.max_stamina;
    }

    /// Set the tribute's status
    pub fn set_status(&mut self, status: TributeStatus) {
        self.status = status;
    }

    /// Applies statuses to the tribute based on events in the current area.
    pub(crate) fn apply_area_effects(&mut self, area_details: &AreaDetails) {
        for event in &area_details.events {
            match event {
                AreaEvent::Wildfire => self.set_status(TributeStatus::Burned),
                AreaEvent::Flood => self.set_status(TributeStatus::Drowned),
                AreaEvent::Earthquake => self.set_status(TributeStatus::Buried),
                AreaEvent::Avalanche => self.set_status(TributeStatus::Buried),
                AreaEvent::Blizzard => self.set_status(TributeStatus::Frozen),
                AreaEvent::Landslide => self.set_status(TributeStatus::Buried),
                AreaEvent::Heatwave => self.set_status(TributeStatus::Overheated),
                AreaEvent::Sandstorm => self.set_status(TributeStatus::Burned), // Sand burns and abrades
                AreaEvent::Drought => self.set_status(TributeStatus::Overheated), // Dehydration effect
                AreaEvent::Rockslide => self.set_status(TributeStatus::Buried), // Similar to landslide
            }
        }
    }

    /// Applies any effects from elsewhere in the game to the tribute.
    /// This may result in status or attribute changes.
    pub(crate) fn process_status(
        &mut self,
        area_details: &AreaDetails,
        rng: &mut impl Rng,
        events: &mut Vec<String>,
    ) {
        // First, apply any area events for the current area
        self.apply_area_effects(area_details);

        match &self.status {
            // TODO: Add more variation to effects.
            TributeStatus::Wounded => {
                self.takes_physical_damage(WOUNDED_DAMAGE);
            }
            TributeStatus::Sick => {
                self.reduce_strength(SICK_STRENGTH_REDUCTION);
                self.reduce_movement(SICK_MOVEMENT_REDUCTION);
            }
            TributeStatus::Electrocuted => {
                self.takes_physical_damage(ELECTROCUTED_DAMAGE);
            }
            TributeStatus::Frozen => {
                self.reduce_movement(FROZEN_MOVEMENT_REDUCTION);
            }
            TributeStatus::Overheated => {
                self.reduce_movement(OVERHEATED_MOVEMENT_REDUCTION);
            }
            TributeStatus::Dehydrated => {
                self.reduce_strength(DEHYDRATED_STRENGTH_REDUCTION);
            }
            TributeStatus::Starving => {
                self.reduce_strength(STARVING_STRENGTH_REDUCTION);
            }
            TributeStatus::Poisoned => {
                self.takes_mental_damage(POISONED_MENTAL_DAMAGE);
            }
            TributeStatus::Broken => {
                // die roll for which bone breaks
                let bone = rng.random_range(0..4);
                match bone {
                    // Leg
                    0 => self.reduce_movement(BROKEN_BONE_LEG_MOVEMENT_REDUCTION),
                    // Arm
                    1 => self.reduce_strength(BROKEN_BONE_ARM_STRENGTH_REDUCTION),
                    // Skull
                    2 => self.reduce_intelligence(BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION),
                    // Rib
                    _ => self.takes_physical_damage(BROKEN_BONE_RIB_DAMAGE),
                }
            }
            TributeStatus::Infected => {
                self.takes_physical_damage(INFECTED_DAMAGE);
                self.takes_mental_damage(INFECTED_MENTAL_DAMAGE);
            }
            TributeStatus::Drowned => {
                self.takes_physical_damage(DROWNED_DAMAGE);
                self.takes_mental_damage(DROWNED_MENTAL_DAMAGE);
            }
            TributeStatus::Mauled(animal) => {
                let number_of_animals = rng.random_range(2..=5);
                let damage = animal.damage() * number_of_animals;
                self.takes_physical_damage(damage);
            }
            TributeStatus::Burned => {
                self.takes_physical_damage(BURNED_DAMAGE);
            }
            TributeStatus::Buried => {
                self.takes_physical_damage(BURIED_DAMAGE);
            }
            TributeStatus::Healthy | TributeStatus::RecentlyDead | TributeStatus::Dead => {}
        }

        self.events.clear();

        if self.attributes.health == 0 {
            let killer = self.status.clone();
            events.push(
                GameOutput::TributeDiesFromStatus(self.name.as_str(), &killer.to_string())
                    .to_string(),
            );
            self.statistics.killed_by = Some(killer.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::Tribute;
    use crate::tributes::statuses::TributeStatus;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    fn takes_physical_damage(mut tribute: Tribute) {
        let health = tribute.attributes.health;
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, health - 10);
    }

    #[rstest]
    fn takes_no_physical_damage_when_dead(mut tribute: Tribute) {
        tribute.dies();
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, 0);
    }

    #[rstest]
    fn heals(mut tribute: Tribute) {
        tribute.attributes.health = 50;
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 60);
    }

    #[rstest]
    fn does_not_heal_when_dead(mut tribute: Tribute) {
        tribute.dies();
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 0);
    }

    #[rstest]
    fn takes_mental_damage(mut tribute: Tribute) {
        let sanity = tribute.attributes.sanity;
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, sanity - 10);
    }

    #[rstest]
    fn takes_no_mental_damage_when_insane(mut tribute: Tribute) {
        tribute.attributes.sanity = 0;
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 0);
    }

    #[rstest]
    fn heals_mental_damage(mut tribute: Tribute) {
        tribute.attributes.sanity = 50;
        tribute.heals_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 60);
    }

    #[rstest]
    fn short_rests(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        tribute.short_rests();
        assert_eq!(tribute.attributes.movement, 100);
    }

    #[rstest]
    fn long_rests(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        tribute.attributes.health = 50;
        tribute.attributes.sanity = 50;
        tribute.long_rests();
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.attributes.health, 55);
        assert_eq!(tribute.attributes.sanity, 55);
    }

    #[rstest]
    fn dies(mut tribute: Tribute) {
        tribute.dies();
        assert_eq!(tribute.attributes.health, 0);
        assert_eq!(tribute.status, TributeStatus::Dead);
        assert!(!tribute.attributes.is_hidden);
        assert_eq!(tribute.items.len(), 0);
    }

    #[rstest]
    fn is_alive(mut tribute: Tribute) {
        assert!(tribute.is_alive());
        tribute.dies();
        assert!(!tribute.is_alive());
    }

    #[rstest]
    fn hides_success(mut tribute: Tribute) {
        tribute.attributes.intelligence = 100;
        let hidden = tribute.hides();
        assert!(hidden);
        assert!(tribute.attributes.is_hidden);
    }

    #[rstest]
    fn hides_fail(mut tribute: Tribute) {
        tribute.attributes.intelligence = 0;
        let hidden = tribute.hides();
        assert!(!hidden);
        assert!(!tribute.attributes.is_hidden);
    }

    #[rstest]
    fn misses_home(mut tribute: Tribute) {
        tribute.attributes.bravery = 20;
        tribute.attributes.sanity = 20;
        let sanity = tribute.attributes.sanity;
        tribute.misses_home();
        assert!(tribute.attributes.sanity < sanity);
    }

    #[rstest]
    fn is_visible(mut tribute: Tribute) {
        assert!(tribute.is_visible());
        tribute.attributes.is_hidden = true;
        assert!(!tribute.is_visible());
    }

    #[rstest]
    fn process_status_mauled(mut tribute: Tribute, mut small_rng: SmallRng) {
        use crate::threats::animals::Animal;
        let health = tribute.attributes.health;
        tribute.status = TributeStatus::Mauled(Animal::Bear);
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert!(tribute.attributes.health < health);
    }

    #[rstest]
    #[case(TributeStatus::Wounded)]
    #[case(TributeStatus::Sick)]
    #[case(TributeStatus::Electrocuted)]
    #[case(TributeStatus::Frozen)]
    #[case(TributeStatus::Overheated)]
    #[case(TributeStatus::Dehydrated)]
    #[case(TributeStatus::Starving)]
    #[case(TributeStatus::Poisoned)]
    #[case(TributeStatus::Broken)]
    #[case(TributeStatus::Infected)]
    #[case(TributeStatus::Drowned)]
    #[case(TributeStatus::Burned)]
    #[case(TributeStatus::Buried)]
    fn process_status_no_effect(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
        #[case] status: TributeStatus,
    ) {
        tribute.status = status.clone();
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert!(tribute.is_alive());
    }

    #[rstest]
    fn process_status_dies(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 1;
        tribute.status = TributeStatus::Electrocuted;
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert_eq!(tribute.attributes.health, 0);
        assert_eq!(tribute.status, TributeStatus::RecentlyDead);
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[rstest]
    fn process_status_from_area_event(mut tribute: Tribute, mut small_rng: SmallRng) {
        use crate::areas::Area;
        use crate::areas::events::AreaEvent;

        let mut area_details = AreaDetails::new(Some("Forest".to_string()), Area::Cornucopia);
        area_details.events.push(AreaEvent::Wildfire);

        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert_eq!(tribute.status, TributeStatus::Burned);
    }
}
