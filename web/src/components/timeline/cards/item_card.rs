use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct ItemCardProps {
    pub message: GameMessage,
}

#[component]
pub fn ItemCard(props: ItemCardProps) -> Element {
    let body = match &props.message.payload {
        MessagePayload::ItemFound {
            tribute, item, area,
        } => format!("{} found {} in {}", tribute.name, item.name, area.name),
        MessagePayload::ItemUsed { tribute, item } => {
            format!("{} used {}", tribute.name, item.name)
        }
        MessagePayload::ItemDropped {
            tribute, item, area,
        } => format!("{} dropped {} in {}", tribute.name, item.name, area.name),
        MessagePayload::SponsorGift {
            recipient,
            item,
            donor,
        } => format!("{} received {} from {}", recipient.name, item.name, donor),
        _ => "item event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-yellow-500 bg-yellow-50 theme2:bg-yellow-950 p-3",
            header { class: "font-semibold", "🎒 {body}" }
        }
    }
}
