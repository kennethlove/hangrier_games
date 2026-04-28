use crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, death_card::DeathCard,
    item_card::ItemCard, movement_card::MovementCard, state_card::StateCard,
};
use dioxus::prelude::*;
use shared::messages::{GameMessage, MessageKind, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct EventCardProps {
    pub game_identifier: String,
    pub message: GameMessage,
}

#[component]
pub fn EventCard(props: EventCardProps) -> Element {
    let kind = props.message.payload.kind();
    let payload = props.message.payload.clone();
    rsx! {
        match kind {
            MessageKind::Death => {
                if let MessagePayload::TributeKilled { victim, killer, cause } = payload {
                    rsx! { DeathCard {
                        game_identifier: props.game_identifier.clone(),
                        victim,
                        killer,
                        cause,
                    } }
                } else { rsx! {} }
            }
            MessageKind::Combat => {
                if let MessagePayload::Combat(engagement) = payload {
                    rsx! { CombatCard {
                        game_identifier: props.game_identifier.clone(),
                        attacker: engagement.attacker,
                        target: engagement.target,
                        outcome: engagement.outcome,
                        detail_lines: engagement.detail_lines,
                    } }
                } else { rsx! {} }
            }
            MessageKind::Alliance => rsx! { AllianceCard { message: props.message.clone() } },
            MessageKind::Movement => rsx! { MovementCard { message: props.message.clone() } },
            MessageKind::Item => rsx! { ItemCard { message: props.message.clone() } },
            MessageKind::State => rsx! { StateCard { message: props.message.clone() } },
        }
    }
}
