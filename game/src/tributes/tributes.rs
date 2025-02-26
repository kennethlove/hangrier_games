use super::actions::{Action, AttackOutcome, AttackResult, TributeAction};
use super::brains::Brain;
use super::statuses::TributeStatus;
use crate::areas::Area;
use crate::games::{Game, GAME};
use crate::items::{Attribute, Item};
use crate::messages::GameMessage;
use crate::tributes::events::TributeEvent;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::prelude::*;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::str::FromStr;
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
    #[serde(rename="player_name")]
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
    pub items: Vec<Item>,
    /// Events that have happened to the tribute
    #[serde(default)]
    pub events: Vec<TributeEvent>
}

impl Tribute {
    /// Creates a new Tribute with full health, sanity, and movement.
    pub fn new(name: String, district: Option<u32>, avatar: Option<String>) -> Self {
        let brain = Brain::default();
        let district  = district.unwrap_or(0);
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
        }
    }

    pub fn random() -> Self {
        let name = Name(EN).fake();
        let mut rng = thread_rng();
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
    pub fn takes_physical_damage(&mut self, damage: u32) {
        self.attributes.health = self.attributes.health.saturating_sub(damage);
    }

    /// Reduces mental health.
    pub fn takes_mental_damage(&mut self, damage: u32) {
        self.attributes.sanity = self.attributes.sanity.saturating_sub(damage);
    }

    /// Restores health.
    pub fn heals(&mut self, health: u32) {
        self.attributes.health = std::cmp::min(100, self.attributes.health + health);
    }

    /// Restores mental health.
    pub fn heals_mental_damage(&mut self, health: u32) {
        self.attributes.sanity = std::cmp::min(100, self.attributes.sanity + health);
    }

    /// Consumes movement and removes hidden status.
    pub fn moves(&mut self) {
        self.attributes.movement =
            std::cmp::max(0, self.attributes.movement - self.attributes.speed);
        self.attributes.is_hidden = false;
    }

    /// Restores movement.
    pub fn short_rests(&mut self) {
        self.attributes.movement = 100;
    }

    pub fn long_rests(&mut self) {
        self.short_rests();
        self.heals(5);
        self.heals_mental_damage(5);
    }

    /// Marks the tribute as recently dead and reveals them.
    pub fn dies(&mut self) {
        self.status = TributeStatus::RecentlyDead;
        self.attributes.is_hidden = false;
    }

    pub fn is_alive(&self) -> bool {
        match (&self.status, self.attributes.health) {
            (_, 0) => false,
            (TributeStatus::RecentlyDead | TributeStatus::Dead, _) => false,
            _ => true,
        }
    }

    /// Hides the tribute from view.
    pub fn hides(&mut self) {
        self.attributes.is_hidden = true;
    }

    /// Reveals the tribute to view.
    pub fn reveals(&mut self) {
        self.attributes.is_hidden = false;
    }

    /// Tribute is lonely/homesick/etc., loses some sanity.
    pub fn suffers(&mut self) {
        let loneliness = self.attributes.bravery as f64 / 100.0; // how lonely is the tribute?

        if loneliness.round() < 0.25 {
            if self.attributes.sanity < 25 {
                self.takes_mental_damage(self.attributes.bravery);
            }
            self.takes_mental_damage(self.attributes.bravery);
        }
    }

    pub fn attacks(&mut self, target: &mut Tribute) -> AttackOutcome {
        if self == target {
            println!("{}", GameMessage::TributeSelfHarm(self.clone()));
        }

        // `self` is the attacker
        match attack_contest(self, target) {
            AttackResult::AttackerWins => {
                target.takes_physical_damage(self.attributes.strength);
                target.statistics.defeats += 1;
                self.statistics.wins += 1;

                println!(
                    "{}",
                    GameMessage::TributeAttackWin(self.clone(), target.clone())
                );

                if target.attributes.health > 0 {
                    println!(
                        "{}",
                        GameMessage::TributeAttackWound(self.clone(), target.clone())
                    );
                    return AttackOutcome::Wound(self.clone(), target.clone());
                }
            }
            AttackResult::AttackerWinsDecisively => {
                // Take double damage
                target.takes_physical_damage(self.attributes.strength * 2);
                target.statistics.defeats += 1;
                self.statistics.wins += 1;

                println!(
                    "{}",
                    GameMessage::TributeAttackWinExtra(self.clone(), target.clone())
                );

                if target.attributes.health > 0 {
                    println!(
                        "{}",
                        GameMessage::TributeAttackWound(self.clone(), target.clone())
                    );
                    return AttackOutcome::Wound(self.clone(), target.clone());
                }
            }
            AttackResult::DefenderWins => {
                self.takes_physical_damage(target.attributes.strength);
                self.statistics.defeats += 1;
                target.statistics.wins += 1;

                println!(
                    "{}",
                    GameMessage::TributeAttackLose(self.clone(), target.clone())
                );

                if self.attributes.health > 0 {
                    println!(
                        "{}",
                        GameMessage::TributeAttackWound(target.clone(), self.clone())
                    );
                    return AttackOutcome::Wound(target.clone(), self.clone());
                }
            }
            AttackResult::DefenderWinsDecisively => {
                self.takes_physical_damage(target.attributes.strength * 2);
                self.statistics.defeats += 1;
                target.statistics.wins += 1;

                println!(
                    "{}",
                    GameMessage::TributeAttackLoseExtra(self.clone(), target.clone())
                );

                if self.attributes.health > 0 {
                    println!(
                        "{}",
                        GameMessage::TributeAttackWound(target.clone(), self.clone())
                    );
                    return AttackOutcome::Wound(target.clone(), self.clone());
                }
            }
            AttackResult::Miss => {
                println!(
                    "{}",
                    GameMessage::TributeAttackMiss(self.clone(), target.clone())
                );
                self.statistics.draws += 1;
                target.statistics.draws += 1;

                return AttackOutcome::Miss(self.clone(), target.clone());
            }
        };

        if self.attributes.health == 0 {
            // Attacker was killed by target
            println!(
                "{}",
                GameMessage::TributeAttackDied(self.clone(), target.clone())
            );
            self.statistics.killed_by = Some(target.name.clone());
            self.status = TributeStatus::RecentlyDead;
            self.dies();
            AttackOutcome::Kill(target.clone(), self.clone())
        } else if target.attributes.health == 0 {
            // Target was killed by attacker
            println!(
                "{}",
                GameMessage::TributeAttackSuccessKill(self.clone(), target.clone())
            );
            target.statistics.killed_by = Some(self.name.clone());
            target.status = TributeStatus::RecentlyDead;
            target.dies();
            AttackOutcome::Kill(self.clone(), target.clone())
        } else {
            AttackOutcome::Miss(self.clone(), target.clone())
        }
    }

    pub fn is_visible(&self) -> bool {
        let is_hidden = self.attributes.is_hidden;
        if is_hidden {
            let mut rng = thread_rng();
            !rng.gen_bool(self.attributes.intelligence as f64 / 100.0)
        } else {
            true
        }
    }

    pub fn travels(&self, closed_areas: Vec<Area>, suggested_area: Option<Area>) -> TravelResult {
        let mut rng = thread_rng();
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
            println!(
                "{}",
                GameMessage::TributeTravelAlreadyThere(self.clone(), new_area.clone().unwrap())
            );
            return TravelResult::Failure;
        }

        let handle_suggested_area = || -> TravelResult {
            if new_area.is_some() {
                println!(
                    "{}",
                    GameMessage::TributeTravel(
                        self.clone(),
                        area.clone(),
                        new_area.clone().unwrap()
                    )
                );
                return TravelResult::Success(new_area.unwrap());
            }
            TravelResult::Failure
        };

        match self.attributes.movement {
            // No movement left, can't move
            0 => {
                println!(
                    "{}",
                    GameMessage::TributeTravelTooTired(self.clone(), area.clone())
                );
                TravelResult::Failure
            }
            // Low movement, can only move to suggested area
            1..=10 => match handle_suggested_area() {
                TravelResult::Success(area) => TravelResult::Success(area),
                TravelResult::Failure => {
                    println!(
                        "{}",
                        GameMessage::TributeTravelTooTired(self.clone(), area.clone())
                    );
                    TravelResult::Failure
                }
            },
            // High movement, can move to any open neighbor or the suggested area
            _ => {
                match handle_suggested_area() {
                    TravelResult::Success(area) => return TravelResult::Success(area),
                    TravelResult::Failure => (),
                }
                // let neighbors = area.clone().unwrap().neighbors;
                let neighbors: Vec<Area> = area.clone().neighbors();
                for area in &neighbors {
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
                        //     println!(
                        //         "{}",
                        //         GameMessage::TributeTravelFollow(self.clone(), area.clone())
                        //     );
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
                            println!(
                                "{}",
                                GameMessage::TributeTravelStay(self.clone(), area.clone())
                            );
                            return TravelResult::Success(area.clone());
                        }
                        continue;
                    }
                    break new_area.clone();
                };
                println!(
                    "{}",
                    GameMessage::TributeTravel(
                        self.clone(),
                        area.clone(),
                        new_area.clone()
                    )
                );
                TravelResult::Success(new_area)
            }
        }
    }

    pub fn process_status(&mut self) {
        let status = self.status.clone();
        match status {
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
                    0 => { // Leg
                        self.attributes.speed = std::cmp::max(1, self.attributes.speed - 5);
                    },
                    1 => { // Arm
                        self.attributes.strength = std::cmp::max(1, self.attributes.strength - 5);
                    },
                    2 => { // Skull
                        self.attributes.intelligence = std::cmp::max(1, self.attributes.intelligence - 5);
                    },
                    _ => { // Rib
                        self.attributes.dexterity = std::cmp::max(1, self.attributes.dexterity - 5);
                    },
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

        if self.attributes.health == 0 {
            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    pub fn handle_event(&mut self, tribute_event: TributeEvent) {
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
            println!(
                "{}",
                GameMessage::TributeDiesFromTributeEvent(self.clone(), tribute_event.clone())
            );
            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    pub fn do_day_night(
        &mut self,
        suggested_action: Option<Action>,
        probability: Option<f64>,
        day: bool,
    ) {
        // Tribute is already dead, do nothing.
        if !self.is_alive() {
            println!("{}", GameMessage::TributeAlreadyDead(self.clone()));
        }

        // Update the tribute based on the period's events.
        self.process_status();

        // Nighttime terror
        if !day && self.is_alive() {
            self.suffers();
        }

        // Gift from patrons?
        let chance = match self.district {
            1 | 2 => 1.0 / 10.0,
            3 | 4 => 1.0 / 15.0,
            5 | 6 => 1.0 / 20.0,
            7 | 8 => 1.0 / 25.0,
            9 | 10 => 1.0 / 30.0,
            _ => 1.0 / 50.0,
        };

        if thread_rng().gen_bool(chance) {
            let item = Item::new_random_consumable();
            println!("{}", GameMessage::SponsorGift(self.clone(), item.clone()));
        }

        // Tribute died to the period's events.
        if self.status == TributeStatus::RecentlyDead || self.attributes.health == 0 {
            println!("{}", GameMessage::TributeDead(self.clone()));
        }

        let area = self.area.clone();
        let closed_areas: Vec<Area> = GAME.with_borrow(|g| {
            g.areas.clone()
                .iter()
                .filter(|a| !a.open)
                .map(|a| {
                    Area::from_str(a.area.clone().as_str()).expect("Invalid area")
                })
                .collect()
        });

        if let Some(action) = suggested_action {
            self.brain
                .set_preferred_action(action, probability.unwrap());
        }

        let nearby_tributes: Vec<Tribute> = GAME.with_borrow(|g| {
            g.living_tributes().iter().filter(|t| t.area == area).cloned().collect()
        });
        let mut brain = self.brain.clone();
        let action = brain.act(&self, nearby_tributes.len());
        match &action {
            Action::Move(area) => match self.travels(closed_areas.clone(), area.clone()) {
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
                println!("{}", GameMessage::TributeHide(self.clone()));
            }
            Action::Rest | Action::None => {
                self.long_rests();
                self.take_action(&action, None);
                println!("{}", GameMessage::TributeLongRest(self.clone()));
            }
            Action::Attack => {
                if let Some(mut target) = self.pick_target() {
                    if target.is_visible() {
                        match self.attacks(&mut target) {
                            AttackOutcome::Kill(mut attacker, mut target) => {
                                if attacker.attributes.health == 0 {
                                    attacker.dies();
                                }
                                if target.attributes.health == 0 {
                                    target.dies();
                                }
                                if attacker.identifier == target.identifier {
                                    attacker.attributes.health = target.attributes.health.clone();
                                    attacker.statistics.day_killed =
                                        target.statistics.day_killed.clone();
                                    attacker.statistics.killed_by =
                                        target.statistics.killed_by.clone();
                                    attacker.status = target.status.clone();
                                    // return target;
                                }
                            }
                            _ => (),
                        }
                        self.take_action(&action, Some(&target));
                    } else {
                        println!(
                            "{}",
                            GameMessage::TributeAttackHidden(self.clone(), target.clone())
                        );
                        self.take_action(&Action::Attack, None);
                    }
                }
            }
            Action::TakeItem => {
                if let Some(item) = self.take_nearby_item() {
                    self.take_action(&action, None);
                    println!(
                        "{}",
                        GameMessage::TributeTakeItem(self.clone(), item.clone())
                    );
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
                            println!(
                                "{}",
                                GameMessage::TributeUseItem(self.clone(), item.clone())
                            );
                            self.take_action(&action, None);
                        }
                        false => {
                            println!(
                                "{}",
                                GameMessage::TributeCannotUseItem(self.clone(), item.clone())
                            );
                            self.short_rests();
                            self.take_action(&Action::Rest, None);
                        }
                    };
                }
            }
            Action::UseItem(item) => {
                let items = self.consumable_items();
                if let Some(item) = item {
                    if items.contains(item) {
                        match self.use_consumable(item.clone()) {
                            true => {
                                println!(
                                    "{}",
                                    GameMessage::TributeUseItem(self.clone(), item.clone())
                                );
                                self.take_action(&action, None);
                            }
                            false => {
                                println!(
                                    "{}",
                                    GameMessage::TributeCannotUseItem(self.clone(), item.clone())
                                );
                                self.short_rests();
                                self.take_action(&Action::Rest, None);
                            }
                        };
                    }
                }
            }
        }
        dbg!(&self);
    }

    /// Save the tribute's latest action
    fn take_action(&mut self, action: &Action, target: Option<&Tribute>) {
        self.brain
            .previous_actions
            .push(TributeAction::new(action.clone(), target.cloned()));
    }

    /// Take item from area
    fn take_nearby_item(&mut self) -> Option<Item> {
        let mut rng = thread_rng();
        let area = GAME.with_borrow(|game| { game.where_am_i(&self) });
        if area.is_none() {
            None
        } else {
            todo!();
            // let mut area = area.unwrap();
            // let items = area.available_items();
            // let item = items.choose(&mut rng).unwrap();
            //
            // self.take_item(&item);
            // area.remove_item(&item);
            //
            // Some(item.clone())
        }
    }

    /// Take a prescribed item
    fn take_item(&mut self, item: &Item) {
        self.items.push(item.clone());
    }

    fn use_consumable(&mut self, chosen_item: Item) -> bool {
        let items = self.consumable_items();

        #[allow(unused_assignments)]
        let mut item = items.iter().last().unwrap().clone();
        // If the tribute has the item...
        if let Some(selected_item) = items.iter().filter(|i| i.name == chosen_item.name).last() {
            // select it
            item = selected_item.clone();
        } else {
            // otherwise, quit because you can't use an item you don't have
            return false;
        }
        item.quantity -= 1;

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

    pub fn available_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.quantity > 0)
            .cloned()
            .collect()
    }

    pub fn weapons(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_weapon())
            .cloned()
            .collect()
    }

    pub fn defensive_items(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_defensive())
            .cloned()
            .collect()
    }

    pub fn consumable_items(&self) -> Vec<Item> {
        self.available_items()
            .iter()
            .filter(|i| i.is_consumable())
            .cloned()
            .collect()
    }

    pub fn pick_target(&self) -> Option<Tribute> {
        let game = GAME.with_borrow(|game| { game.clone() });
        let area = game.where_am_i(&self);
        let mut tributes: Vec<Tribute> = Vec::new();
        if let Some(area) = area {
            todo!();
            // tributes = area
            //     .living_tributes()
            //     .iter()
            //     .filter(|t| t.id != self.id)
            //     .cloned()
            //     .collect();
        }

        match tributes.len() {
            0 => {
                // there are no other targets
                match self.attributes.sanity {
                    0..=9 => {
                        // attempt suicide
                        println!("{}", GameMessage::TributeSuicide(self.clone()));
                        Some(self.clone())
                    }
                    _ => None, // Attack no one
                }
            }
            _ => {
                // there ARE targets
                let enemies: Vec<Tribute> = tributes
                    .iter()
                    .filter(|t| t.district != self.district)
                    .filter(|t| t.is_visible())
                    .cloned()
                    .collect();

                match enemies.len() {
                    0 | 1 => Some(enemies.first()?.clone()), // Easy choice
                    _ => {
                        let mut rng = thread_rng();
                        Some(enemies.choose(&mut rng)?.clone()) // Get a random enemy
                    }
                }
            }
        }
    }

    pub fn status(&self) -> TributeStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: TributeStatus) {
        self.status = status;
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
        println!(
            "{}",
            GameMessage::TributeHorrified(tribute.clone(), terror.round() as u32)
        );
        tribute.takes_mental_damage(terror.round() as u32);
    }
}

fn attack_contest(attacker: &Tribute, target: &Tribute) -> AttackResult {
    let mut tribute1_roll = thread_rng().gen_range(1..=20); // Base roll
    tribute1_roll += attacker.attributes.strength; // Add strength

    if let Some(weapon) = attacker.weapons().iter_mut().last() {
        tribute1_roll += weapon.effect as u32; // Add weapon damage
        weapon.quantity = weapon.quantity.saturating_sub(1);
        if weapon.quantity == 0 {
            println!(
                "{}",
                GameMessage::WeaponBreak(attacker.clone(), weapon.clone())
            );
        }
    }

    let mut tribute2_roll = thread_rng().gen_range(1..=20); // Base roll
    tribute2_roll += target.attributes.defense; // Add defense

    if let Some(shield) = target.defensive_items().iter_mut().last() {
        tribute2_roll += shield.effect as u32; // Add weapon defense
        shield.quantity = shield.quantity.saturating_sub(1);
        if shield.quantity == 0 {
            println!(
                "{}",
                GameMessage::ShieldBreak(attacker.clone(), shield.clone())
            );
        }
    }

    if tribute1_roll > tribute2_roll {
        if tribute1_roll as f32 >= tribute2_roll as f32 * 1.5 {
            // Attacker wins significantly
            AttackResult::AttackerWinsDecisively
        } else {
            AttackResult::AttackerWins
        }
    } else if tribute2_roll > tribute1_roll {
        if tribute2_roll as f32 >= tribute1_roll as f32 * 1.5 {
            // Defender wins significantly
            AttackResult::DefenderWinsDecisively
        } else {
            AttackResult::DefenderWins
        }
    } else {
        AttackResult::Miss
    }
}

impl Default for Tribute {
    fn default() -> Self {
        Self::new("Tribute".to_string(), None, None)
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
    fn new() -> Self {
        let mut rng = thread_rng();

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::Game;
    use rstest::{fixture, rstest};

    #[fixture]
    fn game() -> Game {
        Game::default()
    }

    #[test]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), None, None);
        assert!((50u32..=100).contains(&tribute.attributes.health));
        assert!((50u32..=100).contains(&tribute.attributes.sanity));
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.status, TributeStatus::Healthy);
    }

    #[rstest]
    fn takes_physical_damage(_game: Game) {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        let hp = tribute.attributes.health.clone();
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, hp - 10);
    }

    #[rstest]
    fn takes_mental_damage(_game: Game) {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        let mp = tribute.attributes.sanity.clone();
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, mp - 10);
    }

    #[rstest]
    fn moves_and_rests(_game: Game) {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.attributes.speed = 50;
        tribute.moves();
        assert_eq!(tribute.attributes.movement, 50);
        tribute.short_rests();
        assert_eq!(tribute.attributes.movement, 100);
    }

    #[rstest]
    fn is_hidden_true(_game: Game) {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.attributes.intelligence = 100;
        tribute.attributes.is_hidden = true;
        assert!(!tribute.is_visible());
    }
}
