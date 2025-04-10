use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use game::messages::GameMessage;

async fn fetch_game_day_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::GameDayLog(identifier, day)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/log/{}", API_HOST, identifier, day))
            .await
            .unwrap();

        match response.json::<Vec<GameMessage>>().await {
            Ok(logs) => {
                QueryResult::Ok(QueryValue::Logs(logs))
            }
            Err(_err) => {
                QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
            },
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameDayLog(day: u32) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let log_query = use_get_query(
        [
            QueryKey::GameDayLog(identifier.clone(), day),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        fetch_game_day_log,
    );

    match log_query.result().value() {
        QueryResult::Ok(QueryValue::Logs(logs)) => {
            rsx! {
                ul {
                    class: r#"
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-stone-800
                    "#,
                    for log in logs {
                        li {
                            "{log.content}"
                        }
                    }
                }
            }
        }
        QueryResult::Err(_) => {
            rsx! { p { class: "theme1:text-green-200 theme2:text-green-200", "Failed to load." } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { class: "theme1:text-green-200 theme2:text-green-200", "Loading..." } }
        }
        _ => { rsx! {} }
    }
}
