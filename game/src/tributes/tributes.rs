use super::actions::{AttackOutcome, AttackResult, Action, TributeAction};
use super::brains::TributeBrain;
use super::statuses::TributeStatus;
use crate::areas::Area;
use crate::tributes::events::TributeEvent;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use crate::games::Game;
use crate::items::{Attribute, Item};
use crate::messages::GameMessage;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tribute {
    /// What is their identifier?
    pub id: Option<i32>,
    /// What game do they belong to?
    pub game: Option<Game>,
    /// Where are they in the game?
    pub area: Option<Area>,
    /// What is their current status?
    pub status: TributeStatus,
    /// This is their thinker
    pub brain: TributeBrain,
    /// How they present themselves to the real world
    pub avatar: Option<String>,
    /// Who created them in the real world
    pub human_player_name: Option<String>,
    /// What they like to go by
    pub name: String,
    /// Where they're from
    pub district: i32,
    /// Stats like fights won
    pub statistics: Statistics,
    /// Attributes like health
    pub attributes: Attributes,
    /// Actions the tribute has taken
    pub previous_actions: Vec<TributeAction>,
    /// Items the tribute owns
    pub items: Vec<Item>,
}

impl Tribute {
    /// Creates a new Tribute with full health, sanity, and movement.
    pub fn new(name: String, district: Option<i32>, avatar: Option<String>) -> Self {
        let brain = TributeBrain::new();
        let district = district.unwrap_or(0);
        let attributes = Attributes::new();
        let statistics = Statistics::default();

        Self {
            id: None,
            game: None,
            name: name.clone(),
            district,
            area: Some(Area::default()),
            brain,
            status: TributeStatus::Healthy,
            avatar,
            human_player_name: None,
            attributes,
            statistics,
            previous_actions: vec![],
            items: vec![],
        }
    }

    pub fn delete(id: i32) {
        models::tribute::Tribute::delete(id);
    }

    pub fn update(&self, update: UpdateTribute) {
        let tribute_model = models::Tribute::from(self.clone());
        tribute_model.update(update);
    }

    pub fn avatar(&self) -> String {
        format!("assets/{}", self.avatar.clone().unwrap_or("hangry-games.png".to_string()))
    }

    /// Reduces health.
    pub fn takes_physical_damage(&mut self, damage: i32) {
        self.attributes.health = std::cmp::max(0, self.attributes.health - damage);
    }

    /// Reduces mental health.
    pub fn takes_mental_damage(&mut self, damage: i32) {
        self.sanity = std::cmp::max(0, self.attributes.sanity - damage);
    }

    /// Restores health.
    pub fn heals(&mut self, health: i32) {
        self.attributes.health = std::cmp::min(100, self.attributes.health + health);
    }

    /// Restores mental health.
    pub fn heals_mental_damage(&mut self, health: i32) {
        self.attributes.sanity = std::cmp::min(100, self.attributes.sanity + health);
    }

    /// Consumes movement and removes hidden status.
    pub fn moves(&mut self) {
        self.attributes.movement = std::cmp::max(0, self.attributes.movement - self.attributes.speed.unwrap());
        self.attributes.is_hidden = Some(false);
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
        self.attributes.is_hidden = Some(false);
    }

    pub fn is_alive(&self) -> bool {
        match (self.status.clone(), self.attributes.health) {
            (_, 0) => false,
            (TributeStatus::RecentlyDead | TributeStatus::Dead, _) => false,
            _ => true,
        }
    }

    /// Moves the tribute from one area to another, removes hidden status.
    pub fn changes_area(&mut self, area: Area) {
        self.area = Some(area);
        self.attributes.is_hidden = Some(false);
    }

    /// Removes the tribute from the game arena, removes hidden status.
    pub fn leaves_area(&mut self) {
        self.area = None;
        self.attributes.is_hidden = Some(false);
    }

    /// Hides the tribute from view.
    pub fn hides(&mut self) {
        self.attributes.is_hidden = Some(true);
    }

    /// Reveals the tribute to view.
    pub fn reveals(&mut self) {
        self.attributes.is_hidden = Some(false);
    }

    /// Tribute is lonely/homesick/etc., loses some sanity.
    pub fn suffers(&mut self) {
        let game = get_game_by_id(self.game_id.unwrap()).unwrap();
        let district_mates = get_all_living_tributes(&game).iter()
            .filter(|t| t.district == self.district)
            .filter(|t| self.area == Some(Area::from(get_area_by_id(t.area_id).unwrap())))
            .count() as f64;

        let loneliness = self.attributes.bravery.unwrap_or(0) as f64 / 100.0;  // how lonely is the tribute?
        let terror = (self.attributes.sanity as f64 / 100.0) * game.day.unwrap() as f64; // how scared are they?
        let connectedness = district_mates * loneliness;
        let terror = terror - connectedness;

        if terror.round() > 1.0 {
            create_full_log(
                self.game.unwrap().id.clone(),
                GameMessage::TributeSuffer(self.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            self.takes_mental_damage(terror.round() as i32);
        }
    }

    pub fn attacks(&mut self, target: &mut Tribute) -> AttackOutcome {
        if self == target {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeSelfHarm(self.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                Some("attack".to_string()),
                Some(self.id.unwrap())
            );
        }

        match attack_contest(self.clone(), target.clone()) {
            AttackResult::AttackerWins => {
                target.takes_physical_damage(self.attributes.strength.unwrap());
                target.statistics.defeats = Some(target.statistics.defeats.unwrap_or(0) + 1);
                self.statistics.defeats = Some(self.statistics.defeatswrap_or(0) + 1);

                create_full_log(
                    self.game.unwrap().id.unwrap(),
                    GameMessage::TributeAttackWin(self.clone(), target.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some("attack".to_string()),
                    Some(target.id.unwrap())
                );

                if target.health > 0 {
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeAttackWound(self.clone(), target.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(self.id.unwrap()),
                        Some("attack".to_string()),
                        Some(target.id.unwrap())
                    );
                    return AttackOutcome::Wound(self.clone(), target.clone())
                }
            }
            AttackResult::AttackerWinsDecisively => {
                target.takes_physical_damage(self.attributes.strength.unwrap() * 2);
                target.statistics.defeats = Some(target.statistics.defeats.unwrap_or(0) + 1);
                self.statistics.wins = Some(self.statistics.wins.unwrap_or(0) + 1);

                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeAttackWinExtra(self.clone(), target.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some("attack".to_string()),
                    Some(target.id.unwrap())
                );

                if target.health > 0 {
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeAttackWound(self.clone(), target.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(self.id.unwrap()),
                        Some("attack".to_string()),
                        Some(target.id.unwrap())
                    );
                    return AttackOutcome::Wound(self.clone(), target.clone())
                }
            }
            AttackResult::DefenderWins => {
                self.takes_physical_damage(target.attributes.strength.unwrap());
                self.statistics.defeats = Some(self.statistics.defeats.unwrap() + 1);
                target.statistics.wins = Some(target.statistics.wins.unwrap() + 1);

                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeAttackLose(self.clone(), target.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some("attack".to_string()),
                    Some(target.id.unwrap())
                );

                if self.health > 0 {
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeAttackWound(target.clone(), self.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(target.id.unwrap()),
                        Some("attack".to_string()),
                        Some(self.id.unwrap())
                    );
                    return AttackOutcome::Wound(target.clone(), self.clone())
                }
            }
            AttackResult::DefenderWinsDecisively => {
                self.takes_physical_damage(target.attributes.strength.unwrap() * 2);
                self.statistics.defeats = Some(self.statistics.defeats.unwrap() + 1);
                target.statistics.wins = Some(target.statistics.wins.unwrap() + 1);

                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeAttackLoseExtra(self.clone(), target.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some("attack".to_string()),
                    Some(target.id.unwrap())
                );

                if self.health > 0 {
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeAttackWound(target.clone(), self.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(target.id.unwrap()),
                        Some("attack".to_string()),
                        Some(self.id.unwrap())
                    );
                    return AttackOutcome::Wound(target.clone(), self.clone())
                }
            }
            AttackResult::Miss => {
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeAttackMiss(self.clone(), target.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some("attack".to_string()),
                    Some(target.id.unwrap())
                );
                self.statistics.draws = Some(self.statistics.draws.unwrap() + 1);
                target.statistics.draws = Some(target.statistics.draws.unwrap() + 1);

                return AttackOutcome::Miss(self.clone(), target.clone())
            }
        };

        if self.attributes.health <= 0 {
            // Attacker was killed by target
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeAttackDied(self.clone(), target.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(target.id.unwrap()),
                Some("attack".to_string()),
                Some(self.id.unwrap())
            );
            self.statistics.killed_by = Some(target.name.clone());
            self.status = TributeStatus::RecentlyDead;
            self.dies();
            AttackOutcome::Kill(target.clone(), self.clone())
        } else if target.attributes.health <= 0 {
            // Target was killed by attacker
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeAttackSuccessKill(self.clone(), target.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                Some("attack".to_string()),
                Some(target.id.unwrap())
            );
            target.statistics.killed_by = Some(self.name.clone());
            target.status = TributeStatus::RecentlyDead;
            target.dies();
            AttackOutcome::Kill(self.clone(), target.clone())
        } else {
            AttackOutcome::Miss(self.clone(), target.clone())
        }

        // apply_violence_stress(self);
    }

    pub fn is_visible(&self) -> bool {
        let is_hidden = self.attributes.is_hidden.unwrap_or(false);
        if is_hidden {
            let mut rng = thread_rng();
            !rng.gen_bool(self.attributes.intelligence.unwrap() as f64 / 100.0)
        } else {
            true
        }
    }

    pub fn travels(&self, closed_areas: Vec<Area>, suggested_area: Option<String>) -> TravelResult {
        let mut rng = thread_rng();
        let area = self.clone().area.unwrap();

        let suggested_area = {
            let suggested_area = suggested_area.clone();
            if suggested_area.is_some() {
                let suggested_area = Area::from_str(suggested_area.unwrap().as_str()).unwrap();
                if closed_areas.contains(&suggested_area) {
                    None
                } else {
                    Some(suggested_area)
                }
            } else {
                None
            }
        };

        if suggested_area.is_some() && suggested_area.clone().unwrap() == area {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeTravelAlreadyThere(self.clone(), suggested_area.clone().unwrap()).to_string(),
                Some(area.id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            return TravelResult::Failure;
        }

        let handle_suggested_area = || -> TravelResult {
            if suggested_area.is_some() {
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeTravel(self.clone(), area.clone(), suggested_area.clone().unwrap()).to_string(),
                    Some(area.id()),
                    Some(self.id.unwrap()),
                    Some("Move".to_string()),
                    Some(suggested_area.clone().unwrap().id())
                );
                return TravelResult::Success(suggested_area.unwrap());
            }
            TravelResult::Failure
        };

        match self.attributes.movement {
            // No movement left, can't move
            0 => {
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeTravelTooTired(self.clone(), area.clone()).to_string(),
                    Some(area.id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
                TravelResult::Failure
            },
            // Low movement, can only move to suggested area
            1..=10 => {
                match handle_suggested_area() {
                    TravelResult::Success(area) => TravelResult::Success(area),
                    TravelResult::Failure => {
                        create_full_log(
                            self.game_id.unwrap(),
                            GameMessage::TributeTravelTooTired(self.clone(), area.clone()).to_string(),
                            Some(area.id()),
                            Some(self.id.unwrap()),
                            None,
                            None
                        );
                        TravelResult::Failure
                    }
                }
            },
            // High movement, can move to any open neighbor or the suggested area
            _ => {
                match handle_suggested_area() {
                    TravelResult::Success(area) => return TravelResult::Success(area),
                    TravelResult::Failure => ()
                }
                let neighbors = area.neighbors();
                for area in &neighbors {
                    if area.tributes(self.game_id.unwrap()).iter()
                        .filter(|t| t.district == self.district)
                        .count() > 0 {
                        create_full_log(
                            self.game_id.unwrap(),
                            GameMessage::TributeTravelFollow(self.clone(), area.clone()).to_string(),
                            Some(self.area.clone().unwrap().id()),
                            Some(self.id.unwrap()),
                            Some("Move".to_string()),
                            Some(area.id())
                        );
                        return TravelResult::Success(area.clone());
                    }
                }
                let mut count = 0;
                let new_area = loop {
                    let new_area = neighbors.choose(&mut rng).unwrap();
                    if new_area == &area || closed_areas.contains(new_area) {
                        count += 1;

                        if count == 10 {
                            create_full_log(
                                self.game_id.unwrap(),
                                GameMessage::TributeTravelStay(self.clone(), area.clone()).to_string(),
                                Some(area.id()),
                                Some(self.id.unwrap()),
                                Some("Move".to_string()),
                                Some(area.id())
                            );
                            return TravelResult::Success(area.clone());
                        }

                        continue;
                    }
                    break new_area.clone();
                };
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeTravel(self.clone(), area.clone(), new_area.clone()).to_string(),
                    Some(area.id()),
                    Some(self.id.unwrap()),
                    Some("Move".to_string()),
                    Some(new_area.id())
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
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeBleeds(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Sick => {
                self.attributes.strength = Some(std::cmp::max(1, self.attributes.strength.unwrap() - 1));
                self.attributes.speed = Some(std::cmp::max(1, self.attributes.speed.unwrap() - 1));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeSick(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Electrocuted => {
                self.takes_physical_damage(20);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeElectrocuted(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Frozen => {
                self.attributes.speed = Some(std::cmp::max(1, self.attributes.speed.unwrap() - 1));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeFrozen(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Overheated => {
                self.attributes.speed = Some(std::cmp::max(1, self.attributes.speed.unwrap() - 1));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeOverheated(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Dehydrated => {
                self.attributes.strength = Some(std::cmp::max(1, self.attributes.strength.unwrap() - 1));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeDehydrated(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Starving => {
                self.attributes.strength = Some(std::cmp::max(1, self.attributes.strength.unwrap() - 1));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeStarving(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Poisoned => {
                self.takes_mental_damage(5);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributePoisoned(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Broken => {
                // coin flip for which bone breaks
                let leg_bone = thread_rng().gen_bool(0.5);

                // TODO: Add in other bones? Ribs and skull make sense.

                if leg_bone {
                    self.attributes.speed = Some(std::cmp::max(1, self.attributes.speed.unwrap() - 5));
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeBrokenLeg(self.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(self.id.unwrap()),
                        None,
                        None
                    );
                } else {
                    self.attributes.strength = Some(std::cmp::max(1, self.attributes.strength.unwrap() - 5));
                    create_full_log(
                        self.game_id.unwrap(),
                        GameMessage::TributeBrokenArm(self.clone()).to_string(),
                        Some(self.area.clone().unwrap().id()),
                        Some(self.id.unwrap()),
                        None,
                        None
                    );
                }
            },
            TributeStatus::Infected => {
                self.takes_physical_damage(2);
                self.takes_mental_damage(2);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeInfected(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Drowned => {
                self.takes_physical_damage(2);
                self.takes_mental_damage(2);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeDrowned(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Mauled(animal) => {
                let number_of_animals = thread_rng().gen_range(2..=5);
                let damage = animal.damage() * number_of_animals;
                self.takes_physical_damage(damage);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeMauled(self.clone(), number_of_animals, animal.clone(), damage).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            TributeStatus::Burned => {
                self.takes_physical_damage(5);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeBurned(self.clone()).to_string(),
                    Some(self.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            }
            _ => {}
        }

        if self.attributes.health <= 0 {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeDiesFromStatus(self.clone(), self.status.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    pub fn handle_event(&mut self, tribute_event: TributeEvent) {
        match tribute_event {
            TributeEvent::AnimalAttack(ref animal) => {
                self.status = TributeStatus::Mauled(animal.clone());
            },
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
            },
            TributeEvent::Dehydration => {
                self.status = TributeStatus::Dehydrated;
            },
            TributeEvent::Starvation => {
                self.status = TributeStatus::Starving;
            },
            TributeEvent::Poisoning => {
                self.status = TributeStatus::Poisoned;
            },
            TributeEvent::BrokenBone => {
                self.status = TributeStatus::Broken;
            },
            TributeEvent::Infection => {
                self.status = TributeStatus::Infected;
            },
            TributeEvent::Drowning => {
                self.status = TributeStatus::Drowned;
            },
            TributeEvent::Burn => {
                self.status = TributeStatus::Burned;
            },
        }
        if self.attributes.health <= 0 {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeDiesFromTributeEvent(self.clone(), tribute_event.clone()).to_string(),
                Some(self.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            self.statistics.killed_by = Some(self.status.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    pub fn do_day_night(&mut self, suggested_action: Option<Action>, probability: Option<f64>, day: bool) -> Tribute {
        let mut tribute = Tribute::from(get_tribute_by_id(self.id.unwrap()));

        // Tribute is already dead, do nothing.
        if !tribute.is_alive() {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeAlreadyDead(tribute.clone()).to_string(),
                Some(tribute.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            return tribute.clone();
        }

        // Update the tribute based on the period's events.
        tribute.process_status();

        // Nighttime terror
        if !day && tribute.is_alive() {
            tribute.suffers();
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
            let item = Item::new_generic_consumable(self.game_id, None, self.id);
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::SponsorGift(tribute.clone(), item.clone()).to_string(),
                None,
                Some(self.id.unwrap()),
                None,
                None
            );
        }

        // Tribute died to the period's events.
        if tribute.status == TributeStatus::RecentlyDead || tribute.attributes.health <= 0 {
            create_full_log(
                self.game_id.unwrap(),
                GameMessage::TributeDead(tribute.clone()).to_string(),
                Some(tribute.area.clone().unwrap().id()),
                Some(self.id.unwrap()),
                None,
                None
            );
            return self.clone();
        }

        let game = get_game_by_id(self.game_id.unwrap()).unwrap();
        let area = tribute.area.clone().unwrap();
        let closed_areas = game.closed_areas().clone();

        let brain = &mut tribute.brain.clone();

        if suggested_action.is_some() {
            brain.set_preferred_action(suggested_action.unwrap(), probability.unwrap());
        }

        let nearby_tributes = get_all_living_tributes(&game).iter()
            .filter(|t| t.area().is_some())
            .map(|t| Tribute::from(t.clone()))
            .filter(|t| t.clone().area.unwrap() == area)
            .collect::<Vec<_>>().len();

        let action = brain.act(&tribute, nearby_tributes, closed_areas.clone());

        match &action {
            Action::Move(area) => {
                match self.travels(closed_areas.clone(), area.clone()) {
                    TravelResult::Success(area) => {
                        tribute.changes_area(area.clone());
                        self.take_action(action.clone(), Some(area.clone().to_string()));
                        // No need to log the move, it's already done in self.travels.
                    },
                    TravelResult::Failure => {
                        tribute.short_rests();
                        self.take_action(action.clone(), None);
                    }
                }
            },
            Action::Hide => {
                tribute.hides();
                self.take_action(action.clone(), None);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeHide(tribute.clone()).to_string(),
                    Some(tribute.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some(action.clone().as_str().to_string()),
                    Some(self.id.unwrap())
                );
            },
            Action::Rest | Action::None => {
                tribute.long_rests();
                self.take_action(action, None);
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeLongRest(tribute.clone()).to_string(),
                    Some(tribute.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    None,
                    None
                );
            },
            Action::Attack => {
                if let Some(mut target) = pick_target(tribute.clone().into()) {
                    if target.is_visible() {
                        match tribute.attacks(&mut target) {
                            AttackOutcome::Kill(mut attacker, mut target) => {
                                if attacker.attributes.health <= 0 {
                                    attacker.dies();
                                }
                                if target.attributes.health <= 0 {
                                    target.dies();
                                }
                                if attacker.id == target.id {
                                    attacker.attributes.health = target.attributes.health.clone();
                                    attacker.statistics.day_killed = target.statistics.day_killed.clone();
                                    attacker.statistics.killed_by = target.statistics.killed_by.clone();
                                    attacker.status = target.status.clone();
                                    return target;
                                }
                                update_tribute(attacker.id.unwrap(), attacker.clone().into());
                                update_tribute(target.id.unwrap(), target.clone().into());
                            },
                            _ => ()
                        }
                        self.take_action(action, Some(target.clone().name));
                    } else {
                        create_full_log(
                            self.game_id.unwrap(),
                            GameMessage::TributeAttackHidden(tribute.clone(), target.clone()).to_string(),
                            Some(tribute.area.clone().unwrap().id()),
                            Some(self.id.unwrap()),
                            Some(action.clone().as_str().to_string()),
                            Some(target.id.unwrap())
                        );
                        self.take_action(Action::Attack, None);
                    }
                }
            },
            Action::TakeItem => {
                let item = tribute.take_nearby_item(area);
                self.take_action(action.clone(), Some(item.name.clone()));
                create_full_log(
                    self.game_id.unwrap(),
                    GameMessage::TributeTakeItem(tribute.clone(), item.clone()).to_string(),
                    Some(tribute.area.clone().unwrap().id()),
                    Some(self.id.unwrap()),
                    Some(action.clone().as_str().to_string()),
                    Some(item.id.unwrap())
                );
            },
            Action::UseItem(None) => {
                // Get consumable items
                let mut items = self.consumable_items();
                if items.is_empty() {
                    tribute.long_rests();
                    self.take_action(Action::Rest, None);
                } else {
                    // Use random item
                    let item = items.choose_mut(&mut thread_rng()).unwrap();
                    match tribute.use_consumable(item.clone()) {
                        true => {
                            create_full_log(
                                self.game_id.unwrap(),
                                GameMessage::TributeUseItem(tribute.clone(), item.clone()).to_string(),
                                Some(tribute.area.clone().unwrap().id()),
                                Some(self.id.unwrap()),
                                Some(action.clone().as_str().to_string()),
                                Some(item.id.unwrap())
                            );
                            self.take_action(action, Some(item.name.clone()));
                        },
                        false => {
                            create_full_log(
                                self.game_id.unwrap(),
                                GameMessage::TributeCannotUseItem(tribute.clone(), item.clone()).to_string(),
                                Some(tribute.area.clone().unwrap().id()),
                                Some(self.id.unwrap()),
                                Some(action.clone().as_str().to_string()),
                                Some(item.id.unwrap())
                            );
                            tribute.short_rests();
                            self.take_action(Action::Rest, None);
                        }
                    };
                }
            }
            Action::UseItem(item) => {
                let items = tribute.consumable_items();
                if let Some(item) = item {
                    let selected_item = items.iter().find(|i| i.name == item.clone());
                    if selected_item.is_some() {
                        match tribute.use_consumable(selected_item.unwrap().clone()) {
                            true => {
                                create_full_log(
                                    self.game_id.unwrap(),
                                    GameMessage::TributeUseItem(tribute.clone(), selected_item.unwrap().clone()).to_string(),
                                    Some(tribute.area.clone().unwrap().id()),
                                    Some(self.id.unwrap()),
                                    Some(action.clone().as_str().to_string()),
                                    Some(selected_item.unwrap().id.unwrap())
                                );
                                self.take_action(action, Some(selected_item.unwrap().name.clone()));
                            },
                            false => {
                                create_full_log(
                                    self.game_id.unwrap(),
                                    GameMessage::TributeCannotUseItem(tribute.clone(), selected_item.unwrap().clone()).to_string(),
                                    Some(tribute.area.clone().unwrap().id()),
                                    Some(self.id.unwrap()),
                                    Some(action.clone().as_str().to_string()),
                                    Some(selected_item.unwrap().id.unwrap())
                                );
                                tribute.short_rests();
                                self.take_action(Action::Rest, None);
                            }
                        };
                    }
                }
            }
        }
        tribute.clone()
    }

    fn take_action(&mut self, action: &Action, target: Option<&Tribute>) {
        self.actions.push(TributeAction::new(action.clone(), target.cloned()));
    }

    fn take_nearby_item(&self, area: Area) -> Item {
        let mut rng = thread_rng();
        let mut items = area.available_items(self.game_id.unwrap());
        let item = items.choose_mut(&mut rng).unwrap();
        self.take_item(item.clone());
        item.clone()
    }

    fn take_item(&self, item: Item) {
        let tribute = TributeModel::from(self.clone());
        tribute.takes_item(item.id.unwrap());
    }

    fn use_consumable(&mut self, chosen_item: Item) -> bool {
        let items = self.consumable_items();
        #[allow(unused_assignments)]
        let mut item = items.iter().last().unwrap().clone();
        if let Some(selected_item) = items.iter()
            .filter(|i| i.name == chosen_item.name)
            .filter(|i| i.quantity > 0)
            .last()
        {
            item = selected_item.clone();
        } else {
            return false;
        }
        item.quantity -= 1;

        // Apply item effect
        match item.attribute {
            Attribute::Health => {
                self.heals(item.effect);
            },
            Attribute::Sanity => {
                self.heals_mental_damage(item.effect);
            },
            Attribute::Movement => {
                self.attributes.movement = std::cmp::min(100, self.attributes.movement + item.effect);
            },
            Attribute::Bravery => {
                self.attributes.bravery = Some(std::cmp::min(100, self.attributes.bravery.unwrap() + item.effect));
            },
            Attribute::Speed => {
                self.attributes.speed = Some(std::cmp::min(100, self.attributes.speed.unwrap() + item.effect));
            },
            Attribute::Strength => {
                self.attributes.strength = Some(std::cmp::min(50, self.attributes.strength.unwrap() + item.effect));
            },
            _ => ()
        }

        if item.quantity <= 0 {
            // No uses left
            TributeModel::from(self.clone()).uses_consumable(item.id.unwrap());
        } else {
            // Update item quantity
            update_item(models::UpdateItem::from(item.clone()).into());
        }
        update_tribute(self.id.unwrap(), self.clone().into());
        true
    }

    pub fn items(&self) -> Vec<Item> {
        let items = models::item::Item::get_by_tribute(self.game_id.unwrap(), self.id.unwrap());
        items.iter().filter(|i| i.quantity > 0).cloned().map(Item::from).collect()
    }

    pub fn weapons(&self) -> Vec<Item> {
        self.items().iter().cloned().filter(|i| i.is_weapon()).collect()
    }

    pub fn defensive_items(&self) -> Vec<Item> {
        self.items().iter().cloned().filter(|i| i.is_defensive()).collect()
    }

    pub fn consumable_items(&self) -> Vec<Item> {
        self.items().iter().cloned().filter(|i| i.is_consumable()).collect()
    }
}

#[derive(Debug)]
pub enum TravelResult {
    Success(Area),
    Failure,
}

#[allow(dead_code)]
fn apply_violence_stress(tribute: &mut Tribute) {
    let kills = tribute.statistics.kills.unwrap_or(0);
    let wins = tribute.statistics.wins.unwrap_or(0);
    let sanity = tribute.attributes.sanity;
    let mut terror = 20.0;

    if kills + wins > 0 {
        terror = (100.0 / (kills + wins) as f64) * (sanity as f64 / 100.0) / 2.0;
    }

    if terror.round() > 0.0 {
        create_full_log(
            tribute.game_id.unwrap(),
            GameMessage::TributeHorrified(tribute.clone(), terror.round() as i32).to_string(),
            Some(tribute.area.clone().unwrap().id()),
            Some(tribute.id.unwrap()),
            None,
            None
        );
        tribute.takes_mental_damage(terror.round() as i32);
    }
}

fn attack_contest(attacker: Tribute, target: Tribute) -> AttackResult {
    let mut tribute1_roll = thread_rng().gen_range(1..=20); // Base roll
    tribute1_roll += attacker.attributes.strength.unwrap(); // Add strength

    if let Some(weapon) = attacker.weapons().iter_mut().last() {
        tribute1_roll += weapon.effect; // Add weapon damage
        weapon.quantity -= 1;
        if weapon.quantity <= 0 {
            create_full_log(
                attacker.game_id.unwrap(),
                GameMessage::WeaponBreak(attacker.clone(), weapon.clone()).to_string(),
                Some(attacker.area.clone().unwrap().id()),
                Some(attacker.id.unwrap()),
                Some("Weapon".to_string()),
                Some(weapon.id.unwrap())
            );
            weapon.delete();
        }
        update_item(models::UpdateItem::from(weapon.clone()).into());
    }

    // Add luck in here?

    let mut tribute2_roll = thread_rng().gen_range(1..=20); // Base roll
    tribute2_roll += target.attributes.defense.unwrap(); // Add defense

    if let Some(shield) = target.items().iter_mut().filter(|i| i.is_defensive()).next() {
        tribute2_roll += shield.effect; // Add weapon defense
        shield.quantity -= 1;
        if shield.quantity <= 0 {
            create_full_log(
                target.game_id.unwrap(),
                GameMessage::ShieldBreak(target.clone(), shield.clone()).to_string(),
                Some(target.area.clone().unwrap().id()),
                Some(target.id.unwrap()),
                Some("Shield".to_string()),
                Some(shield.id.unwrap())
            );
            shield.delete();
        }
        update_item(models::UpdateItem::from(shield.clone()).into());
    }

    let response = {
        if tribute1_roll > tribute2_roll {
            if tribute1_roll >= tribute2_roll + 5 { // Attacker wins significantly
                AttackResult::AttackerWinsDecisively
            } else {
                AttackResult::AttackerWins
            }
        } else if tribute2_roll > tribute1_roll {
            if tribute2_roll >= tribute1_roll + 5 { // Defender wins significantly
                AttackResult::DefenderWinsDecisively
            } else {
                AttackResult::DefenderWins
            }
        } else {
            AttackResult::Miss
        }
    };
    response
}

pub fn pick_target(tribute: TributeModel) -> Option<Tribute> {
    let area = get_area_by_id(tribute.area_id).unwrap();
    let tributes = area.tributes(tribute.game_id.unwrap()).iter()
        .map(|t| Tribute::from(t.clone()))
        .filter(|t| t.is_alive())
        .filter(|t| t.id.unwrap() != tribute.id)
        .collect::<Vec<_>>();

    match tributes.len() {
        0 => { // there are no other targets
            match tribute.sanity {
                0..=9 => { // attempt suicide
                    create_full_log(
                        tribute.game_id.unwrap(),
                        GameMessage::TributeSuicide(Tribute::from(tribute.clone())).to_string(),
                        Some(area.id),
                        Some(tribute.id),
                        Some("Tribute".to_string()),
                        Some(tribute.id)
                    );
                    Some(tribute.into())
                },
                10..=19 => match thread_rng().gen_bool(0.2) {
                    true => { // attempt suicide
                        create_full_log(
                            tribute.game_id.unwrap(),
                            GameMessage::TributeSuicide(Tribute::from(tribute.clone())).to_string(),
                            Some(area.id),
                            Some(tribute.id),
                            Some("Tribute".to_string()),
                            Some(tribute.id)
                        );
                        Some(tribute.into())
                    },
                    false => None, // Attack no one
                },
                _ => None, // Attack no one
            }
        },
        _ => {
            let mut targets = tributes.clone();
            let enemy_targets: Vec<Tribute> = targets.iter().cloned()
                .filter(|t| t.district != tribute.district)
                .filter(|t| t.is_visible())
                .collect();

            match tribute.sanity {
                0..20 => (), // Sanity is low, target everyone
                _ => targets = enemy_targets.clone() // Sane enough not to attack district mate
            }

            match targets.len() {
                0 | 1 => Some(targets.first()?.clone()), // Easy choice
                _ => {
                    let mut rng = thread_rng();
                    Some(targets.choose(&mut rng)?.clone()) // Get a random enemy
                }
            }
        }
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
    /// How many games have they survived?
    pub games: u32,
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
            health: 100,
            sanity: 100,
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

    #[test]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), None, None);
        assert_eq!(tribute.attributes.health, 100);
        assert_eq!(tribute.attributes.sanity, 100);
        assert_eq!(tribute.attributes.movement, 100);
        assert_eq!(tribute.status, TributeStatus::Healthy);
    }

    #[test]
    fn takes_physical_damage() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.takes_physical_damage(10);
        assert_eq!(tribute.attributes.health, 90);
    }

    #[test]
    fn takes_mental_damage() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.takes_mental_damage(10);
        assert_eq!(tribute.attributes.sanity, 90);
    }

    #[test]
    fn moves_and_rests() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.attributes.speed = 50;
        tribute.moves();
        assert_eq!(tribute.attributes.movement, 50);
        tribute.short_rests();
        assert_eq!(tribute.attributes.movement, 100);
    }

    #[test]
    fn is_hidden_true() {
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.attributes.intelligence = 100;
        tribute.attributes.is_hidden = true;
        assert!(!tribute.is_visible());
    }
}
