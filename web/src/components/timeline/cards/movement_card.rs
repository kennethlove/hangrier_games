use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct MovementCardProps {
    pub message: GameMessage,
}

#[component]
pub fn MovementCard(props: MovementCardProps) -> Element {
    let body = match &props.message.payload {
        MessagePayload::TributeMoved { tribute, from, to } => {
            format!("{} moved from {} to {}", tribute.name, from.name, to.name)
        }
        MessagePayload::TributeHidden { tribute, area } => {
            format!("{} hid in {}", tribute.name, area.name)
        }
        MessagePayload::AreaClosed { area } => format!("Area closed: {}", area.name),
        MessagePayload::AreaEvent {
            area, description, ..
        } => format!("{}: {}", area.name, description),
        _ => "movement event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-sky-500 bg-sky-50 theme2:bg-sky-950 p-3",
            header { class: "font-semibold", "🧭 {body}" }
        }
    }
}
