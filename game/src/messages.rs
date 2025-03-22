use crate::areas::events::AreaEvent;
use crate::areas::Area;
use crate::items::Item;
use crate::threats::animals::Animal;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;
use uuid::Uuid;

// Collection on strings to be used as output for the game
#[allow(dead_code)]
pub enum GameOutput {
    GameDayStart(u32),
    FirstDayStart,
    FeastDayStart,
    TributesLeft(u32),
    GameNightStart(u32),
    DailyDeathAnnouncement(u32),
    DeathAnnouncement(Tribute),
    NoOneWins,
    TributeWins(Tribute),
    TributeRest(Tribute),
    TributeLongRest(Tribute),
    TributeHide(Tribute),
    TributeTravel(Tribute, Area, Area),
    TributeTakeItem(Tribute, Item),
    TributeCannotUseItem(Tribute, Item),
    TributeUseItem(Tribute, Item),
    TributeTravelTooTired(Tribute, Area),
    TributeTravelAlreadyThere(Tribute, Area),
    TributeTravelFollow(Tribute, Area),
    TributeTravelStay(Tribute, Area),
    TributeBleeds(Tribute),
    TributeSick(Tribute),
    TributeElectrocuted(Tribute),
    TributeFrozen(Tribute),
    TributeOverheated(Tribute),
    TributeDehydrated(Tribute),
    TributeStarving(Tribute),
    TributePoisoned(Tribute),
    TributeBrokenArm(Tribute),
    TributeBrokenLeg(Tribute),
    TributeInfected(Tribute),
    TributeDrowned(Tribute),
    TributeMauled(Tribute, u32, Animal, u32),
    TributeBurned(Tribute),
    TributeHorrified(Tribute, u32),
    TributeSuffer(Tribute),
    TributeSelfHarm(Tribute),
    TributeSuicide(Tribute),
    TributeAttackWin(Tribute, Tribute),
    TributeAttackWinExtra(Tribute, Tribute),
    TributeAttackWound(Tribute, Tribute),
    TributeAttackLose(Tribute, Tribute),
    TributeAttackLoseExtra(Tribute, Tribute),
    TributeAttackMiss(Tribute, Tribute),
    TributeAttackDied(Tribute, Tribute),
    TributeAttackSuccessKill(Tribute, Tribute),
    TributeAttackHidden(Tribute, Tribute),
    TributeDiesFromStatus(Tribute, TributeStatus),
    TributeDiesFromAreaEvent(Tribute, AreaEvent), // Died in area
    TributeDiesFromTributeEvent(Tribute, TributeEvent),
    TributeAlreadyDead(Tribute),
    TributeDead(Tribute),
    WeaponBreak(Tribute, Item),
    ShieldBreak(Tribute, Item),
    SponsorGift(Tribute, Item),
    AreaEvent(AreaEvent, Area),
    AreaClose(Area),
    AreaOpen(Area),
    TrappedInArea(Tribute, Area),
    DiedInArea(Tribute, Area),
}

impl Display for GameOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GameOutput::GameDayStart(day_number) => {
                write!(f, "=== ☀️ Day {} begins! ===", day_number)
            }
            GameOutput::FirstDayStart => {
                write!(f, "=== 🎉 The Hunger Games begin! 🎉 ===")
            }
            GameOutput::FeastDayStart => {
                write!(f, "=== 😋 Day 3: Feast Day ===")
            }
            GameOutput::TributesLeft(tribute_count) => {
                write!(f, "=== 📌 Tributes alive: {} ===", tribute_count)
            }
            GameOutput::GameNightStart(day_number) => {
                write!(f, "=== 🌙 Night {} begins ===", day_number)
            }
            GameOutput::DailyDeathAnnouncement(death_count) => {
                write!(f, "=== 💀 Tributes dead: {} ===", death_count)
            }
            GameOutput::DeathAnnouncement(tribute) => {
                write!(f, "=== 🪦 {} has died ===", tribute.name)
            }
            GameOutput::NoOneWins => {
                write!(f, "=== 🎭 No one wins! ===")
            }
            GameOutput::TributeWins(tribute) => {
                write!(f, "=== 🏆 The winner is {} ===", tribute.name)
            }
            GameOutput::TributeRest(tribute) => {
                write!(f, "😪 {} rests", tribute.name)
            }
            GameOutput::TributeLongRest(tribute) => {
                write!(f, "💤 {} rests and recovers a little health and sanity", tribute.name)
            }
            GameOutput::TributeHide(tribute) => {
                write!(f, "🫥 {} tries to hide", tribute.name)
            }
            GameOutput::TributeTravel(tribute, area_a, area_b) => {
                write!(f, "🚶 {} moves from {} to {}", tribute.name, area_a, area_b)
            }
            GameOutput::TributeTakeItem(tribute, item) => {
                write!(f, "🔨 {} takes a(n) {}", tribute.name, item.name)
            }
            GameOutput::TributeCannotUseItem(tribute, item) => {
                write!(f, "❌ {} cannot use a(n) {}", tribute.name, item.name)
            }
            GameOutput::TributeUseItem(tribute, item) => {
                write!(f, "💊 {} uses a(n) {}, gains {} {}", tribute.name, item.name, item.effect, item.attribute)
            }
            GameOutput::TributeTravelTooTired(tribute, area) => {
                write!(f, "😴 {} is too tired to move from {}, rests instead", tribute.name, area)
            }
            GameOutput::TributeTravelAlreadyThere(tribute, area) => {
                write!(f, "🤔 {} is already in the {}, stays put", tribute.name, area)
            }
            GameOutput::TributeTravelFollow(tribute, area) => {
                write!(f, "🫡 {} follows their district mate to {}", tribute.name, area)
            }
            GameOutput::TributeTravelStay(tribute, area) => {
                write!(f, "🪑 {} stays in {}", tribute.name, area)
            }
            GameOutput::TributeBleeds(tribute) => {
                write!(f, "🩸 {} bleeds from their wounds.", tribute.name)
            }
            GameOutput::TributeSick(tribute) => {
                write!(f, "🤒 {} contracts dysentery, loses strength and speed", tribute.name)
            }
            GameOutput::TributeElectrocuted(tribute) => {
                write!(f, "🌩️ {} is struck by lightning, loses health", tribute.name)
            }
            GameOutput::TributeFrozen(tribute) => {
                write!(f, "🥶 {} suffers from hypothermia, loses speed.", tribute.name)
            }
            GameOutput::TributeOverheated(tribute) => {
                write!(f, "🥵 {} suffers from heat stroke, loses speed.", tribute.name)
            }
            GameOutput::TributeDehydrated(tribute) => {
                write!(f, "🌵 {} is severely dehydrated, loses strength", tribute.name)
            }
            GameOutput::TributeStarving(tribute) => {
                write!(f, "🍴 {} is ravenously hungry, loses strength", tribute.name)
            }
            GameOutput::TributePoisoned(tribute) => {
                write!(f, "🧪 {} eats something poisonous, loses sanity", tribute.name)
            }
            GameOutput::TributeBrokenArm(tribute) => {
                write!(f, "🦴 {} injures their arm, loses strength.", tribute.name)
            }
            GameOutput::TributeBrokenLeg(tribute) => {
                write!(f, "🦴 {} injures their leg, loses speed.", tribute.name)
            }
            GameOutput::TributeInfected(tribute) => {
                write!(f, "🤢 {} gets an infection, loses health and sanity", tribute.name)
            }
            GameOutput::TributeDrowned(tribute) => {
                write!(f, "🏊 {} partially drowns, loses health and sanity", tribute.name)
            }
            GameOutput::TributeMauled(tribute, count, animal, damage) => {
                write!(f, "🐾 {} is attacked by {} {}, takes {} damage!", tribute.name, count, animal.plural(), damage)
            }
            GameOutput::TributeBurned(tribute) => {
                write!(f, "🔥 {} gets burned, loses health", tribute.name)
            }
            GameOutput::TributeHorrified(tribute, damage) => {
                write!(f, "😱 {} is horrified by the violence, loses {} sanity.", tribute.name, damage)
            }
            GameOutput::TributeSuffer(tribute) => {
                write!(f, "😭 {} suffers from loneliness and terror.", tribute.name)
            }
            GameOutput::TributeSelfHarm(tribute) => {
                write!(f, "🤦 {} tries to attack themself!", tribute.name)
            }
            GameOutput::TributeSuicide(tribute) => {
                write!(f, "🪒 {} attempts suicide.", tribute.name)
            }
            GameOutput::TributeAttackWin(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins!", tribute.name, target.name)
            }
            GameOutput::TributeAttackWinExtra(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins decisively!", tribute.name, target.name)
            }
            GameOutput::TributeAttackWound(tribute, target) => {
                write!(f, "🤕 {} wounds {}", tribute.name, target.name)
            }
            GameOutput::TributeAttackLose(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses!", tribute.name, target.name)
            }
            GameOutput::TributeAttackLoseExtra(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses decisively!", tribute.name, target.name)
            }
            GameOutput::TributeAttackMiss(tribute, target) => {
                write!(f, "😰 {} attacks {}, but misses!", tribute.name, target.name)
            }
            GameOutput::TributeAttackDied(tribute, target) => {
                write!(f, "☠️ {} is killed by {}", tribute.name, target.name)
            }
            GameOutput::TributeAttackSuccessKill(tribute, target) => {
                write!(f, "☠️ {} successfully kills {}", tribute.name, target.name)
            }
            GameOutput::TributeAttackHidden(tribute, target) => {
                write!(f, "🤔 {} can't attack {}, they're hidden", tribute.name, target.name)
            }
            GameOutput::TributeDiesFromStatus(tribute, status) => {
                write!(f, "💀 {} dies from {}", tribute.name, status)
            }
            GameOutput::TributeDiesFromAreaEvent(tribute, area_event) => {
                write!(f, "🪦 {} died in the {}.", tribute.name, area_event)
            }
            GameOutput::TributeDiesFromTributeEvent(tribute, tribute_event) => {
                write!(f, "💀 {} dies by {}", tribute.name, tribute_event)
            }
            GameOutput::TributeAlreadyDead(tribute) => {
                write!(f, "‼️ {} is already dead!", tribute.name)
            }
            GameOutput::TributeDead(tribute) => {
                write!(f, "❗️ {} is dead!", tribute.name)
            }
            GameOutput::WeaponBreak(tribute, weapon) => {
                write!(f, "🗡️ {} breaks their {}", tribute.name, weapon.name)
            }
            GameOutput::ShieldBreak(tribute, shield) => {
                write!(f, "🛡️ {} breaks their {}", tribute.name, shield.name)
            }
            GameOutput::SponsorGift(tribute, item) => {
                write!(f, "🎁 {} receives a(n) {} ({}x {} +{})", tribute.name, item.name, item.quantity, item.attribute, item.effect)
            }
            GameOutput::AreaEvent(area_event, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== ⚠️ A(n) {} has occurred in the {} ===", area_event, area_name)
            }
            GameOutput::AreaClose(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== 🔔 The {} is uninhabitable ===", area_name)
            }
            GameOutput::AreaOpen(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== 🔔 The {} is habitable again ===", area_name)
            }
            GameOutput::TrappedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "💥 {} is trapped in the {}.", tribute.name, area_name)
            }
            GameOutput::DiedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "💥 {} died in the {}.", tribute.name, area_name)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag="type", content="value")]
pub enum MessageSource {
    #[serde(rename = "Game")]
    Game(String), // Game identifier
    #[serde(rename = "Area")]
    Area(String), // Area name
    #[serde(rename = "Tribute")]
    Tribute(String), // Tribute identifier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

pub static GLOBAL_MESSAGES: Lazy<Mutex<VecDeque<GameMessage>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

pub fn add_message(
    source: MessageSource,
    game_day: u32,
    subject: String,
    content: String,
) -> Result<(), String> {
    let message = GameMessage {
        identifier: Uuid::new_v4().to_string(),
        source,
        game_day,
        subject,
        timestamp: Utc::now(),
        content,
    };

    GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .push_back(message);

    Ok(())
}

pub fn add_game_message(game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Game(game_id.to_string()),
        0,
        game_id.to_string(),
        content,
    )
}

pub fn add_area_message(area_name: &str, game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Area(area_name.to_string()),
        0,
        format!("{game_id}:{area_name}"),
        content,
    )
}

pub fn add_tribute_message(tribute_id: &str, game_id: &str, content: String) -> Result<(), String> {
    add_message(
        MessageSource::Tribute(tribute_id.to_string()),
        0,
        format!("{game_id}:{tribute_id}"),
        content,
    )
}

pub fn get_all_messages() -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .cloned()
        .collect())
}

pub fn get_messages_by_source(source: &MessageSource) -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|msg| msg.source == *source)
        .cloned()
        .collect())
}

pub fn get_messages_by_day(day: u32) -> Result<Vec<GameMessage>, String> {
    Ok(GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|msg| msg.game_day == day)
        .cloned()
        .collect())
}

pub fn clear_messages() -> Result<(), String> {
    GLOBAL_MESSAGES
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    Ok(())
}

