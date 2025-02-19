use crate::cache::{QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_init_query_client, use_query_client};
use shared::EditGame;
use crate::routes::Routes;

#[component]
pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();

    let delete_game_signal: Signal<Option<String>> = use_signal(|| None);
    use_context_provider(|| delete_game_signal);
    
    let edit_game_signal: Signal<Option<EditGame>> = use_signal(|| None);
    use_context_provider(|| edit_game_signal);

    let copyright = "&copy; 2025";

    rsx! {
        h1 { "Hangry Games" }

        Router::<Routes> {}

        p {
            dangerous_inner_html: "{copyright}",
        }

    }
}

