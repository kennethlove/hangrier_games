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
                write!(f, "{}", format!("=== ☀️ Day {} begins! ===", day_number))
            }
            GameMessage::FirstDayStart => {
                write!(f, "=== 🎉 The Hunger Games begin! 🎉 ===")
            }
            GameMessage::FeastDayStart => {
                write!(f, "=== 😋 Day 3: Feast Day ===")
            }
            GameMessage::TributesLeft(tribute_count) => {
                write!(
                    f,
                    "{}",
                    format!("=== 📌 Tributes alive: {} ===", tribute_count)
                )
            }
            GameMessage::GameNightStart(day_number) => {
                write!(f, "{}", format!("=== 🌙 Night {} begins ===", day_number))
            }
            GameMessage::DailyDeathAnnouncement(death_count) => {
                write!(
                    f,
                    "{}",
                    format!("=== 💀 Tributes dead: {} ===", death_count)
                )
            }
            GameMessage::DeathAnnouncement(tribute) => {
                write!(f, "{}", format!("=== 🪦 {} has died ===", tribute.name))
            }
            GameMessage::NoOneWins => {
                write!(f, "=== 🎭 No one wins! ===")
            }
            GameMessage::TributeWins(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("=== 🏆 The winner is {} ===", tribute.name)
                )
            }
            GameMessage::TributeRest(tribute) => {
                write!(f, "{}", format!("😪 {} rests", tribute.name))
            }
            GameMessage::TributeLongRest(tribute) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "💤 {} rests and recovers a little health and sanity",
                        tribute.name
                    )
                )
            }
            GameMessage::TributeHide(tribute) => {
                write!(f, "{}", format!("🫥 {} tries to hide", tribute.name))
            }
            GameMessage::TributeTravel(tribute, area_a, area_b) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🚶 {} moves from {} to {}",
                        tribute.name,
                        area_a.to_string(),
                        area_b.to_string()
                    )
                )
            }
            GameMessage::TributeTakeItem(tribute, item) => {
                write!(
                    f,
                    "{}",
                    format!("🔨 {} takes a(n) {}", tribute.name, item.name)
                )
            }
            GameMessage::TributeCannotUseItem(tribute, item) => {
                write!(
                    f,
                    "{}",
                    format!("❌ {} cannot use a(n) {}", tribute.name, item.name)
                )
            }
            GameMessage::TributeUseItem(tribute, item) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "💊 {} uses a(n) {}, gains {} {}",
                        tribute.name, item.name, item.effect, item.attribute
                    )
                )
            }
            GameMessage::TributeTravelTooTired(tribute, area) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "😴 {} is too tired to move from {}, rests instead",
                        tribute.name,
                        area.to_string()
                    )
                )
            }
            GameMessage::TributeTravelAlreadyThere(tribute, area) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🤔 {} is already in the {}, stays put",
                        tribute.name,
                        area.to_string()
                    )
                )
            }
            GameMessage::TributeTravelFollow(tribute, area) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🫡 {} follows their district mate to {}",
                        tribute.name,
                        area.to_string()
                    )
                )
            }
            GameMessage::TributeTravelStay(tribute, area) => {
                write!(
                    f,
                    "{}",
                    format!("🪑 {} stays in {}", tribute.name, area.to_string())
                )
            }
            GameMessage::TributeBleeds(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🩸 {} bleeds from their wounds.", tribute.name)
                )
            }
            GameMessage::TributeSick(tribute) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🤒 {} contracts dysentery, loses strength and speed",
                        tribute.name
                    )
                )
            }
            GameMessage::TributeElectrocuted(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🌩️ {} is struck by lightning, loses health", tribute.name)
                )
            }
            GameMessage::TributeFrozen(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🥶 {} suffers from hypothermia, loses speed.", tribute.name)
                )
            }
            GameMessage::TributeOverheated(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🥵 {} suffers from heat stroke, loses speed.", tribute.name)
                )
            }
            GameMessage::TributeDehydrated(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🌵 {} is severely dehydrated, loses strength", tribute.name)
                )
            }
            GameMessage::TributeStarving(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🍴 {} is ravenously hungry, loses strength", tribute.name)
                )
            }
            GameMessage::TributePoisoned(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🧪 {} eats something poisonous, loses sanity", tribute.name)
                )
            }
            GameMessage::TributeBrokenArm(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🦴 {} injures their arm, loses strength.", tribute.name)
                )
            }
            GameMessage::TributeBrokenLeg(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🦴 {} injures their leg, loses speed.", tribute.name)
                )
            }
            GameMessage::TributeInfected(tribute) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🤢 {} gets an infection, loses health and sanity",
                        tribute.name
                    )
                )
            }
            GameMessage::TributeDrowned(tribute) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🏊 {} partially drowns, loses health and sanity",
                        tribute.name
                    )
                )
            }
            GameMessage::TributeMauled(tribute, count, animal, damage) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🐾 {} is attacked by {} {}, takes {} damage!",
                        tribute.name,
                        count,
                        animal.plural(),
                        damage
                    )
                )
            }
            GameMessage::TributeBurned(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🔥 {} gets burned, loses health", tribute.name)
                )
            }
            GameMessage::TributeHorrified(tribute, damage) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "😱 {} is horrified by the violence, loses {} sanity.",
                        tribute.name, damage
                    )
                )
            }
            GameMessage::TributeSuffer(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("😭 {} suffers from loneliness and terror.", tribute.name)
                )
            }
            GameMessage::TributeSelfHarm(tribute) => {
                write!(
                    f,
                    "{}",
                    format!("🤦 {} tries to attack themself!", tribute.name)
                )
            }
            GameMessage::TributeSuicide(tribute) => {
                write!(f, "{}", format!("🪒 {} attempts suicide.", tribute.name))
            }
            GameMessage::TributeAttackWin(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("🔪 {} attacks {}, and wins!", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackWinExtra(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🔪 {} attacks {}, and wins decisively!",
                        tribute.name, target.name
                    )
                )
            }
            GameMessage::TributeAttackWound(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("🤕 {} wounds {}", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackLose(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("🤣 {} attacks {}, but loses!", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackLoseExtra(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🤣 {} attacks {}, but loses decisively!",
                        tribute.name, target.name
                    )
                )
            }
            GameMessage::TributeAttackMiss(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("😰 {} attacks {}, but misses!", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackDied(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("☠️ {} is killed by {}", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackSuccessKill(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!("☠️ {} successfully kills {}", tribute.name, target.name)
                )
            }
            GameMessage::TributeAttackHidden(tribute, target) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🤔 {} can't attack {}, they're hidden",
                        tribute.name, target.name
                    )
                )
            }
            GameMessage::TributeDiesFromStatus(tribute, status) => {
                write!(
                    f,
                    "{}",
                    format!("💀 {} dies from {}", tribute.name, status.to_string())
                )
            }
            GameMessage::TributeDiesFromAreaEvent(tribute, area_event) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🪦 {} died in the {}.",
                        tribute.name,
                        area_event.to_string()
                    )
                )
            }
            GameMessage::TributeDiesFromTributeEvent(tribute, tribute_event) => {
                write!(
                    f,
                    "{}",
                    format!("💀 {} dies by {}", tribute.name, tribute_event.to_string())
                )
            }
            GameMessage::TributeAlreadyDead(tribute) => {
                write!(f, "{}", format!("‼️ {} is already dead!", tribute.name))
            }
            GameMessage::TributeDead(tribute) => {
                write!(f, "{}", format!("❗️ {} is dead!", tribute.name))
            }
            GameMessage::WeaponBreak(tribute, weapon) => {
                write!(
                    f,
                    "{}",
                    format!("🗡️ {} breaks their {}", tribute.name, weapon.name)
                )
            }
            GameMessage::ShieldBreak(tribute, shield) => {
                write!(
                    f,
                    "{}",
                    format!("🛡️ {} breaks their {}", tribute.name, shield.name)
                )
            }
            GameMessage::SponsorGift(tribute, item) => {
                write!(
                    f,
                    "{}",
                    format!(
                        "🎁 {} receives a(n) {} ({}x {} +{})",
                        tribute.name, item.name, item.quantity, item.attribute, item.effect
                    )
                )
            }
            GameMessage::AreaEvent(area_event, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(
                    f,
                    "{}",
                    format!(
                        "=== ⚠️ A(n) {} has occurred in the {} ===",
                        area_event.to_string(),
                        area_name
                    )
                )
            }
            GameMessage::AreaClose(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(
                    f,
                    "{}",
                    format!("=== 🔔 The {} is uninhabitable ===", area_name)
                )
            }
            GameMessage::AreaOpen(area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(
                    f,
                    "{}",
                    format!("=== 🔔 The {} is habitable again ===", area_name)
                )
            }
            GameMessage::TrappedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(
                    f,
                    "{}",
                    format!("💥 {} is trapped in the {}.", tribute.name, area_name)
                )
            }
            GameMessage::DiedInArea(tribute, area) => {
                let area_name = area.to_string().replace("The ", "");
                write!(
                    f,
                    "{}",
                    format!("💥 {} died in the {}.", tribute.name, area_name)
                )
            }
        }
    }
}
