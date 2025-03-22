use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use game::globals::LogMessage;
use shared::LogEntry;
use crate::API_HOST;
use crate::cache::{QueryError, QueryKey, QueryValue};

async fn fetch_full_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Log(identifier, day)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/log/{}", API_HOST.clone(), identifier, day))
            .await
            .unwrap();

        match response.json::<Vec<LogMessage>>().await {
            Ok(logs) => {
                QueryResult::Ok(QueryValue::Logs(logs))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn FullGameLog(day: u32) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let log_query = use_get_query(
        [
            QueryKey::Log(identifier.clone(), day),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        fetch_full_log,
    );

    match log_query.result().value() {
        QueryResult::Ok(QueryValue::Logs(logs)) => {
            rsx! {
                ul {
                    for log in logs {
                        li {
                            "{log.message}"
                        }
                    }
                }
            }
        }
        QueryResult::Err(_) => {
            rsx! { p { "Failed to load." } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}
