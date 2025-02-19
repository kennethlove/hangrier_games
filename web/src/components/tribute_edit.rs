use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;
use reqwest::{Response, StatusCode};
use shared::{DeleteTribute, EditTribute};
use std::ops::Deref;
use std::time::Duration;

async fn edit_tribute(tribute: EditTribute) -> MutationResult<MutationValue, MutationError> {
    let game_identifier = GAME.with_borrow(|g| { g.identifier.clone() });
    let identifier = tribute.clone().0;

    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/tributes/{}", API_HOST.clone(), game_identifier, identifier);

    let response = client
        .put(url)
        .json(&tribute.clone())
        .send().await;

    if response.expect("Failed to update tribute").status().is_success() {
        MutationResult::Ok(MutationValue::TributeUpdated(identifier))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn TributeEdit(identifier: String, district: u32, name: String) -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    let onclick = move |_| {
        edit_tribute_signal.set(Some(EditTribute(identifier.clone(), district, name.clone())));
    };

    rsx! {
        button {
            onclick,
            "e"
        }
    }
}

#[component]
pub fn EditTributeModal() -> Element {
    let edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    rsx! {
        dialog {
            role: "confirm",
            open: edit_tribute_signal.read().clone().is_some(),

            EditTributeForm {}
        }
    }
}

#[component]
pub fn EditTributeForm() -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();
    let tribute_details = edit_tribute_signal.read().clone().unwrap_or_default();
    let name = tribute_details.2.clone();
    let district = tribute_details.1;

    let game: Signal<Option<Game>> = use_context();
    if game.peek().is_none() {
        return rsx! {}
    }
    let game = game.unwrap();
    let game_identifier = game.identifier.clone();

    let mutate = use_mutation(edit_tribute);

    let dismiss = move |_| {
        edit_tribute_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let save = move |e: Event<FormData>| {
        let game_identifier = game_identifier.clone();
        let tribute_details = edit_tribute_signal.read().clone().expect("No details provided");
        let identifier = tribute_details.0.clone();

        let data = e.data().values();
        let name = data.get("name").expect("No name value").0[0].clone();
        let district: u32 = data.get("district").expect("No district value").0[0].clone().parse().unwrap();

        if !name.is_empty() && (1..=12u32).contains(&district) {
            let edit_tribute = EditTribute(identifier.clone(), district.clone(), name.clone());
            spawn(async move {
                mutate.manual_mutate(edit_tribute.clone()).await;
                edit_tribute_signal.set(Some(edit_tribute.clone()));

                if let MutationResult::Ok(MutationValue::TributeUpdated(identifier)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Game(game_identifier.clone())]);
                    edit_tribute_signal.set(None);
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
            label {
                "District",

                select {
                    name: "district",
                    for n in 1..=12u32 {
                        option {
                            value: n,
                            selected: n == district,
                            "{n}"
                        }
                    }
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
