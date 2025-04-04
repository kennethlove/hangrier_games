use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::Button;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use std::ops::Deref;

async fn create_game(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = match name {
        Some(name) => Game::new(&name),
        None => Game::default()
    };

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
                if let MutationResult::Ok(MutationValue::NewGame(_game)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Games]);
                }
            }
        });
    };

    rsx! {
        Button {
            extra_classes: Some(r#"
            theme1:bg-radial
            theme1:from-amber-300
            theme1:to-red-600
            theme1:border-red-600
            theme1:text-red-900

            theme2:text-green-800
            theme2:bg-linear-to-b
            theme2:from-green-400
            theme2:to-teal-500
            theme2:border-none
            theme2:hover:text-green-200
            theme2:hover:from-green-500
            theme2:hover:to-teal-600
            "#.into()),
            onclick,
            "Quickstart"
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mut game_name_signal: Signal<String> = use_signal(String::default);
    let mutate = use_mutation(create_game);

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return; }

        spawn(async move {
            mutate.manual_mutate(Some(name)).await;
            if mutate.result().is_ok() {

                match mutate.result().deref() {
                    MutationResult::Ok(MutationValue::NewGame(_game)) => {
                        client.invalidate_queries(&[QueryKey::Games]);
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
            input {
                class: r#"
                block
                focus:outline-none
                border
                w-half
                px-2
                py-1
                theme1:border-amber-600
                theme1:text-amber-300
                theme1:placeholder-amber-600

                theme2:border-green-400
                theme2:text-green-200
                theme2:placeholder-green-200/50

                "#,
                id: "game-name",
                name: "game-name",
                r#type: "text",
                placeholder: "Game name",
                value: game_name_signal.read().clone(),
                oninput: move |e| {
                    game_name_signal.set(e.value().clone());
                }
            }
            Button {
                extra_classes: Some(r#"
                theme1:bg-radial
                theme1:from-amber-300
                theme1:to-red-600
                theme1:border-red-600
                theme1:text-red-900

                theme2:text-green-800
                theme2:bg-linear-to-b
                theme2:from-green-400
                theme2:to-teal-500
                theme2:border-none
                theme2:hover:text-green-200
                theme2:hover:from-green-500
                theme2:hover:to-teal-600
                "#.into()),
                r#type: "submit",
                "Create game"
            }
        }
    }
}
