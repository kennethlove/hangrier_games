use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, Phase};

#[derive(Props, PartialEq, Clone)]
pub struct PhaseBoundaryCardProps {
    pub message: GameMessage,
}

fn phase_visual(phase: Phase) -> (&'static str, &'static str) {
    match phase {
        Phase::Dawn => ("🌅", "border-sky-400 bg-sky-50 "),
        Phase::Day => ("☀️", "border-amber-500 bg-amber-50 "),
        Phase::Dusk => ("🌇", "border-purple-500 bg-purple-50 "),
        Phase::Night => ("🌙", "border-indigo-600 bg-indigo-50 "),
    }
}

/// Renders phase boundary announcements (`PhaseStarted`, `PhaseEnded`).
/// Uses the inner `phase` field to pick a per-phase icon + accent color
/// so all four phases (Dawn / Day / Dusk / Night) are visually distinct.
#[component]
pub fn PhaseBoundaryCard(props: PhaseBoundaryCardProps) -> Element {
    let (icon, accent) = match &props.message.payload {
        MessagePayload::PhaseStarted { phase, .. } | MessagePayload::PhaseEnded { phase, .. } => {
            phase_visual(*phase)
        }
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
