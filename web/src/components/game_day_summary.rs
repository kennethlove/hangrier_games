use std::time::Duration;

use crate::API_HOST;
use crate::cache::{QueryError, QueryKey, QueryValue};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use game::messages::GameMessage;

async fn fetch_game_day_summary(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::GameDaySummary(identifier, day)) = keys.first() {
        let response = reqwest::get(format!(
            "{}/api/games/{}/summarize/{}",
            API_HOST.clone(),
            identifier,
            day
        ))
        .await
        .unwrap();

        dioxus_logger::tracing::info!("{:?}", response);

        match response.json::<String>().await {
            Ok(summary) => QueryResult::Ok(QueryValue::Summary(summary)),
            Err(err) => {
                dioxus_logger::tracing::error!("{:?}", err);
                QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameDaySummary(day: u32) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let summary_query = use_get_query(
        [
            QueryKey::GameDaySummary(identifier.clone(), day),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games,
        ],
        fetch_game_day_summary,
    );

    match summary_query.result().value() {
        QueryResult::Ok(QueryValue::Summary(summary)) => {
            rsx! {
                for p in summary.split("\n") {
                    p {
                        "{p}"
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
        _ => {
            rsx! {}
        }
    }
}
