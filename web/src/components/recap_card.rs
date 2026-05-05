//! RecapCard — collapsible summary shown above the timeline for finished games.
//!
//! Persists collapsed state in `localStorage` per game so reloads remember the
//! reader's choice. Shows the winner (or "all tributes died") and days played.

use crate::routes::Routes;
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use shared::DisplayGame;

#[derive(Props, PartialEq, Clone)]
pub struct RecapCardProps {
    pub game: DisplayGame,
}

#[component]
pub fn RecapCard(props: RecapCardProps) -> Element {
    let key = format!("recap_collapsed:{}", props.game.identifier);
    let initial: bool = LocalStorage::get(&key).unwrap_or(false);
    let mut collapsed = use_signal(|| initial);

    let toggle = {
        let key = key.clone();
        move |_| {
            let new = !collapsed();
            collapsed.set(new);
            let _ = LocalStorage::set(&key, new);
        }
    };

    let days_played = props.game.day.unwrap_or(0);
    let game_id = props.game.identifier.clone();

    rsx! {
        section { class: "rounded-lg border bg-amber-50   p-4 mb-4",
            header {
                class: "flex items-center justify-between cursor-pointer",
                onclick: toggle,
                h2 { class: "text-xl font-semibold", "Game Recap" }
                span { if collapsed() { "▸" } else { "▾" } }
            }
            if !collapsed() {
                div { class: "mt-3 space-y-1 text-sm",
                    p {
                        match props.game.winner.as_ref() {
                            Some(w) => rsx! {
                                "🏆 Winner: "
                                Link {
                                    to: Routes::TributeDetail {
                                        game_identifier: game_id.clone(),
                                        tribute_identifier: w.identifier.clone(),
                                    },
                                    class: "underline",
                                    "{w.name}"
                                }
                            },
                            None => rsx! { "All tributes died" },
                        }
                    }
                    p { "Days played: {days_played}" }
                }
            }
        }
    }
}
