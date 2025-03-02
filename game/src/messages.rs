use crate::areas::events::AreaEvent;
use crate::areas::Area;
use crate::items::Item;
use crate::threats::animals::Animal;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::Tribute;
use std::fmt::{Display, Formatter};

// Collection on strings to be used as output for the game
#[allow(dead_code)]
pub enum GameMessage {
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

impl Display for GameMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GameMessage::GameDayStart(day_number) => {
                write!(f, "=== ☀️ Day {} begins! ===", day_number)
            }
            GameMessage::FirstDayStart => {
                write!(f, "=== 🎉 The Hunger Games begin! 🎉 ===")
            }
            GameMessage::FeastDayStart => {
                write!(f, "=== 😋 Day 3: Feast Day ===")
            }
            GameMessage::TributesLeft(tribute_count) => {
                write!(f, "=== 📌 Tributes alive: {} ===", tribute_count)
            }
            GameMessage::GameNightStart(day_number) => {
                write!(f, "=== 🌙 Night {} begins ===", day_number)
            }
            GameMessage::DailyDeathAnnouncement(death_count) => {
                write!(f, "=== 💀 Tributes dead: {} ===", death_count)
            }
            GameMessage::DeathAnnouncement(tribute) => {
                write!(f, "=== 🪦 {} has died ===", tribute.name)
            }
            GameMessage::NoOneWins => {
                write!(f, "=== 🎭 No one wins! ===")
            }
            GameMessage::TributeWins(tribute) => {
                write!(f, "=== 🏆 The winner is {} ===", tribute.name)
            }
            GameMessage::TributeRest(tribute) => {
                write!(f, "😪 {} rests", tribute.name)
            }
            GameMessage::TributeLongRest(tribute) => {
                write!(f, "💤 {} rests and recovers a little health and sanity", tribute.name)
            }
            GameMessage::TributeHide(tribute) => {
                write!(f, "🫥 {} tries to hide", tribute.name)
            }
            GameMessage::TributeTravel(tribute, area_a, area_b) => {
                write!(f, "🚶 {} moves from {} to {}", tribute.name, area_a, area_b)
            }
            GameMessage::TributeTakeItem(tribute, item) => {
                write!(f, "🔨 {} takes a(n) {}", tribute.name, item.name)
            }
            GameMessage::TributeCannotUseItem(tribute, item) => {
                write!(f, "❌ {} cannot use a(n) {}", tribute.name, item.name)
            }
            GameMessage::TributeUseItem(tribute, item) => {
                write!(f, "💊 {} uses a(n) {}, gains {} {}", tribute.name, item.name, item.effect, item.attribute)
            }
            GameMessage::TributeTravelTooTired(tribute, area) => {
                write!(f, "😴 {} is too tired to move from {}, rests instead", tribute.name, area)
            }
            GameMessage::TributeTravelAlreadyThere(tribute, area) => {
                write!(f, "🤔 {} is already in the {}, stays put", tribute.name, area)
            }
            GameMessage::TributeTravelFollow(tribute, area) => {
                write!(f, "🫡 {} follows their district mate to {}", tribute.name, area)
            }
            GameMessage::TributeTravelStay(tribute, area) => {
                write!(f, "🪑 {} stays in {}", tribute.name, area)
            }
            GameMessage::TributeBleeds(tribute) => {
                write!(f, "🩸 {} bleeds from their wounds.", tribute.name)
            }
            GameMessage::TributeSick(tribute) => {
                write!(f, "🤒 {} contracts dysentery, loses strength and speed", tribute.name)
            }
            GameMessage::TributeElectrocuted(tribute) => {
                write!(f, "🌩️ {} is struck by lightning, loses health", tribute.name)
            }
            GameMessage::TributeFrozen(tribute) => {
                write!(f, "🥶 {} suffers from hypothermia, loses speed.", tribute.name)
            }
            GameMessage::TributeOverheated(tribute) => {
                write!(f, "🥵 {} suffers from heat stroke, loses speed.", tribute.name)
            }
            GameMessage::TributeDehydrated(tribute) => {
                write!(f, "🌵 {} is severely dehydrated, loses strength", tribute.name)
            }
            GameMessage::TributeStarving(tribute) => {
                write!(f, "🍴 {} is ravenously hungry, loses strength", tribute.name)
            }
            GameMessage::TributePoisoned(tribute) => {
                write!(f, "🧪 {} eats something poisonous, loses sanity", tribute.name)
            }
            GameMessage::TributeBrokenArm(tribute) => {
                write!(f, "🦴 {} injures their arm, loses strength.", tribute.name)
            }
            GameMessage::TributeBrokenLeg(tribute) => {
                write!(f, "🦴 {} injures their leg, loses speed.", tribute.name)
            }
            GameMessage::TributeInfected(tribute) => {
                write!(f, "🤢 {} gets an infection, loses health and sanity", tribute.name)
            }
            GameMessage::TributeDrowned(tribute) => {
                write!(f, "🏊 {} partially drowns, loses health and sanity", tribute.name)
            }
            GameMessage::TributeMauled(tribute, count, animal, damage) => {
                write!(f, "🐾 {} is attacked by {} {}, takes {} damage!", tribute.name, count, animal.plural(), damage)
            }
            GameMessage::TributeBurned(tribute) => {
                write!(f, "🔥 {} gets burned, loses health", tribute.name)
            }
            GameMessage::TributeHorrified(tribute, damage) => {
                write!(f, "😱 {} is horrified by the violence, loses {} sanity.", tribute.name, damage)
            }
            GameMessage::TributeSuffer(tribute) => {
                write!(f, "😭 {} suffers from loneliness and terror.", tribute.name)
            }
            GameMessage::TributeSelfHarm(tribute) => {
                write!(f, "🤦 {} tries to attack themself!", tribute.name)
            }
            GameMessage::TributeSuicide(tribute) => {
                write!(f, "🪒 {} attempts suicide.", tribute.name)
            }
            GameMessage::TributeAttackWin(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins!", tribute.name, target.name)
            }
            GameMessage::TributeAttackWinExtra(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins decisively!", tribute.name, target.name)
            }
            GameMessage::TributeAttackWound(tribute, target) => {
                write!(f, "🤕 {} wounds {}", tribute.name, target.name)
            }
            GameMessage::TributeAttackLose(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses!", tribute.name, target.name)
            }
            GameMessage::TributeAttackLoseExtra(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses decisively!", tribute.name, target.name)
            }
            GameMessage::TributeAttackMiss(tribute, target) => {
                write!(f, "😰 {} attacks {}, but misses!", tribute.name, target.name)
            }
            GameMessage::TributeAttackDied(tribute, target) => {
                write!(f, "☠️ {} is killed by {}", tribute.name, target.name)
            }
            GameMessage::TributeAttackSuccessKill(tribute, target) => {
                write!(f, "☠️ {} successfully kills {}", tribute.name, target.name)
            }
            GameMessage::TributeAttackHidden(tribute, target) => {
                write!(f, "🤔 {} can't attack {}, they're hidden", tribute.name, target.name)
            }
            GameMessage::TributeDiesFromStatus(tribute, status) => {
                write!(f, "💀 {} dies from {}", tribute.name, status)
            }
            GameMessage::TributeDiesFromAreaEvent(tribute, area_event) => {
                write!(f, "🪦 {} died in the {}.", tribute.name, area_event)
            }
            GameMessage::TributeDiesFromTributeEvent(tribute, tribute_event) => {
                write!(f, "💀 {} dies by {}", tribute.name, tribute_event)
            }
            GameMessage::TributeAlreadyDead(tribute) => {
                write!(f, "‼️ {} is already dead!", tribute.name)
            }
            GameMessage::TributeDead(tribute) => {
                write!(f, "❗️ {} is dead!", tribute.name)
            }
            GameMessage::WeaponBreak(tribute, weapon) => {
                write!(f, "🗡️ {} breaks their {}", tribute.name, weapon.name)
            }
            GameMessage::ShieldBreak(tribute, shield) => {
                write!(f, "🛡️ {} breaks their {}", tribute.name, shield.name)
            }
            GameMessage::SponsorGift(tribute, item) => {
                write!(f, "🎁 {} receives a(n) {} ({}x {} +{})", tribute.name, item.name, item.quantity, item.attribute, item.effect)
            }
            GameMessage::AreaEvent(area_event, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== ⚠️ A(n) {} has occurred in the {} ===", area_event, area_name)
            }
            GameMessage::AreaClose(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== 🔔 The {} is uninhabitable ===", area_name)
            }
            GameMessage::AreaOpen(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "=== 🔔 The {} is habitable again ===", area_name)
            }
            GameMessage::TrappedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "💥 {} is trapped in the {}.", tribute.name, area_name)
            }
            GameMessage::DiedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(f, "💥 {} died in the {}.", tribute.name, area_name)
            }
        }
    }
}
