use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::components::tribute_edit::EditTributeModal;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::areas::AreaDetails;
use game::games::GameStatus;
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;
use std::collections::HashMap;

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}", API_HOST.clone(), identifier))
            .await
            .unwrap();

        match response.json::<Game>().await {
            Ok(game) => {
                GAME.set(game.clone());
                QueryResult::Ok(QueryValue::Game(game))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
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
    let game_ready = game.ready;

    let game_status = match game.status {
        GameStatus::NotStarted => {
            if game_ready {
                game_next_step = "Start".to_string();
            } else {
                game_next_step = "Wait".to_string();
            }
            "Not started".to_string()
        }
        GameStatus::InProgress => {
            game_next_step = "Finish".to_string();
            "In progress".to_string()
        }
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

#[component]
pub fn GameDetailPage(identifier: String) -> Element {
    let game_query = use_get_query(
        [QueryKey::Game(identifier.clone()), QueryKey::Games],
        fetch_game,
    );
    let mut game_signal: Signal<Option<Game>> = use_context();

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            game_signal.set(Some(game.clone()));
            rsx! {
                GameDetails { game: game.clone() }
            }
        }
        QueryResult::Err(e) => {
            dioxus_logger::tracing::error!("{:?}", e);
            rsx! { "Failed to load" }
        }
        _ => {
            rsx! {
                p { "Loading..." }
            }
        }
    }
}

#[component]
pub fn GameDetails(game: Game) -> Element {
    rsx! {
        div {
            h1 {
                "{game.name}",
                GameEdit { identifier: game.identifier.clone(), name: game.name.clone() }
            }

            GameStatusState { }

            h3 { "Areas" }

            GameAreaList { }

            h3 { "Tributes" }

            GameTributes { }

        }
    }
}
