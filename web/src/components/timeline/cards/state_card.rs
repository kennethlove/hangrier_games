use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct StateCardProps {
    pub message: GameMessage,
}

#[component]
pub fn StateCard(props: StateCardProps) -> Element {
    let body = match &props.message.payload {
        MessagePayload::TributeWounded {
            victim,
            attacker,
            hp_lost,
        } => match attacker {
            Some(a) => format!("{} wounded by {} (-{hp_lost} HP)", victim.name, a.name),
            None => format!("{} wounded (-{hp_lost} HP)", victim.name),
        },
        MessagePayload::TributeRested {
            tribute,
            hp_restored,
        } => format!("{} rested (+{hp_restored} HP)", tribute.name),
        MessagePayload::TributeStarved { tribute, hp_lost } => {
            format!("{} is starving (-{hp_lost} HP)", tribute.name)
        }
        MessagePayload::TributeDehydrated { tribute, hp_lost } => {
            format!("{} is dehydrated (-{hp_lost} HP)", tribute.name)
        }
        MessagePayload::SanityBreak { tribute } => {
            format!("{} suffered a sanity break", tribute.name)
        }
        _ => "state event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-gray-400 bg-gray-50 theme2:bg-gray-900 p-2 text-sm",
            "🌫️ {body}"
        }
    }
}
