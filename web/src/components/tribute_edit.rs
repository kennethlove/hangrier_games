use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::GAME;
use game::tributes::Tribute;
use reqwest::{Response, StatusCode};
use shared::{DeleteTribute, EditTribute};
use std::ops::Deref;
use std::time::Duration;

async fn edit_tribute(tribute: EditTribute) -> MutationResult<MutationValue, MutationError> {
    let game_name = GAME.with_borrow(|g| { g.name.clone() });
    let identifier = tribute.clone().2;

    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/tributes/{}", API_HOST.clone(), game_name, identifier);

    let response = client
        .put(url)
        .json(&tribute.clone())
        .send().await;

    if response.unwrap().status().is_success() {
        MutationResult::Ok(MutationValue::TributeUpdated(identifier))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn TributeEdit(name: String, district: u32, identifier: String) -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    let onclick = move |_| {
        edit_tribute_signal.set(Some(EditTribute(name.clone(), district.clone(), identifier.clone())));
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
    let name = tribute_details.0.clone();
    let district = tribute_details.1;

    let game_name = GAME.with_borrow(|g| { g.name.clone() });

    let mutate = use_mutation(edit_tribute);

    let dismiss = move |_| {
        edit_tribute_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let save = move |e: Event<FormData>| {
        let game_name = game_name.clone();
        let tribute_details = edit_tribute_signal.read().clone().expect("No details provided");
        let identifier = tribute_details.2.clone();

        let data = e.data().values();
        let name = data.get("name").expect("No name value").0[0].clone();
        let district: u32 = data.get("district").expect("No district value").0[0].clone().parse().unwrap();

        if !name.is_empty() && (1..=12u32).contains(&district) {
            spawn(async move {
                mutate.manual_mutate(EditTribute(name.clone(), district.clone(), identifier.clone())).await;
                if let MutationResult::Ok(MutationValue::TributeUpdated(identifier)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Tributes(game_name.clone())]);
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
