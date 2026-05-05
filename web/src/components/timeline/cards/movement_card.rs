use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::{AreaRef, GameMessage, MessagePayload};

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
            if description.trim().is_empty() {
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
    rsx! {
        article { class: "rounded border-l-4 border-sky-500 bg-sky-50  p-3",
            header { class: "font-semibold",
                "🧭 "
                {body}
            }
        }
    }
}
