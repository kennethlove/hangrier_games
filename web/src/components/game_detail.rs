use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_day_log::GameDayLog;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::components::info_detail::InfoDetail;
use crate::components::{Button, ThemedButton};
use crate::storage::{use_persistent, AppState};
use crate::{LoadingState, API_HOST};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use dioxus_query::prelude::{use_get_query, use_mutation, use_query_client, MutationResult, QueryResult, UseMutation, UseQueryClient};
use game::games::Game;
use game::games::GameStatus;
use game::tributes::Tribute;
use reqwest::StatusCode;
use std::ops::Deref;

async fn fetch_game(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}", &*API_HOST, identifier))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) =>  {
                if let Ok(game) = response.json::<Game>().await {
                    QueryResult::Ok(QueryValue::Game(Box::new(game)))
                } else {
                    QueryResult::Err(QueryError::BadJson)
                }
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn next_step(args: (String, String)) -> MutationResult<MutationValue, MutationError> {
    let identifier = args.0.clone();
    let token = args.1.clone();
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/next", &*API_HOST, identifier);

    let request = client.request(reqwest::Method::PUT, url)
        .bearer_auth(token);

    match request.send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::NO_CONTENT => MutationResult::Ok(MutationValue::GameFinished(identifier)),
                StatusCode::CREATED => MutationResult::Ok(MutationValue::GameStarted(identifier)),
                StatusCode::OK => {
                    tracing::info!("{:?}", &response);
                    MutationResult::Ok(MutationValue::GameAdvanced(identifier))
                },
                _ => MutationResult::Err(MutationError::UnableToAdvanceGame),
            }
        }
        Err(_) => {
            MutationResult::Err(MutationError::UnableToAdvanceGame)
        }
    }
}

async fn handle_next_step(
    game_id: String,
    token: String,
    mutate: UseMutation<MutationValue, MutationError, (String, String)>,
    client: UseQueryClient<QueryValue, QueryError, QueryKey>,
    mut loading_signal: Signal<LoadingState>,
) {
    loading_signal.set(LoadingState::Loading);
    mutate.manual_mutate((game_id.clone(), token)).await;

    let mut invalidate_keys = None;

    match mutate.result().deref() {
        MutationResult::Ok(mutation_result) => {
            match mutation_result {
                MutationValue::GameAdvanced(game_identifier)
                | MutationValue::GameFinished(game_identifier)
                | MutationValue::GameStarted(game_identifier) => {
                    invalidate_keys = Some(vec![QueryKey::Game(game_identifier.into()), QueryKey::Games]);
                }
                _ => {}
            }
            loading_signal.set(LoadingState::Loaded);
        },
        MutationResult::Err(MutationError::UnableToAdvanceGame) => {
            tracing::error!("Failed to advance game {}", game_id);
            // Potentially reset loading state or show an error message
            loading_signal.set(LoadingState::Loaded); // Or an error state
        },
        MutationResult::Err(e) => {
            tracing::error!("Mutation failed for game {}: {:?}", game_id, e);
            loading_signal.set(LoadingState::Loaded); // Or an error state
        },
        _ => {
            // Handle pending/uninitialized states if needed
            // Usually covered by the initial loading_signal.set(LoadingState::Loading)
        }
    }

    if let Some(keys) = invalidate_keys {
        client.invalidate_queries(&keys);
    }
}

#[component]
pub fn GamePage(identifier: String) -> Element {
    rsx! {
        div {
            class: r#"
            mt-4
            flex
            flex-col
            gap-4
            "#,
            GameState { identifier: identifier.clone() }
            GameStats { identifier: identifier.clone() }
            GameDetails { identifier: identifier.clone() }
        }
    }
}

#[component]
fn GameState(identifier: String) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let loading_signal = use_context::<Signal<LoadingState>>();

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let token_clone = token.clone();
    let game_query = use_get_query(
        [QueryKey::Game(identifier.clone())],
        move |keys: Vec<QueryKey>| { fetch_game(keys, token.clone()) },
    );

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game_data)) => {
            let game = *game_data.clone();
            let game_id = game.identifier.clone();
            let g_id = game_id.clone();
            let game_name = game.name.clone();
            let game_status = game.status.clone();
            let is_mine = game.is_mine;
            let is_ready = game.ready;
            let is_finished = game.status == GameStatus::Finished;
            let game_private = game.private;

            let mutate = use_mutation(next_step);

            let game_next_step = match game_status {
                GameStatus::NotStarted => {
                    if is_ready { "Start" } else { "Wait" }.to_string()
                },
                GameStatus::InProgress => "Play next step".to_string(),
                GameStatus::Finished => "Clone".to_string(),
            };

            let next_step_handler = move |_| {
                let game_id_clone = game_id.clone();
                let token_clone = token_clone.clone();
                let mutate_clone = mutate.clone();
                let client_clone = client.clone();
                let loading_signal_clone = loading_signal.clone();

                spawn(async move {
                    handle_next_step(
                        game_id_clone,
                        token_clone,
                        mutate_clone,
                        client_clone,
                        loading_signal_clone
                    ).await;
                });
            };

            rsx! {
                div {
                    class: "flex flex-col gap-2",
                    h2 {
                        class: r#"
                        flex
                        flex-row
                        place-content-between

                        theme1:text-2xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-300

                        theme2:font-[Playfair_Display]
                        theme2:text-3xl
                        theme2:text-green-200

                        theme3:font-[Orbitron]
                        theme3:text-2xl
                        theme3:text-stone-700
                        "#,

                        "{game_name}"

                        if is_mine {
                            span {
                                class: "pl-2",
                                GameEdit {
                                    identifier: g_id.clone(),
                                    name: game_name.clone(),
                                    private: game_private,
                                    icon_class: r#"
                                    size-4

                                    theme1:fill-amber-500
                                    theme1:hover:fill-amber-200

                                    theme2:fill-green-200/50
                                    theme2:hover:fill-green-200

                                    theme3:fill-amber-700/75
                                    theme3:hover:fill-amber-700
                                    "#
                                }
                            }
                        }
                    }
                    if is_mine {
                        ThemedButton {
                            class: "place-self-center-safe",
                            onclick: next_step_handler,
                            disabled: is_finished,
                            "{game_next_step}"
                        }
                    }
                }
            }
        },
        QueryResult::Err(e) => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,
                    "Failed to load: {e:?}"
                }
            }
        },
        _ => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,
                    "Loading..."
                }
            }
        }
    }
}

#[component]
fn GameStats(identifier: String) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let game_query = use_get_query(
        [QueryKey::Game(identifier.clone())],
        move |keys: Vec<QueryKey>| { fetch_game(keys, token.clone()) },
    );

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            let game_day = game.day.unwrap_or(0);
            let tribute_count = game
                .clone()
                .tributes
                .into_iter()
                .filter(|t| t.is_alive())
                .collect::<Vec<Tribute>>()
                .len();

            let game_status = match game.status {
                GameStatus::NotStarted => "Not started".to_string(),
                GameStatus::InProgress => "In progress".to_string(),
                GameStatus::Finished => "Finished".to_string(),
            };

            let winner_name = {
                if game.winner().is_some() {
                    game.winner().unwrap().name
                } else {
                    String::new()
                }
            };
            let g = game.clone();

            rsx! {
                div {
                    class: "flex flex-col gap-2 mt-4",

                    if !winner_name.is_empty() {
                        h1 {
                            class: "block text-3xl",
                            "Winner: {winner_name}!"
                        }
                    }

                    div {
                        class: "flex flex-row place-content-between pr-2",

                        p {
                            class: r#"
                            flex-grow
                            theme1:text-amber-300
                            theme2:text-green-200

                            theme3:text-stone-700
                            "#,

                            span {
                                class: r#"
                                block
                                text-sm
                                theme1:text-amber-500
                                theme1:font-semibold
                                theme2:text-teal-500
                                theme3:text-yellow-600
                                theme3:font-semibold
                                "#,

                                "status"
                            }
                            "{game_status}"
                        }
                        p {
                            class: r#"
                            flex-grow
                            theme1:text-amber-300
                            theme2:text-green-200
                            theme3:text-stone-700
                            "#,

                            span {
                                class: r#"
                                block
                                text-sm
                                theme1:text-amber-500
                                theme1:font-semibold
                                theme2:text-teal-500
                                theme3:text-yellow-600
                                theme3:font-semibold
                                "#,

                                "day"
                            }
                            "{game_day}"
                        }
                        p {
                            class: r#"
                            theme1:text-amber-300
                            theme2:text-green-200
                            theme3:text-stone-700
                            "#,

                            span {
                                class: r#"
                                block
                                text-sm
                                theme1:text-amber-500
                                theme1:font-semibold
                                theme2:text-teal-500
                                theme3:text-yellow-600
                                theme3:font-semibold
                                "#,

                                "tributes alive"
                            }
                            "{tribute_count}"
                        }
                    }
                }
            }
        },
        QueryResult::Err(e) => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,

                    "Failed to load: {e:?}"
                }
            }
        },
        _ => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,
                    "Loading..."
                }
            }
        }
    }
}

#[component]
fn GameDetails(identifier: String) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let game_query = use_get_query(
        [QueryKey::Game(identifier.clone())],
        move |keys: Vec<QueryKey>| { fetch_game(keys, token.clone()) },
    );

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            let game_ = game.clone();

            let mut game_signal: Signal<Option<Game>> = use_context();
            use_effect({
                let game = game.clone();
                move || {
                    game_signal.set(Some(*game.clone()));
                }
            });

            rsx! {
                div {
                    class: r#"
                    grid
                    gap-4
                    grid-cols-1
                    lg:grid-cols-2
                    c:xl:grid-cols-3
                    xl:grid-cols-[1fr_1fr_18rem]
                    "#,

                    InfoDetail {
                        title: "Areas",
                        open: false,
                        GameAreaList { game: *game_.clone() }
                    }

                    InfoDetail {
                        title: "Tributes",
                        open: false,
                        GameTributes { game: *game_.clone() }
                    }

                    if game.day.unwrap_or(0) > 0 {
                        InfoDetail {
                            title: "Day log",
                            open: false,
                            GameDayLog { game: *game_.clone(), day: game_.day.unwrap_or_default() }
                        }
                    }
                }
            }
        },
        QueryResult::Err(e) => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,

                    "Failed to load: {e:?}"
                }
            }
        },
        _ => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,
                    "Loading..."
                }
            }
        }
    }
}
