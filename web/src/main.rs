use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::{Game, GameStatus};
use num_traits::ToPrimitive;
use serde::Deserialize;
use std::env;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
enum QueryKey {
    AllGames,
    Game(usize),
    Games,
}

#[derive(PartialEq, Debug)]
enum QueryError {
    GameNotFound(usize),
    NoGames,
    Unknown
}

#[derive(PartialEq, Debug)]
enum QueryValue {
    Games(Vec<Game>),
    GameName(String),
}

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    dioxus_logger::tracing::info!("Fetching games");
    if let Some(QueryKey::AllGames) = keys.first() {
        let body = reqwest::get("http://127.0.0.1:3000/api/games")
            .await.unwrap()
            .json::<Vec<Game>>()
            .await.unwrap();
        QueryResult::Ok(QueryValue::Games(body))
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

fn main() {
    launch(App);
}

fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();
    rsx! {
        h1 { "Hangry Games" }
        GamesList {}
    }
}

#[component]
fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);
    let games = games_query.result();
    let games = games.value();
    dioxus_logger::tracing::info!("games {:?}", games);
    match games {
        QueryResult::Err(QueryError::NoGames) => {
            rsx! { p { "No games" } }
        }
        QueryResult::Ok(games) => {
            match games {
                QueryValue::Games(games) => {
                    if games.is_empty() {
                        rsx! { p { "No games yet" } }
                    } else {
                        rsx! {
                            for game in games {
                                p { "{game.name}" }
                            }
                        }
                    }
                },
                _ => {
                    rsx! { p { "Wrong result type" } }
                }
            }

        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! { p { "No idea how you got here." } }
        }
    }
}
