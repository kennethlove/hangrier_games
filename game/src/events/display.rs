use super::GameEvent;
use std::fmt::{Display, Formatter};

use indefinite::{indefinite, indefinite_capitalized};

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
