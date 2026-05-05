use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, Phase};

#[derive(Props, PartialEq, Clone)]
pub struct CycleCardProps {
    pub message: GameMessage,
}

fn phase_visual(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::Dawn => ("🌄", "border-rose-400 bg-rose-50 "),
        Phase::Day => ("☀️", "border-amber-500 bg-amber-50 "),
        Phase::Dusk => ("🌆", "border-violet-500 bg-violet-50 "),
        Phase::Night => ("🌙", "border-indigo-500 bg-indigo-50 "),
    }
}

/// Renders cycle-boundary and game-lifecycle announcements
/// (`CycleStart`, `CycleEnd`, `GameEnded`). Uses the inner `phase`
/// field to pick a per-phase icon + accent color so all four phases
/// (Dawn / Day / Dusk / Night) are visually distinct.
#[component]
pub fn CycleCard(props: CycleCardProps) -> Element {
    let (icon, accent) = match &props.message.payload {
        MessagePayload::CycleStart { phase, .. } | MessagePayload::CycleEnd { phase, .. } => {
            phase_visual(*phase)
        }
        MessagePayload::GameEnded { .. } => ("🏁", "border-emerald-600 bg-emerald-50 "),
        _ => ("📣", "border-sky-500 bg-sky-50 "),
    };
    rsx! {
        article { class: "rounded border-l-4 p-3 {accent}",
            header { class: "font-semibold",
                "{icon} {props.message.content}"
            }
        }
    }
}
