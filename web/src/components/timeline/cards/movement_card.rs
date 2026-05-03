use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::{AreaRef, GameMessage, MessagePayload, MessageSource};

#[derive(Props, PartialEq, Clone)]
pub struct MovementCardProps {
    pub game_identifier: String,
    pub message: GameMessage,
}

#[component]
fn AreaLink(game_identifier: String, area: AreaRef) -> Element {
    rsx! {
        Link {
            to: Routes::AreaDetail {
                game_identifier: game_identifier.clone(),
                area_identifier: area.identifier.clone(),
            },
            class: "underline",
            "{area.name}"
        }
    }
}

#[component]
pub fn MovementCard(props: MovementCardProps) -> Element {
    // Cycle announcements ("Night 4 falls...", "End of day 3.") flow
    // through Game::log() which synthesises a fallback `AreaEvent`
    // payload with `area.name = <game uuid>` and an empty
    // `description`. Render the human-readable `content` (and drop the
    // compass flair) for those instead of the raw uuid prefix.
    let is_game_announcement = matches!(props.message.source, MessageSource::Game(_));
    let gid = props.game_identifier.clone();
    let body = match &props.message.payload {
        MessagePayload::TributeMoved { tribute, from, to } => rsx! {
            "{tribute.name} moved from "
            AreaLink { game_identifier: gid.clone(), area: from.clone() }
            " to "
            AreaLink { game_identifier: gid.clone(), area: to.clone() }
        },
        MessagePayload::TributeHidden { tribute, area } => rsx! {
            "{tribute.name} hid in "
            AreaLink { game_identifier: gid.clone(), area: area.clone() }
        },
        MessagePayload::AreaClosed { area } => rsx! {
            "Area closed: "
            AreaLink { game_identifier: gid.clone(), area: area.clone() }
        },
        MessagePayload::AreaEvent {
            area, description, ..
        } => {
            if is_game_announcement || description.trim().is_empty() {
                rsx! { "{props.message.content}" }
            } else {
                rsx! {
                    AreaLink { game_identifier: gid.clone(), area: area.clone() }
                    ": {description}"
                }
            }
        }
        _ => rsx! { "{props.message.content}" },
    };
    let prefix = if is_game_announcement { "" } else { "🧭 " };
    rsx! {
        article { class: "rounded border-l-4 border-sky-500 bg-sky-50 theme2:bg-sky-950 p-3",
            header { class: "font-semibold",
                "{prefix}"
                {body}
            }
        }
    }
}
