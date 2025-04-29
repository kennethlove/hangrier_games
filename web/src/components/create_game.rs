use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::{Button, Input, ThemedButton};
use crate::LoadingState;
use crate::env::APP_API_HOST as API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use std::ops::Deref;
use dioxus::html::link::disabled;
use crate::storage::{use_persistent, AppState};

async fn create_game(args: (Option<String>, String)) -> MutationResult<MutationValue, MutationError> {
    let name = args.0.clone();
    let token = args.1.clone();
    let client = reqwest::Client::new();
    let json_body = match name {
        Some(name) => Game::new(&name),
        None => Game::default()
    };

    let response = client.request(
        reqwest::Method::POST,
        format!("{}/api/games", &*API_HOST))
        .bearer_auth(token)
        .json(&json_body);

    match response.send().await {
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
    let storage = use_persistent("hangry-games", AppState::default);
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mutate = use_mutation(create_game);
    let mut loading_signal = use_context::<Signal<LoadingState>>();

    let onclick = move |_| {
        loading_signal.set(LoadingState::Loading);
        let token = storage.get().jwt.expect("No JWT found");
        spawn(async move {
            mutate.manual_mutate((None, token)).await;
            if mutate.result().is_ok() {
                if let MutationResult::Ok(MutationValue::NewGame(_game)) = mutate.result().deref() {
                    loading_signal.set(LoadingState::Loaded);
                    client.invalidate_queries(&[QueryKey::Games]);
                }
            }
        });
    };

    rsx! {
        ThemedButton {
            onclick,
            "Quickstart"
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mut game_name_signal: Signal<String> = use_signal(String::default);
    let mutate = use_mutation(create_game);
    let mut loading_signal = use_context::<Signal<LoadingState>>();

    let onsubmit = move |_| {
        let token = storage.get().jwt.expect("No JWT found");
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return; }
        loading_signal.set(LoadingState::Loading);

        spawn(async move {
            mutate.manual_mutate((Some(name), token)).await;
            if mutate.result().is_ok() {

                match mutate.result().deref() {
                    MutationResult::Ok(MutationValue::NewGame(_game)) => {
                        client.invalidate_queries(&[QueryKey::Games]);
                        loading_signal.set(LoadingState::Loaded);
                        game_name_signal.set(String::default());
                    },
                    MutationResult::Err(MutationError::UnableToCreateGame) => {},
                    _ => {}
                }
            }
        });
    };

    rsx! {
        form {
            class: "flex flex-row justify-center gap-2",
            onsubmit,
            label {
                r#for: "game-name",
                class: "sr-only",
                "Game name"
            }
            Input {
                id: "game-name",
                name: "game-name",
                r#type: "text",
                placeholder: "Game name",
                value: game_name_signal.read().clone(),
                oninput: move |e: Event<FormData>| {
                    game_name_signal.set(e.value().clone());
                }
            }
            ThemedButton {
                r#type: "submit",
                "Create game"
            }
        }
    }
}
