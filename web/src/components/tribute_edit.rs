use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::icons::edit::EditIcon;
use crate::components::Button;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use shared::EditTribute;
use std::ops::Deref;
use crate::storage::{use_persistent, AppState};

async fn edit_tribute(args: (EditTribute, String, String)) -> MutationResult<MutationValue, MutationError> {
    let tribute = args.clone().0;
    let identifier = args.clone().0.0;
    let game_identifier = args.clone().1;
    let token = args.clone().2;

    let client = reqwest::Client::new();
    let url: String = format!(
        "{}/api/games/{}/tributes/{}",
        API_HOST,
        game_identifier,
        identifier
    );

    let response = client
        .put(url)
        .bearer_auth(token)
        .json(&tribute.clone())
        .send().await;

    if response
        .expect("Failed to update tribute")
        .status()
        .is_success()
    {
        MutationResult::Ok(MutationValue::TributeUpdated(identifier))
    } else {
        MutationResult::Err(MutationError::Unknown)
    }
}

#[component]
pub fn TributeEdit(identifier: String, district: u32, name: String) -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    let onclick = move |_| {
        edit_tribute_signal.set(Some(EditTribute(
            identifier.clone(),
            district,
            name.clone(),
        )));
    };

    rsx! {
        Button {
            class: r#"
            border-none
            "#,
            onclick,
            EditIcon {
                class: r#"
                size-4
                theme1:fill-amber-500
                theme1:hover:fill-amber-200

                theme2:fill-green-200/50
                theme2:hover:fill-green-200

                theme3:fill-amber-600/50
                theme3:hover:fill-amber-600
                "#,
            }
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

            div { class: "fixed inset-0 backdrop-blur-sm backdrop-grayscale" }
            div {
                class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                div {
                    class: "flex items-center gap-8 min-h-full justify-center",
                    EditTributeForm {}
                }
            }
        }
    }
}

#[component]
pub fn EditTributeForm() -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);

    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();
    let tribute_details = edit_tribute_signal.read().clone().unwrap_or_default();
    let name = tribute_details.2.clone();
    let district = tribute_details.1;

    let game: Signal<Option<Game>> = use_context();
    if game.peek().is_none() {
        return rsx! {};
    }
    let game = game.unwrap();
    let game_identifier = game.identifier.clone();

    let mutate = use_mutation(edit_tribute);

    let dismiss = move |_| {
        edit_tribute_signal.set(None);
    };

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let save = move |e: Event<FormData>| {
        let token = storage.get().jwt.expect("No JWT found");
        let game_identifier = game_identifier.clone();
        let tribute_details = edit_tribute_signal
            .read()
            .clone()
            .expect("No details provided");
        let identifier = tribute_details.0.clone();

        let data = e.data().values();
        let name = data.get("name").expect("No name value").0[0].clone();
        let district: u32 = data.get("district").expect("No district value").0[0]
            .clone()
            .parse()
            .unwrap();

        if !name.is_empty() && (1..=12u32).contains(&district) {
            let edit_tribute = EditTribute(identifier.clone(), district, name.clone());
            spawn(async move {
                mutate
                    .manual_mutate((edit_tribute.clone(), game_identifier.clone(), token))
                    .await;
                edit_tribute_signal.set(Some(edit_tribute));

                if let MutationResult::Ok(MutationValue::TributeUpdated(_identifier)) =
                    mutate.result().deref()
                {
                    client.invalidate_queries(&[QueryKey::Tributes(game_identifier.clone())]);
                    edit_tribute_signal.set(None);
                }
            });
        }
    };

    rsx! {
        form {
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
            theme3:border-gold-rich
            theme3:border-3
            "#,
            onsubmit: save,

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

                "Edit tribute"
            }
            div {
                label {
                    "Name",

                    input {
                        class: "border ml-2 px-2 py-1",
                        r#type: "text",
                        name: "name",
                        value: name,
                    }
                }
                label {
                    class: "block mt-2",
                    "District",

                    select {
                        class: "border ml-2 px-2 py-1",
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
                div {
                    class: "flex justify-end gap-2",
                    Button {
                        r#type: "submit",
                        "Update"
                    }
                    Button {
                        r#type: "dialog",
                        onclick: dismiss,
                        "Cancel"
                    }
                }
            }
        }
    }
}
