use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_mutation, use_query_client, MutationResult, QueryResult};
use game::games::Game;
use game::games::GameStatus;
use game::tributes::Tribute;
use reqwest::StatusCode;
use std::ops::Deref;

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}", API_HOST.clone(), identifier))
            .await
            .expect("Failed to fetch game details");

        match response.json::<Game>().await {
            Ok(game) => {
                // GAME.set(game.clone());
                QueryResult::Ok(QueryValue::Game(Box::new(game)))
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

    dioxus_logger::tracing::info!("{:?}", &response);

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
    let game_signal: Signal<Option<Game>> = use_context();
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
        let game_finished = game.status == GameStatus::Finished;
        let tribute_count = game.clone()
            .tributes.into_iter()
            .filter(|t| t.is_alive())
            .collect::<Vec<Tribute>>()
            .len();
        let winner_name = {
            if game.winner().is_some() { game.winner().unwrap().name } else { String::new() }
        };

        let next_step_handler = move |_| {
            let game_id = game_id.clone();
            let mut game = game.clone();

            let client = use_query_client::<QueryValue, QueryError, QueryKey>();

            spawn(async move {
                mutate.manual_mutate(game_id.clone()).await;

                dioxus_logger::tracing::info!("{}", "here");

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
                    onclick: next_step_handler,
                    disabled: game_finished,
                    "{game_next_step}"
                }
            }
            h3 {
                "Game round: Day { game_day }, Night { game_day }"
            }
            h4 { "{tribute_count} tributes remain alive."}
            if !winner_name.is_empty() {
                h1 { "{winner_name} wins!"}
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
            game_signal.set(Some(*game.clone()));
            rsx! {
                GameDetails { game: *game.clone() }
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

            h3 { "Output" }

            for log in game.log {
                pre { "{log.message}" }
            }

            h3 { "Areas" }

            GameAreaList { }

            h3 { "Tributes" }

            GameTributes { }

        }
    }
}
