use crate::cache::{QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_init_query_client, use_query_client};

use crate::components::{
    CreateGameButton,
    CreateGameForm,
    GamesList,
};

pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let copyright = "&copy; 2025";

    rsx! {
        h1 { "Hangry Games" }
        CreateGameButton {}
        CreateGameForm {}
        GamesList {}

        button {
            onclick: move |_| {
                client.invalidate_query(QueryKey::Games)
            },
            label { "Refresh" }
        }
        p {
            dangerous_inner_html: "{copyright}",
        }
    }
}

