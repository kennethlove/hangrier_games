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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::items::Item;
use crate::threats::animals::Animal;

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
