use crate::items::Item;
use crate::threats::animals::Animal;
use indefinite::indefinite;
use indefinite::indefinite_capitalized;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

// Collection on strings to be used as output for the game
#[allow(dead_code)]
pub enum GameOutput<'a> {
    GameDayStart(u32),
    GameDayEnd(u32),
    FirstDayStart,
    FeastDayStart,
    TributesLeft(u32),
    GameNightStart(u32),
    GameNightEnd(u32),
    DailyDeathAnnouncement(u32),
    DeathAnnouncement(&'a str),
    NoOneWins,
    TributeWins(&'a str),
    TributeRest(&'a str),
    TributeLongRest(&'a str),
    TributeHide(&'a str),
    TributeTravel(&'a str, &'a str, &'a str),
    TributeTakeItem(&'a str, &'a str),
    TributeCannotUseItem(&'a str, &'a str),
    TributeUseItem(&'a str, &'a Item),
    TributeTravelTooTired(&'a str, &'a str),
    TributeTravelAlreadyThere(&'a str, &'a str),
    TributeTravelFollow(&'a str, &'a str),
    TributeTravelStay(&'a str, &'a str),
    TributeBleeds(&'a str),
    TributeSick(&'a str),
    TributeElectrocuted(&'a str),
    TributeFrozen(&'a str),
    TributeOverheated(&'a str),
    TributeDehydrated(&'a str),
    TributeStarving(&'a str),
    TributePoisoned(&'a str),
    TributeBrokenArm(&'a str),
    TributeBrokenLeg(&'a str),
    TributeInfected(&'a str),
    TributeDrowned(&'a str),
    TributeMauled(&'a str, u32, &'a str, u32),
    TributeBurned(&'a str),
    TributeHorrified(&'a str, u32),
    TributeSuffer(&'a str),
    TributeSelfHarm(&'a str),
    TributeSuicide(&'a str),
    TributeAttackWin(&'a str, &'a str),
    TributeAttackWinExtra(&'a str, &'a str),
    TributeAttackWound(&'a str, &'a str),
    TributeAttackLose(&'a str, &'a str),
    TributeAttackLoseExtra(&'a str, &'a str),
    TributeAttackMiss(&'a str, &'a str),
    TributeAttackDied(&'a str, &'a str),
    TributeAttackSuccessKill(&'a str, &'a str),
    TributeAttackHidden(&'a str, &'a str),
    TributeDiesFromStatus(&'a str, &'a str),
    TributeDiesFromAreaEvent(&'a str, &'a str), // Died in area
    TributeDiesFromTributeEvent(&'a str, &'a str),
    TributeAlreadyDead(&'a str),
    TributeDead(&'a str),
    WeaponBreak(&'a str, &'a str),
    ShieldBreak(&'a str, &'a str),
    SponsorGift(&'a str, &'a Item),
    AreaEvent(&'a str, &'a str),
    AreaClose(&'a str),
    AreaOpen(&'a str),
    TrappedInArea(&'a str, &'a str),
    DiedInArea(&'a str, &'a str),
    TributeDeath(&'a str),
    TributeTravelNoOptions(&'a str, &'a str),
    TributeBetrayal(&'a str, &'a str),
    TributeForcedBetrayal(&'a str, &'a str),
    NoOneToAttack(&'a str),
    AllAlone(&'a str),
}

impl <'a> Display for GameOutput<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GameOutput::GameDayStart(day_number) => {
                write!(f, "=== ☀️ Day {} begins! ===", day_number)
            }
            GameOutput::GameDayEnd(day_number) => {
                write!(f, "=== ☀️ Day {} ends! ===", day_number)
            }
            GameOutput::FirstDayStart => {
                write!(f, "=== 🎉 The Hangry Games begin! 🎉 ===")
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
            GameOutput::GameNightEnd(day_number) => {
                write!(f, "=== 🌙 Night {} ends ===", day_number)
            }
            GameOutput::DailyDeathAnnouncement(death_count) => {
                write!(f, "=== 💀 Tributes dead: {} ===", death_count)
            }
            GameOutput::DeathAnnouncement(tribute) => {
                write!(f, "=== 🪦 {} has died ===", tribute)
            }
            GameOutput::NoOneWins => {
                write!(f, "=== 🎭 No one wins! ===")
            }
            GameOutput::TributeWins(tribute) => {
                write!(f, "=== 🏆 The winner is {} ===", tribute)
            }
            GameOutput::TributeRest(tribute) => {
                write!(f, "😪 {} rests", tribute)
            }
            GameOutput::TributeLongRest(tribute) => {
                write!(f, "💤 {} rests and recovers a little health and sanity", tribute)
            }
            GameOutput::TributeHide(tribute) => {
                write!(f, "🫥 {} tries to hide", tribute)
            }
            GameOutput::TributeTravel(tribute, area_a, area_b) => {
                write!(f, "🚶 {} moves from {} to {}", tribute, area_a, area_b)
            }
            GameOutput::TributeTakeItem(tribute, item) => {
                let object = indefinite(&item);
                write!(f, "🔨 {} takes {}", tribute, object)
            }
            GameOutput::TributeCannotUseItem(tribute, item) => {
                let object = indefinite(&item);
                write!(f, "❌ {} cannot use {}", tribute, object)
            }
            GameOutput::TributeUseItem(tribute, item) => {
                let object = indefinite(&item.name);
                write!(f, "💊 {} uses {}, gains {} {}", tribute, object, item.effect, item.attribute)
            }
            GameOutput::TributeTravelTooTired(tribute, area) => {
                write!(f, "😴 {} is too tired to move from {}, rests instead", tribute, area)
            }
            GameOutput::TributeTravelAlreadyThere(tribute, area) => {
                write!(f, "🤔 {} is already in the {}, stays put", tribute, area)
            }
            GameOutput::TributeTravelFollow(tribute, area) => {
                write!(f, "🫡 {} follows their district mate to {}", tribute, area)
            }
            GameOutput::TributeTravelStay(tribute, area) => {
                write!(f, "🪑 {} stays in {}", tribute, area)
            }
            GameOutput::TributeTravelNoOptions(tribute, area) => {
                write!(f, "📍 {} has nowhere to go, stays in {}", tribute, area)
            }
            GameOutput::TributeBleeds(tribute) => {
                write!(f, "🩸 {} bleeds from their wounds.", tribute)
            }
            GameOutput::TributeSick(tribute) => {
                write!(f, "🤒 {} contracts dysentery, loses strength and speed", tribute)
            }
            GameOutput::TributeElectrocuted(tribute) => {
                write!(f, "🌩️ {} is struck by lightning, loses health", tribute)
            }
            GameOutput::TributeFrozen(tribute) => {
                write!(f, "🥶 {} suffers from hypothermia, loses speed.", tribute)
            }
            GameOutput::TributeOverheated(tribute) => {
                write!(f, "🥵 {} suffers from heat stroke, loses speed.", tribute)
            }
            GameOutput::TributeDehydrated(tribute) => {
                write!(f, "🌵 {} is severely dehydrated, loses strength", tribute)
            }
            GameOutput::TributeStarving(tribute) => {
                write!(f, "🍴 {} is ravenously hungry, loses strength", tribute)
            }
            GameOutput::TributePoisoned(tribute) => {
                write!(f, "🧪 {} eats something poisonous, loses sanity", tribute)
            }
            GameOutput::TributeBrokenArm(tribute) => {
                write!(f, "🦴 {} injures their arm, loses strength.", tribute)
            }
            GameOutput::TributeBrokenLeg(tribute) => {
                write!(f, "🦴 {} injures their leg, loses speed.", tribute)
            }
            GameOutput::TributeInfected(tribute) => {
                write!(f, "🤢 {} gets an infection, loses health and sanity", tribute)
            }
            GameOutput::TributeDrowned(tribute) => {
                write!(f, "🏊 {} partially drowns, loses health and sanity", tribute)
            }
            GameOutput::TributeMauled(tribute, count, animal, damage) => {
                let animal = Animal::from_str(animal).unwrap();
                write!(f, "🐾 {} is attacked by {} {}, takes {} damage!", tribute, count, animal.plural(), damage)
            }
            GameOutput::TributeBurned(tribute) => {
                write!(f, "🔥 {} gets burned, loses health", tribute)
            }
            GameOutput::TributeHorrified(tribute, damage) => {
                write!(f, "😱 {} is horrified by the violence, loses {} sanity.", tribute, damage)
            }
            GameOutput::TributeSuffer(tribute) => {
                write!(f, "😭 {} suffers from loneliness and terror.", tribute)
            }
            GameOutput::TributeSelfHarm(tribute) => {
                write!(f, "🤦 {} tries to attack themself!", tribute)
            }
            GameOutput::TributeSuicide(tribute) => {
                write!(f, "🪒 {} attempts suicide.", tribute)
            }
            GameOutput::TributeAttackWin(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins!", tribute, target)
            }
            GameOutput::TributeAttackWinExtra(tribute, target) => {
                write!(f, "🔪 {} attacks {}, and wins decisively!", tribute, target)
            }
            GameOutput::TributeAttackWound(tribute, target) => {
                write!(f, "🤕 {} wounds {}", tribute, target)
            }
            GameOutput::TributeAttackLose(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses!", tribute, target)
            }
            GameOutput::TributeAttackLoseExtra(tribute, target) => {
                write!(f, "🤣 {} attacks {}, but loses decisively!", tribute, target)
            }
            GameOutput::TributeAttackMiss(tribute, target) => {
                write!(f, "😰 {} attacks {}, but misses!", tribute, target)
            }
            GameOutput::TributeAttackDied(tribute, target) => {
                write!(f, "☠️ {} is killed by {}", tribute, target)
            }
            GameOutput::TributeAttackSuccessKill(tribute, target) => {
                write!(f, "☠️ {} successfully kills {}", tribute, target)
            }
            GameOutput::TributeAttackHidden(tribute, target) => {
                write!(f, "🤔 {} can't attack {}, they're hidden", tribute, target)
            }
            GameOutput::TributeDiesFromStatus(tribute, status) => {
                write!(f, "💀 {} dies from {}", tribute, status)
            }
            GameOutput::TributeDiesFromAreaEvent(tribute, area_event) => {
                write!(f, "🪦 {} died in the {}.", tribute, area_event)
            }
            GameOutput::TributeDiesFromTributeEvent(tribute, tribute_event) => {
                write!(f, "💀 {} dies by {}", tribute, tribute_event)
            }
            GameOutput::TributeAlreadyDead(tribute) => {
                write!(f, "‼️ {} is already dead!", tribute)
            }
            GameOutput::TributeDead(tribute) => {
                write!(f, "❗️ {} is dead!", tribute)
            }
            GameOutput::WeaponBreak(tribute, weapon) => {
                write!(f, "🗡️ {} breaks their {}", tribute, weapon)
            }
            GameOutput::ShieldBreak(tribute, shield) => {
                write!(f, "🛡️ {} breaks their {}", tribute, shield)
            }
            GameOutput::SponsorGift(tribute, item) => {
                let object = indefinite(&item.name);
                write!(f, "🎁 {} receives {} ({}x {} +{})", tribute, object, item.quantity, item.attribute, item.effect)
            }
            GameOutput::AreaEvent(area_event, area) => {
                let area_name = area.replace("The ", "");
                let event = indefinite_capitalized(&area_event);
                write!(f, "=== ⚠️ {} has occurred in the {} ===", event, area_name)
            }
            GameOutput::AreaClose(area) => {
                let area_name = area.replace("The ", "");
                write!(f, "=== 🔔 The {} is uninhabitable ===", area_name)
            }
            GameOutput::AreaOpen(area) => {
                let area_name = area.replace("The ", "");
                write!(f, "=== 🔔 The {} is habitable again ===", area_name)
            }
            GameOutput::TrappedInArea(tribute, area) => {
                let area_name = area.replace("The ", "");
                write!(f, "💥 {} is trapped in the {}.", tribute, area_name)
            }
            GameOutput::DiedInArea(tribute, area) => {
                let area_name = area.replace("The ", "");
                write!(f, "💥 {} died in the {}.", tribute, area_name)
            }
            GameOutput::TributeDeath(tribute) => {
                write!(f, "⚰️ {} has died.", tribute)
            }
            GameOutput::TributeBetrayal(tribute, target) => {
                write!(f, "💔 {} betrays {}!", tribute, target)
            }
            GameOutput::TributeForcedBetrayal(tribute, target) => {
                write!(f, "💔💔 {} is forced to betray {}!", tribute, target)
            }
            GameOutput::NoOneToAttack(tribute) => {
                write!(f, "🤷 {} has no one to attack!", tribute)
            }
            GameOutput::AllAlone(tribute) => {
                write!(f, "😢 {} is all alone!", tribute)
            }
        }
    }
}
