use dioxus::html::q::dangerous_inner_html;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::{Game, GameStatus};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
enum QueryKey {
    AllGames,
    CreateGame(Option<String>),
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

#[derive(PartialEq, Debug)]
enum MutationValue {
    NewGame(Game),
    GameDeleted(String),
}

#[derive(PartialEq, Debug)]
enum MutationError {
    UnableToCreateGame,
    Unknown,
}

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        let response = reqwest::get("http://127.0.0.1:3000/api/games")
            .await.unwrap()
            .json::<Vec<Game>>()
            .await.unwrap();
        QueryResult::Ok(QueryValue::Games(response))
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(name)) = keys.first() {
        let response = reqwest::get(format!("http://127.0.0.1:3000/api/games/{}", name))
            .await.unwrap();

        match response.json::<Game>().await {
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

#[derive(Debug, Serialize)]
struct CreateGame {
    name: Option<String>,
}

async fn create_game(name: Option<String>) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = CreateGame { name: name.clone() };
    let response = client.post("http://127.0.0.1:3000/api/games")
        .json(&json_body)
        .send().await.unwrap();

    match response.json::<Game>().await {
        Ok(game) => {
            let client = use_query_client::<QueryValue, QueryError, QueryKey>();
            client.invalidate_queries(&[QueryKey::Games]);

            MutationResult::Ok(MutationValue::NewGame(game))
        }
        Err(e) => {
            MutationResult::Err(MutationError::UnableToCreateGame)
        }
    }
}

async fn delete_game(name: String) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("http://127.0.0.1:3000/api/games/{}", name))
        .send().await.unwrap();

    if response.status().is_server_error() {
        MutationResult::Err(MutationError::Unknown)
    } else {
        let client = use_query_client::<QueryValue, QueryError, QueryKey>();
        client.invalidate_queries(&[QueryKey::Games]);
        MutationResult::Ok(MutationValue::GameDeleted(name))
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
        CreateGameButton {}
        CreateGameForm {}
        GamesList {}

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
fn CreateGameButton() -> Element {
    let mutate = use_mutation(create_game);

    let onclick = move |_| {
        mutate.mutate(None);
    };

    rsx! {
        p { "{*mutate.result():?}" }
        button {
            onclick,
            label { "quickstart" }
        }
    }
}

#[component]
fn CreateGameForm() -> Element {
    let mut game_name_signal: Signal<String> = use_signal(|| String::default());
    let mutate = use_mutation(create_game);

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() { return }

        let game_query = mutate.mutate(Some(name));

        let client = use_query_client::<QueryValue, QueryError, QueryKey>();
        client.invalidate_queries(&[QueryKey::Games]);
        game_name_signal.set("".to_string());
    };

    rsx! {
        form {
            onsubmit,
            input {
                r#type: "text",
                placeholder: "Game name",
                value: game_name_signal.read().clone(),
                oninput: move |e| {
                    game_name_signal.set(e.value().clone());
                }
            }
            button {
                r#type: "submit",
                label { "create game" }
            }
        }
    }
}

#[component]
fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);
    match games_query.result().value() {
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
                            ul {
                                for game in games {
                                    GameListMember { game: game.clone() }
                                }
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
fn GameListMember(game: Game) -> Element {
    let mutate = use_mutation(delete_game);
    let name = game.name.clone();

    let onclick = move |_| {
        mutate.mutate(name.clone());
    };

    rsx! {
        li {
            "{game.name} ",
            button {
                onclick,
                "x"
            }
        }
    }
}

#[component]
fn GameDetail(name: String) -> Element {
    let game_query = use_get_query([QueryKey::Game(name), QueryKey::Games], fetch_game);
    match game_query.result().value() {
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


