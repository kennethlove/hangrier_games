use crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, combat_swing_card::CombatSwingCard,
    cycle_card::CycleCard, death_card::DeathCard, item_card::ItemCard, movement_card::MovementCard,
    stamina_card::StaminaCard, state_card::StateCard, survival_card::SurvivalCard,
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
            MessageKind::Movement => rsx! { MovementCard {
                game_identifier: props.game_identifier.clone(),
                message: props.message.clone(),
            } },
            MessageKind::Item => rsx! { ItemCard {
                game_identifier: props.game_identifier.clone(),
                message: props.message.clone(),
            } },
            MessageKind::State => match payload {
                MessagePayload::StaminaBandChanged { .. } => {
                    rsx! { StaminaCard { message: props.message.clone() } }
                }
                MessagePayload::HungerBandChanged { .. }
                | MessagePayload::ThirstBandChanged { .. }
                | MessagePayload::ShelterSought { .. }
                | MessagePayload::Foraged { .. }
                | MessagePayload::Drank { .. }
                | MessagePayload::Ate { .. } => rsx! { SurvivalCard { message: props.message.clone() } },
                MessagePayload::CycleStart { .. }
                | MessagePayload::CycleEnd { .. }
                | MessagePayload::GameEnded { .. } => rsx! { CycleCard { message: props.message.clone() } },
                _ => rsx! { StateCard { message: props.message.clone() } },
            },
            MessageKind::CombatSwing => {
                if let MessagePayload::CombatSwing(beat) = payload {
                    rsx! { CombatSwingCard {
                        game_identifier: props.game_identifier.clone(),
                        beat,
                    } }
                } else { rsx! {} }
            }
        }
    }
}
