use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use shared::CreateGame;
use crate::routes::Routes;
use std::ops::Deref;
use reqwest::{Error, Response};

async fn create_game(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = CreateGame { name: name.clone() };
    let response = client.post("http://127.0.0.1:3000/api/games")
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
    let navigator = use_navigator();

    let onclick = move |_| {
        spawn(async move {
            mutate.manual_mutate(None).await;
            client.invalidate_queries(&[QueryKey::Games]);
            if mutate.result().is_ok() {

                if let MutationResult::Ok(MutationValue::NewGame(game)) = mutate.result().deref() {
                    navigator.push(Routes::GameDetail { name: game.name.clone() });
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
    let navigator = use_navigator();

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return; }

        spawn(async move {
            mutate.manual_mutate(Some(name)).await;
            if mutate.result().is_ok() {

                match mutate.result().deref() {
                    MutationResult::Ok(MutationValue::NewGame(game)) => {
                        client.invalidate_queries(&[QueryKey::Games]);
                        navigator.push(Routes::GameDetail { name: game.name.clone() });
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
