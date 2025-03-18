use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::EditGameModal;
use crate::components::tribute_edit::EditTributeModal;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::use_init_query_client;
use game::games::Game;
use shared::{DeleteGame, EditGame, EditTribute};

#[component]
pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();
    
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
        document::Stylesheet {
            href: asset!("/assets/dist/main.css")
        }

        div {
            class: "container",
            h1 { "Hangry Games" }

            Router::<Routes> {}

            p {
                dangerous_inner_html: "{copyright}",
            }
        }

        EditGameModal {}
        EditTributeModal {}
    }
}
