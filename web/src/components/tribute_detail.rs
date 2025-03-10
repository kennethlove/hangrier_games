use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_edit::TributeEdit;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::tributes::Tribute;

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
                    QueryResult::Ok(QueryValue::Tribute(Box::new(tribute)))
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
pub fn TributeDetail(game_identifier: String, tribute_identifier: String) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let game_identifier = game.identifier.clone();

    let tribute_query = use_get_query(
        [
            QueryKey::Tribute(tribute_identifier.clone()),
            QueryKey::Tributes(game_identifier.clone()),
            QueryKey::Game(game_identifier.clone()),
        ],
        fetch_tribute,
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::Tribute(tribute)) => {
            rsx! {
                h1 { "{tribute.name}"
                    TributeEdit {
                        identifier: tribute.clone().identifier,
                        district: tribute.district,
                        name: tribute.clone().name,
                    }
                }
                h2 { "{tribute.district}" }
                h3 { "Location: {tribute.area}" }
                h3 { "Items" }
                ul {
                    for item in tribute.clone().items {
                        li {
                            onclick: move |_| {},
                            "{item.name}" }
                    }
                }

                h3 { "Log" }
                dl {
                    for log in tribute.clone().log {
                        dt { "Day {log.day}"}
                        dd {"{log.message}" }
                    }
                }
            }
        }
        QueryResult::Err(QueryError::TributeNotFound(identifier)) => {
            rsx! { p { "{identifier} not found." } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! { } }
    }
}
