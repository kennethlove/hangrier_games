//! Temporary placeholder for the legacy day-log UI.
//!
//! The previous `GameDayLog` and `GameDaySummary` components rendered events
//! built from the now-removed `MessageKind::AllianceFormed` /
//! `MessageKind::BetrayalTriggered` / `MessageKind::TrustShockBreak` variants
//! and the deleted `structured_event` helper. They have been deleted as part
//! of the typed `MessagePayload` migration (PR1).
//!
//! The replacement timeline UI is delivered in PR2 (beads
//! `hangrier_games-h8z`), which consumes the new
//! `GET /api/games/:game_identifier/timeline-summary` endpoint introduced in
//! PR1. Until then, this stub keeps the web crate compiling and gives players
//! a clear "coming soon" affordance instead of stale, incorrect output.

use dioxus::prelude::*;
use shared::DisplayGame;

#[component]
pub fn GameLogStub(game: DisplayGame, day: u32) -> Element {
    let _ = game; // silence unused-field warning until PR2 wires it up
    rsx! {
        p {
            class: r#"
            text-center
            italic

            theme1:text-stone-300
            theme2:text-green-300
            theme3:text-slate-600
            "#,
            "Day {day} log: timeline UI coming soon (PR2)."
        }
    }
}
