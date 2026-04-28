use crate::components::timeline::FilterMode;
use crate::components::timeline::event_card::EventCard;
use dioxus::prelude::*;
use shared::messages::GameMessage;

#[derive(Props, PartialEq, Clone)]
pub struct TimelineProps {
    pub game_identifier: String,
    pub messages: Vec<GameMessage>,
    pub filter: FilterMode,
}

#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let mut sorted: Vec<GameMessage> = props
        .messages
        .into_iter()
        .filter(|m| props.filter.matches(m.payload.kind()))
        .collect();
    sorted.sort_by_key(|m| (m.tick, m.emit_index));
    rsx! {
        if sorted.is_empty() {
            div { class: "rounded border border-dashed p-6 text-center text-sm",
                "Nothing happened this period."
            }
        } else {
            div { class: "space-y-2",
                for (i, msg) in sorted.into_iter().enumerate() {
                    EventCard {
                        key: "{i}",
                        game_identifier: props.game_identifier.clone(),
                        message: msg,
                    }
                }
            }
        }
    }
}
