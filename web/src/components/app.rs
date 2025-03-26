use dioxus::dioxus_core::AttributeValue;
use dioxus::document::Script;
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::EditGameModal;
use crate::components::tribute_edit::EditTributeModal;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::use_init_query_client;
use game::games::Game;
use shared::{DeleteGame, EditGame, EditTribute};
use crate::storage::{use_persistent, AppState};

#[component]
pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();

    let mut storage = use_persistent("hangry-games", || AppState::default());

    let dark_mode_signal: Signal<bool> = use_signal(|| storage.get().dark_mode);
    use_context_provider(|| dark_mode_signal);

    let game_signal: Signal<Option<Game>> = use_signal(|| None);
    use_context_provider(|| game_signal);

    let delete_game_signal: Signal<Option<DeleteGame>> = use_signal(|| None);
    use_context_provider(|| delete_game_signal);

    let edit_game_signal: Signal<Option<EditGame>> = use_signal(|| None);
    use_context_provider(|| edit_game_signal);
    
    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    let copyright = "&copy; 2025";

    rsx! {
        document::Link {
            rel: "preconnect",
            href: "https://api.fonts.coollabs.io",
            crossorigin: Some("true".into())
        }
        document::Link {
            href: "https://api.fonts.coollabs.io/css2?family=Cinzel:wght@400..900&display=swap",
            rel: "stylesheet"
        }
        document::Link {
            href: "https://api.fonts.coollabs.io/icon?family=Material+Icons",
            rel: "stylesheet"
        }

        document::Stylesheet {
            href: asset!("/assets/dist/main.css")
        }

        div {
            class: if *dark_mode_signal.read() { "dark" } else { "" },
            div {
                class: "grid grid-flow-row min-v-full min-h-screen bg-green-900 p-2 frame",

                Router::<Routes> {}

                p {
                    dangerous_inner_html: "{copyright}",
                }
            }

            EditGameModal {}
            EditTributeModal {}
        }
    }
}

