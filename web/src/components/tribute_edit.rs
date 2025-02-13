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
    let name = tribute.clone().0;
    let district = tribute.clone().1;
    let identifier = tribute.clone().2;

    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/tributes/{}", API_HOST.clone(), game_name, identifier);

    let response = client
        .put(url)
        .json(&tribute.clone())
        .send().await;

    if response.unwrap().status().is_success() {
        MutationResult::Ok(MutationValue::TributeUpdated(name, district))
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

            if edit_tribute_signal.read().is_some() {
                EditTributeForm {}
            }
        }
    }
}

#[component]
pub fn EditTributeForm() -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    let game_name = GAME.with_borrow(|g| { g.name.clone() });

    let tribute_details = edit_tribute_signal.read().clone().unwrap();
    let mut name_signal: Signal<String> = use_signal(|| tribute_details.0.clone());
    let mut district_signal: Signal<u32> = use_signal(|| tribute_details.1.clone());

    let mutate = use_mutation(edit_tribute);

    let dismiss = move |_| {
        edit_tribute_signal.set(None);
    };

    let save = move |_| {
        let game_name = game_name.clone();
        let tribute_details = edit_tribute_signal.read().clone().expect("No details provided");
        let name = name_signal.read().clone();
        let district = district_signal.read().clone();
        let identifier = tribute_details.2;

        match (name.is_empty(), (1u32..=12u32).contains(&district)) {
            (false, true) => {
                spawn(async move {
                    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
                    mutate.manual_mutate(EditTribute(name, district, identifier)).await;
                    if let MutationResult::Ok(MutationValue::TributeUpdated(name, district)) = mutate.result().deref() {
                        client.invalidate_queries(&[QueryKey::Tributes(game_name.clone())]);
                        edit_tribute_signal.set(None);
                    }
                });
            }
            (_, _) => {}
        }
    };

    rsx! {
        form {
            label {
                "Name",

                input {
                    r#type: "text",
                    name: "name",
                    value: name_signal.read().clone(),
                    oninput: move |e| {
                        name_signal.set(e.value().clone());
                    }
                }
            }
            label {
                "District",

                input {
                    r#type: "number",
                    name: "district",
                    value: district_signal.read().clone(),
                    max: 12,
                    min: 1,
                    oninput: move |e| {
                        district_signal.set(e.value().clone().parse().unwrap());
                    }
                }
            }
            button {
                r#type: "button",
                onclick: save,
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
