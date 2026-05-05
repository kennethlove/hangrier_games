//! TributeFilterChips — per-tribute filter row for the timeline.
//!
//! Renders one chip per tribute in the game (plus an "All" chip). Selecting
//! a tribute filters the timeline to events that involve that tribute via
//! `MessagePayload::involves`. State is held in `PeriodFilters` and persisted
//! to gloo-storage like the category filter.

use crate::components::game_tributes::GameTributesQ;
use crate::components::timeline::PeriodFilters;
use dioxus::prelude::*;
use dioxus_query::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TributeFilterChipsProps {
    pub game_identifier: String,
}

#[component]
pub fn TributeFilterChips(props: TributeFilterChipsProps) -> Element {
    let mut filters: Signal<PeriodFilters> = use_context();
    let game_id = props.game_identifier.clone();
    {
        let mut f = filters.write();
        f.hydrate(&game_id);
    }
    let current = filters.read().tribute_filter(&game_id);

    let q = use_query(Query::new(game_id.clone(), GameTributesQ));
    let reader = q.read();
    let state = reader.state();

    let chip_class = |active: bool| -> &'static str {
        if active {
            "rounded-full px-3 py-1 text-sm bg-amber-500 text-amber-50  "
        } else {
            "rounded-full px-3 py-1 text-sm border border-amber-400 text-amber-700    "
        }
    };

    let on_all = {
        let game_id = game_id.clone();
        move |_| filters.write().set_tribute_filter(&game_id, None)
    };

    match &*state {
        QueryStateData::Settled { res: Ok(resp), .. } => {
            let mut tributes = resp.tributes.clone();
            tributes.sort_by(|a, b| a.name.cmp(&b.name));
            rsx! {
                div { class: "flex flex-wrap gap-2 mb-4",
                    button {
                        class: chip_class(current.is_none()),
                        onclick: on_all,
                        "All tributes"
                    }
                    for tribute in tributes {
                        {
                            let game_id = game_id.clone();
                            let id = tribute.identifier.clone();
                            let active = current.as_deref() == Some(id.as_str());
                            let label = tribute.name.clone();
                            rsx! {
                                button {
                                    key: "{id}",
                                    class: chip_class(active),
                                    onclick: move |_| {
                                        let next = if active {
                                            None
                                        } else {
                                            Some(id.clone())
                                        };
                                        filters.write().set_tribute_filter(&game_id, next);
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => rsx! {},
    }
}
