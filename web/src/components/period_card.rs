//! PeriodCard — entry tile for a single Day/Night period in the timeline hub.
//!
//! Renders death/event counts and links into the per-period view. The current
//! (in-progress) period is highlighted with a ring + "live" badge.

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

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Dawn => "Dawn",
        Phase::Day => "Day",
        Phase::Dusk => "Dusk",
        Phase::Night => "Night",
    }
}

fn phase_visual(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::Dawn => ("🌄", "border-l-4 border-l-rose-400"),
        Phase::Day => ("☀️", "border-l-4 border-l-amber-500"),
        Phase::Dusk => ("🌆", "border-l-4 border-l-violet-500"),
        Phase::Night => ("🌙", "border-l-4 border-l-indigo-500"),
    }
}

#[component]
pub fn PeriodCard(props: PeriodCardProps) -> Element {
    let label = phase_label(props.phase);
    let (icon, accent) = phase_visual(props.phase);

    let current_class = if props.is_current {
        "ring-2 ring-amber-400  "
    } else {
        ""
    };

    let route = Routes::GamePeriodPage {
        identifier: props.game_identifier.clone(),
        day: props.day,
        phase: props.phase,
        filter: String::new(),
        tribute: String::new(),
    };

    rsx! {
        Link {
            to: route,
            class: "block rounded-lg border p-4 hover:shadow-lg transition {accent} {current_class}",
            div { class: "flex items-center justify-between mb-2",
                h3 { class: "font-semibold", "{icon} Day {props.day} — {label}" }
                if props.is_current {
                    span { class: "text-xs uppercase tracking-wide text-amber-600  ",
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
