use crate::cache::MutationError;
use crate::components::game_tributes::GameTributesQ;
use crate::components::icons::edit::EditIcon;
use crate::components::modal::{Modal, Props as ModalProps};
use crate::components::tribute_detail::TributeQ;
use crate::components::{Button, Input};
use crate::env::APP_API_HOST;
use crate::storage::{AppState, use_persistent};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use shared::EditTribute;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct EditTributeM;

impl MutationCapability for EditTributeM {
    type Ok = String;
    type Err = MutationError;
    type Keys = (EditTribute, String, String);

    async fn run(&self, args: &(EditTribute, String, String)) -> Result<String, MutationError> {
        let tribute = args.0.clone();
        let identifier = tribute.identifier.clone();
        let game_identifier = args.1.clone();
        let token = args.2.clone();
        let client = reqwest::Client::new();
        let url: String = format!(
            "{}/api/games/{}/tributes/{}",
            APP_API_HOST, game_identifier, identifier
        );
        let response = client
            .put(url)
            .bearer_auth(token)
            .json(&tribute)
            .send()
            .await;
        match response {
            Ok(r) if r.status().is_success() => Ok(identifier),
            _ => Err(MutationError::Unknown),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<TributeQ>::invalidate_all().await;
            QueriesStorage::<GameTributesQ>::invalidate_all().await;
        }
    }
}

#[component]
pub fn TributeEdit(
    identifier: String,
    name: String,
    avatar: String,
    game_identifier: String,
) -> Element {
    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();

    let onclick = move |_| {
        edit_tribute_signal.set(Some(EditTribute {
            identifier: identifier.clone(),
            name: name.clone(),
            avatar: avatar.clone(),
            game_identifier: game_identifier.clone(),
        }));
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

    let props = ModalProps {
        title: "Edit Tribute".to_string(),
        open: edit_tribute_signal.read().clone().is_some(),
        children: Some(rsx! {
            div {
                class: "flex items-center gap-8 min-h-full justify-center",
                EditTributeForm {}
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
pub fn EditTributeForm() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);

    let mut edit_tribute_signal: Signal<Option<EditTribute>> = use_context();
    let tribute_details = edit_tribute_signal.read().clone().unwrap_or_default();
    let name = tribute_details.name.clone();
    let avatar = tribute_details.avatar.clone();
    let game_identifier = tribute_details.game_identifier.clone();
    let identifier = tribute_details.identifier.clone();

    let mut avatar_preview = use_signal(|| avatar.clone());
    let mut upload_status = use_signal(String::new);

    let mutate = use_mutation(Mutation::new(EditTributeM));

    let dismiss = move |_| {
        edit_tribute_signal.set(None);
    };

    let game_identifier_for_upload = game_identifier.clone();

    let save = move |e: Event<FormData>| {
        let token = storage.get().jwt.unwrap_or_default();
        let game_identifier = game_identifier.clone();
        let tribute_details = edit_tribute_signal
            .read()
            .clone()
            .expect("No details provided");
        let identifier = tribute_details.identifier.clone();
        let current_avatar = avatar_preview.read().clone();

        let name = match e.data().get_first("name") {
            Some(FormValue::Text(s)) => s,
            _ => return,
        };

        if !name.is_empty() {
            let edit_tribute = EditTribute {
                identifier: identifier.clone(),
                name: name.clone(),
                avatar: current_avatar,
                game_identifier: game_identifier.clone(),
            };
            spawn(async move {
                let reader = mutate
                    .mutate_async((edit_tribute.clone(), game_identifier.clone(), token))
                    .await;
                let state = reader.state();
                if matches!(&*state, MutationStateData::Settled { res: Ok(_), .. }) {
                    edit_tribute_signal.set(None);
                }
            });
        }
    };

    let upload_avatar = move |e: Event<FormData>| {
        let token = storage.get().jwt.unwrap_or_default();
        let game_id = game_identifier_for_upload.clone();
        let tribute_id = identifier.clone();

        upload_status.set("Uploading...".to_string());

        spawn(async move {
            let files = e.files();

            if let Some(file) = files.into_iter().next() {
                let file_name = file.name();
                let Ok(bytes) = file.read_bytes().await else {
                    upload_status.set("Upload error: failed to read file".to_string());
                    return;
                };
                let file_data = bytes.to_vec();

                let client = reqwest::Client::new();
                let url = format!(
                    "{}/api/games/{}/tributes/{}/avatar",
                    APP_API_HOST, game_id, tribute_id
                );

                let part = reqwest::multipart::Part::bytes(file_data).file_name(file_name.clone());

                let form = reqwest::multipart::Form::new().part("avatar", part);

                let response = client
                    .post(&url)
                    .bearer_auth(token)
                    .multipart(form)
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await
                            && let Some(url) = json.get("url").and_then(|v| v.as_str())
                        {
                            avatar_preview.set(url.to_string());
                            upload_status.set("Upload successful!".to_string());
                        }
                    }
                    Ok(resp) => {
                        upload_status.set(format!("Upload failed: {}", resp.status()));
                    }
                    Err(e) => {
                        upload_status.set(format!("Upload error: {}", e));
                    }
                }
            }
        });
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
            "#,
            onsubmit: save,

            div {
                label {
                    "Name",

                    Input {
                        class: "border ml-2 px-2 py-1",
                        r#type: "text",
                        name: "name",
                        value: name,
                    }
                }
            }

            div {
                label {
                    class: "block mb-2",
                    "Avatar"
                }

                // Show current avatar if exists
                if !avatar_preview.read().is_empty() {
                    img {
                        class: "w-32 h-32 object-cover rounded mb-2",
                        src: "{avatar_preview}",
                        alt: "Tribute avatar"
                    }
                }

                // File upload form
                form {
                    class: "mb-2",
                    onchange: upload_avatar,
                    onsubmit: |e| e.prevent_default(),

                    input {
                        r#type: "file",
                        name: "avatar",
                        accept: "image/jpeg,image/png,image/webp",
                        class: "border px-2 py-1"
                    }
                }

                if !upload_status.read().is_empty() {
                    p {
                        class: "text-sm italic",
                        "{upload_status}"
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
