use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use shared::DeleteGame;
use std::ops::Deref;

async fn delete_game(delete_game_info: DeleteGame) -> MutationResult<MutationValue, MutationError> {
    let identifier = delete_game_info.0;
    let name = delete_game_info.1;
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}", API_HOST.clone(), identifier);

    let response = client
        .delete(url)
        .send().await;

    if response.unwrap().status().is_success() {
        MutationResult::Ok(MutationValue::GameDeleted(identifier, name))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn GameDelete(game_identifier: String, game_name: String) -> Element {
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();

    let onclick = move |_| {
        let identifier = game_identifier.clone();
        let name = game_name.clone();

        delete_game_signal.set(Some(DeleteGame(identifier, name)));
    };

    rsx! {
        button {
            class: "border px-2 py-1",
            onclick,
            "x"
        }
    }
}

#[component]
pub fn DeleteGameModal() -> Element {
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();
    let delete_game_info = delete_game_signal.read().clone();
    let mutate = use_mutation(delete_game);
    
    let name = {
        if let Some(details) = delete_game_info.clone() {
            details.1
        } else { String::new() }
    };

    let dismiss = move |_| {
        delete_game_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let delete = move |_| {
        if let Some(dg) = delete_game_info.clone() {
            spawn(async move {
                mutate.manual_mutate(dg.clone()).await;
                if let MutationResult::Ok(MutationValue::GameDeleted(_, _)) = mutate.result().deref() {
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
            div { class: "fixed inset-0 bg-red-200/25 transition-opacity backdrop-blur-sm backgrop-grayscale" }
            div {
                class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                div {
                    class: "flex items-center gap-4 min-h-full justify-center",
                    div {
                        class: "relative transform overflow-hidden p-2",
                        div {
                            class: "mx-auto border p-2 bg-red-200",
                            div {
                                h1 {
                                    class: "block",
                                    "Delete game?"
                                }
                                p { r#"Are you sure you want to delete "{name}"?"#}
                            }
                            div {
                                class: "flex justify-end gap-4",
                                button {
                                    class: "border px-2 py-1",
                                    r#type: "button",
                                    onclick: delete,
                                    "Yes"
                                }
                                button {
                                    class: "border px-2 py-1",
                                    r#type: "button",
                                    onclick: dismiss,
                                    "No"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
