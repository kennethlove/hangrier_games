//! FilterChips — All/Subset toggle row for filtering timeline events by category.
//!
//! Hydrates and persists the per-game filter through `PeriodFilters` context.
//! Uses real `MessageKind` variants (Death/Combat/Alliance/Movement/Item).
//! `MessageKind::State` is intentionally omitted: it always passes the filter
//! so structural events stay visible regardless of the chip selection.

use crate::components::timeline::{FilterMode, PeriodFilters};
use dioxus::prelude::*;
use shared::messages::MessageKind;
use std::collections::HashSet;

#[derive(Props, PartialEq, Clone)]
pub struct FilterChipsProps {
    pub game_identifier: String,
}

const CATEGORIES: &[(MessageKind, &str)] = &[
    (MessageKind::Death, "Deaths"),
    (MessageKind::Combat, "Combat"),
    (MessageKind::Alliance, "Alliances"),
    (MessageKind::Movement, "Movement"),
    (MessageKind::Item, "Items"),
];

#[component]
pub fn FilterChips(props: FilterChipsProps) -> Element {
    let mut filters: Signal<PeriodFilters> = use_context();
    let game_id = props.game_identifier.clone();
    {
        let mut f = filters.write();
        f.hydrate(&game_id);
    }
    let current = filters.read().filter_for(&game_id);

    let chip_class = |active: bool| -> &'static str {
        if active {
            "rounded-full px-3 py-1 text-sm bg-amber-500 text-amber-50  "
        } else {
            "rounded-full px-3 py-1 text-sm border border-amber-400 text-amber-700    "
        }
    };

    let on_all = {
        let game_id = game_id.clone();
        move |_| filters.write().set_filter(&game_id, FilterMode::All)
    };

    rsx! {
        div { class: "flex flex-wrap gap-2 mb-4",
            button {
                class: chip_class(current.is_all()),
                onclick: on_all,
                "All"
            }
            for (kind, label) in CATEGORIES.iter().copied() {
                {
                    let game_id = game_id.clone();
                    let active = match &current {
                        FilterMode::All => false,
                        FilterMode::Subset(s) => s.contains(&kind),
                    };
                    rsx! {
                        button {
                            key: "{label}",
                            class: chip_class(active),
                            onclick: move |_| {
                                let mut f = filters.write();
                                let next = match f.filter_for(&game_id) {
                                    FilterMode::All => {
                                        let mut s = HashSet::new();
                                        s.insert(kind);
                                        FilterMode::Subset(s)
                                    }
                                    FilterMode::Subset(mut s) => {
                                        if s.contains(&kind) {
                                            s.remove(&kind);
                                        } else {
                                            s.insert(kind);
                                        }
                                        if s.is_empty() {
                                            FilterMode::All
                                        } else {
                                            FilterMode::Subset(s)
                                        }
                                    }
                                };
                                f.set_filter(&game_id, next);
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
