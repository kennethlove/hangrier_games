use dioxus::html::q::dangerous_inner_html;
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
    Game(String),
    Games,
}

#[derive(PartialEq, Debug)]
enum QueryError {
    GameNotFound(String),
    NoGames,
    Unknown
}

#[derive(PartialEq, Debug)]
enum QueryValue {
    Games(Vec<Game>),
    Game(Game),
}

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
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

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(name)) = keys.first() {
        let body = reqwest::get(format!("http://127.0.0.1:3000/api/games/{}", name))
            .await.unwrap();

        match body.json::<Game>().await {
            Ok(game) => {
                QueryResult::Ok(QueryValue::Game(game))
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(name.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

fn main() {
    launch(App);
}

fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();
    let copyright = "&copy; 2025";
    rsx! {
        h1 { "Hangry Games" }
        GamesList {}
        GameDetail { name: "dotingly-distasteful-sport".to_string() }
        GameDetail { name: "unchangingly-senseless-object".to_string() }
        button {
            onclick: move |_| {
                client.invalidate_query(QueryKey::Games)
            },
            label { "Refresh" }
        }
        p {
            dangerous_inner_html: "{copyright}",
        }
    }
}

#[component]
fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);
    let games = games_query.result();
    let games = games.value();
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

#[component]
fn GameDetail(name: String) -> Element {
    let game_query = use_get_query([QueryKey::Game(name), QueryKey::Games], fetch_game);
    let game = game_query.result();
    let game = game.value();
    match game {
        QueryResult::Ok(QueryValue::Game(game_result)) => {
            rsx! {
                h1 { "{game_result.name}" }
                h2 { "{game_result.status}" }
            }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! { p { "Game not found" } }
        }
    }
}


