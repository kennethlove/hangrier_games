use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;
use reqwest::{Response, StatusCode};
use shared::{DeleteTribute, EditGame, EditTribute};
use std::ops::Deref;
use std::time::Duration;

async fn edit_game(game: EditGame) -> MutationResult<MutationValue, MutationError> {
    let identifier = game.0.clone();

    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}", API_HOST.clone(),identifier);

    let response = client
        .put(url)
        .json(&game.clone())
        .send().await;

    if response.expect("Failed to update game").status().is_success() {
        MutationResult::Ok(MutationValue::GameUpdated(identifier))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn GameEdit(identifier: String, name: String) -> Element {
    let mut edit_game_signal: Signal<Option<EditGame>> = use_context();

    let onclick = move |_| {
        edit_game_signal.set(Some(EditGame(identifier.clone(), name.clone())));
    };

    rsx! {
        button {
            onclick,
            "e"
        }
    }
}

#[component]
pub fn EditGameModal() -> Element {
    let edit_game_signal: Signal<Option<EditGame>> = use_context();

    rsx! {
        dialog {
            role: "confirm",
            open: edit_game_signal.read().clone().is_some(),

            EditGameForm {}
        }
    }
}

#[component]
pub fn EditGameForm() -> Element {
    let mut edit_game_signal: Signal<Option<EditGame>> = use_context();
    let game_details = edit_game_signal.read().clone().unwrap_or_default();
    let name = game_details.1.clone();
    let identifier = game_details.0.clone();

    let mutate = use_mutation(edit_game);

    let dismiss = move |_| {
        edit_game_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let save = move |e: Event<FormData>| {
        let identifier = identifier.clone();

        let data = e.data().values();
        let name = data.get("name").expect("No name value").0[0].clone();

        if !name.is_empty() {
            let edit_game = EditGame(identifier.clone(), name.clone());
            spawn(async move {
                mutate.manual_mutate(edit_game.clone()).await;
                edit_game_signal.set(Some(edit_game.clone()));

                if let MutationResult::Ok(MutationValue::GameUpdated(identifier)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Game(identifier.clone())]);
                    edit_game_signal.set(None);
                }
            });
        }
    };

    rsx! {
        form {
            onsubmit: save,
            label {
                "Name",

                input {
                    r#type: "text",
                    name: "name",
                    value: name,
                }
            }
            button {
                r#type: "submit",
                "Update"
            }
            button {
                r#type: "dialog",
                onclick: dismiss,
                "Cancel"
            }
        }
    }
}
