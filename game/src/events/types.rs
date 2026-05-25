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
