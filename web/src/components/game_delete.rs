use crate::cache::MutationError;
use crate::components::Button;
use crate::components::games_list::GamesListQ;
use crate::components::icons::delete::DeleteIcon;
use crate::http::WithCredentials;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use gloo_storage::Storage;
use shared::DeleteGame;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DeleteGameM;

impl MutationCapability for DeleteGameM {
    type Ok = (String, String);
    type Err = MutationError;
    type Keys = DeleteGame;

    async fn run(&self, args: &DeleteGame) -> Result<(String, String), MutationError> {
        let identifier = args.0.clone();
        let name = args.1.clone();
        let client = reqwest::Client::new();
        let url: String = crate::api_url::api_url(&format!("/api/games/{}", identifier));
        let response = client.delete(url).with_credentials().send().await;
        match response {
            Ok(r) if r.status().is_success() => Ok((identifier, name)),
            _ => Err(MutationError::Unknown),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        }
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
    let mut delete_game_signal: Signal<Option<DeleteGame>> = use_context();
    let delete_game_info = delete_game_signal.read().clone();
    let mutate = use_mutation(Mutation::new(DeleteGameM));

    let name = {
        if let Some(details) = delete_game_info.clone() {
            details.1
        } else {
            String::new()
        }
    };

    let dismiss = move |_| {
        delete_game_signal.set(None);
    };

    let delete = move |_| {
        if let Some(dg) = delete_game_info.clone() {
            spawn(async move {
                let reader = mutate.mutate_async(dg.clone()).await;
                let state = reader.state();
                if let MutationStateData::Settled {
                    res: Ok((id, _)), ..
                } = &*state
                {
                    gloo_storage::LocalStorage::delete(format!("recap_collapsed:{id}"));
                    gloo_storage::LocalStorage::delete(format!("period_filters:{id}"));
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

                        "#,

                        h1 {
                            class: r#"
                            block
                            p-2
                            text-lg

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
