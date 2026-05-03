use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::{AreaRef, GameMessage, ItemRef, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct ItemCardProps {
    pub game_identifier: String,
    pub message: GameMessage,
}

#[component]
fn ItemLink(game_identifier: String, item: ItemRef) -> Element {
    rsx! {
        Link {
            to: Routes::ItemDetail {
                game_identifier: game_identifier.clone(),
                item_identifier: item.identifier.clone(),
            },
            class: "underline",
            "{item.name}"
        }
    }
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
pub fn ItemCard(props: ItemCardProps) -> Element {
    let gid = props.game_identifier.clone();
    let inner = match &props.message.payload {
        MessagePayload::ItemFound {
            tribute,
            item,
            area,
        } => rsx! {
            "{tribute.name} found "
            ItemLink { game_identifier: gid.clone(), item: item.clone() }
            " in "
            AreaLink { game_identifier: gid.clone(), area: area.clone() }
        },
        MessagePayload::ItemUsed { tribute, item } => rsx! {
            "{tribute.name} used "
            ItemLink { game_identifier: gid.clone(), item: item.clone() }
        },
        MessagePayload::ItemDropped {
            tribute,
            item,
            area,
        } => rsx! {
            "{tribute.name} dropped "
            ItemLink { game_identifier: gid.clone(), item: item.clone() }
            " in "
            AreaLink { game_identifier: gid.clone(), area: area.clone() }
        },
        MessagePayload::SponsorGift {
            recipient,
            item,
            donor,
        } => rsx! {
            "{recipient.name} received "
            ItemLink { game_identifier: gid.clone(), item: item.clone() }
            " from {donor}"
        },
        _ => rsx! { "item event" },
    };
    rsx! {
        article { class: "rounded border-l-4 border-yellow-500 bg-yellow-50 theme2:bg-yellow-950 p-3",
            header { class: "font-semibold",
                "🎒 "
                {inner}
            }
        }
    }
}
