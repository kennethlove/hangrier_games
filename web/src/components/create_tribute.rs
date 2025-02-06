use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::{Game, GAME};
use reqwest::{Error, Response};
use shared::CreateGame;
use std::ops::Deref;
use game::tributes::Tribute;

async fn create_tribute(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let game_name = GAME.with_borrow(|g| { g.name.clone() });

    let client = reqwest::Client::new();
    let json_body = match name {
        Some(name) => Tribute::new(name, None, None),
        None => Tribute::random()
    };

    let response = client.post(format!("{}/api/games/{}/tributes", API_HOST.clone(), game_name))
        .json(&json_body)
        .send().await;

    match response {
        Ok(response) => {
            match response.json::<Tribute>().await {
                Ok(tribute) => {
                    MutationResult::Ok(MutationValue::NewTribute(tribute))
                }
                Err(_) => {
                    MutationResult::Err(MutationError::UnableToCreateTribute)
                }
            }
        }
        Err(e) => {
            dioxus_logger::tracing::error!("error creating tribute: {:?}", e);
            MutationResult::Err(MutationError::UnableToCreateTribute)
        }
    }
}

#[component]
pub fn CreateTributeButton(game_name: String) -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mutate = use_mutation(create_tribute);

    let onclick = move |_| {
        let game_name = game_name.clone();
        spawn(async move {
            mutate.manual_mutate(None).await;
            if mutate.result().is_ok() {
                if let MutationResult::Ok(MutationValue::NewTribute(tribute)) = mutate.result().deref() {
                    client.invalidate_queries(&[QueryKey::Tributes(game_name)]);
                }
            } else {}
        });
    };

    rsx! {
        button {
            r#type: "button",
            onclick,
            label { "random tribute" }
        }
    }
}

#[component]
pub fn CreateTributeForm(game_name: String) -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let mutate = use_mutation(create_tribute);
    let mut tribute_name_signal = use_signal(|| String::new());

    let onsubmit = move |_| {
        let game_name = game_name.clone();
        spawn(async move {
            let name = tribute_name_signal.read().to_string();
            mutate.manual_mutate(Some(name)).await;

            if mutate.result().is_ok() {
                match mutate.result().deref() {
                    MutationResult::Ok(MutationValue::NewTribute(tribute)) => {
                        client.invalidate_queries(&[QueryKey::Tributes(game_name)]);
                        tribute_name_signal.set(String::default());
                    },
                    MutationResult::Err(MutationError::UnableToCreateTribute) => {},
                    _ => {}
                }
            } else {}
        });
    };

    rsx! {
        form {
            onsubmit,
            input {
                r#type: "text",
                placeholder: "Game name",
                value: tribute_name_signal.read().clone(),
                oninput: move |e| {
                    tribute_name_signal.set(e.value().clone());
                }
            }
            button {
                r#type: "submit",
                label { "create tribute" }
            }
        }
    }
}
