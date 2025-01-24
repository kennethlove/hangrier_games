use std::net::{IpAddr, Ipv4Addr};
use std::ops::Deref;
use std::time::Duration;
use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use reqwest::{Response, StatusCode};

async fn delete_game(name: String) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let url: String = format!("http://127.0.0.1:3000/api/games/{}", name);

    let response = client
        .delete(url)
        .send().await;

    dioxus_logger::tracing::info!("{:?}", &response);

    if !response.unwrap().status().is_success() {
        MutationResult::Err(MutationError::Unknown)
    } else {
        MutationResult::Ok(MutationValue::GameDeleted(name))
    }
}

#[component]
pub fn GameDelete(game_name: String) -> Element {
    let mut delete_game_signal: Signal<Option<String>> = use_context();

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
    let mut delete_game_signal: Signal<Option<String>> = use_context();
    let game_name = delete_game_signal.peek().clone();
    let name = game_name.clone().unwrap_or_default();
    let mutate = use_mutation(delete_game);

    let dismiss = move |_| {
        delete_game_signal.set(None);
    };

    let delete = move |e: Event<MouseData>| {
        e.prevent_default();
        if let Some(name) = game_name.clone() {
            spawn(async move {
                let client = use_query_client::<QueryValue, QueryError, QueryKey>();

                mutate.manual_mutate(name.clone()).await;
                if let MutationResult::Ok(MutationValue::GameDeleted(name)) = mutate.result().deref() {
                    let timeout = gloo_timers::callback::Timeout::new(1, move || {
                        client.invalidate_queries(&[QueryKey::Games]);
                        delete_game_signal.set(None);
                    });
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
                p { "Delete {name}"}
                button {
                    r#type: "button",
                    onclick: delete,
                    "Continue"
                }
                button {
                    r#type: "button",
                    onclick: dismiss,
                    "Close"
                }
            }
        }
    }
}
