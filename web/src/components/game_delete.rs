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
    let mutate = use_mutation(delete_game);
    let name = game_name.clone();

    let onclick = move |_| {
        mutate.mutate(name.clone());
    };

    rsx! {
        button {
            onclick,
            "x"
        }
    }
}
