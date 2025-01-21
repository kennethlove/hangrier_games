use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use shared::CreateGame;

async fn create_game(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = CreateGame { name: name.clone() };
    let response = client.post("http://127.0.0.1:3000/api/games")
        .json(&json_body)
        .send().await.unwrap();

    match response.json::<Game>().await {
        Ok(game) => {
            let client = use_query_client::<QueryValue, QueryError, QueryKey>();
            client.invalidate_queries(&[QueryKey::Games]);

            MutationResult::Ok(MutationValue::NewGame(game))
        }
        Err(e) => {
            MutationResult::Err(MutationError::UnableToCreateGame)
        }
    }
}

#[component]
pub fn CreateGameButton() -> Element {
    let mutate = use_mutation(create_game);

    let onclick = move |_| {
        mutate.mutate(None);
    };

    rsx! {
        p { "{*mutate.result():?}" }
        button {
            onclick,
            label { "quickstart" }
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let mut game_name_signal: Signal<String> = use_signal(|| String::default());
    let mutate = use_mutation(create_game);

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return; }

        let game_query = mutate.mutate(Some(name));

        let client = use_query_client::<QueryValue, QueryError, QueryKey>();
        client.invalidate_queries(&[QueryKey::Games]);
        game_name_signal.set("".to_string());
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

