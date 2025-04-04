use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_mutation, use_query_client, MutationResult, QueryResult};
use game::games::Game;
use game::games::GameStatus;
use game::messages::GameMessage;
use game::tributes::Tribute;
use reqwest::StatusCode;
use std::ops::Deref;
use crate::components::full_game_log::GameDayLog;
use crate::components::game_day_summary::GameDaySummary;

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
        let game_name = game.name.clone();
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
        let g = game.clone();

        let next_step_handler = move |_| {
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
            div {
                class: "flex flex-col gap-2 border p-2",
                div {
                    class: "flex flex-row gap-4 place-content-between",
                    h2 {
                        class: "text-xl",
                        "{game_name}"
                    }
                    div {
                        class: "flex flex-row gap-2",
                        GameEdit { identifier: g.identifier, name: g.name }
                        button {
                            class: "button border px-2 py-1",
                            onclick: next_step_handler,
                            disabled: game_finished,
                            "{game_next_step}"
                        }
                    }
                }

                if !winner_name.is_empty() {
                    h1 { "Winner: {winner_name}!"}
                }

                div {
                    class: "flex flex-row place-content-between",
                    p {
                        span {
                            class: "block text-sm",
                            "status"
                        }
                        "{game_status}"
                    }
                    p {
                        span {
                            class: "block text-sm",
                            "day"
                        }
                        "{game_day}"
                    }
                    p {
                        span {
                            class: "block text-sm",
                            "tributes alive"
                        }
                        "{tribute_count}"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn GamePage(identifier: String) -> Element {
    rsx! {
        div {
            class: "mb-4",
            GameStatusState {}
        }
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
            div {
                class: "grid grid-cols-2 gap-8",

                div {
                    class: "flex flex-col gap-4",
                    div {
                        h3 {
                            class: "text-xl mb-2",
                            "Areas",
                            i {
                                class: "ra ra-aura"
                            }
                        }
                        GameAreaList { }
                    }

                    div {
                        h3 {
                            class: "text-xl mb-2",
                            "Day summary",
                        }
                        GameDaySummary { day: game.day.unwrap_or_default() }
                    }

                    div {
                        h3 {
                            class: "text-xl mb-2",
                            "Day log"
                        }
                        GameDayLog { day: game.day.unwrap_or_default() }
                    }
                }
                div {
                    h3 {
                        class: "text-xl mb-2",
                        "Tributes"
                    }

                    GameTributes { }
                }
            }
        }
    }
}
