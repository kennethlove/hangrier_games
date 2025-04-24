use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::GAME;
use shared::DeleteTribute;
use std::ops::Deref;

#[allow(dead_code)]
async fn delete_tribute(name: String) -> MutationResult<MutationValue, MutationError> {
    let game_name = GAME.with_borrow(|g| { g.name.clone() });

    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/tributes/{}", API_HOST.clone(), game_name, name);

    let response = client
        .delete(url)
        .send().await;

    if response.unwrap().status().is_success() {
        MutationResult::Ok(MutationValue::TributeDeleted(name))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn TributeDelete(tribute_name: String) -> Element {
    let mut delete_tribute_signal: Signal<Option<DeleteTribute>> = use_context();

    let onclick = move |_| {
        delete_tribute_signal.set(Some(DeleteTribute::from(tribute_name.clone())));
    };

    rsx! {
        button {
            onclick,
            "x"
        }
    }
}

#[component]
pub fn DeleteTributeModal() -> Element {
    let mut delete_tribute_signal: Signal<Option<DeleteTribute>> = use_context();
    let game_name = GAME.with_borrow(|g| { g.name.clone() });
    let tribute_name  = delete_tribute_signal.peek().clone();
    let name = tribute_name.clone().unwrap_or_default().clone();
    let mutate = use_mutation(delete_tribute);
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let dismiss = move |_| {
        delete_tribute_signal.set(None);
    };

    let delete = move |_| {
        if let Some(tribute_name) = tribute_name.clone() {
            let game_name = game_name.clone();
            spawn(async move {
                mutate.manual_mutate(tribute_name.clone()).await;
                if let MutationResult::Ok(MutationValue::TributeDeleted(_tribute_name)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Tributes(game_name.clone())]);
                    delete_tribute_signal.set(None);
                }
            });
        }
    };

    rsx! {
        dialog {
            role: "confirm",
            open: delete_tribute_signal.read().clone().is_some(),
            div {
                h1 { "Delete Tribute" }
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
