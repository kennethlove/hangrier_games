use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;

async fn delete_game(name: String) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("http://127.0.0.1:3000/api/games/{}", name))
        .send().await.unwrap();

    if response.status().is_server_error() {
        MutationResult::Err(MutationError::Unknown)
    } else {
        let client = use_query_client::<QueryValue, QueryError, QueryKey>();
        client.invalidate_queries(&[QueryKey::Games]);
        MutationResult::Ok(MutationValue::GameDeleted(name))
    }
}

#[component]
pub fn GameDelete(game_name: String) -> Element {
    let mut delete_game_signal: Signal<Option<String>> = use_context();

    let mutate = use_mutation(delete_game);
    let name = game_name.clone();

    let onclick = move |_| {
        // mutate.mutate(name.clone());
        delete_game_signal.set(Some(name.clone()));
    };

    rsx! {
        button {
            onclick,
            "x"
        }
    }
}
#[component]
pub fn DeleteGameModal() -> Element {
    let mut delete_game_signal: Signal<Option<String>> = use_context();
    let game_name = delete_game_signal.peek().clone();
    let name = game_name.clone().unwrap_or_default();
    let mutate = use_mutation(delete_game);

    let dismiss = move |_| {
        delete_game_signal.set(None);
    };

    let delete = move |_| {
        if let Some(name) = game_name.clone() {
            mutate.mutate(name.clone());
            delete_game_signal.set(None);
        }
    };

    rsx! {
        dialog {
            role: "confirm",
            open: delete_game_signal.read().clone().is_some(),
            div {
                h1 { "Delete game" }
                p { "Delete {name}"}
                form {
                    method: "dialog",
                    button {
                        r#type: "button",
                        onclick: delete,
                        "Continue"
                    }
                    button {
                        r#type: "button",
                        onclick: dismiss,
                        "Close"
                    }
                }
            }
        }
    }
}
