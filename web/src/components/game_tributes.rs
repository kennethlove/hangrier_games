use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_edit::TributeEdit;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::TributeKey;

async fn fetch_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/tributes", API_HOST.clone(), identifier))
            .await
            .unwrap();

        match response.json::<Vec<TributeKey>>().await {
            Ok(tributes) => {
                QueryResult::Ok(QueryValue::GameTributes(tributes))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributes() -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let tribute_query = use_get_query(
        [
            QueryKey::Tributes(identifier.clone()),
            QueryKey::Game(identifier.clone())
        ],
        fetch_tributes,
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::GameTributes(tributes)) => {
            rsx! {
                ul {
                    for tribute in tributes {
                        GameTributeListMember {
                            identifier: tribute.clone()
                        }
                    }
                }
            }
        }
        QueryResult::Err(e) => {
            dioxus_logger::tracing::error!("{:?}", e);
            rsx! { p { "Something went wrong" } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}

async fn fetch_tribute(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tribute(identifier)) = keys.first() {
        if let Some(QueryKey::Game(game_identifier)) = keys.last() {
            let response = reqwest::get(
                format!(
                    "{}/api/games/{}/tributes/{}",
                    API_HOST.clone(),
                    game_identifier,
                    identifier
                ))
                .await
                .unwrap();

            match response.json::<Option<Tribute>>().await {
                Ok(Some(tribute)) => {
                    QueryResult::Ok(QueryValue::Tribute(tribute))
                }
                _ => QueryResult::Err(QueryError::TributeNotFound(identifier.to_string()))
            }
        } else {
            QueryResult::Err(QueryError::Unknown)
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributeListMember(identifier: String) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let game_identifier = game.identifier.clone();

    let tribute_query = use_get_query(
        [
            QueryKey::Tribute(identifier.clone()),
            QueryKey::Tributes(game_identifier.clone()),
            QueryKey::Game(game_identifier.clone()),
        ],
        fetch_tribute,
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::Tribute(tribute)) => {
            rsx! {
                li {
                    "{tribute.name} - {tribute.district}",
                    TributeEdit {
                        identifier: tribute.clone().identifier,
                        district: tribute.district,
                        name: tribute.clone().name,
                    }
                    ul {
                        for item in tribute.clone().items {
                            li { "{item.name}" }
                        }
                    }
                }
            }
        },
        QueryResult::Err(QueryError::TributeNotFound(identifier)) => {
            rsx! { p { "{identifier} not found." } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! { } }
    }
}
