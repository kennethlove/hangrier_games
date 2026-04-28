//! PeriodCard — entry tile for a single Day/Night period in the timeline hub.
//!
//! Renders death/event counts and links into the per-period view. The current
//! (in-progress) period is highlighted with a ring + "live" badge.
//!
//! NOTE: The destination route `Routes::GamePeriodPage` is added in T8.
//! Until then this links to `Routes::Home {}` as a placeholder so the build
//! stays green commit-by-commit. T8 swaps it in.

use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::Phase;

#[derive(Props, PartialEq, Clone)]
pub struct PeriodCardProps {
    pub game_identifier: String,
    pub day: u32,
    pub phase: Phase,
    pub deaths: u32,
    pub event_count: u32,
    pub is_current: bool,
}

#[component]
pub fn PeriodCard(props: PeriodCardProps) -> Element {
    let _ = props.game_identifier; // wired to GamePeriodPage in T8
    let phase_label = match props.phase {
        Phase::Day => "Day",
        Phase::Night => "Night",
    };

    let current_class = if props.is_current {
        "ring-2 ring-amber-400 theme2:ring-green-400 theme3:ring-purple-400"
    } else {
        ""
    };

    rsx! {
        Link {
            to: Routes::Home {}, // TODO(T8): swap to Routes::GamePeriodPage { ... }
            class: "block rounded-lg border p-4 hover:shadow-lg transition theme1:bg-amber-50 theme1:border-amber-200 theme2:bg-slate-800 theme2:border-green-700 theme3:bg-purple-900 theme3:border-purple-600 {current_class}",
            div { class: "flex items-center justify-between mb-2",
                h3 { class: "font-semibold", "Day {props.day} — {phase_label}" }
                if props.is_current {
                    span { class: "text-xs uppercase tracking-wide text-amber-600 theme2:text-green-400 theme3:text-purple-300",
                        "live"
                    }
                }
            }
            div { class: "flex items-center gap-4 text-sm",
                span { "💀 {props.deaths} deaths" }
                span { "📜 {props.event_count} events" }
            }
        }
    }
}
