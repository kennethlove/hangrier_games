use crate::components::tribute_edit::TributeEdit;
use dioxus::prelude::*;
use game::games::Game;

#[component]
pub fn GameTributes() -> Element {
    let game_signal: Signal<Option<Game>> = use_context();
    
    let game = game_signal.read().clone();
    let tributes = game.unwrap().tributes;

    rsx! {
        ul {
            for tribute in tributes {
                li {
                    "{tribute.name} - {tribute.district}",
                    TributeEdit {
                        identifier: tribute.clone().identifier,
                        district: tribute.district,
                        name: tribute.clone().name,
                    }
                }
            }
        }
    }
}
