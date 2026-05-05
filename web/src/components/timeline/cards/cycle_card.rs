use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct CycleCardProps {
    pub message: GameMessage,
}

/// Renders cycle-boundary and game-lifecycle announcements
/// (`CycleStart`, `CycleEnd`, `GameEnded`). Falls back to the
/// pre-rendered `content` string for the human-readable text since
/// the typed payload only carries structural data (day/phase/winner).
#[component]
pub fn CycleCard(props: CycleCardProps) -> Element {
    let (icon, accent) = match &props.message.payload {
        MessagePayload::CycleStart { .. } => ("🌅", "border-amber-500 bg-amber-50 "),
        MessagePayload::CycleEnd { .. } => ("🌙", "border-indigo-500 bg-indigo-50 "),
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
