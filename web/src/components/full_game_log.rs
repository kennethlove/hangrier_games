use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use game::messages::GameMessage;

async fn fetch_full_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::FullGameLog(identifier, day)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/log/{}", API_HOST.clone(), identifier, day))
            .await
            .unwrap();

        dioxus_logger::tracing::info!("{:?}", response);

        match response.json::<Vec<GameMessage>>().await {
            Ok(logs) => {
                QueryResult::Ok(QueryValue::Logs(logs))
            }
            Err(err) => {
                dioxus_logger::tracing::error!("{:?}", err);
                QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
            },
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
            QueryKey::FullGameLog(identifier.clone(), day),
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
                            "{log.content}"
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
