use crate::components::timeline::FilterMode;
use crate::components::timeline::event_card::EventCard;
use dioxus::prelude::*;
use shared::messages::GameMessage;

#[derive(Props, PartialEq, Clone)]
pub struct TimelineProps {
    pub game_identifier: String,
    pub messages: Vec<GameMessage>,
    pub filter: FilterMode,
    pub tribute_filter: Option<String>,
}

#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let mut sorted: Vec<GameMessage> = props
        .messages
        .into_iter()
        .filter(|m| props.filter.matches(m.payload.kind()))
        .filter(|m| {
            props
                .tribute_filter
                .as_deref()
                .is_none_or(|id| m.payload.involves(id))
        })
        .collect();
    // `emit_index` is the monotonic per-phase emission counter, so it is
    // the only correct chronological order. `tick` is a per-action group
    // id (boundary messages all share tick=0); sorting by it would float
    // every cycle-start, area-event, env-death, and "has fallen" message
    // ahead of per-tribute action messages emitted between them.
    sorted.sort_by_key(|m| m.emit_index);
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
