use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_tributes::GameTributes;
use crate::components::tribute_edit::EditTributeModal;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::GameStatus;
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;
use std::collections::HashMap;
use crate::components::game_edit::GameEdit;

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}", API_HOST.clone(), identifier))
            .await.unwrap();

        match response.json::<Game>().await {
            Ok(game) => {
                GAME.set(game.clone());
                QueryResult::Ok(QueryValue::Game(game))
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
fn GameStatusState() -> Element {
    let game: Signal<Option<Game>> = use_context();
    let game = game.read().clone().unwrap();

    let game_next_step: String;

    let game_ready = game_is_ready(&game.tributes);

    let game_status= match game.status {
        GameStatus::NotStarted => {
            if game_ready {
                game_next_step = "Start".to_string();
            } else {
                game_next_step = "Wait".to_string();
            }
            "Not started".to_string()
        },
        GameStatus::InProgress => {
            game_next_step = "Finish".to_string();
            "In progress".to_string()
        },
        GameStatus::Finished => {
            game_next_step = "Clone".to_string();
            "Finished".to_string()
        }
    };

    rsx! {
        h2 {
            class: "game-status",
            "Game Status: {game_status}"
            button {
                class: "button",
                onclick: move |_| {
                },
                "{game_next_step}"
            }
        }
    }
}

fn game_is_ready(tributes: &Vec<Tribute>) -> bool {
    if tributes.is_empty() { return false; }
    
    let mut tribute_spread: HashMap<u32, u32> = HashMap::new();
    for tribute in tributes {
        if tribute_spread.contains_key(&tribute.district) {
            let count = tribute_spread.get(&tribute.district).unwrap();
            tribute_spread.insert(tribute.district, count + 1);
        } else {
            tribute_spread.insert(tribute.district, 1);
        }
    }

    let mut valid = true;

    for count in tribute_spread.values() {
        if *count != 2 { valid = false; }
    }

    valid
}

#[component]
pub fn GameDetailPage(identifier: String) -> Element {
    let game_query = use_get_query([QueryKey::Game(identifier.clone()), QueryKey::Games], fetch_game);
    let mut game_signal: Signal<Option<Game>> = use_context();

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            game_signal.set(Some(game.clone()));
            let detail = Gdp { game: game.clone() };
            rsx! {
                GameDetails { gdp: detail }
            }
        }
        QueryResult::Err(e) => {
            dioxus_logger::tracing::error!("{:?}", e);
            rsx! { "Failed to load" }
        }
        _ => {
            rsx! {
                p { "Loading...outer" }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Props)]
pub struct Gdp {
    game: Game,
}

#[component]
pub fn GameDetails(gdp: Gdp) -> Element {
    let game = gdp.game;

    rsx! {
        div {
            h1 {
                "{game.name}",
                GameEdit { identifier: game.identifier.clone(), name: game.name.clone() }
            }

            GameStatusState {}

            h3 { "Tributes" }

            GameTributes { }

            h3 { "Items" }
            ul {
                for item in game.items.iter() {
                    li { "{item.name}" }
                }
                // for (area, details) in game.areas.iter() {
                //     li {
                //         "{area}: {details.open}"
                //         ul {
                //             for item in &details.items {
                //                 li {
                //                     "{item.name}",
                //                 }
                //             }
                //         }
                //     }
                // }
            }
        }
    }
}

#[component]
fn RefreshButton(game_identifier: String) -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let onclick = move |_| {
        client.invalidate_queries(&[QueryKey::Tributes(game_identifier.clone())]);
    };

    rsx! {
        button {
            r#type: "button",
            onclick: onclick,
            "Refresh"
        }
    }
}
