use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use shared::EditGame;
use std::ops::Deref;
use crate::components::icons::edit::EditIcon;

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
pub fn GameEdit(identifier: String, name: String, icon_class: String) -> Element {
    let mut edit_game_signal: Signal<Option<EditGame>> = use_context();
    let title = format!("Edit {name}");

    let onclick = move |_| {
        let name = name.clone();
        edit_game_signal.set(Some(EditGame(identifier.clone(), name.clone())));
    };

    rsx! {
        button {
            class: "button cursor-pointer",
            title,
            onclick,
            EditIcon { class: icon_class }
            label {
                class: "sr-only",
                "edit"
            }
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
            div { class: "fixed inset-0 backdrop-blur-sm backdrop-grayscale" }
            div {
                class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                div {
                    class: "flex items-center gap-8 min-h-full justify-center",
                    EditGameForm {}
                }
            }
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
            class: "mx-auto p-2 bg-stone-200 grid grid-col gap-4",
            onsubmit: save,
            h1 {
                class: "block theme1:bg-red-900 p-2 text-stone-200 text-lg",
                "Edit game"
            }
            label {
                "Name",

                input {
                    class: "border ml-2 px-2 py-1",
                    r#type: "text",
                    name: "name",
                    value: name,
                }
            }
            div {
                class: "flex justify-end gap-2",
                button {
                    class: "border px-2 py-1",
                    r#type: "submit",
                    "Update"
                }
                button {
                    class: "border px-2 py-1",
                    onclick: dismiss,
                    r#type: "button",
                    "Cancel"
                }
            }
        }
    }
}
