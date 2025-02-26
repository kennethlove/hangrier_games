use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_mutation, use_query_client, MutationResult, QueryResult};
use game::games::GameStatus;
use game::games::{Game, GAME};
use std::ops::Deref;
use reqwest::StatusCode;

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

async fn next_step(identifier: String) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/next", API_HOST.clone(), identifier);

    let response = client
        .put(url)
        .send().await.expect("Failed to advance game");

    match response.status() {
        StatusCode::NO_CONTENT => {
            MutationResult::Ok(MutationValue::GameFinished(identifier))
        }
        StatusCode::CREATED => {
            MutationResult::Ok(MutationValue::GameStarted(identifier))
        }
        StatusCode::OK => {
            MutationResult::Ok(MutationValue::GameAdvanced(identifier))
        }
        _ => {
            MutationResult::Err(MutationError::UnableToAdvanceGame)
        }
    }
}

#[component]
fn GameStatusState() -> Element {
    let mut game_signal: Signal<Option<Game>> = use_context();
    let game = game_signal.read();
    
    if let Some(game) = game.clone() {
        let game_next_step: String;

        let game_status = match game.status {
            GameStatus::NotStarted => {
                if game.ready {
                    game_next_step = "Start".to_string();
                } else {
                    game_next_step = "Wait".to_string();
                }
                "Not started".to_string()
            }
            GameStatus::InProgress => {
                game_next_step = "Play next step".to_string();
                "In progress".to_string()
            }
            GameStatus::Finished => {
                game_next_step = "Clone".to_string();
                "Finished".to_string()
            }
        };

        let mutate = use_mutation(next_step);
        let game_id = game.identifier.clone();
        let game_day = game.day.unwrap_or(0);

        let next_step = move |_| {
            let game_id = game_id.clone();
            let mut game = game.clone();
            
            let client = use_query_client::<QueryValue, QueryError, QueryKey>();

            spawn(async move {
                mutate.manual_mutate(game_id.clone()).await;

                match mutate.result().deref() {
                    MutationResult::Ok(mutation_result) => {
                        match mutation_result {
                            MutationValue::GameAdvanced(game_identifier) => {
                                client.invalidate_queries(&[QueryKey::Game(game_identifier.into())]);
                            }
                            MutationValue::GameFinished(_) => {
                                game.end();
                            }
                            MutationValue::GameStarted(game_identifier) => {
                                game.start();
                                client.invalidate_queries(&[QueryKey::Game(game_identifier.into())]);
                            }
                            _ => {}
                        }
                    }
                    MutationResult::Err(MutationError::UnableToAdvanceGame) => {
                        dioxus_logger::tracing::error!("Failed to advance game");
                    }
                    _ => {}
                }
            });
        };

        rsx! {
            h2 {
                class: "game-status",
                "Game Status: {game_status}"
                button {
                    class: "button",
                    onclick: next_step,
                    "{game_next_step}"
                }
            }
            h3 {
                "Game round: Day { game_day }, Night { game_day }"
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn GamePage(identifier: String) -> Element {
    rsx! {
        GameStatusState {}
        GameDetailPage { identifier }
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

            h3 { "Areas" }

            GameAreaList { }

            h3 { "Tributes" }

            GameTributes { }

        }
    }
}
