use crate::cache::MutationError;
use crate::components::game_detail::DisplayGameQ;
use crate::components::games_list::GamesListQ;
use crate::components::icons::edit::EditIcon;
use crate::components::modal::{Modal, Props as ModalProps};
use crate::components::{Button, Input};
use crate::env::APP_API_HOST;
use crate::http::WithCredentials;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use shared::EditGame;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct EditGameM;

impl MutationCapability for EditGameM {
    type Ok = String;
    type Err = MutationError;
    type Keys = EditGame;

    async fn run(&self, args: &EditGame) -> Result<String, MutationError> {
        let identifier = args.identifier.clone();
        let client = reqwest::Client::new();
        let url: String = format!("{}/api/games/{}", APP_API_HOST, identifier);
        let response = client
            .put(url)
            .with_credentials()
            .json(&args.clone())
            .send()
            .await;
        match response {
            Ok(r) if r.status().is_success() => Ok(identifier),
            _ => Err(MutationError::Unknown),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<DisplayGameQ>::invalidate_all().await;
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        }
    }
}

#[component]
pub fn GameEdit(identifier: String, name: String, icon_class: String, private: bool) -> Element {
    let mut edit_game_signal: Signal<Option<EditGame>> = use_context();
    let title = format!("Edit {name}");

    let onclick = move |_| {
        let name = name.clone();
        let private = private;
        edit_game_signal.set(Some(EditGame {
            identifier: identifier.clone(),
            name: name.clone(),
            private,
        }));
    };

    rsx! {
        Button {
            class: "border-none",
            title,
            onclick,
            EditIcon { class: icon_class }
            label {
                class: "sr-only",
                "edit"
            }
        }
    }
}

#[component]
pub fn EditGameModal() -> Element {
    let edit_game_signal: Signal<Option<EditGame>> = use_context();

    let props = ModalProps {
        title: "Edit Game".to_string(),
        open: edit_game_signal.read().clone().is_some(),
        children: Some(rsx! {
            div {
                class: "flex items-center gap-8 min-h-full justify-center",
                EditGameForm {}
            }
        }),
    };

    rsx! {
        Modal {
            modal_props: props
        }
    }
}

#[component]
pub fn EditGameForm() -> Element {
    let mut edit_game_signal: Signal<Option<EditGame>> = use_context();
    let game_details = edit_game_signal.read().clone().unwrap_or_default();
    let name = game_details.name.clone();
    let identifier = game_details.identifier.clone();
    let private = game_details.private;

    let mutate = use_mutation(Mutation::new(EditGameM));

    let dismiss = move |_| {
        edit_game_signal.set(None);
    };

    let save = move |e: Event<FormData>| {
        let identifier = identifier.clone();

        let name = match e.data().get_first("name") {
            Some(FormValue::Text(s)) => s,
            _ => return,
        };
        let private = match e.data().get_first("private") {
            Some(FormValue::Text(s)) => s == "on",
            _ => false,
        };

        if !name.is_empty() {
            let edit_game = EditGame {
                identifier: identifier.clone(),
                name: name.clone(),
                private,
            };
            spawn(async move {
                let reader = mutate.mutate_async(edit_game.clone()).await;
                let state = reader.state();
                if matches!(&*state, MutationStateData::Settled { res: Ok(_), .. }) {
                    edit_game_signal.set(None);
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

            "#,
            onsubmit: save,
            label {
                "Name",

                Input {
                    class: "border ml-2 px-2 py-1",
                    r#type: "text",
                    name: "name",
                    value: name,
                }
            }
            fieldset {
                class: "grid grid-cols-2 gap-4",
                legend {
                    "Allow spectators?"
                }
                label {
                    "No"

                    input {
                        class: "border ml-2 px-2 py-1",
                        r#type: "radio",
                        name: "private",
                        checked: private,
                        value: "on",
                    }
                }
                label {
                    "Yes",

                    input {
                        class: "border ml-2 px-2 py-1",
                        r#type: "radio",
                        name: "private",
                        checked: !private,
                        value: "off",
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
                    onclick: dismiss,
                    "Cancel"
                }
            }
        }
    }
}
