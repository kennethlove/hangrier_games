use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, MessageSource};

#[derive(Props, PartialEq, Clone)]
pub struct MovementCardProps {
    pub message: GameMessage,
}

#[component]
pub fn MovementCard(props: MovementCardProps) -> Element {
    // Cycle announcements ("Night 4 falls...", "End of day 3.") flow
    // through Game::log() which synthesises a fallback `AreaEvent`
    // payload with `area.name = <game uuid>` and an empty
    // `description`. Render the human-readable `content` (and drop the
    // compass flair) for those instead of the raw uuid prefix.
    let is_game_announcement = matches!(props.message.source, MessageSource::Game(_));
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
        } => {
            if is_game_announcement || description.trim().is_empty() {
                props.message.content.clone()
            } else {
                format!("{}: {}", area.name, description)
            }
        }
        _ => props.message.content.clone(),
    };
    let prefix = if is_game_announcement { "" } else { "🧭 " };
    rsx! {
        article { class: "rounded border-l-4 border-sky-500 bg-sky-50 theme2:bg-sky-950 p-3",
            header { class: "font-semibold", "{prefix}{body}" }
        }
    }
}
