use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use reqwest::{Error, Response};
use shared::CreateGame;
use std::ops::Deref;

async fn create_game(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = CreateGame { name: name.clone() };
    let response = client.post(format!("{}/api/games", API_HOST.clone()))
        .json(&json_body)
        .send().await;

    match response {
        Ok(response) => {
            match response.json::<Game>().await {
                Ok(game) => {
                    MutationResult::Ok(MutationValue::NewGame(game))
                }
                Err(_) => {
                    MutationResult::Err(MutationError::UnableToCreateGame)
                }
            }
        }
        Err(e) => {
            dioxus_logger::tracing::error!("error creating game: {:?}", e);
            MutationResult::Err(MutationError::UnableToCreateGame)
        }
    }
}

#[component]
pub fn CreateGameButton() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mutate = use_mutation(create_game);

    let onclick = move |_| {
        spawn(async move {
            mutate.manual_mutate(None).await;
            if mutate.result().is_ok() {
                if let MutationResult::Ok(MutationValue::NewGame(game)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Games]);
                }
            } else {}
        });
    };

    rsx! {
        button {
            r#type: "button",
            onclick,
            label { "quickstart" }
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mut game_name_signal: Signal<String> = use_signal(|| String::default());
    let mutate = use_mutation(create_game);

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return; }

        spawn(async move {
            mutate.manual_mutate(Some(name)).await;
            if mutate.result().is_ok() {

                match mutate.result().deref() {
                    MutationResult::Ok(MutationValue::NewGame(game)) => {
                        client.invalidate_queries(&[QueryKey::Games]);
                        game_name_signal.set(String::default());
                    },
                    MutationResult::Err(MutationError::UnableToCreateGame) => {},
                    _ => {}
                }
            } else {}
        });
    };

    rsx! {
        form {
            onsubmit,
            input {
                r#type: "text",
                placeholder: "Game name",
                value: game_name_signal.read().clone(),
                oninput: move |e| {
                    game_name_signal.set(e.value().clone());
                }
            }
            button {
                r#type: "submit",
                label { "create game" }
            }
        }
    }
}
