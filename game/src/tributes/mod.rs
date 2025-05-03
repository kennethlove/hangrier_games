pub mod actions;
pub mod brains;
pub mod events;
pub mod statuses;

use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::games::Game;
use crate::items::OwnsItems;
use crate::items::{Attribute, Item};
use crate::messages::add_tribute_message;
use crate::output::GameOutput;
use crate::tributes::events::TributeEvent;
use actions::{Action, AttackOutcome, AttackResult, TributeAction};
use brains::Brain;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::prelude::*;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use statuses::TributeStatus;
use std::cmp::{Ordering, PartialEq};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

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

    fn use_item(&mut self, item: Item) -> Option<Item> {
        let used_item = self.items.iter_mut().find(|i| *i == &item);

        if let Some(used_item) = used_item {
            used_item.quantity = used_item.quantity.saturating_sub(1);
            if used_item.quantity >= 1 {
                Some(used_item.clone())
            } else {
                self.remove_item(item.clone());
                None
            }
        } else { None }
    }

    fn remove_item(&mut self, item: Item) { self.items.retain(|i| i != &item); }
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

    /// Restores health.
    fn heals(&mut self, health: u32) {
        self.attributes.health = std::cmp::min(100, self.attributes.health + health);
    }

    /// Restores mental health.
    fn heals_mental_damage(&mut self, health: u32) {
        self.attributes.sanity = std::cmp::min(100, self.attributes.sanity + health);
    }

    /// Restores movement.
    fn short_rests(&mut self) { self.attributes.movement = 100; }

    /// Restores movement, some health, and some sanity
    fn long_rests(&mut self) {
        self.short_rests();
        self.heals(5);
        self.heals_mental_damage(5);
    }

    /// Marks the tribute as dead and reveals them.
    pub fn dies(&mut self) {
        self.status = TributeStatus::Dead;
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
    fn hides(&mut self) { self.attributes.is_hidden = true; }

    /// Tribute is lonely/homesick/etc., loses some sanity.
    fn suffers(&mut self) {
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
    async fn attacks(&mut self, target: &mut Tribute) -> AttackOutcome {
        // Is the tribute attempting suicide?
        if self == target {
            add_tribute_message(
                &self.identifier,
                &self.statistics.game,
                format!("{}", GameOutput::TributeSelfHarm(self.clone()))
            ).expect("");

            // Attack always succeeds
            target.takes_physical_damage(self.attributes.strength);

            add_tribute_message(
                &self.identifier,
                &self.statistics.game,
                format!("{}", GameOutput::TributeAttackWin(self.clone(), target.clone()))
            ).expect("");

            if target.attributes.health > 0 {
                add_tribute_message(
                    &self.identifier,
                    &self.statistics.game,
                    format!("{}", GameOutput::TributeAttackWound(self.clone(), target.clone()))
                ).expect("");
                return AttackOutcome::Wound(self.clone(), target.clone());
            }
        }

        // `self` is the attacker
        match attack_contest(self, target).await {
            AttackResult::AttackerWins => {
                target.takes_physical_damage(self.attributes.strength);
                target.statistics.defeats += 1;
                self.statistics.wins += 1;

                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeAttackWin(self.clone(), target.clone()))
                ).expect("");

                if target.attributes.health > 0 {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeAttackWound(self.clone(), target.clone()))
                    ).expect("");
                    return AttackOutcome::Wound(self.clone(), target.clone());
                }
            }
            AttackResult::AttackerWinsDecisively => {
                // Take double damage
                target.takes_physical_damage(self.attributes.strength * 2);
                target.statistics.defeats += 1;
                self.statistics.wins += 1;

                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeAttackWinExtra(self.clone(), target.clone()))
                ).expect("");

                if target.attributes.health > 0 {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeAttackWound(self.clone(), target.clone()))
                    ).expect("");
                    return AttackOutcome::Wound(self.clone(), target.clone());
                }
            }
            AttackResult::DefenderWins => {
                self.takes_physical_damage(target.attributes.strength);
                self.statistics.defeats += 1;
                target.statistics.wins += 1;

                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeAttackLose(self.clone(), target.clone()))
                ).expect("");

                if self.attributes.health > 0 {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeAttackWound(target.clone(), self.clone()))
                    ).expect("");
                    return AttackOutcome::Wound(target.clone(), self.clone());
                }
            }
            AttackResult::DefenderWinsDecisively => {
                self.takes_physical_damage(target.attributes.strength * 2);
                self.statistics.defeats += 1;
                target.statistics.wins += 1;


                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeAttackLoseExtra(self.clone(), target.clone()))
                ).expect("");

                if self.attributes.health > 0 {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeAttackWound(target.clone(), self.clone()))
                    ).expect("");
                    return AttackOutcome::Wound(target.clone(), self.clone());
                }
            }
            AttackResult::Miss => {
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeAttackMiss(self.clone(), target.clone()))
                ).expect("");
                self.statistics.draws += 1;
                target.statistics.draws += 1;

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        if self.attributes.health == 0 {
            // Attacker was killed by target
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeAttackDied(self.clone(), target.clone()))
            ).expect("");

            self.statistics.killed_by = Some(target.name.clone());
            self.status = TributeStatus::RecentlyDead;
            AttackOutcome::Kill(target.clone(), self.clone())
        } else if target.attributes.health == 0 {
            // Target was killed by attacker
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeAttackSuccessKill(self.clone(), target.clone()))
            ).expect("");

            target.statistics.killed_by = Some(self.name.clone());
            target.status = TributeStatus::RecentlyDead;
            AttackOutcome::Kill(self.clone(), target.clone())
        } else {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeAttackMiss(self.clone(), target.clone()))
            ).expect("");
            AttackOutcome::Miss(self.clone(), target.clone())
        }
    }

    pub fn is_visible(&self) -> bool {
        let is_hidden = self.attributes.is_hidden;
        if is_hidden {
            let mut rng = SmallRng::from_entropy();
            !rng.gen_bool(self.attributes.intelligence as f64 / 100.0)
        } else {
            true
        }
    }

    async fn travels(&self, closed_areas: Vec<Area>, suggested_area: Option<Area>) -> TravelResult {
        let mut rng = SmallRng::from_entropy();
        let area = self.area.clone();
        let mut new_area: Option<Area> = None;

        if let Some(suggestion) = suggested_area {
            if closed_areas.contains(&suggestion) {
                new_area = None;
            } else {
                new_area = Some(suggestion);
            }
        }

        if new_area.is_some() && new_area == Some(area.clone()) {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeTravelAlreadyThere(self.clone(), new_area.clone().unwrap())),
            ).expect("");
            return TravelResult::Failure;
        }

        let handle_suggested_area = async || -> TravelResult {
            if let Some(new_area) = new_area {
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeTravel(self.clone(), area.clone(), new_area.clone())),
                ).expect("");
                return TravelResult::Success(new_area);
            }
            TravelResult::Failure
        };

        match self.attributes.movement {
            // No movement left, can't move
            0 => {
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeTravelTooTired(self.clone(), area.clone())),
                ).expect("");
                TravelResult::Failure
            }
            // Low movement, can only move to suggested area
            1..=10 => match handle_suggested_area().await {
                TravelResult::Success(area) => TravelResult::Success(area),
                TravelResult::Failure => {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeTravelTooTired(self.clone(), area.clone())),
                    ).expect("");
                    TravelResult::Failure
                }
            },
            // High movement, can move to any open neighbor or the suggested area
            _ => {
                if let TravelResult::Success(area) = handle_suggested_area().await {
                    return TravelResult::Success(area);
                }

                // let neighbors = area.clone().unwrap().neighbors;
                let neighbors: Vec<Area> = area.clone().neighbors();
                for _area in &neighbors {
                    // If the tribute has more loyalty than not
                    if self.attributes.loyalty >= 50 {
                        // TODO: revisit this
                        // If a neighboring area has a living district-mate
                        // if area
                        //     .living_tributes()
                        //     .iter()
                        //     .filter(|t| t.district == self.district)
                        //     .count()
                        //     > 0
                        // {
                        //     println!("{}", GameMessage::TributeTravelFollow(self.clone(), area.clone()));
                        //     return TravelResult::Success(area.clone());
                        // }
                    }
                }

                let mut count = 0;
                let new_area = loop {
                    let new_area = neighbors.choose(&mut rng).unwrap();
                    if new_area == &area.clone() || closed_areas.contains(new_area) {
                        count += 1;

                        if count == 10 {
                            add_tribute_message(
                                self.identifier.as_str(),
                                self.statistics.game.as_str(),
                                format!("{}", GameOutput::TributeTravelStay(self.clone(), area.clone())),
                            ).expect("");
                            return TravelResult::Success(area.clone());
                        }
                        continue;
                    }
                    break new_area.clone();
                };
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeTravel(self.clone(), area.clone(), new_area.clone())),
                ).expect("");
                TravelResult::Success(new_area)
            }
        }
    }

    fn process_status(&mut self, game: &Game) {
        // First, apply any area events for the current area
        self.apply_area_effects(game);

        match self.status.clone() {
            TributeStatus::Wounded => {
                self.takes_physical_damage(1);
            }
            TributeStatus::Sick => {
                self.attributes.strength = std::cmp::max(1, self.attributes.strength - 1);
                self.attributes.speed = std::cmp::max(1, self.attributes.speed - 1);
            }
            TributeStatus::Electrocuted => {
                self.takes_physical_damage(20);
            }
            TributeStatus::Frozen => {
                self.attributes.speed = std::cmp::max(1, self.attributes.speed - 1);
            }
            TributeStatus::Overheated => {
                self.attributes.speed = std::cmp::max(1, self.attributes.speed - 1);
            }
            TributeStatus::Dehydrated => {
                self.attributes.strength = std::cmp::max(1, self.attributes.strength - 1);
            }
            TributeStatus::Starving => {
                self.attributes.strength = std::cmp::max(1, self.attributes.strength - 1);
            }
            TributeStatus::Poisoned => {
                self.takes_mental_damage(5);
            }
            TributeStatus::Broken => {
                // die roll for which bone breaks
                let bone = thread_rng().gen_range(0..4);
                match bone {
                    0 => {
                        // Leg
                        self.attributes.speed = std::cmp::max(1, self.attributes.speed - 5);
                    }
                    1 => {
                        // Arm
                        self.attributes.strength = std::cmp::max(1, self.attributes.strength - 5);
                    }
                    2 => {
                        // Skull
                        self.attributes.intelligence =
                            std::cmp::max(1, self.attributes.intelligence - 5);
                    }
                    _ => {
                        // Rib
                        self.attributes.dexterity = std::cmp::max(1, self.attributes.dexterity - 5);
                    }
                }
            }
            TributeStatus::Infected => {
                self.takes_physical_damage(2);
                self.takes_mental_damage(2);
            }
            TributeStatus::Drowned => {
                self.takes_physical_damage(2);
                self.takes_mental_damage(2);
            }
            TributeStatus::Mauled(animal) => {
                let number_of_animals = thread_rng().gen_range(2..=5);
                let damage = animal.damage() * number_of_animals;
                self.takes_physical_damage(damage as u32);
            }
            TributeStatus::Burned => {
                self.takes_physical_damage(5);
            }
            _ => {}
        }

        self.events.clear();

        if self.attributes.health == 0 {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeDiesFromStatus(self.clone(), self.status.clone())),
            ).expect("");
            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

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
                format!("{}", GameOutput::TributeDiesFromTributeEvent(self.clone(), tribute_event.clone())),
            ).expect("");

            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    // Problem seems to be in here
    pub async fn do_day_night(
        &mut self,
        suggested_action: Option<Action>,
        probability: Option<f64>,
        day: bool,
        game: &mut Game,
    ) -> Tribute {

        // Tribute is already dead, do nothing.
        if !self.is_alive() {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeAlreadyDead(self.clone())),
            ).expect("");
        }

        // Update the tribute based on the period's events.
        self.process_status(game);

        // Tribute died to the period's events.
        if self.status == TributeStatus::RecentlyDead || self.attributes.health == 0 {
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::TributeDead(self.clone())),
            ).expect("");
        }

        let areas = game.areas.clone();
        let closed_areas: Vec<AreaDetails> = areas.clone().iter()
            .filter(|a| !a.events.is_empty())
            .cloned().collect();

        let number_of_nearby_tributes: usize = game.living_tributes().iter()
            .filter(|t| t.area == self.area)
            .collect::<Vec<_>>()
            .len();

        // Any generous patrons this round?
        if let Some(gift) = self.receive_patron_gift().await {
            self.add_item(gift.clone());
            add_tribute_message(
                self.identifier.as_str(),
                self.statistics.game.as_str(),
                format!("{}", GameOutput::SponsorGift(self.clone(), gift.clone())),
            ).expect("");
        }

        // Nighttime terror
        if !day && self.is_alive() { self.suffers(); }

        if let Some(action) = suggested_action {
            self.brain.set_preferred_action(action, probability.unwrap());
        }

        let mut brain = self.brain.clone();
        let action = brain.act(self, number_of_nearby_tributes);

        match &action {
            Action::Move(area) => match self.travels(
                closed_areas
                    .into_iter()
                    .map(|ca| Area::from_str(&ca.area).unwrap())
                    .collect::<Vec<Area>>(),
                area.clone(),
            ).await {
                TravelResult::Success(area) => {
                    self.area = area.clone();
                    // self.clone().game.unwrap().move_tribute(&self, area);
                }
                TravelResult::Failure => {
                    self.short_rests();
                }
            },
            Action::Hide => {
                self.hides();
                self.take_action(&action, None);
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeHide(self.clone())),
                ).expect("");
            }
            Action::Rest | Action::None => {
                self.long_rests();
                self.take_action(&action, None);
                add_tribute_message(
                    self.identifier.as_str(),
                    self.statistics.game.as_str(),
                    format!("{}", GameOutput::TributeLongRest(self.clone())),
                ).expect("");
            }
            Action::Attack => {
                if let Some(mut target) = self.pick_target(game).await {
                    if target.is_visible() {
                        if let AttackOutcome::Kill(mut attacker, mut target) =
                            self.attacks(&mut target).await
                        {
                            if attacker.attributes.health == 0 {
                                attacker.dies();
                            }
                            if target.attributes.health == 0 {
                                target.dies();
                            }
                            if attacker.identifier == target.identifier {
                                attacker.attributes.health = target.attributes.health;
                                attacker.statistics.day_killed = target.statistics.day_killed;
                                attacker.statistics.killed_by = target.statistics.killed_by.clone();
                                attacker.status = target.status.clone();
                                // return target;
                            }
                        }
                        self.take_action(&action, Some(&target));
                    } else {
                        add_tribute_message(
                            self.identifier.as_str(),
                            self.statistics.game.as_str(),
                            format!("{}", GameOutput::TributeAttackHidden(self.clone(), target.clone())),
                        ).expect("");
                        self.take_action(&Action::Attack, None);
                    }
                }
            }
            Action::TakeItem => {
                info!(target: "api", "Taking item");
                if let Some(item) = self.take_nearby_item(game) {
                    add_tribute_message(
                        self.identifier.as_str(),
                        self.statistics.game.as_str(),
                        format!("{}", GameOutput::TributeTakeItem(self.clone(), item.clone())),
                    ).expect("");
                    self.take_action(&action, None);
                }
            }
            Action::UseItem(None) => {
                // Get consumable items
                let mut items = self.consumable_items();
                if items.is_empty() {
                    self.long_rests();
                    self.take_action(&Action::Rest, None);
                } else {
                    // Use random item
                    let item = items.choose_mut(&mut thread_rng()).unwrap();
                    match self.use_consumable(item.clone()) {
                        true => {
                            add_tribute_message(
                                self.identifier.as_str(),
                                self.statistics.game.as_str(),
                                format!("{}", GameOutput::TributeUseItem(self.clone(), item.clone())),
                            ).expect("");
                            self.use_item(item.clone());
                            info!(target: "api", "true, Items: {:?}", &self.items);
                            self.take_action(&action, None);
                        }
                        false => {
                            add_tribute_message(
                                self.identifier.as_str(),
                                self.statistics.game.as_str(),
                                format!("{}", GameOutput::TributeCannotUseItem(self.clone(), item.clone())),
                            ).expect("");
                            info!(target: "api", "false, Items: {:?}", &self.items);
                            self.short_rests();
                            self.take_action(&Action::Rest, None);
                        }
                    };
                }
            }
            Action::UseItem(item) => {
                info!(target: "api", "Using item");
                let items = self.consumable_items();
                if let Some(item) = item {
                    if items.contains(item) {
                        match self.use_consumable(item.clone()) {
                            true => {
                                add_tribute_message(
                                    self.identifier.as_str(),
                                    self.statistics.game.as_str(),
                                    format!("{}", GameOutput::TributeUseItem(self.clone(), item.clone())),
                                ).expect("");
                                self.use_item(item.clone());
                                self.take_action(&action, None);
                            }
                            false => {
                                add_tribute_message(
                                    self.identifier.as_str(),
                                    self.statistics.game.as_str(),
                                    format!("{}", GameOutput::TributeCannotUseItem(self.clone(), item.clone())),
                                ).expect("");
                                self.short_rests();
                                self.take_action(&Action::Rest, None);
                            }
                        };
                    }
                }
            }
        }
        self.clone()
    }

    /// Receive a patron gift, broken down by district
    async fn receive_patron_gift(&mut self) -> Option<Item> {
        // Gift from patrons?
        let chance = match self.district {
            1 | 2 => 1.0 / 10.0,
            3 | 4 => 1.0 / 15.0,
            5 | 6 => 1.0 / 20.0,
            7 | 8 => 1.0 / 25.0,
            9 | 10 => 1.0 / 30.0,
            _ => 1.0 / 50.0,
        };

        if SmallRng::from_entropy().gen_bool(chance) { Some(Item::new_random_consumable()) } else { None }
    }

    /// Save the tribute's latest action
    fn take_action(&mut self, action: &Action, target: Option<&Tribute>) {
        self.brain
            .previous_actions
            .push(TributeAction::new(action.clone(), target.cloned()));
    }

    /// Take item from area
    fn take_nearby_item(&mut self, game: &mut Game) -> Option<Item> {
        let mut rng = SmallRng::from_entropy();
        let area_index = game.areas.iter().position(|a| {
            a.area == self.area.to_string()
        }).expect("Area not found");
        let mut area_details = game.areas.swap_remove(area_index);

        let items = area_details.items.clone();
        if items.is_empty() {
            game.areas.push(area_details);
            None
        } else {
            let item = items.choose(&mut rng).unwrap().clone();
            if let Some(item) = area_details.use_item(item.clone()) {
                info!(target: "api", "Taking nearby item: {:?}", item);
                self.add_item(item.clone());

                game.areas.push(area_details);

                return Some(item.clone());
            }
            game.areas.push(area_details);
            None
        }
    }

    /// Use consumable item from inventory
    fn use_consumable(&mut self, chosen_item: Item) -> bool {
        let items = self.consumable_items();

        #[allow(unused_assignments)]
        let mut item = items.iter().last().unwrap().clone();
        // If the tribute has the item...
        if let Some(selected_item) = items
            .iter()
            .filter(|i| i.identifier == chosen_item.identifier)
            .next_back()
        {
            // select it
            item = selected_item.clone();
        } else {
            // otherwise, quit because you can't use an item you don't have
            return false;
        }

        if self.use_item(item.clone()).is_none() {
            return false;
        }

        // Apply item effect
        match item.attribute {
            Attribute::Health => {
                self.heals(item.effect as u32);
            }
            Attribute::Sanity => {
                self.heals_mental_damage(item.effect as u32);
            }
            Attribute::Movement => {
                self.attributes.movement =
                    std::cmp::min(100, self.attributes.movement as i32 + item.effect) as u32;
            }
            Attribute::Bravery => {
                self.attributes.bravery =
                    std::cmp::min(100, self.attributes.bravery as i32 + item.effect) as u32;
            }
            Attribute::Speed => {
                self.attributes.speed =
                    std::cmp::min(100, self.attributes.speed as i32 + item.effect) as u32;
            }
            Attribute::Strength => {
                self.attributes.strength =
                    std::cmp::min(50, self.attributes.strength as i32 + item.effect) as u32;
            }
            _ => (),
        }

        true
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
    fn defensive_items(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_defensive())
            .cloned()
            .collect()
    }

    /// Which items are marked as consumable?
    pub fn consumable_items(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_consumable())
            .cloned()
            .collect()
    }

    /// Pick an appropriate target from nearby tributes
    async fn pick_target(&self, game: &mut Game) -> Option<Tribute> {
        let tributes: Vec<Tribute> = game
            .living_tributes()
            .iter()
            .filter(|t| t.area == self.area)
            .cloned()
            .collect();

        match tributes.len() {
            0 => {
                // there are no other targets
                match self.attributes.sanity {
                    0..=9 => {
                        // attempt suicide
                        add_tribute_message(
                            self.identifier.as_str(),
                            self.statistics.game.as_str(),
                            format!("{}", GameOutput::TributeSuicide(self.clone())),
                        ).expect("");
                        Some(self.clone())
                    }
                    _ => None, // Attack no one
                }
            }
            _ => {
                // there ARE targets
                let enemies: Vec<Tribute> = tributes
                    .iter()
                    .filter(|t| t.district != self.district && t.is_visible())
                    .cloned()
                    .collect();

                match enemies.len() {
                    0 => None,                           // No enemies means no attack
                    1 => Some(enemies.first()?.clone()), // Easy choice
                    _ => {
                        let mut rng = SmallRng::from_entropy();
                        let enemy = enemies.choose(&mut rng)?;
                        Some(enemy.clone())
                    }
                }
            }
        }
    }

    pub fn set_status(&mut self, status: TributeStatus) { self.status = status; }

    fn apply_area_effects(&mut self, game: &Game) {
        let area_details = game.areas.iter()
            .find(|a| a.area == self.area.to_string())
            .expect("Area not found");

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
}

#[derive(Debug)]
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
        info!(target: "api::tribute", "{}", GameOutput::TributeHorrified(tribute.clone(), terror.round() as u32));
        tribute.takes_mental_damage(terror.round() as u32);
    }
}

/// Generate attack data for each tribute.
/// Each rolls a d20 to decide basic attack/defense value.
/// Strength and any weapon are added to the attack roll.
/// Defense and any shield are added to the defense roll.
/// If eiter roll is more than 1.5x the other, that triggers a "decisive" victory.
async fn attack_contest(attacker: &mut Tribute, target: &Tribute) -> AttackResult {
    // Get attack roll + strength modifier
    let mut attack_roll: i32 = thread_rng().gen_range(1..=20); // Base roll
    attack_roll += attacker.attributes.strength as i32; // Add strength

    // If the attacker has a weapon, use it
    if let Some(weapon) = attacker.weapons().iter_mut().last() {
        attack_roll += weapon.effect; // Add weapon damage
        weapon.quantity = weapon.quantity.saturating_sub(1);
        if weapon.quantity == 0 {
            add_tribute_message(
                attacker.identifier.as_str(),
                attacker.statistics.game.as_str(),
                format!("{}", GameOutput::WeaponBreak(attacker.clone(), weapon.clone())),
            ).expect("");
        }
    }

    // Get defense roll + defense modifier
    let mut defense_roll: i32 = target.attributes.defense as i32; // Add defense

    // If the defender has a shield, use it
    if let Some(shield) = target.defensive_items().iter_mut().last() {
        defense_roll += shield.effect; // Add shield defense
        shield.quantity = shield.quantity.saturating_sub(1);
        if shield.quantity == 0 {
            add_tribute_message(
                target.identifier.as_str(),
                target.statistics.game.as_str(),
                format!("{}", GameOutput::WeaponBreak(target.clone(), shield.clone())),
            ).expect("");
        }
    }

    // Compare attack vs defense
    match attack_roll.cmp(&defense_roll) {
        Ordering::Less => { // If the defender wins
            let difference = defense_roll as f64 - (attack_roll as f64 * 1.5);
            if difference > 0.0 {
                // Defender wins significantly
                AttackResult::DefenderWinsDecisively
            } else {
                AttackResult::DefenderWins
            }
        }
        Ordering::Equal => AttackResult::Miss, // If they tie
        Ordering::Greater => { // If the attacker wins
            let difference = attack_roll as f64 - (defense_roll as f64 * 1.5);

            if difference > 0.0 {
                // Attacker wins significantly
                AttackResult::AttackerWinsDecisively
            } else {
                AttackResult::AttackerWins
            }
        }
    }
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
            health: rng.gen_range(50..=100),
            sanity: rng.gen_range(50..=100),
            movement: 100,
            strength: rng.gen_range(1..=50),
            defense: rng.gen_range(1..=50),
            bravery: rng.gen_range(1..=100),
            loyalty: rng.gen_range(1..=100),
            speed: rng.gen_range(1..=100),
            dexterity: rng.gen_range(1..=100),
            intelligence: rng.gen_range(1..=100),
            persuasion: rng.gen_range(1..=100),
            luck: rng.gen_range(1..=100),
            is_hidden: false,
        }
    }

    pub fn as_map(&self) -> HashMap<String, String> {
        let json = serde_json::to_value(self).unwrap();
        let map: HashMap<String, serde_json::Value> = serde_json::from_value(json).unwrap();

        map.into_iter()
            .map(|(k, v)| (k, v.to_string().trim_matches('"').to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), None, None);
        assert!((50u32..=100).contains(&tribute.attributes.health));
        assert!((50u32..=100).contains(&tribute.attributes.sanity));
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.status, TributeStatus::Healthy);
    }

    #[test]
    fn takes_physical_damage() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        let hp = tribute.attributes.health.clone();
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, hp - 10);
    }

    #[test]
    fn takes_mental_damage() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        let mp = tribute.attributes.sanity.clone();
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, mp - 10);
    }

    #[test]
    fn is_hidden_true() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.attributes.intelligence = 100;
        tribute.attributes.is_hidden = true;
        assert!(!tribute.is_visible());
    }
}
