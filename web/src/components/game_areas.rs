use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::areas::AreaDetails;
use game::games::Game;

async fn fetch_areas(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Areas(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/areas", API_HOST.clone(), identifier))
            .await
            .unwrap();

        match response.json::<Vec<AreaDetails>>().await {
            Ok(areas) => {
                QueryResult::Ok(QueryValue::Areas(areas))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameAreaList() -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let area_query = use_get_query(
        [
            QueryKey::Areas(identifier.clone()),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        fetch_areas,
    );

    match area_query.result().value() {
        QueryResult::Ok(QueryValue::Areas(areas)) => {
            rsx! {
                ul {
                    for area in areas {
                        li {
                            "{area.name}, open: {area.open}",
                            p { "items" }
                            ul {
                                for item in area.clone().items {
                                    li { "{item.name}" }
                                }
                            }
                            p { "events" }
                            ul {
                                for event in area.clone().events {
                                    li { "{event}" }
                                }
                            }
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

