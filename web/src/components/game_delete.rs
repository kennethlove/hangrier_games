use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use reqwest::{Response, StatusCode};
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Deref;
use std::time::Duration;
use shared::DeleteGame;

async fn delete_game(name: String) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}", API_HOST.clone(), name);

    let response = client
        .delete(url)
        .send().await;

    if response.unwrap().status().is_success() {
        MutationResult::Ok(MutationValue::GameDeleted(name))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn GameDelete(game_name: String) -> Element {
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();

    let name = game_name.clone();

    let onclick = move |_| {
        delete_game_signal.set(Some(name.clone()));
    };

    rsx! {
        button {
            onclick,
            "x"
        }
    }
}
#[component]
pub fn DeleteGameModal() -> Element {
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();
    let game_name = delete_game_signal.peek().clone();
    let name = game_name.clone().unwrap_or_default();
    let mutate = use_mutation(delete_game);

    let dismiss = move |_| {
        delete_game_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let delete = move |_| {
        if let Some(name) = game_name.clone() {
            spawn(async move {
                mutate.manual_mutate(name.clone()).await;
                if let MutationResult::Ok(MutationValue::GameDeleted(name)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Games]);
                    delete_game_signal.set(None);
                }
            });
        }
    };

    rsx! {
        dialog {
            role: "confirm",
            open: delete_game_signal.read().clone().is_some(),
            div {
                h1 { "Delete game" }
                p { r#"Delete "{name}"?"#}
                button {
                    r#type: "button",
                    onclick: delete,
                    "Yes"
                }
                button {
                    r#type: "button",
                    onclick: dismiss,
                    "No"
                }
            }
        }
    }
}
