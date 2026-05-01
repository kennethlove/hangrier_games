use crate::cache::MutationError;
use crate::components::game_tributes::GameTributesQ;
use crate::env::APP_API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::GAME;
use shared::DeleteTribute;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DeleteTributeM;

impl MutationCapability for DeleteTributeM {
    type Ok = String;
    type Err = MutationError;
    type Keys = String;

    async fn run(&self, name: &String) -> Result<String, MutationError> {
        let game_name = GAME.with_borrow(|g| g.name.clone());
        let client = reqwest::Client::new();
        let url: String = format!(
            "{}/api/games/{}/tributes/{}",
            APP_API_HOST, game_name, name
        );
        let response = client.delete(url).send().await;
        match response {
            Ok(r) if r.status().is_success() => Ok(name.clone()),
            _ => Err(MutationError::Unknown),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<GameTributesQ>::invalidate_all().await;
        }
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
    let tribute_name = delete_tribute_signal.peek().clone();
    let name = tribute_name.clone().unwrap_or_default().clone();
    let mutate = use_mutation(Mutation::new(DeleteTributeM));

    let dismiss = move |_| {
        delete_tribute_signal.set(None);
    };

    let delete = move |_| {
        if let Some(tribute_name) = tribute_name.clone() {
            spawn(async move {
                let reader = mutate.mutate_async(tribute_name.clone()).await;
                let state = reader.state();
                if matches!(&*state, MutationStateData::Settled { res: Ok(_), .. }) {
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
