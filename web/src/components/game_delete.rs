use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use shared::DeleteGame;
use std::ops::Deref;
use crate::components::Button;
use crate::components::icons::delete::DeleteIcon;

async fn delete_game(delete_game_info: DeleteGame) -> MutationResult<MutationValue, MutationError> {
    let identifier = delete_game_info.0;
    let name = delete_game_info.1;
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}", API_HOST, identifier);

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
pub fn GameDelete(game_identifier: String, game_name: String, icon_class: String) -> Element {
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();
    let title = format!("Delete {game_name}");

    let onclick = move |_| {
        let identifier = game_identifier.clone();
        let name = game_name.clone();

        delete_game_signal.set(Some(DeleteGame(identifier, name)));
    };

    rsx! {
        Button {
            extra_classes: Some("border-none".to_string()),
            title,
            onclick,
            DeleteIcon { class: icon_class }
            label {
                class: "sr-only",
                "Delete"
            }
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
            div { class: "fixed inset-0 backdrop-blur-sm backdrop-grayscale" }
            div {
                class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                div {
                    class: "flex items-center gap-8 min-h-full justify-center",
                    div {
                        class: r#"
                        mx-auto
                        p-2
                        grid
                        grid-col
                        gap-4
                        theme1:bg-stone-200
                        theme1:text-stone-900
                        theme2:bg-green-200
                        theme1:text-green-900
                        "#,

                        h1 {
                            class: r#"
                            block
                            p-2
                            text-lg
                            theme1:bg-red-900
                            theme1:text-stone-200
                            theme2:bg-green-800
                            theme2:text-green-200
                            "#,
                            "Delete game?"
                        }
                        p {
                            class: "text-md",
                            "Are you sure you want to delete ",
                            br {},
                            r#""{name}"?"#
                        }
                        div {
                            class: "flex justify-end gap-2",
                            Button {
                                onclick: delete,
                                "Yes"
                            }
                            Button {
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
