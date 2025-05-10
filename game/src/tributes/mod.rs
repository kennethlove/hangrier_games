pub mod actions;
pub mod brains;
pub mod events;
pub mod statuses;

use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::{Attribute, Item};
use crate::items::{ItemError, OwnsItems};
use crate::messages::add_tribute_message;
use crate::output::GameOutput;
use crate::tributes::events::TributeEvent;
use actions::{Action, AttackOutcome, AttackResult, TributeAction};
use brains::Brain;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use statuses::TributeStatus;
use std::cmp::{Ordering, PartialEq};
use tracing::info;
use uuid::Uuid;

/// Consts
const DEFAULT_HEAL: u32 = 5;
const DEFAULT_MENTAL_HEAL: u32 = 5;
const SANITY_BREAK_LEVEL: u32 = 9;
const LOYALTY_BREAK_LEVEL: f64 = 0.25;
const DECISIVE_WIN_MULTIPLIER: f64 = 1.5;

/// Damages
const WOUNDED_DAMAGE: u32 = 1;
const SICK_STRENGTH_REDUCTION: u32 = 1;
const SICK_SPEED_REDUCTION: u32 = 1;
const ELECTROCUTED_DAMAGE: u32 = 20;
const FROZEN_SPEED_REDUCTION: u32 = 1;
const OVERHEATED_SPEED_REDUCTION: u32 = 1;
const DEHYDRATED_STRENGTH_REDUCTION: u32 = 1;
const STARVING_STRENGTH_REDUCTION: u32 = 1;
const POISONED_MENTAL_DAMAGE: u32 = 5;
const BROKEN_BONE_LEG_SPEED_REDUCTION: u32 = 10;
const BROKEN_BONE_ARM_STRENGTH_REDUCTION: u32 = 5;
const BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION: u32 = 5;
const BROKEN_BONE_RIB_DEXTERITY_REDUCTION: u32 = 5;
const INFECTED_DAMAGE: u32 = 2;
const INFECTED_MENTAL_DAMAGE: u32 = 5;
const DROWNED_DAMAGE: u32 = 2;
const DROWNED_MENTAL_DAMAGE: u32 = 2;
const BURNED_DAMAGE: u32 = 5;
const BURIED_SPEED_REDUCTION: u32 = 5;

/// Attributes
const MAX_HEALTH: u32 = 100;
const MAX_SANITY: u32 = 100;
const MAX_MOVEMENT: u32 = 100;
const MAX_STRENGTH: u32 = 50;
const MAX_DEFENSE: u32 = 50;
const MAX_BRAVERY: u32 = 100;
const MAX_LOYALTY: u32 = 100;
const MAX_SPEED: u32 = 100;
const MAX_DEXTERITY: u32 = 100;
const MAX_INTELLIGENCE: u32 = 100;
const MAX_PERSUASION: u32 = 100;
const MAX_LUCK: u32 = 100;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tribute {
    /// Identifier
    pub identifier: String,
    /// Where are they?
    pub area: Area,
    /// What is their current status?
    pub status: TributeStatus,
    /// This is their thinker
    #[serde(skip)]
    pub brain: Brain,
    /// How they present themselves to the real world
    pub avatar: Option<String>,
    /// Who created them in the real world
    #[serde(rename = "player_name")]
    pub human_player_name: Option<String>,
    /// What they like to go by
    pub name: String,
    /// Where they're from
    pub district: u32,
    /// Stats like fights won
    pub statistics: Statistics,
    /// Attributes like health
    pub attributes: Attributes,
    /// Items the tribute owns
    #[serde(default)]
    pub items: Vec<Item>,
    /// Events that have happened to the tribute
    #[serde(default)]
    pub events: Vec<TributeEvent>,
    #[serde(default)]
    pub editable: bool,
}

impl Default for Tribute {
    fn default() -> Self { Self::new("Tribute".to_string(), None, None) }
}

impl OwnsItems for Tribute {
    fn add_item(&mut self, item: Item) { self.items.push(item); }

    fn has_item(&self, item: &Item) -> bool {
        self.items.iter().any(|i| i == item)
    }

    fn use_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let used_item = self.items
            .iter_mut()
            .find(|i| i.identifier == item.identifier);

        if let Some(used_item) = used_item {
            if used_item.quantity == 0 {
                return Err(ItemError::ItemNotUsable);
            }

            used_item.quantity = used_item.quantity.saturating_sub(1);
            if used_item.quantity == 0 {
                self.remove_item(item)
            } else { Ok(()) }
        } else {
            Err(ItemError::ItemNotFound)
        }
    }

    fn remove_item(&mut self, item: &Item) -> Result<(), ItemError> {
        let index = self.items.iter().position(|i| i.identifier == item.identifier);
        if let Some(index) = index {
            self.items.remove(index);
            Ok(())
        } else {
            Err(ItemError::ItemNotFound)
        }
    }
}

impl Tribute {
    /// Creates a new Tribute with full health, sanity, and movement.
    pub fn new(name: String, district: Option<u32>, avatar: Option<String>) -> Self {
        let brain = Brain::default();
        let district = district.unwrap_or(0);
        let attributes = Attributes::new();
        let statistics = Statistics::default();

        let id: String = Uuid::new_v4().to_string();

        Self {
            identifier: id,
            area: Area::Cornucopia,
            name: name.clone(),
            district,
            brain,
            status: TributeStatus::default(),
            avatar,
            human_player_name: None,
            attributes,
            statistics,
            items: vec![],
            events: vec![],
            editable: true,
        }
    }

    pub fn random() -> Self {
        let name = Name(EN).fake();
        let mut rng = SmallRng::from_entropy();
        let district = rng.gen_range(1..=12);
        Tribute::new(name, Some(district), None)
    }

    pub fn avatar(&self) -> String {
        format!(
            "assets/{}",
            self.avatar
                .clone()
                .unwrap_or("hangry-games.png".to_string())
        )
    }

    /// Reduces health.
    fn takes_physical_damage(&mut self, damage: u32) {
        self.attributes.health = self.attributes.health.saturating_sub(damage);
    }

    /// Reduces mental health.
    fn takes_mental_damage(&mut self, damage: u32) {
        self.attributes.sanity = self.attributes.sanity.saturating_sub(damage);
    }

    /// Reduces attack strength.
    fn reduce_strength(&mut self, amount: u32) {
        self.attributes.strength = self.attributes.strength.saturating_sub(amount).max(1);
    }

    /// Increases attack strength.
    fn increase_strength(&mut self, amount: u32) {
        self.attributes.strength = self.attributes.strength.saturating_add(amount).min(MAX_STRENGTH);
    }

    /// Reduces movement speed.
    fn reduce_speed(&mut self, amount: u32) {
        self.attributes.speed = self.attributes.speed.saturating_sub(amount).max(1);
    }

    /// Increases movement speed.
    fn increase_speed(&mut self, amount: u32) {
        self.attributes.speed = self.attributes.speed.saturating_add(amount).min(MAX_SPEED);
    }

    /// Reduces intelligence which affects decision-making and hiding.
    fn reduce_intelligence(&mut self, amount: u32) {
        self.attributes.intelligence = self.attributes.intelligence.saturating_sub(amount).max(1);
    }

    /// Increases bravery which affects decision-making.
    fn increase_bravery(&mut self, amount: u32) {
        self.attributes.bravery = self.attributes.bravery.saturating_add(amount).min(MAX_BRAVERY);
    }

    /// Increases movement which allows more travel
    /// TODO: Use movement more effectively.
    fn increase_movement(&mut self, amount: u32) {
        self.attributes.movement = self.attributes.movement.saturating_add(amount).min(MAX_MOVEMENT);
    }

    /// Reduces dexterity which currently affects nothing.
    /// TODO: Use dexterity for something.
    fn reduce_dexterity(&mut self, amount: u32) {
        self.attributes.dexterity = self.attributes.dexterity.saturating_sub(amount).max(1);
    }

    /// Restores health.
    fn heals(&mut self, health: u32) {
        if self.is_alive() {
            self.attributes.health = self.attributes.health.saturating_add(health).min(MAX_HEALTH);
        }
    }

    /// Restores mental health.
    fn heals_mental_damage(&mut self, sanity: u32) {
        self.attributes.sanity = self.attributes.sanity.saturating_add(sanity).min(MAX_SANITY);
    }

    /// Restores movement.
    fn short_rests(&mut self) { self.attributes.movement = MAX_MOVEMENT; }

    /// Restores movement, some health, and some sanity
    fn long_rests(&mut self) {
        self.short_rests();
        self.heals(DEFAULT_HEAL);
        self.heals_mental_damage(DEFAULT_MENTAL_HEAL);
    }

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
    fn hides(&mut self) -> bool {
        let mut rng = SmallRng::from_entropy();
        let hidden = rng.gen_bool(self.attributes.intelligence as f64 / 100.0);
        self.attributes.is_hidden = hidden;
        hidden
    }

    /// Helper function to see if the tribute is hidden
    pub fn is_visible(&self) -> bool {
        !self.attributes.is_hidden
    }

    /// Tribute is lonely/homesick/etc., loses some sanity.
    fn misses_home(&mut self) {
        let loneliness = self.attributes.bravery as f64 / 100.0; // how lonely is the tribute?

        if loneliness.round() < 0.25 {
            if self.attributes.sanity < 25 {
                self.takes_mental_damage(self.attributes.bravery);
            }
            self.takes_mental_damage(self.attributes.bravery);
        }
    }

    /// Tribute attacks another tribute
    /// Potentially fatal to either tribute
    fn attacks(&mut self, target: &mut Tribute, rng: &mut impl Rng) -> AttackOutcome {
        // Is the tribute attempting suicide?
        if self == target {
            self.try_log_action(
                GameOutput::TributeSelfHarm(self.name.as_str()),
                "self-harm"
            );

            // Attack always succeeds
            target.takes_physical_damage(self.attributes.strength);

            self.try_log_action(
                GameOutput::TributeAttackWin(self.name.as_str(), target.name.as_str()),
                "attack against self"
            );

            if target.attributes.health > 0 {
                self.try_log_action(
                    GameOutput::TributeAttackWound(self.name.as_str(), target.name.as_str()),
                    "wounded self"
                );
                return AttackOutcome::Wound(self.clone(), target.clone());
            } else {
                self.try_log_action(
                    GameOutput::TributeSuicide(self.name.as_str()),
                    "successful suicide"
                );
                return AttackOutcome::Kill(self.clone(), target.clone());
            }
        }

        let tribute_name = self.name.clone();
        let target_name = target.name.clone();
        // `self` is the attacker
        match attack_contest(self, target, rng) {
            AttackResult::AttackerWins => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength,
                    GameOutput::TributeAttackWin(tribute_name.as_str(), target_name.as_str()),
                    "attack win"
                );
            }
            AttackResult::AttackerWinsDecisively => {
                apply_combat_results(
                    self,
                    target,
                    self.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackWinExtra(tribute_name.as_str(), target_name.as_str()),
                    "attack win extra"
                );
            }
            AttackResult::DefenderWins => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength,
                    GameOutput::TributeAttackLose(tribute_name.as_str(), target_name.as_str()),
                    "attack lose"
                );
            }
            AttackResult::DefenderWinsDecisively => {
                apply_combat_results(
                    target,
                    self,
                    target.attributes.strength * 2, // double damage
                    GameOutput::TributeAttackLoseExtra(tribute_name.as_str(), target_name.as_str()),
                    "attack lose extra"
                );
            }
            AttackResult::Miss => {
                self.statistics.draws += 1;
                target.statistics.draws += 1;

                self.try_log_action(
                    GameOutput::TributeAttackMiss(tribute_name.as_str(), target_name.as_str()),
                    "missed attack"
                );

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        if self.attributes.health == 0 {
            // Target killed attacker
            self.statistics.killed_by = Some(target_name.clone());
            self.status = TributeStatus::RecentlyDead;

            self.try_log_action(
                GameOutput::TributeAttackDied(tribute_name.as_str(), target_name.as_str()),
                "attacker died"
            );

            AttackOutcome::Kill(target.clone(), self.clone())
        } else if target.attributes.health == 0 {
            // Attacker killed Target
            target.statistics.killed_by = Some(tribute_name.clone());
            target.status = TributeStatus::RecentlyDead;

            self.try_log_action(
                GameOutput::TributeAttackSuccessKill(tribute_name.as_str(), target_name.as_str()),
                "killed target"
            );

            AttackOutcome::Kill(self.clone(), target.clone())
        } else {
            self.try_log_action(
                GameOutput::TributeAttackWound(tribute_name.as_str(), target_name.as_str()),
                "wounded target"
            );
            AttackOutcome::Wound(self.clone(), target.clone())
        }
    }

    /// Moves a tribute to a new area.
    /// If the tribute has no movement, they cannot move.
    /// If the tribute is already in the suggested area, they stay put.
    /// If the tribute has low movement, they can only move to the suggested area or stay put.
    /// If the tribute has high movement, they can move to any open neighbor or the suggested area.
    async fn travels(&self, closed_areas: Vec<Area>, suggested_area: Option<Area>) -> TravelResult {
        let mut rng = SmallRng::from_entropy();
        // Where is the tribute?
        let current_area = self.area.clone();

        // 1. Can the tribute move at all?
        if self.attributes.movement == 0 {
            let current_area = self.area.to_string();
            self.try_log_action(
                GameOutput::TributeTravelTooTired(self.name.as_str(), current_area.as_str()),
                "too tired"
            );
            return TravelResult::Failure;
        }

        // 2. Determine the target area based on suggestion and validity.
        let mut target_area: Option<Area> = None;
        if let Some(suggestion) = suggested_area {
            if !closed_areas.contains(&suggestion) {
                if suggestion == current_area {
                    let suggestion = suggestion.to_string();
                    self.try_log_action(
                        GameOutput::TributeTravelAlreadyThere(self.name.as_str(), suggestion.as_str()),
                        "already there"
                    );
                    return TravelResult::Failure;
                }
                target_area = Some(suggestion);
            }
        }

        // 3. Handle movement based on tribute's movement attribute.
        match self.attributes.movement {
            // Low movement: can only move to suggested_area if it's valid and set.
            1..=10 => {
                if let Some(new_area) = target_area {
                    let current_area = current_area.to_string();
                    let new_area_name = new_area.to_string();
                    self.try_log_action(
                        GameOutput::TributeTravel(self.name.as_str(), current_area.as_str(), new_area_name.as_str()),
                        "travel"
                    );
                    TravelResult::Success(new_area)
                } else {
                    let current_area = current_area.to_string();
                    self.try_log_action(
                        GameOutput::TributeTravelTooTired(self.name.as_str(), current_area.as_str()),
                        "too tired"
                    );
                    TravelResult::Failure
                }
            }
            // High movement: can move to any open neighbor or the suggested area.
            _ => {
                if let Some(new_area) = target_area {
                    let current_area = current_area.to_string();
                    let new_area_name = new_area.to_string();
                    self.try_log_action(
                        GameOutput::TributeTravel(self.name.as_str(), current_area.as_str(), new_area_name.as_str()),
                        "travel"
                    );
                    return TravelResult::Success(new_area)
                }

                let neighbors = current_area.neighbors();
                let available_neighbors: Vec<Area> = neighbors
                    .into_iter()
                    .filter(|area| area != &current_area && !closed_areas.contains(area))
                    .collect();

                if available_neighbors.is_empty() {
                    let current_area_name = current_area.to_string();
                    self.try_log_action(
                        GameOutput::TributeTravelNoOptions(self.name.as_str(), current_area_name.as_str()),
                        "no options"
                    );
                    return TravelResult::Success(current_area.clone())
                }

                // TODO: Loyalty bit goes here

                let chosen_neighbor = available_neighbors.choose(&mut rng).unwrap();
                let current_area_name = current_area.to_string();
                let chosen_area_name = chosen_neighbor.to_string();
                self.try_log_action(
                    GameOutput::TributeTravel(self.name.as_str(), current_area_name.as_str(), chosen_area_name.as_str()),
                    "travel"
                );
                TravelResult::Success(chosen_neighbor.clone())
            }
        }
    }

    /// Applies any effects from elsewhere in the game to the tribute.
    /// This may result in status or attribute changes.
    fn process_status(&mut self, area_details: &AreaDetails, rng: &mut impl Rng) {
        // First, apply any area events for the current area
        self.apply_area_effects(area_details);

        match &self.status {
            // TODO: Add more variation to effects.
            TributeStatus::Wounded => { self.takes_physical_damage(WOUNDED_DAMAGE); }
            TributeStatus::Sick => {
                self.reduce_strength(SICK_STRENGTH_REDUCTION);
                self.reduce_speed(SICK_SPEED_REDUCTION);
            }
            TributeStatus::Electrocuted => { self.takes_physical_damage(ELECTROCUTED_DAMAGE); }
            TributeStatus::Frozen => { self.reduce_speed(FROZEN_SPEED_REDUCTION); }
            TributeStatus::Overheated => { self.reduce_speed(OVERHEATED_SPEED_REDUCTION); }
            TributeStatus::Dehydrated => { self.reduce_strength(DEHYDRATED_STRENGTH_REDUCTION); }
            TributeStatus::Starving => { self.reduce_strength(STARVING_STRENGTH_REDUCTION); }
            TributeStatus::Poisoned => { self.takes_mental_damage(POISONED_MENTAL_DAMAGE); }
            TributeStatus::Broken => {
                // die roll for which bone breaks
                let bone = rng.gen_range(0..4);
                match bone {
                    // Leg
                    0 => self.reduce_speed(BROKEN_BONE_LEG_SPEED_REDUCTION),
                    // Arm
                    1 => self.reduce_strength(BROKEN_BONE_ARM_STRENGTH_REDUCTION),
                    // Skull
                    2 => self.reduce_intelligence(BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION),
                    // Rib
                    _ => self.reduce_dexterity(BROKEN_BONE_RIB_DEXTERITY_REDUCTION),
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
                let number_of_animals = rng.gen_range(2..=5);
                let damage = animal.damage() * number_of_animals;
                self.takes_physical_damage(damage);
            }
            TributeStatus::Burned => {
                self.takes_physical_damage(BURNED_DAMAGE);
            }
            TributeStatus::Buried => {
                self.reduce_speed(BURIED_SPEED_REDUCTION);
            }
            TributeStatus::Healthy | TributeStatus::RecentlyDead | TributeStatus::Dead => {}
        }

        self.events.clear();

        if self.attributes.health == 0 {
            let killer = self.status.clone();
            self.try_log_action(
                GameOutput::TributeDiesFromStatus(self.name.as_str(), &*killer.to_string()),
                "dies from status"
            );
            self.statistics.killed_by = Some(killer.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    /// Applies the effects of a tribute event on a tribute.
    // TODO: Use it or lose it
    pub async fn handle_event(&mut self, tribute_event: TributeEvent) {
        match tribute_event {
            TributeEvent::AnimalAttack(ref animal) => {
                self.status = TributeStatus::Mauled(animal.clone());
            }
            TributeEvent::Dysentery => {
                self.status = TributeStatus::Sick;
            }
            TributeEvent::LightningStrike => {
                self.status = TributeStatus::Electrocuted;
            }
            TributeEvent::Hypothermia => {
                self.status = TributeStatus::Frozen;
            }
            TributeEvent::HeatStroke => {
                self.status = TributeStatus::Overheated;
            }
            TributeEvent::Dehydration => {
                self.status = TributeStatus::Dehydrated;
            }
            TributeEvent::Starvation => {
                self.status = TributeStatus::Starving;
            }
            TributeEvent::Poisoning => {
                self.status = TributeStatus::Poisoned;
            }
            TributeEvent::BrokenBone => {
                self.status = TributeStatus::Broken;
            }
            TributeEvent::Infection => {
                self.status = TributeStatus::Infected;
            }
            TributeEvent::Drowning => {
                self.status = TributeStatus::Drowned;
            }
            TributeEvent::Burn => {
                self.status = TributeStatus::Burned;
            }
        }
        if self.attributes.health == 0 {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeDiesFromTributeEvent(self.name.as_str(), &*tribute_event.to_string())),
            ).expect("");

            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    /// Send a tribute through a game cycle.
    /// This is the main function that runs the tribute's actions.
    /// 1. Ignore dead tributes.
    /// 2. Process status effects including area events.
    /// 3. Check for gifts from sponsors.
    /// 4. Check for nighttime effects.
    /// 5. Check for suggested actions.
    /// 6. Get the tribute's action from the brain.
    /// 7. Perform the action.
    /// 8. Log the action.
    pub async fn do_day_night(
        &mut self,
        suggested_action: Option<Action>,
        probability: Option<f64>,
        day: bool,
        area_details: &mut AreaDetails,
        closed_areas: Vec<Area>,
        number_of_nearby_tributes: usize,
        targets: Vec<Tribute>,
        living_tributes_count: usize,
        rng: &mut impl Rng,
    ) {
        // Tribute is already dead, do nothing.
        if !self.is_alive() {
            self.try_log_action(
                GameOutput::TributeAlreadyDead(self.name.as_str()),
                "already dead"
            );
            return;
        }

        // Update the tribute based on the period's events.
        self.process_status(area_details, rng);

        // Tribute died to the period's events.
        if self.status == TributeStatus::RecentlyDead || self.attributes.health == 0 {
            self.try_log_action(
                GameOutput::TributeDead(self.name.as_str()),
                "died to events"
            );
        }

        // Any generous patrons this round?
        if let Some(gift) = self.receive_patron_gift(&mut *rng).await {
            self.add_item(gift.clone());
            self.try_log_action(
                GameOutput::SponsorGift(self.name.as_str(), gift.clone()),
                "received gift"
            );
        }

        // Nighttime terror
        if !day && self.is_alive() { self.misses_home(); }

        if let Some(action) = suggested_action {
            self.brain.set_preferred_action(
                action,
                probability.unwrap_or(1.0) // If no probability is set, perform the preferred action.
            );
        }

        let tribute = self.clone();
        let action = self.brain.act(&tribute, number_of_nearby_tributes, &mut *rng);

        match &action {
            Action::Move(area) => match self.travels(closed_areas, area.clone()).await {
                TravelResult::Success(area) => {
                    self.area = area.clone();
                }
                TravelResult::Failure => {
                    self.short_rests();
                }
            },
            Action::Hide => {
                self.hides();
                self.take_action(&action, None);
                self.try_log_action(
                    GameOutput::TributeHide(self.name.as_str()),
                    "hides"
                );
            }
            Action::Rest | Action::None => {
                self.long_rests();
                self.take_action(&action, None);
                self.try_log_action(
                    GameOutput::TributeLongRest(self.name.as_str()),
                    "long rests"
                );
            }
            // Try to attack another tribute
            Action::Attack => {
                if let Some(mut target) = self.pick_target(targets, living_tributes_count).await {
                    self.attacks(&mut target, &mut *rng);
                    self.take_action(&action, Some(&target));
                } else {
                    self.take_action(&Action::Rest, None);
                }
            }
            Action::TakeItem => {
                if let Some(item) = self.take_nearby_item(area_details) {
                    self.try_log_action(
                        GameOutput::TributeTakeItem(self.name.as_str(), item.name.as_str()),
                        "took item"
                    );
                    self.take_action(&action, None);
                }
            }
            Action::UseItem(None) => {
                // Get consumable items
                let items = self.consumables();
                if items.is_empty() {
                    self.long_rests();
                    self.take_action(&Action::Rest, None);
                } else {
                    // Use random item
                    // let item = items.choose_mut(rng).unwrap();
                    let item: Item;
                    if let Some(chosen_item) = items.choose(rng) {
                        item = chosen_item.clone();
                    } else {
                        self.long_rests();
                        self.take_action(&Action::Rest, None);
                        return;
                    }

                    match self.use_consumable(item.clone()) {
                        Ok(()) => {
                            self.try_log_action(
                                GameOutput::TributeUseItem(self.name.as_str(), item.clone()),
                                "used random item"
                            );
                            // self.use_item(item.clone()).expect("Failed to use item");
                            self.take_action(&action, None);
                        }
                        Err(_) => {
                            self.try_log_action(
                                GameOutput::TributeCannotUseItem(self.name.as_str(), item.name.as_str()),
                                "cannot use random item"
                            );
                            self.short_rests();
                            self.take_action(&Action::Rest, None);
                        }
                    };
                }
            }
            Action::UseItem(item) => {
                let items = self.consumables();
                if let Some(item) = item {
                    if items.contains(item) {
                        match self.use_consumable(item.clone()) {
                            Ok(()) => {
                                self.try_log_action(
                                    GameOutput::TributeUseItem(self.name.as_str(), item.clone()),
                                    "used specific item"
                                );
                                if let Err(_) = self.use_item(item) {
                                    self.try_log_action(
                                        GameOutput::TributeCannotUseItem(self.name.as_str(), item.name.as_str()),
                                        "cannot use specific item"
                                    );
                                } else {
                                    self.take_action(&action, None);
                                }
                            }
                            Err(_) => {
                                self.try_log_action(
                                    GameOutput::TributeCannotUseItem(self.name.as_str(), item.name.as_str()),
                                    "cannot use specific item"
                                );
                                self.short_rests();
                                self.take_action(&Action::Rest, None);
                            }
                        };
                    }
                }
            }
        }
    }

    /// Receive a patron gift, broken down by district
    async fn receive_patron_gift(&mut self, mut rng: impl Rng) -> Option<Item> {
        // Gift from patrons?
        let chance = match self.district {
            1 | 2 => 1.0 / 10.0,
            3 | 4 => 1.0 / 15.0,
            5 | 6 => 1.0 / 20.0,
            7 | 8 => 1.0 / 25.0,
            9 | 10 => 1.0 / 30.0,
            11 | 12 => 1.0 / 50.0,
            _ => 1.0, // Mainly for testing/debugging purposes
        };

        if rng.gen_bool(chance) { Some(Item::new_random_consumable()) } else { None }
    }

    /// Save the tribute's latest action
    fn take_action(&mut self, action: &Action, target: Option<&Tribute>) {
        self.brain
            .previous_actions
            .push(TributeAction::new(action.clone(), target.cloned()));
    }

    /// Take an item from the current area
    fn take_nearby_item(&mut self, area_details: &mut AreaDetails) -> Option<Item> {
        let mut rng = SmallRng::from_entropy();
        let items = area_details.items.clone();
        if items.is_empty() {
            None
        } else {
            let item = items.choose(&mut rng).unwrap().clone();
            if let Ok(()) = area_details.use_item(&item) {
                self.add_item(item.clone());

                return Some(item.clone());
            }
            None
        }
    }

    /// Use consumable item from inventory
    fn use_consumable(&mut self, chosen_item: Item) -> Result<(), ItemError> {
        let items = self.consumables();
        let item: Item;

        // If the tribute has the item...
        match items.iter().find(|i| i.identifier == chosen_item.identifier) {
            Some(selected_item) => {
                // select it
                item = selected_item.clone();
            }
            None => {
                // otherwise, quit because you can't use an item you don't have
                return Err(ItemError::ItemNotFound);
            }
        }

        if self.use_item(&item).is_err() {
            return Err(ItemError::ItemNotUsable);
        }

        // Apply item effect
        match item.attribute {
            Attribute::Health => self.heals(item.effect as u32),
            Attribute::Sanity => self.heals_mental_damage(item.effect as u32),
            Attribute::Movement => self.increase_movement(item.effect as u32),
            Attribute::Bravery => self.increase_bravery(item.effect as u32),
            Attribute::Speed => self.increase_speed(item.effect as u32),
            Attribute::Strength => self.increase_strength(item.effect as u32),
            _ => return Err(ItemError::InvalidAttribute),
        }
        Ok(())
    }

    /// What items does the tribute have?
    fn available_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.quantity > 0)
            .cloned()
            .collect()
    }

    /// Which items are marked as weapons?
    fn weapons(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_weapon())
            .cloned()
            .collect()
    }

    /// Which items are marked as shields?
    fn shields(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_defensive())
            .cloned()
            .collect()
    }

    /// Which items are marked as consumable?
    pub fn consumables(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_consumable())
            .cloned()
            .collect()
    }

    /// Pick an appropriate target from nearby tributes prioritizing targets as follows:
    /// (for this function, "nearby" means in the same area and "ally" means
    /// from the same district)
    /// 1. If there are enemy tributes nearby, target them.
    /// 2. If there are no enemies and the tribute is feeling suicidal, target self.
    /// 3. If there are no enemies nearby, but they exist elsewhere, target no one.
    /// 4. If there are no enemies nearby and no enemies left in the game:
    /// 4a. If loyalty is low, target ally.
    /// 4b. Otherwise, target no one.
    async fn pick_target(&self, mut targets: Vec<Tribute>, living_tributes_count: usize) -> Option<Tribute> {
        // If there are no targets, check if the tribute is feeling suicidal.
        if targets.is_empty() {
            match self.attributes.sanity {
                0..=SANITY_BREAK_LEVEL => {
                    // attempt suicide
                    self.try_log_action(
                        GameOutput::TributeSuicide(self.name.as_str()),
                        "suicide"
                    );
                    Some(self.clone())
                }
                _ => None, // Attack no one
            }
        } else {
            let enemies: Vec<Tribute> = targets
                .iter()
                .filter(|t| t.district != self.district)
                .cloned()
                .collect();

            match enemies.len() {
                0 => { // No enemies, check for a "friend"
                    // If there are two of us in the area
                    if targets.len() == 1 {
                        let target = targets.pop().unwrap();
                        // And we're the only two left in the game
                        if living_tributes_count == 2 {
                            // Kill the other tribute
                            self.try_log_action(
                                GameOutput::TributeBetrayal(self.name.as_str(), target.name.as_str()),
                                "betrayal"
                            );
                            Some(target.clone())
                        } else {
                            // Otherwise, how loyal am I?
                            let loyalty = self.attributes.loyalty as f64 / 100.0;
                            if loyalty < LOYALTY_BREAK_LEVEL {
                                // Kill the other tribute
                                self.try_log_action(
                                    GameOutput::TributeForcedBetrayal(self.name.as_str(), target.name.as_str()),
                                    "forced betrayal"
                                );
                                Some(target.clone())
                            } else {
                                // Otherwise, don't attack
                                self.try_log_action(
                                    GameOutput::NoOneToAttack(self.name.as_str()),
                                    "no one to target"
                                );
                                None
                            }
                        }
                    } else {
                        self.try_log_action(
                            GameOutput::AllAlone(self.name.as_str()),
                            "all alone when trying to find a target"
                        );
                        None
                    }
                },
                1 => Some(enemies.first()?.clone()), // Easy choice
                _ => {
                    let mut rng = SmallRng::from_entropy();
                    let enemy = enemies.choose(&mut rng)?;
                    Some(enemy.clone())
                }
            }
        }
    }

    pub fn set_status(&mut self, status: TributeStatus) { self.status = status; }

    /// Applies statuses to the tribute based on events in the current area.
    fn apply_area_effects(&mut self, area_details: &AreaDetails) {
        for event in &area_details.events {
            match event {
                AreaEvent::Wildfire => self.set_status(TributeStatus::Burned),
                AreaEvent::Flood => self.set_status(TributeStatus::Drowned),
                AreaEvent::Earthquake => self.set_status(TributeStatus::Buried),
                AreaEvent::Avalanche => self.set_status(TributeStatus::Buried),
                AreaEvent::Blizzard => self.set_status(TributeStatus::Frozen),
                AreaEvent::Landslide => self.set_status(TributeStatus::Buried),
                AreaEvent::Heatwave => self.set_status(TributeStatus::Overheated),
            }
        }
    }

    /// Helper to attempt to add a tribute message, logging a warning on failure.
    /// The success of this function does not affect the outcome of the calling method.
    fn try_log_action(&self, game_event_output: impl std::fmt::Display, action_description: &str) {
        let content = format!("{}", game_event_output);

        if let Err(e) = add_tribute_message(
            self.identifier.as_str(),
            self.statistics.game.as_str(),
            content,
        ) {
            tracing::warn!(
                target: "game::tribute",
                "Failed to log action: {}. Error: {}",
                action_description,
                e
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TravelResult {
    Success(Area),
    Failure,
}

#[allow(dead_code)]
fn apply_violence_stress(tribute: &mut Tribute) {
    let kills = tribute.statistics.kills;
    let wins = tribute.statistics.wins;
    let sanity = tribute.attributes.sanity;
    let mut terror = 20.0;

    if kills + wins > 0 {
        terror = (100.0 / (kills + wins) as f64) * (sanity as f64 / 100.0) / 2.0;
    }

    if terror.round() > 0.0 {
        // println!("{}", GameMessage::TributeHorrified(tribute.clone(), terror.round() as u32));
        info!(target: "api::tribute", "{}", GameOutput::TributeHorrified(tribute.name.as_str(), terror.round() as u32));
        tribute.takes_mental_damage(terror.round() as u32);
    }
}

/// Generate attack data for each tribute.
/// Each rolls a d20 to decide a basic attack / defense value.
/// Strength and any weapon are added to the attack roll.
/// Defense and any shield are added to the defense roll.
/// If either roll is more than 1.5x the other, that triggers a "decisive" victory.
fn attack_contest(attacker: &mut Tribute, target: &mut Tribute, rng: &mut impl Rng) -> AttackResult {
    // Get attack roll and strength modifier
    let mut attack_roll: i32 = rng.gen_range(1..=20); // Base roll
    attack_roll += attacker.attributes.strength as i32; // Add strength

    // If the attacker has a weapon, use it
    if let Some(weapon) = attacker.weapons().iter_mut().last() {
        attack_roll += weapon.effect; // Add weapon damage
        weapon.quantity = weapon.quantity.saturating_sub(1);
        if weapon.quantity == 0 {
            attacker.try_log_action(
                GameOutput::WeaponBreak(attacker.name.as_str(), weapon.name.as_str()),
                "weapon break"
            );
            if let Err(err) = attacker.remove_item(weapon) {
                eprintln!("Failed to remove weapon: {}", err);
            }
        }
    }

    // Get defense roll and defense modifier
    let mut defense_roll: i32 = rng.gen_range(1..=20); // Base roll
    defense_roll += target.attributes.defense as i32; // Add defense

    // If the defender has a shield, use it
    if let Some(shield) = target.shields().iter_mut().last() {
        defense_roll += shield.effect; // Add shield defense
        shield.quantity = shield.quantity.saturating_sub(1);
        if shield.quantity == 0 {
            target.try_log_action(
                GameOutput::ShieldBreak(target.name.as_str(), shield.name.as_str()),
                "shield break"
            );
            if let Err(err) = target.remove_item(shield) {
                eprintln!("Failed to remove shield: {}", err);
            };
        }
    }

    // Compare attack vs. defense
    match attack_roll.cmp(&defense_roll) {
        Ordering::Less => { // If the defender wins
            let difference = defense_roll as f64 - (attack_roll as f64 * DECISIVE_WIN_MULTIPLIER);
            if difference > 0.0 {
                // Defender wins significantly
                AttackResult::DefenderWinsDecisively
            } else {
                AttackResult::DefenderWins
            }
        }
        Ordering::Equal => AttackResult::Miss, // If they tie
        Ordering::Greater => { // If the attacker wins
            let difference = attack_roll as f64 - (defense_roll as f64 * DECISIVE_WIN_MULTIPLIER);

            if difference > 0.0 {
                // Attacker wins significantly
                AttackResult::AttackerWinsDecisively
            } else {
                AttackResult::AttackerWins
            }
        }
    }
}

/// Apply the results of a combat encounter.
/// Adjust statistics and log the result.
fn apply_combat_results(
    winner: &mut Tribute,
    loser: &mut Tribute,
    damage_to_loser: u32,
    log_event: GameOutput,
    log_description: &str,
) {
    loser.takes_physical_damage(damage_to_loser);
    loser.statistics.defeats += 1;
    winner.statistics.wins += 1;
    winner.try_log_action(log_event, log_description);
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Statistics {
    /// What day, if any, were they killed?
    pub day_killed: Option<u32>,
    /// Who or what killed them?
    pub killed_by: Option<String>,
    /// How many tributes did they kill?
    pub kills: u32,
    /// How many fights did they win?
    pub wins: u32,
    /// How many fights did they lose?
    pub defeats: u32,
    /// How many fights ended in a draw?
    pub draws: u32,
    /// Which game do these stats relate to?
    pub game: String,
}

/// Update statistics for a pair of tributes based on the attack result
pub fn update_stats(attacker: &mut Tribute, defender: &mut Tribute, result: AttackResult) {
    match result {
        AttackResult::AttackerWins | AttackResult::AttackerWinsDecisively => {
            defender.statistics.defeats += 1;
            attacker.statistics.wins += 1;
        },
        AttackResult::DefenderWins | AttackResult::DefenderWinsDecisively => {
            attacker.statistics.defeats += 1;
            defender.statistics.wins += 1;
        },
        AttackResult::Miss => {
            attacker.statistics.draws += 1;
            defender.statistics.draws += 1;
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attributes {
    /// How much damage can they take?
    pub health: u32,
    /// How much suffering can they handle? Are they still sane?
    pub sanity: u32,
    /// How far can they move before they need a rest?
    pub movement: u32,
    /// How hard do they hit?
    pub strength: u32,
    /// How hard of a hit can they take?
    pub defense: u32,
    /// Will they jump into dangerous situations?
    pub bravery: u32,
    /// Are they a backstabber?
    pub loyalty: u32,
    /// How far can they move each turn?
    pub speed: u32,
    /// How well do they avoid attacks?
    pub dexterity: u32,
    /// How well do they avoid traps?
    pub intelligence: u32,
    /// Can they talk their way out of, or into, things?
    pub persuasion: u32,
    /// Are they likely to get gifts or come out slightly ahead?
    pub luck: u32,
    /// Can other tributes see them?
    pub is_hidden: bool,
}

impl Default for Attributes {
    /// Provides a maxed-out set of Attributes
    fn default() -> Self {
        Self {
            health: 100,
            sanity: 100,
            movement: 100,
            strength: 50,
            defense: 50,
            bravery: 100,
            loyalty: 100,
            speed: 100,
            dexterity: 100,
            intelligence: 100,
            persuasion: 100,
            luck: 100,
            is_hidden: false,
        }
    }
}

impl Attributes {
    /// Provides a randomized set of Attributes
    pub fn new() -> Self {
        let mut rng = SmallRng::from_entropy();

        Self {
            health: rng.gen_range(50..=MAX_HEALTH),
            sanity: rng.gen_range(50..=MAX_SANITY),
            movement: MAX_MOVEMENT,
            strength: rng.gen_range(1..=MAX_STRENGTH),
            defense: rng.gen_range(1..=MAX_DEFENSE),
            bravery: rng.gen_range(1..=MAX_BRAVERY),
            loyalty: rng.gen_range(1..=MAX_LOYALTY),
            speed: rng.gen_range(1..=MAX_SPEED),
            dexterity: rng.gen_range(1..=MAX_DEXTERITY),
            intelligence: rng.gen_range(1..=MAX_INTELLIGENCE),
            persuasion: rng.gen_range(1..=MAX_PERSUASION),
            luck: rng.gen_range(1..=MAX_LUCK),
            is_hidden: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::areas::events::AreaEvent;
    use crate::areas::Area::{Cornucopia, East, North, South, West};
    use crate::areas::{Area, AreaDetails};
    use crate::games::Game;
    use crate::items::{Attribute, Item, ItemType, OwnsItems};
    use crate::threats::animals::Animal;
    use crate::tributes::actions::{Action, AttackOutcome, AttackResult};
    use crate::tributes::statuses::TributeStatus;
    use crate::tributes::{attack_contest, TravelResult, Tribute};
    use rand::prelude::SmallRng;
    use rand::SeedableRng;
    use rstest::{fixture, rstest};
    use std::str::FromStr;

    #[fixture]
    fn tribute() -> Tribute { Tribute::random() }

    #[fixture]
    fn target() -> Tribute { Tribute::random() }

    #[test]
    fn default() {
        let tribute = Tribute::default();
        assert_eq!(tribute.name, "Tribute".to_string());
        assert_eq!(tribute.district, 0);
        assert_eq!(tribute.avatar, None);
    }

    #[test]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), Some("avatar.png".to_string()));
        assert_eq!(tribute.name, "Katniss".to_string());
        assert_eq!(tribute.district, 12);
        assert_eq!(tribute.avatar, Some("avatar.png".to_string()));
        assert_eq!(tribute.status, TributeStatus::Healthy);
    }

    #[test]
    fn random() {
        let tribute = Tribute::random();
        assert!((1u32..=12u32).contains(&tribute.district));
    }

    #[rstest]
    fn add_item(mut tribute: Tribute) {
        let item = Item::new_random_consumable();
        tribute.add_item(item.clone());
        assert_eq!(tribute.items.len(), 1);
        assert_eq!(tribute.items[0], item);
    }

    #[rstest]
    fn has_item(mut tribute: Tribute) {
        let item = Item::new_random_consumable();
        tribute.add_item(item.clone());
        assert!(tribute.has_item(&item));
    }

    #[rstest]
    fn use_item(mut tribute: Tribute) {
        let mut item = Item::new_random_consumable(); // default quantity is 1
        item.quantity = 1;
        tribute.add_item(item.clone());
        tribute.use_item(&item).expect("Failed to use item");
        assert_eq!(tribute.items.len(), 0);
    }

    #[rstest]
    fn use_item_reusable(mut tribute: Tribute) {
        let mut item = Item::new_random_consumable();
        item.quantity = 2;
        tribute.add_item(item.clone());
        assert_eq!(tribute.use_item(&item), Ok(()));
        assert_eq!(tribute.items.len(), 1);
        assert_eq!(tribute.items[0].quantity, 1);
    }

    #[rstest]
    fn takes_physical_damage(mut tribute: Tribute) {
        let hp = tribute.attributes.health.clone();
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, hp - 10);
    }

    #[rstest]
    fn takes_no_physical_damage_when_dead(mut tribute: Tribute) {
        tribute.attributes.health = 0;
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, 0);
    }

    #[rstest]
    fn heals(mut tribute: Tribute) {
        tribute.attributes.health = 10;
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 20);
    }

    #[rstest]
    fn does_not_heal_when_dead(mut tribute: Tribute) {
        tribute.attributes.health = 0;

        tribute.status = TributeStatus::RecentlyDead;
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 0);

        tribute.status = TributeStatus::Dead;
        tribute.heals(10);
        assert_eq!(tribute.attributes.health, 0);
    }

    #[rstest]
    fn takes_mental_damage(mut tribute: Tribute) {
        let mp = tribute.attributes.sanity.clone();
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, mp - 10);
    }

    #[rstest]
    fn takes_no_mental_damage_when_insane(mut tribute: Tribute) {
        tribute.attributes.sanity = 0;
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 0);
    }

    #[rstest]
    fn heals_mental_damage(mut tribute: Tribute) {
        tribute.attributes.sanity = 10;
        tribute.heals_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 20);
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
        tribute.attributes.health = 5;
        tribute.attributes.sanity = 5;
        tribute.long_rests();
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.attributes.health, 10);
        assert_eq!(tribute.attributes.sanity, 10);
    }

    #[rstest]
    fn dies(mut tribute: Tribute) {
        tribute.dies();
        assert_eq!(tribute.attributes.health, 0);
        assert_eq!(tribute.status, TributeStatus::Dead);
        assert!(tribute.items.is_empty());
        assert!(tribute.is_visible());
    }

    #[rstest]
    fn is_alive(mut tribute: Tribute) {
        assert!(tribute.is_alive());
        tribute.dies();
        assert!(!tribute.is_alive());
    }

    #[rstest]
    fn hides_success(mut tribute: Tribute) {
        tribute.attributes.intelligence = 100; // So the hiding is always successful
        tribute.hides();
        assert!(tribute.attributes.is_hidden);
        assert!(!tribute.is_visible());
    }

    #[rstest]
    fn hides_fail(mut tribute: Tribute) {
        tribute.attributes.intelligence = 0;
        tribute.hides();
        assert!(!tribute.attributes.is_hidden);
        assert!(tribute.is_visible());
    }

    #[rstest]
    fn misses_home(mut tribute: Tribute) {
        tribute.attributes.bravery = 2;
        tribute.attributes.sanity = 50;
        tribute.misses_home();
        assert_eq!(tribute.attributes.sanity, 48);

        tribute.attributes.sanity = 20;
        tribute.misses_home();
        assert_eq!(tribute.attributes.sanity, 16);
    }

    #[rstest]
    fn is_visible(mut tribute: Tribute) {
        tribute.attributes.intelligence = 100; // guaranteed hide
        assert!(tribute.is_visible());

        tribute.hides();
        assert!(!tribute.is_visible());
    }

    #[rstest]
    #[tokio::test]
    async fn travels_success(tribute: Tribute) {
        let open_area = AreaDetails::new(Some("Forest".to_string()), Cornucopia);
        let result = tribute.travels(vec![East, South, North, West], None).await;
        assert_eq!(result, TravelResult::Success(Area::from_str(open_area.area.as_str()).unwrap()));
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_no_movement(mut tribute: Tribute) {
        tribute.attributes.movement = 0;
        let result = tribute.travels(vec![], None).await;
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_already_there(mut tribute: Tribute) {
        tribute.area = North;
        let result = tribute.travels(vec![Cornucopia, East, West, South], Some(North)).await;
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_low_movement_no_suggestion(mut tribute: Tribute) {
        tribute.attributes.movement = 5;
        let result = tribute.travels(vec![Cornucopia, East, West, North], None).await;
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_fail_low_movement_suggestion(mut tribute: Tribute) {
        tribute.attributes.movement = 5;
        let result = tribute.travels(vec![Cornucopia, East, West, North], Some(North)).await;
        assert_eq!(result, TravelResult::Failure);
    }

    #[rstest]
    #[tokio::test]
    async fn travels_success_low_movement_suggestion(mut tribute: Tribute) {
        tribute.area = North;
        tribute.attributes.movement = 5;
        let open_area = AreaDetails::new(Some("Forest".to_string()), Cornucopia);
        let result = tribute.travels(vec![East, South], Some(Cornucopia)).await;
        assert_eq!(result, TravelResult::Success(Area::from_str(open_area.area.as_str()).unwrap()));
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
    #[case(TributeStatus::Buried)]
    #[case(TributeStatus::Burned)]
    #[case(TributeStatus::Drowned)]
    #[case(TributeStatus::Infected)]
    fn process_status(mut tribute: Tribute, #[case] status: TributeStatus) {
        let mut game = Game::default();
        game.areas.push(AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia));
        tribute.status = status;
        let clone = tribute.clone();
        let mut rng = SmallRng::from_entropy();

        tribute.process_status(&game.areas[0], &mut rng);
        assert_ne!(clone, tribute);
    }

    #[rstest]
    fn process_status_mauled(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        let mut game = Game::default();
        let bear = Animal::Bear;
        let hp = tribute.attributes.health;

        game.areas.push(AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia));
        tribute.status = TributeStatus::Mauled(bear.clone());

        tribute.process_status(&game.areas[0], &mut rng);
        assert!(hp - bear.damage() >= tribute.attributes.health);
    }

    #[rstest]
    #[case(TributeStatus::Healthy)]
    #[case(TributeStatus::RecentlyDead)]
    #[case(TributeStatus::Dead)]
    fn process_status_no_effect(mut tribute: Tribute, #[case] status: TributeStatus) {
        let mut rng = SmallRng::from_entropy();
        let mut game = Game::default();
        game.areas.push(AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia));
        tribute.status = status;
        let clone = tribute.clone();

        tribute.process_status(&game.areas[0], &mut rng);
        assert_eq!(clone, tribute);
    }

    #[rstest]
    fn process_status_dies(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        let mut game = Game::default();
        game.areas.push(AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia));
        tribute.status = TributeStatus::Wounded;
        tribute.attributes.health = 1;

        tribute.process_status(&game.areas[0], &mut rng);
        assert_eq!(TributeStatus::RecentlyDead, tribute.status);
    }

    #[rstest]
    #[case(AreaEvent::Wildfire, TributeStatus::Burned)]
    #[case(AreaEvent::Flood, TributeStatus::Drowned)]
    #[case(AreaEvent::Earthquake, TributeStatus::Buried)]
    #[case(AreaEvent::Avalanche, TributeStatus::Buried)]
    #[case(AreaEvent::Blizzard, TributeStatus::Frozen)]
    #[case(AreaEvent::Landslide, TributeStatus::Buried)]
    #[case(AreaEvent::Heatwave, TributeStatus::Overheated)]
    fn apply_area_effects(mut tribute: Tribute, #[case] event: AreaEvent, #[case] status: TributeStatus) {
        let mut game = Game::default();
        let mut area_details = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let area = Area::from_str(area_details.area.as_str()).unwrap();
        area_details.events.push(event);
        game.areas.push(area_details.clone());
        tribute.area = area.clone();

        tribute.apply_area_effects(&game.areas[0]);
        assert_eq!(tribute.status, status);
    }

    #[rstest]
    fn process_status_from_area_event(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        let mut game = Game::default();
        let mut area_details = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let area = Area::from_str(area_details.area.as_str()).unwrap();
        let event = AreaEvent::Wildfire;
        area_details.events.push(event);
        game.areas.push(area_details.clone());
        tribute.area = area.clone();

        tribute.process_status(&game.areas[0], &mut rng);
        assert_eq!(tribute.status, TributeStatus::Burned);
    }

    #[rstest]
    #[tokio::test]
    async fn receive_patron_gift(mut tribute: Tribute) {
        let rng = SmallRng::from_entropy();
        tribute.district = 13;
        let gift = tribute.receive_patron_gift(rng).await;
        assert!(gift.is_some());
    }

    #[rstest]
    fn take_action(
        mut tribute: Tribute,
        target: Tribute,
    ) {
        let action = Action::Attack;
        tribute.take_action(&action, Some(&target));
        assert_eq!(tribute.brain.previous_actions.len(), 1);
        assert_eq!(tribute.brain.previous_actions[0].action, action);
        assert_eq!(tribute.brain.previous_actions[0].target, Some(target));
    }

    #[rstest]
    fn take_nearby_item(mut tribute: Tribute) {
        let mut game = Game::default();
        let mut area_details = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let item = Item::new_random_consumable();
        area_details.items.push(item.clone());
        game.areas.push(area_details.clone());
        assert_eq!(game.areas.len(), 1);
        assert_eq!(tribute.items.len(), 0);

        tribute.take_nearby_item(&mut game.areas[0]);
        assert_eq!(tribute.items.len(), 1);
        assert_eq!(tribute.items[0], item);
        assert_eq!(game.areas.len(), 1);
        assert_eq!(game.areas.get(0).unwrap().items.len(), 0);
    }

    #[rstest]
    fn use_consumable(mut tribute: Tribute) {
        tribute.attributes.health = 10;
        let clone = tribute.clone();
        let health_potion = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        tribute.items.push(health_potion.clone());
        assert!(tribute.use_consumable(health_potion.clone()).is_ok());
        assert_eq!(tribute.items.len(), 0);
        assert_ne!(clone, tribute);
        assert_eq!(clone.attributes.health + 1, tribute.attributes.health);
    }

    #[rstest]
    fn use_consumable_fail_item_not_found(mut tribute: Tribute) {
        let health_potion = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        assert!(tribute.use_consumable(health_potion.clone()).is_err());
    }

    #[rstest]
    fn use_consumable_fail_item_not_available(mut tribute: Tribute) {
        let health_potion = Item::new("Health Potion", ItemType::Consumable, 0, Attribute::Health, 1);
        assert!(tribute.use_consumable(health_potion.clone()).is_err());
    }

    #[rstest]
    fn available_items(mut tribute: Tribute) {
        let item1 = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        let item2 = Item::new("Sword", ItemType::Weapon, 0, Attribute::Strength, 5);
        tribute.items.push(item1.clone());
        tribute.items.push(item2.clone());
        assert_eq!(tribute.available_items().len(), 1);
    }

    #[rstest]
    fn weapons(mut tribute: Tribute) {
        let item1 = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        let item2 = Item::new("Sword", ItemType::Weapon, 1, Attribute::Strength, 5);
        tribute.items.push(item1.clone());
        tribute.items.push(item2.clone());
        assert_eq!(tribute.weapons().len(), 1);
    }

    #[rstest]
    fn shields(mut tribute: Tribute) {
        let item1 = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        let item2 = Item::new("Shield", ItemType::Weapon, 1, Attribute::Defense, 5);
        tribute.items.push(item1.clone());
        tribute.items.push(item2.clone());
        assert_eq!(tribute.shields().len(), 1);
    }

    #[rstest]
    fn consumables(mut tribute: Tribute) {
        let item1 = Item::new("Health Potion", ItemType::Consumable, 1, Attribute::Health, 1);
        let item2 = Item::new("Sword", ItemType::Weapon, 1, Attribute::Strength, 5);
        tribute.items.push(item1.clone());
        tribute.items.push(item2.clone());
        assert_eq!(tribute.consumables().len(), 1);
    }

    /// The tributes are from different districts and are in the same area.
    #[rstest]
    #[tokio::test]
    async fn pick_target(
        mut tribute: Tribute,
        mut target: Tribute
    ) {
        let mut game = Game::default();
        let cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        game.areas.push(cornucopia.clone());

        tribute.district = 11;
        tribute.area = Cornucopia;

        target.district = 10;
        target.area = Cornucopia;

        game.tributes.extend_from_slice([tribute.clone(), target.clone()].as_ref());

        let target = tribute.pick_target(vec![target.clone()], 2).await;

        assert_eq!(target, target.clone());
    }

    /// The actor is the only tribute in the area.
    /// Their sanity is low, so they should attempt suicide.
    #[tokio::test]
    async fn pick_target_suicide() {
        let mut game = Game::default();
        let cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        game.areas.push(cornucopia.clone());

        let mut katniss = Tribute::new("Katniss".to_string(), Some(1), None);
        katniss.area = Cornucopia;
        katniss.attributes.sanity = 5;

        game.tributes.push(katniss.clone());

        let target = katniss.pick_target(vec![], 1).await;

        assert_eq!(target, Some(katniss.clone()));
    }

    /// No enemies are in the current area, only allies. Other enemies exist elsewhere.
    /// The actor's loyalty is high, so they should NOT attack their ally.
    #[tokio::test]
    async fn pick_target_no_enemies_not_final_two() {
        let mut game = Game::default();
        let cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let north = AreaDetails::new(Some("North".to_string()), North);
        game.areas.push(cornucopia.clone());
        game.areas.push(north.clone());

        let mut katniss = Tribute::new("Katniss".to_string(), Some(1), None);
        katniss.area = Cornucopia;
        katniss.attributes.loyalty = 95;

        let mut peeta = Tribute::new("Peeta".to_string(), Some(1), None);
        peeta.area = Cornucopia;

        let mut rue = Tribute::new("Rue".to_string(), Some(2), None);
        rue.area = North;

        game.tributes.extend_from_slice([katniss.clone(), peeta.clone(), rue.clone()].as_ref());

        let target = katniss.pick_target(vec![peeta], 3).await;

        assert_eq!(target, None);
    }

    /// No enemies are in the current area, only allies. Other enemies exist elsewhere.
    /// The actor's loyalty is low, so they should attack their ally.
    #[tokio::test]
    async fn pick_target_no_enemies_not_final_two_disloyal() {
        let mut game = Game::default();
        let cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let north = AreaDetails::new(Some("North".to_string()), North);
        game.areas.push(cornucopia.clone());
        game.areas.push(north.clone());

        let mut katniss = Tribute::new("Katniss".to_string(), Some(1), None);
        katniss.area = Cornucopia;
        katniss.attributes.loyalty = 2;

        let mut peeta = Tribute::new("Peeta".to_string(), Some(1), None);
        peeta.area = Cornucopia;

        let mut rue = Tribute::new("Rue".to_string(), Some(2), None);
        rue.area = North;

        game.tributes.extend_from_slice([katniss.clone(), peeta.clone(), rue.clone()].as_ref());

        let target = katniss.pick_target(vec![peeta.clone()], 3).await;

        assert_eq!(target, Some(peeta));
    }

    /// No enemies are in the current area, only allies. No other enemies exist.
    /// The actor should attack their ally.
    #[tokio::test]
    async fn pick_target_no_enemies_final_two() {
        let mut game = Game::default();
        let cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        game.areas.push(cornucopia.clone());

        let mut katniss = Tribute::new("Katniss".to_string(), Some(1), None);
        katniss.area = Cornucopia;

        let mut peeta = Tribute::new("Peeta".to_string(), Some(1), None);
        peeta.area = Cornucopia;

        game.tributes.extend_from_slice([katniss.clone(), peeta.clone()].as_ref());

        let target = katniss.pick_target(vec![peeta.clone()], 2).await;

        assert_eq!(target, Some(peeta));
    }

    #[test]
    fn attack_contest_win() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 10;
        target.attributes.defense = 5;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attack_contest(&mut attacker, &mut target, &mut seeded_rng);
        assert_eq!(result, AttackResult::AttackerWins);
    }

    #[test]
    fn attack_contest_win_decisively() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 0;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attack_contest(&mut attacker, &mut target, &mut seeded_rng);
        assert_eq!(result, AttackResult::AttackerWinsDecisively);
    }

    #[test]
    fn attack_contest_lose() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 15;
        target.attributes.defense = 20;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attack_contest(&mut attacker, &mut target, &mut seeded_rng);
        assert_eq!(result, AttackResult::DefenderWins);
    }

    #[test]
    fn attack_contest_lose_decisively() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 1;
        target.attributes.defense = 20;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attack_contest(&mut attacker, &mut target, &mut seeded_rng);
        assert_eq!(result, AttackResult::DefenderWinsDecisively);
    }

    #[test]
    fn attack_contest_draw() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 23; // Magic number to make the final scores even
        target.attributes.defense = 20;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attack_contest(&mut attacker, &mut target, &mut seeded_rng);
        assert_eq!(result, AttackResult::Miss);
    }

    #[test]
    fn attacks_self() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = attacker.clone();
        let mut rng = SmallRng::from_entropy();

        let outcome = attacker.attacks(&mut target, &mut rng);
        assert_eq!(outcome, AttackOutcome::Wound(attacker, target));
    }

    #[test]
    fn attacks_self_suicide() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 100;
        let mut target = attacker.clone();
        let mut rng = SmallRng::from_entropy();

        let outcome = attacker.attacks(&mut target, &mut rng);
        assert_eq!(outcome, AttackOutcome::Kill(attacker, target));
    }

    #[test]
    fn attacks_wound() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);

        attacker.attributes.strength = 25;
        target.attributes.defense = 20;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attacker.attacks(&mut target, &mut seeded_rng);
        assert_eq!(result, AttackOutcome::Wound(attacker.clone(), target.clone()));
        assert_eq!(attacker.statistics.wins, 1);
        assert_eq!(target.statistics.defeats, 1);
    }

    #[test]
    fn attacks_kill() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);
        target.attributes.health = 1;

        attacker.attributes.strength = 25;
        target.attributes.defense = 2;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attacker.attacks(&mut target, &mut seeded_rng);

        assert_eq!(result, AttackOutcome::Kill(attacker.clone(), target.clone()));
        assert_eq!(target.status, TributeStatus::RecentlyDead);
        assert_eq!(target.statistics.killed_by, Some(attacker.name));
        assert_eq!(target.status, TributeStatus::RecentlyDead);
        assert_eq!(target.attributes.health, 0);
    }

    #[test]
    fn attacks_miss() {
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        let mut target = Tribute::new("Peeta".to_string(), None, None);
        target.attributes.health = 1;

        attacker.attributes.strength = 23;
        target.attributes.defense = 20;

        let mut seeded_rng = SmallRng::seed_from_u64(42);

        let result = attacker.attacks(&mut target, &mut seeded_rng);
        assert_eq!(result, AttackOutcome::Miss(attacker.clone(), target.clone()));
        assert_eq!(attacker.statistics.draws, 1);
        assert_eq!(target.statistics.draws, 1);
    }

    #[tokio::test]
    async fn do_day_night_happy_path() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let mut tribute1 = Tribute::new("Katniss".to_string(), None, None);
        tribute1.area = Cornucopia;
        let clone = tribute1.clone();
        let mut tribute2 = Tribute::new("Peeta".to_string(), None, None);
        tribute2.area = Cornucopia;

        let mut game = Game::default();
        game.areas.push(cornucopia.clone());

        tribute1.do_day_night(
            None,
            None,
            true,
            &mut cornucopia,
            vec![],
            1,
            vec![tribute2],
            24,
            &mut rng
        ).await;

        assert_eq!(tribute1.brain.previous_actions.len(), 1);
        assert_ne!(clone, tribute1);
    }

    #[tokio::test]
    async fn do_day_night_dead() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut cornucopia = AreaDetails::new(Some("Cornucopia".to_string()), Cornucopia);
        let mut tribute1 = Tribute::new("Katniss".to_string(), None, None);
        tribute1.area = Cornucopia;
        tribute1.status = TributeStatus::RecentlyDead;
        let clone = tribute1.clone();

        let mut game = Game::default();
        game.areas.push(cornucopia.clone());

        tribute1.do_day_night(
            None,
            None,
            true,
            &mut cornucopia,
            vec![],
            1,
            vec![],
            2,
            &mut rng
        ).await;

        assert_eq!(tribute1.brain.previous_actions.len(), 0);
        assert_eq!(clone, tribute1);
    }
}

#[cfg(test)]
mod statistics {
    use super::Statistics;

    #[test]
    fn default() {
        let stats = Statistics::default();
        assert_eq!(stats.day_killed, None);
        assert_eq!(stats.killed_by, None);
        assert_eq!(stats.kills, 0);
        assert_eq!(stats.wins, 0);
        assert_eq!(stats.defeats, 0);
        assert_eq!(stats.draws, 0);
        assert_eq!(stats.game, "".to_string());
    }

}

#[cfg(test)]
mod attributes {
    use super::Attributes;

    #[test]
    fn default() {
        let attributes = Attributes::default();
        assert_eq!(attributes.health, 100);
        assert_eq!(attributes.sanity, 100);
        assert_eq!(attributes.movement, 100);
        assert_eq!(attributes.strength, 50);
        assert_eq!(attributes.defense, 50);
        assert_eq!(attributes.bravery, 100);
        assert_eq!(attributes.loyalty, 100);
        assert_eq!(attributes.speed, 100);
        assert_eq!(attributes.dexterity, 100);
        assert_eq!(attributes.intelligence, 100);
        assert_eq!(attributes.persuasion, 100);
        assert_eq!(attributes.luck, 100);
        assert!(!attributes.is_hidden);
    }

    #[test]
    fn new() {
        let attributes = Attributes::new();
        assert!(attributes.health >= 50 && attributes.health <= 100);
        assert!(attributes.sanity >= 50 && attributes.sanity <= 100);
        assert_eq!(attributes.movement, 100);
        assert!(attributes.strength >= 1 && attributes.strength <= 50);
        assert!(attributes.defense >= 1 && attributes.defense <= 50);
        assert!(attributes.bravery >= 1 && attributes.bravery <= 100);
        assert!(attributes.loyalty >= 1 && attributes.loyalty <= 100);
        assert!(attributes.speed >= 1 && attributes.speed <= 100);
        assert!(attributes.dexterity >= 1 && attributes.dexterity <= 100);
        assert!(attributes.intelligence >= 1 && attributes.intelligence <= 100);
        assert!(attributes.persuasion >= 1 && attributes.persuasion <= 100);
        assert!(attributes.luck >= 1 && attributes.luck <= 100);
        assert!(!attributes.is_hidden);
    }
}
