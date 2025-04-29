use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::env::APP_API_HOST as API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use game::messages::GameMessage;
use crate::storage::{use_persistent, AppState};

async fn fetch_game_day_log(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::GameDayLog(identifier, day)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}/log/{}", &*API_HOST, identifier, day))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) => {
                if let Ok(logs) = response.json::<Vec<GameMessage>>().await {
                    QueryResult::Ok(QueryValue::Logs(logs))
                } else {
                    QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
                }
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameDayLog(game: Game, day: u32) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let identifier = game.identifier.clone();

    let log_query = use_get_query(
        [
            QueryKey::GameDayLog(identifier.clone(), day),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        move |keys: Vec<QueryKey>| { fetch_game_day_log(keys, token.clone()) },
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
