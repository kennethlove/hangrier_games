//! Structured `GameEvent` enum — the typed counterpart to [`crate::output::GameOutput`].
//!
//! `GameOutput` is a borrowed, stringly-typed enum used purely to render
//! player-facing log lines. `GameEvent` is owned, serde-friendly, and carries
//! the original typed fields (UUIDs, numbers, names, items) so downstream
//! consumers (DB, websockets, analytics, announcers) can react to *what
//! happened* rather than re-parsing a localized sentence.
//!
//! This module is introduced in mqi.1. No emission sites have switched yet —
//! the engine still emits `GameOutput`. Parity tests below guarantee that
//! every `GameEvent` variant renders to a byte-identical string when fed the
//! same data as the matching `GameOutput` variant, so future emission-site
//! migration (mqi.2) and persistence (mqi.3) can proceed in lockstep without
//! changing player-visible log output.
//!
//! Design decisions documented in
//! `docs/superpowers/specs/2026-04-26-game-event-enum.md`.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::items::Item;
use crate::threats::animals::Animal;
use indefinite::{indefinite, indefinite_capitalized};

/// Structured, owned, serde-friendly counterpart to [`crate::output::GameOutput`].
///
/// Every variant of `GameOutput` has a matching variant here. Fields use
/// owned types so the event can outlive the borrowed sources that produced
/// it, and named struct variants are used throughout so future fields can be
/// added without breaking call sites.
///
/// Where `GameOutput` only carries names (e.g. tribute display names),
/// `GameEvent` carries both a UUID (`*_id`) for stable cross-system reference
/// **and** the rendered name (`*_name`) so [`Display`] can reproduce the
/// exact log line without a name-lookup round-trip.
///
/// Serialization uses serde's default externally-tagged representation, which
/// gives unambiguous JSON shapes for every variant and round-trips cleanly
/// without bespoke deserialization logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    // ---- Day / night cycle markers ----
    GameDayStart {
        day_number: u32,
    },
    GameDayEnd {
        day_number: u32,
    },
    FirstDayStart,
    FeastDayStart,
    TributesLeft {
        tribute_count: u32,
    },
    GameNightStart {
        day_number: u32,
    },
    GameNightEnd {
        day_number: u32,
    },
    DailyDeathAnnouncement {
        death_count: u32,
    },
    DeathAnnouncement {
        tribute_id: Uuid,
        tribute_name: String,
    },
    NoOneWins,
    TributeWins {
        tribute_id: Uuid,
        tribute_name: String,
    },

    // ---- Rest / hide / movement ----
    TributeRest {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeLongRest {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeHide {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeTravel {
        tribute_id: Uuid,
        tribute_name: String,
        from_area: String,
        to_area: String,
    },
    TributeTakeItem {
        tribute_id: Uuid,
        tribute_name: String,
        item_name: String,
    },
    TributeCannotUseItem {
        tribute_id: Uuid,
        tribute_name: String,
        item_name: String,
    },
    TributeUseItem {
        tribute_id: Uuid,
        tribute_name: String,
        item: Item,
    },
    TributeTravelTooTired {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },
    TributeTravelExhausted {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },
    TributeTravelAlreadyThere {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },
    TributeTravelFollow {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },
    TributeTravelStay {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },
    TributeTravelNoOptions {
        tribute_id: Uuid,
        tribute_name: String,
        area: String,
    },

    // ---- Status effects (single-tribute) ----
    TributeBleeds {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeSick {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeElectrocuted {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeFrozen {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeOverheated {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeDehydrated {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeStarving {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributePoisoned {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeBrokenArm {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeBrokenLeg {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeInfected {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeDrowned {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeMauled {
        tribute_id: Uuid,
        tribute_name: String,
        animal_count: u32,
        animal: Animal,
        damage: u32,
    },
    TributeBurned {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeHorrified {
        tribute_id: Uuid,
        tribute_name: String,
        sanity_damage: u32,
    },
    TributeSuffer {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeSelfHarm {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeSuicide {
        tribute_id: Uuid,
        tribute_name: String,
    },

    // ---- Combat ----
    TributeAttackWin {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackWinExtra {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackWound {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackLose {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackLoseExtra {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackMiss {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackDied {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackSuccessKill {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeAttackHidden {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    /// Natural 20 on attack roll.
    TributeCriticalHit {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    /// Natural 1 on attack roll.
    TributeCriticalFumble {
        tribute_id: Uuid,
        tribute_name: String,
    },
    /// Natural 20 on defense roll.
    TributePerfectBlock {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },

    // ---- Death ----
    TributeDiesFromStatus {
        tribute_id: Uuid,
        tribute_name: String,
        status: String,
    },
    TributeDiesFromAreaEvent {
        tribute_id: Uuid,
        tribute_name: String,
        area_event: String,
    },
    TributeDiesFromTributeEvent {
        tribute_id: Uuid,
        tribute_name: String,
        tribute_event: String,
    },
    TributeAlreadyDead {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeDead {
        tribute_id: Uuid,
        tribute_name: String,
    },
    TributeDeath {
        tribute_id: Uuid,
        tribute_name: String,
    },

    // ---- Items / equipment ----
    WeaponBreak {
        tribute_id: Uuid,
        tribute_name: String,
        weapon_name: String,
    },
    WeaponWear {
        tribute_id: Uuid,
        tribute_name: String,
        weapon_name: String,
    },
    ShieldBreak {
        tribute_id: Uuid,
        tribute_name: String,
        shield_name: String,
    },
    ShieldWear {
        tribute_id: Uuid,
        tribute_name: String,
        shield_name: String,
    },
    SponsorGift {
        tribute_id: Uuid,
        tribute_name: String,
        item: Item,
    },

    // ---- Area events ----
    AreaEvent {
        area_event: String,
        area_name: String,
    },
    AreaClose {
        area_name: String,
    },
    AreaOpen {
        area_name: String,
    },
    TrappedInArea {
        tribute_id: Uuid,
        tribute_name: String,
        area_name: String,
    },
    DiedInArea {
        tribute_id: Uuid,
        tribute_name: String,
        area_name: String,
    },

    // ---- Social / alliance ----
    TributeBetrayal {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    TributeForcedBetrayal {
        tribute_id: Uuid,
        tribute_name: String,
        target_id: Uuid,
        target_name: String,
    },
    NoOneToAttack {
        tribute_id: Uuid,
        tribute_name: String,
    },
    AllAlone {
        tribute_id: Uuid,
        tribute_name: String,
    },
    AllianceFormed {
        tribute_a_id: Uuid,
        tribute_a_name: String,
        tribute_b_id: Uuid,
        tribute_b_name: String,
        factor: String,
    },
    BetrayalTriggered {
        betrayer_id: Uuid,
        betrayer_name: String,
        victim_id: Uuid,
        victim_name: String,
    },
    TrustShockBreak {
        tribute_id: Uuid,
        tribute_name: String,
    },
    /// One combat swing carrying the full typed beat.
    CombatSwing {
        beat: crate::tributes::combat_beat::CombatBeat,
    },
}

impl Display for GameEvent {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameEvent::GameDayStart { day_number } => {
                write!(f, "=== ☀️ Day {} begins! ===", day_number)
            }
            GameEvent::GameDayEnd { day_number } => {
                write!(f, "=== ☀️ Day {} ends! ===", day_number)
            }
            GameEvent::FirstDayStart => {
                write!(f, "=== 🎉 The Hangry Games begin! 🎉 ===")
            }
            GameEvent::FeastDayStart => {
                write!(f, "=== 😋 Day 3: Feast Day ===")
            }
            GameEvent::TributesLeft { tribute_count } => {
                write!(f, "=== 📌 Tributes alive: {} ===", tribute_count)
            }
            GameEvent::GameNightStart { day_number } => {
                write!(f, "=== 🌙 Night {} begins ===", day_number)
            }
            GameEvent::GameNightEnd { day_number } => {
                write!(f, "=== 🌙 Night {} ends ===", day_number)
            }
            GameEvent::DailyDeathAnnouncement { death_count } => {
                write!(f, "=== 💀 Tributes dead: {} ===", death_count)
            }
            GameEvent::DeathAnnouncement { tribute_name, .. } => {
                write!(f, "=== 🪦 {} has died ===", tribute_name)
            }
            GameEvent::NoOneWins => {
                write!(f, "=== 🎭 No one wins! ===")
            }
            GameEvent::TributeWins { tribute_name, .. } => {
                write!(f, "=== 🏆 The winner is {} ===", tribute_name)
            }
            GameEvent::TributeRest { tribute_name, .. } => {
                write!(f, "😪 {} rests", tribute_name)
            }
            GameEvent::TributeLongRest { tribute_name, .. } => {
                write!(
                    f,
                    "💤 {} rests and recovers a little health and sanity",
                    tribute_name
                )
            }
            GameEvent::TributeHide { tribute_name, .. } => {
                write!(f, "🫥 {} tries to hide", tribute_name)
            }
            GameEvent::TributeTravel {
                tribute_name,
                from_area,
                to_area,
                ..
            } => {
                write!(
                    f,
                    "🚶 {} moves from {} to {}",
                    tribute_name, from_area, to_area
                )
            }
            GameEvent::TributeTakeItem {
                tribute_name,
                item_name,
                ..
            } => {
                let object = indefinite(item_name);
                write!(f, "🔨 {} takes {}", tribute_name, object)
            }
            GameEvent::TributeCannotUseItem {
                tribute_name,
                item_name,
                ..
            } => {
                let object = indefinite(item_name);
                write!(f, "❌ {} cannot use {}", tribute_name, object)
            }
            GameEvent::TributeUseItem {
                tribute_name, item, ..
            } => {
                let object = indefinite(&item.name);
                write!(
                    f,
                    "💊 {} uses {}, gains {} {}",
                    tribute_name, object, item.effect, item.attribute
                )
            }
            GameEvent::TributeTravelTooTired {
                tribute_name, area, ..
            } => {
                write!(
                    f,
                    "😴 {} is too tired to move from {}, rests instead",
                    tribute_name, area
                )
            }
            GameEvent::TributeTravelExhausted {
                tribute_name, area, ..
            } => {
                write!(
                    f,
                    "🥵 {} is too exhausted to move from {}, lacks stamina",
                    tribute_name, area
                )
            }
            GameEvent::TributeTravelAlreadyThere {
                tribute_name, area, ..
            } => {
                write!(
                    f,
                    "🤔 {} is already in the {}, stays put",
                    tribute_name, area
                )
            }
            GameEvent::TributeTravelFollow {
                tribute_name, area, ..
            } => {
                write!(
                    f,
                    "🫡 {} follows their district mate to {}",
                    tribute_name, area
                )
            }
            GameEvent::TributeTravelStay {
                tribute_name, area, ..
            } => {
                write!(f, "🪑 {} stays in {}", tribute_name, area)
            }
            GameEvent::TributeTravelNoOptions {
                tribute_name, area, ..
            } => {
                write!(
                    f,
                    "📍 {} has nowhere to go, stays in {}",
                    tribute_name, area
                )
            }
            GameEvent::TributeBleeds { tribute_name, .. } => {
                write!(f, "🩸 {} bleeds from their wounds.", tribute_name)
            }
            GameEvent::TributeSick { tribute_name, .. } => {
                write!(
                    f,
                    "🤒 {} contracts dysentery, loses strength and speed",
                    tribute_name
                )
            }
            GameEvent::TributeElectrocuted { tribute_name, .. } => {
                write!(
                    f,
                    "🌩️ {} is struck by lightning, loses health",
                    tribute_name
                )
            }
            GameEvent::TributeFrozen { tribute_name, .. } => {
                write!(
                    f,
                    "🥶 {} suffers from hypothermia, loses speed.",
                    tribute_name
                )
            }
            GameEvent::TributeOverheated { tribute_name, .. } => {
                write!(
                    f,
                    "🥵 {} suffers from heat stroke, loses speed.",
                    tribute_name
                )
            }
            GameEvent::TributeDehydrated { tribute_name, .. } => {
                write!(
                    f,
                    "🌵 {} is severely dehydrated, loses strength",
                    tribute_name
                )
            }
            GameEvent::TributeStarving { tribute_name, .. } => {
                write!(
                    f,
                    "🍴 {} is ravenously hungry, loses strength",
                    tribute_name
                )
            }
            GameEvent::TributePoisoned { tribute_name, .. } => {
                write!(
                    f,
                    "🧪 {} eats something poisonous, loses sanity",
                    tribute_name
                )
            }
            GameEvent::TributeBrokenArm { tribute_name, .. } => {
                write!(f, "🦴 {} injures their arm, loses strength.", tribute_name)
            }
            GameEvent::TributeBrokenLeg { tribute_name, .. } => {
                write!(f, "🦴 {} injures their leg, loses speed.", tribute_name)
            }
            GameEvent::TributeInfected { tribute_name, .. } => {
                write!(
                    f,
                    "🤢 {} gets an infection, loses health and sanity",
                    tribute_name
                )
            }
            GameEvent::TributeDrowned { tribute_name, .. } => {
                write!(
                    f,
                    "🏊 {} partially drowns, loses health and sanity",
                    tribute_name
                )
            }
            GameEvent::TributeMauled {
                tribute_name,
                animal_count,
                animal,
                damage,
                ..
            } => {
                write!(
                    f,
                    "🐾 {} is attacked by {} {}, takes {} damage!",
                    tribute_name,
                    animal_count,
                    animal.plural(),
                    damage
                )
            }
            GameEvent::TributeBurned { tribute_name, .. } => {
                write!(f, "🔥 {} gets burned, loses health", tribute_name)
            }
            GameEvent::TributeHorrified {
                tribute_name,
                sanity_damage,
                ..
            } => {
                write!(
                    f,
                    "😱 {} is horrified by the violence, loses {} sanity.",
                    tribute_name, sanity_damage
                )
            }
            GameEvent::TributeSuffer { tribute_name, .. } => {
                write!(f, "😭 {} suffers from loneliness and terror.", tribute_name)
            }
            GameEvent::TributeSelfHarm { tribute_name, .. } => {
                write!(f, "🤦 {} tries to attack themself!", tribute_name)
            }
            GameEvent::TributeSuicide { tribute_name, .. } => {
                write!(f, "🪒 {} attempts suicide.", tribute_name)
            }
            GameEvent::TributeAttackWin {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "🔪 {} attacks {}, and wins!", tribute_name, target_name)
            }
            GameEvent::TributeAttackWinExtra {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "🔪 {} attacks {}, and wins decisively!",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeAttackWound {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "🤕 {} wounds {}", tribute_name, target_name)
            }
            GameEvent::TributeAttackLose {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "🤣 {} attacks {}, but loses!", tribute_name, target_name)
            }
            GameEvent::TributeAttackLoseExtra {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "🤣 {} attacks {}, but loses decisively!",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeAttackMiss {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "😰 {} attacks {}, but misses!",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeAttackDied {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "☠️ {} is killed by {}", tribute_name, target_name)
            }
            GameEvent::TributeAttackSuccessKill {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "☠️ {} successfully kills {}", tribute_name, target_name)
            }
            GameEvent::TributeAttackHidden {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "🤔 {} can't attack {}, they're hidden",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeCriticalHit {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "💥 {} lands a CRITICAL HIT on {}!",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeCriticalFumble { tribute_name, .. } => {
                write!(
                    f,
                    "😵 {} fumbles their attack badly and hurts themself!",
                    tribute_name
                )
            }
            GameEvent::TributePerfectBlock {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "🛡️ {} perfectly blocks {}'s attack and counters!",
                    tribute_name, target_name
                )
            }
            GameEvent::TributeDiesFromStatus {
                tribute_name,
                status,
                ..
            } => {
                write!(f, "💀 {} dies from {}", tribute_name, status)
            }
            GameEvent::TributeDiesFromAreaEvent {
                tribute_name,
                area_event,
                ..
            } => {
                write!(f, "🪦 {} died in the {}.", tribute_name, area_event)
            }
            GameEvent::TributeDiesFromTributeEvent {
                tribute_name,
                tribute_event,
                ..
            } => {
                write!(f, "💀 {} dies by {}", tribute_name, tribute_event)
            }
            GameEvent::TributeAlreadyDead { tribute_name, .. } => {
                write!(f, "‼️ {} is already dead!", tribute_name)
            }
            GameEvent::TributeDead { tribute_name, .. } => {
                write!(f, "❗️ {} is dead!", tribute_name)
            }
            GameEvent::WeaponBreak {
                tribute_name,
                weapon_name,
                ..
            } => {
                write!(f, "🗡️ {} breaks their {}", tribute_name, weapon_name)
            }
            GameEvent::WeaponWear {
                tribute_name,
                weapon_name,
                ..
            } => {
                write!(
                    f,
                    "🗡️ {}'s {} is showing signs of wear",
                    tribute_name, weapon_name
                )
            }
            GameEvent::ShieldBreak {
                tribute_name,
                shield_name,
                ..
            } => {
                write!(f, "🛡️ {} breaks their {}", tribute_name, shield_name)
            }
            GameEvent::ShieldWear {
                tribute_name,
                shield_name,
                ..
            } => {
                write!(
                    f,
                    "🛡️ {}'s {} is showing signs of wear",
                    tribute_name, shield_name
                )
            }
            GameEvent::SponsorGift {
                tribute_name, item, ..
            } => {
                let object = indefinite(&item.name);
                write!(
                    f,
                    "🎁 {} receives {} (durability {}/{} {} +{})",
                    tribute_name,
                    object,
                    item.current_durability,
                    item.max_durability,
                    item.attribute,
                    item.effect
                )
            }
            GameEvent::AreaEvent {
                area_event,
                area_name,
            } => {
                let area_short = area_name.replace("The ", "");
                let event = indefinite_capitalized(area_event);
                write!(f, "=== ⚠️ {} has occurred in the {} ===", event, area_short)
            }
            GameEvent::AreaClose { area_name } => {
                let area_short = area_name.replace("The ", "");
                write!(f, "=== 🔔 The {} is uninhabitable ===", area_short)
            }
            GameEvent::AreaOpen { area_name } => {
                let area_short = area_name.replace("The ", "");
                write!(f, "=== 🔔 The {} is habitable again ===", area_short)
            }
            GameEvent::TrappedInArea {
                tribute_name,
                area_name,
                ..
            } => {
                let area_short = area_name.replace("The ", "");
                write!(f, "💥 {} is trapped in the {}.", tribute_name, area_short)
            }
            GameEvent::DiedInArea {
                tribute_name,
                area_name,
                ..
            } => {
                let area_short = area_name.replace("The ", "");
                write!(f, "💥 {} died in the {}.", tribute_name, area_short)
            }
            GameEvent::TributeDeath { tribute_name, .. } => {
                write!(f, "⚰️ {} has died.", tribute_name)
            }
            GameEvent::TributeBetrayal {
                tribute_name,
                target_name,
                ..
            } => {
                write!(f, "💔 {} betrays {}!", tribute_name, target_name)
            }
            GameEvent::TributeForcedBetrayal {
                tribute_name,
                target_name,
                ..
            } => {
                write!(
                    f,
                    "💔💔 {} is forced to betray {}!",
                    tribute_name, target_name
                )
            }
            GameEvent::NoOneToAttack { tribute_name, .. } => {
                write!(f, "🤷 {} has no one to attack!", tribute_name)
            }
            GameEvent::AllAlone { tribute_name, .. } => {
                write!(f, "😢 {} is all alone!", tribute_name)
            }
            GameEvent::AllianceFormed {
                tribute_a_name,
                tribute_b_name,
                factor,
                ..
            } => {
                write!(
                    f,
                    "{} and {} form an alliance ({}).",
                    tribute_a_name, tribute_b_name, factor
                )
            }
            GameEvent::BetrayalTriggered {
                betrayer_name,
                victim_name,
                ..
            } => {
                write!(
                    f,
                    "{} betrays {} — true to their treacherous nature.",
                    betrayer_name, victim_name
                )
            }
            GameEvent::TrustShockBreak { tribute_name, .. } => {
                write!(
                    f,
                    "{} is shaken by their ally's death and breaks the bond.",
                    tribute_name
                )
            }
            GameEvent::CombatSwing { beat } => {
                use crate::tributes::combat_beat::CombatBeatExt;
                let lines = beat.to_log_lines();
                write!(f, "{}", lines.join(" "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Attribute, Item, ItemRarity, ItemType};
    use crate::output::GameOutput;

    /// Stable UUIDs so test failures are easy to reason about.
    fn uid_a() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }
    fn uid_b() -> Uuid {
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
    }

    fn sample_item() -> Item {
        Item {
            identifier: "33333333-3333-3333-3333-333333333333".to_string(),
            name: "elixir".to_string(),
            item_type: ItemType::Consumable,
            rarity: ItemRarity::Common,
            current_durability: 3,
            max_durability: 5,
            attribute: Attribute::Health,
            effect: 7,
        }
    }

    /// Single source of truth for the parity table. Each row pairs a
    /// constructed `GameEvent` with a `GameOutput` carrying the same data;
    /// the rendered strings must be byte-identical.
    fn parity_table() -> Vec<(GameEvent, GameOutput<'static>)> {
        let item = sample_item();
        // SAFETY: `Item` is owned, but `GameOutput` borrows. We leak the
        // sample item once for the test table so its references are 'static.
        // This is test-only code; the leak is bounded and intentional.
        let item_ref: &'static Item = Box::leak(Box::new(item.clone()));

        vec![
            (
                GameEvent::GameDayStart { day_number: 4 },
                GameOutput::GameDayStart(4),
            ),
            (
                GameEvent::GameDayEnd { day_number: 4 },
                GameOutput::GameDayEnd(4),
            ),
            (GameEvent::FirstDayStart, GameOutput::FirstDayStart),
            (GameEvent::FeastDayStart, GameOutput::FeastDayStart),
            (
                GameEvent::TributesLeft { tribute_count: 12 },
                GameOutput::TributesLeft(12),
            ),
            (
                GameEvent::GameNightStart { day_number: 2 },
                GameOutput::GameNightStart(2),
            ),
            (
                GameEvent::GameNightEnd { day_number: 2 },
                GameOutput::GameNightEnd(2),
            ),
            (
                GameEvent::DailyDeathAnnouncement { death_count: 3 },
                GameOutput::DailyDeathAnnouncement(3),
            ),
            (
                GameEvent::DeathAnnouncement {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::DeathAnnouncement("Alice"),
            ),
            (GameEvent::NoOneWins, GameOutput::NoOneWins),
            (
                GameEvent::TributeWins {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeWins("Alice"),
            ),
            (
                GameEvent::TributeRest {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeRest("Alice"),
            ),
            (
                GameEvent::TributeLongRest {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeLongRest("Alice"),
            ),
            (
                GameEvent::TributeHide {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeHide("Alice"),
            ),
            (
                GameEvent::TributeTravel {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    from_area: "Cornucopia".into(),
                    to_area: "North".into(),
                },
                GameOutput::TributeTravel("Alice", "Cornucopia", "North"),
            ),
            (
                GameEvent::TributeTakeItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item_name: "elixir".into(),
                },
                GameOutput::TributeTakeItem("Alice", "elixir"),
            ),
            (
                GameEvent::TributeCannotUseItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item_name: "elixir".into(),
                },
                GameOutput::TributeCannotUseItem("Alice", "elixir"),
            ),
            (
                GameEvent::TributeUseItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item: item.clone(),
                },
                GameOutput::TributeUseItem("Alice", item_ref),
            ),
            (
                GameEvent::TributeTravelTooTired {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelTooTired("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelExhausted {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelExhausted("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelAlreadyThere {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelAlreadyThere("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelFollow {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelFollow("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelStay {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelStay("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelNoOptions {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelNoOptions("Alice", "Forest"),
            ),
            (
                GameEvent::TributeBleeds {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBleeds("Alice"),
            ),
            (
                GameEvent::TributeSick {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSick("Alice"),
            ),
            (
                GameEvent::TributeElectrocuted {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeElectrocuted("Alice"),
            ),
            (
                GameEvent::TributeFrozen {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeFrozen("Alice"),
            ),
            (
                GameEvent::TributeOverheated {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeOverheated("Alice"),
            ),
            (
                GameEvent::TributeDehydrated {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDehydrated("Alice"),
            ),
            (
                GameEvent::TributeStarving {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeStarving("Alice"),
            ),
            (
                GameEvent::TributePoisoned {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributePoisoned("Alice"),
            ),
            (
                GameEvent::TributeBrokenArm {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBrokenArm("Alice"),
            ),
            (
                GameEvent::TributeBrokenLeg {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBrokenLeg("Alice"),
            ),
            (
                GameEvent::TributeInfected {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeInfected("Alice"),
            ),
            (
                GameEvent::TributeDrowned {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDrowned("Alice"),
            ),
            (
                GameEvent::TributeMauled {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    animal_count: 3,
                    animal: Animal::Wolf,
                    damage: 12,
                },
                GameOutput::TributeMauled("Alice", 3, "Wolf", 12),
            ),
            (
                GameEvent::TributeBurned {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBurned("Alice"),
            ),
            (
                GameEvent::TributeHorrified {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    sanity_damage: 5,
                },
                GameOutput::TributeHorrified("Alice", 5),
            ),
            (
                GameEvent::TributeSuffer {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSuffer("Alice"),
            ),
            (
                GameEvent::TributeSelfHarm {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSelfHarm("Alice"),
            ),
            (
                GameEvent::TributeSuicide {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSuicide("Alice"),
            ),
            (
                GameEvent::TributeAttackWin {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWin("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackWinExtra {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWinExtra("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackWound {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWound("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackLose {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackLose("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackLoseExtra {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackLoseExtra("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackMiss {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackMiss("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackDied {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackDied("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackSuccessKill {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackSuccessKill("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackHidden {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackHidden("Alice", "Bob"),
            ),
            (
                GameEvent::TributeCriticalHit {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeCriticalHit("Alice", "Bob"),
            ),
            (
                GameEvent::TributeCriticalFumble {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeCriticalFumble("Alice"),
            ),
            (
                GameEvent::TributePerfectBlock {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributePerfectBlock("Alice", "Bob"),
            ),
            (
                GameEvent::TributeDiesFromStatus {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    status: "poison".into(),
                },
                GameOutput::TributeDiesFromStatus("Alice", "poison"),
            ),
            (
                GameEvent::TributeDiesFromAreaEvent {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_event: "wildfire".into(),
                },
                GameOutput::TributeDiesFromAreaEvent("Alice", "wildfire"),
            ),
            (
                GameEvent::TributeDiesFromTributeEvent {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    tribute_event: "Bob".into(),
                },
                GameOutput::TributeDiesFromTributeEvent("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAlreadyDead {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeAlreadyDead("Alice"),
            ),
            (
                GameEvent::TributeDead {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDead("Alice"),
            ),
            (
                GameEvent::TributeDeath {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDeath("Alice"),
            ),
            (
                GameEvent::WeaponBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    weapon_name: "spear".into(),
                },
                GameOutput::WeaponBreak("Alice", "spear"),
            ),
            (
                GameEvent::WeaponWear {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    weapon_name: "spear".into(),
                },
                GameOutput::WeaponWear("Alice", "spear"),
            ),
            (
                GameEvent::ShieldBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    shield_name: "buckler".into(),
                },
                GameOutput::ShieldBreak("Alice", "buckler"),
            ),
            (
                GameEvent::ShieldWear {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    shield_name: "buckler".into(),
                },
                GameOutput::ShieldWear("Alice", "buckler"),
            ),
            (
                GameEvent::SponsorGift {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item: item.clone(),
                },
                GameOutput::SponsorGift("Alice", item_ref),
            ),
            (
                GameEvent::AreaEvent {
                    area_event: "earthquake".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaEvent("earthquake", "The Forest"),
            ),
            (
                GameEvent::AreaClose {
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaClose("The Forest"),
            ),
            (
                GameEvent::AreaOpen {
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaOpen("The Forest"),
            ),
            (
                GameEvent::TrappedInArea {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::TrappedInArea("Alice", "The Forest"),
            ),
            (
                GameEvent::DiedInArea {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::DiedInArea("Alice", "The Forest"),
            ),
            (
                GameEvent::TributeBetrayal {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeBetrayal("Alice", "Bob"),
            ),
            (
                GameEvent::TributeForcedBetrayal {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeForcedBetrayal("Alice", "Bob"),
            ),
            (
                GameEvent::NoOneToAttack {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::NoOneToAttack("Alice"),
            ),
            (
                GameEvent::AllAlone {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::AllAlone("Alice"),
            ),
            (
                GameEvent::AllianceFormed {
                    tribute_a_id: uid_a(),
                    tribute_a_name: "Alice".into(),
                    tribute_b_id: uid_b(),
                    tribute_b_name: "Bob".into(),
                    factor: "trust".into(),
                },
                GameOutput::AllianceFormed("Alice", "Bob", "trust"),
            ),
            (
                GameEvent::BetrayalTriggered {
                    betrayer_id: uid_a(),
                    betrayer_name: "Cato".into(),
                    victim_id: uid_b(),
                    victim_name: "Glimmer".into(),
                },
                GameOutput::BetrayalTriggered("Cato", "Glimmer"),
            ),
            (
                GameEvent::TrustShockBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Rue".into(),
                },
                GameOutput::TrustShockBreak("Rue"),
            ),
        ]
    }

    #[test]
    fn parity_table_covers_every_variant() {
        // Bumps any time a variant is added without a parity row.
        // 77 = current count of GameOutput variants in output.rs.
        assert_eq!(parity_table().len(), 77);
    }

    #[test]
    fn display_matches_game_output_for_every_variant() {
        for (event, output) in parity_table() {
            assert_eq!(
                event.to_string(),
                output.to_string(),
                "Display mismatch for {:?}",
                event
            );
        }
    }

    // ---------- Serde roundtrip coverage ----------
    // One assertion per data shape: unit, single-field, multi-field,
    // optional-field-via-Item.

    fn roundtrip(event: &GameEvent) {
        let json = serde_json::to_string(event).expect("serialize");
        let parsed: GameEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*event, parsed, "roundtrip mismatch: {}", json);
    }

    #[test]
    fn serde_roundtrip_unit_variant() {
        roundtrip(&GameEvent::FirstDayStart);
        roundtrip(&GameEvent::FeastDayStart);
        roundtrip(&GameEvent::NoOneWins);
    }

    #[test]
    fn serde_roundtrip_single_primitive_field() {
        roundtrip(&GameEvent::GameDayStart { day_number: 7 });
        roundtrip(&GameEvent::TributesLeft { tribute_count: 11 });
    }

    #[test]
    fn serde_roundtrip_multi_field_with_uuid() {
        roundtrip(&GameEvent::AllianceFormed {
            tribute_a_id: uid_a(),
            tribute_a_name: "Alice".into(),
            tribute_b_id: uid_b(),
            tribute_b_name: "Bob".into(),
            factor: "shared district".into(),
        });
    }

    #[test]
    fn serde_roundtrip_with_nested_item() {
        roundtrip(&GameEvent::SponsorGift {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            item: sample_item(),
        });
        roundtrip(&GameEvent::TributeUseItem {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            item: sample_item(),
        });
    }

    #[test]
    fn serde_roundtrip_with_animal_enum() {
        roundtrip(&GameEvent::TributeMauled {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            animal_count: 4,
            animal: Animal::TrackerJacker,
            damage: 9,
        });
    }
}
