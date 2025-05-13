use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::icons::delete::DeleteIcon;
use crate::components::Button;
use crate::env::APP_API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult, MutationState};
use shared::DeleteGame;
use std::ops::Deref;
use crate::storage::{use_persistent, AppState};

async fn delete_game(args: (DeleteGame, String)) -> MutationResult<MutationValue, MutationError> {
    let identifier = args.0.0;
    let name = args.0.1;
    let token = args.1;
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}", APP_API_HOST, identifier);

    let response = client
        .delete(url)
        .bearer_auth(token)
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
            class: Some("border-none".to_string()),
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
    let storage = use_persistent("hangry-games", AppState::default);

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
            let token = storage.get().jwt.expect("No JWT found");
            spawn(async move {
                mutate.mutate_async((dg.clone(), token)).await;
                if let MutationState::Settled(Ok(MutationValue::GameDeleted(_, _))) = mutate.result().deref() {
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
                        theme2:text-green-900

                        theme3:bg-stone-50
                        theme3:border-3
                        theme3:border-gold-rich
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

                            theme3:font-[Orbitron]
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
