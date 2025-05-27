use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_day_log::GameDayLog;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::components::info_detail::InfoDetail;
use crate::components::ThemedButton;
use crate::env::APP_API_HOST;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState};
use crate::LoadingState;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_mutation, use_query_client, MutationResult, MutationState, QueryResult, QueryState, UseMutation, UseQueryClient};
use game::games::Game;
use reqwest::StatusCode;
use shared::{DisplayGame, GameStatus};
use std::ops::Deref;

async fn fetch_display_game(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::DisplayGame(identifier)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}/display", APP_API_HOST, identifier))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) =>  {
                match response.error_for_status() {
                    Ok(response) => {
                        if let Ok(game) = response.json::<DisplayGame>().await {
                            Ok(QueryValue::DisplayGame(Box::new(game)))
                        } else {
                            Err(QueryError::BadJson)
                        }
                    }
                    Err(e) => {
                        if e.status() == Some(StatusCode::UNAUTHORIZED) {
                            Err(QueryError::Unauthorized)
                        } else {
                            Err(QueryError::GameNotFound(identifier.to_string()))
                        }
                    }
                }
            }
            Err(e) => {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Err(QueryError::Unauthorized)
                } else {
                    Err(QueryError::GameNotFound(identifier.to_string()))
                }
            }
        }
    } else {
        Err(QueryError::Unknown)
    }
}

async fn _fetch_full_game(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}", APP_API_HOST, identifier))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) =>  {
                match response.error_for_status() {
                    Ok(response) => {
                        if let Ok(game) = response.json::<Game>().await {
                            QueryResult::Ok(QueryValue::Game(Box::new(game)))
                        } else {
                            QueryResult::Err(QueryError::BadJson)
                        }
                    }
                    Err(e) => {
                        if e.status() == Some(StatusCode::UNAUTHORIZED) {
                            QueryResult::Err(QueryError::Unauthorized)
                        } else {
                            QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
                        }
                    }
                }
            }
            Err(e) => {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    QueryResult::Err(QueryError::Unauthorized)
                } else {
                    QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
                }
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn next_step(args: (String, String)) -> MutationResult<MutationValue, MutationError> {
    let identifier = args.0.clone();
    let token = args.1.clone();
    let client = reqwest::Client::new();
    let url: String = format!("{}/api/games/{}/next", APP_API_HOST, identifier);

    let request = client.request(reqwest::Method::PUT, url).bearer_auth(token);

    match request.send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::NO_CONTENT => Ok(MutationValue::GameFinished(identifier)),
                StatusCode::CREATED => Ok(MutationValue::GameStarted(identifier)),
                StatusCode::OK => {
                    Ok(MutationValue::GameAdvanced(identifier))
                },
                _ => Err(MutationError::UnableToAdvanceGame),
            }
        }
        Err(_) => {
            Err(MutationError::UnableToAdvanceGame)
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
    mutate.mutate_async((game_id.clone(), token)).await;

    match mutate.result().deref() {
        MutationState::Settled(Ok(result)) => {
            match result {
                MutationValue::GameStarted(game_identifier) |
                MutationValue::GameFinished(game_identifier) |
                MutationValue::GameAdvanced(game_identifier) => {
                    client.invalidate_queries(&[QueryKey::DisplayGame(game_identifier.clone().into()), QueryKey::Games]);
                    loading_signal.set(LoadingState::Loaded);
                }
                _ => {
                    loading_signal.set(LoadingState::Loaded); // Or an error state
                }
            }
        }
        MutationState::Settled(Err(MutationError::UnableToAdvanceGame)) => {
            loading_signal.set(LoadingState::Loaded); // Or an error state
        }
        MutationState::Settled(Err(_)) => {
            loading_signal.set(LoadingState::Loaded); // Or an error state
        }
        _ => {}
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
    let token = storage.get().jwt.unwrap_or_default();

    let loading_signal = use_context::<Signal<LoadingState>>();

    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let token_clone = token.clone();
    let game_query = use_get_query(
        [QueryKey::DisplayGame(identifier.clone()), QueryKey::Games],
        move |keys: Vec<QueryKey>| { fetch_display_game(keys, token.clone()) },
    );

    match game_query.result().value() {
        QueryState::Settled(Ok(QueryValue::DisplayGame(game_data))) => {
            let game = *game_data.clone();
            let game_id = game.identifier.clone();
            let g_id = game_id.clone();
            let game_name = game.name.clone();
            let game_status = game.status.clone();
            let is_mine = game.is_mine;
            let is_ready = game.ready;
            let is_finished = game.status == GameStatus::Finished;
            let game_private = game.private;
            let creator = game.created_by.username.clone();
            let day = game.day.unwrap_or(0);

            let mutate = use_mutation(next_step);

            let game_next_step = match game_status {
                GameStatus::NotStarted => {
                    if is_ready { "Start" } else { "Wait" }.to_string()
                },
                GameStatus::InProgress => format!("Play day {}", day + 1),
                GameStatus::Finished => "Done!".to_string(),
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
                    div {
                        class: r#"
                        flex
                        flex-row
                        gap-4
                        place-content-between
                        align-middle
                        "#,

                        h2 {
                            class: r#"
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
                        }

                        if is_mine {
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
                        } else {
                            span {
                                class: r#"
                                text-sm
                                theme1:text-stone-200/75
                                theme2:text-green-200/50
                                theme3:text-stone-700
                                "#,
                                "By {creator}"
                            }
                        }
                    }
                    if is_mine && !is_finished {
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
        QueryState::Settled(Err(QueryError::GameNotFound(_))) => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,
                    "Game not found"
                }
            }
        },
        QueryState::Settled(Err(QueryError::Unauthorized)) => {
            rsx! {
                p {
                    class: r#"
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-slate-700
                    "#,

                    h2 {
                        class: r#"
                        text-2xl
                        theme1:text-amber-300
                        theme2:text-green-200
                        theme3:text-slate-700
                        "#,
                        "Unauthorized"
                    }
                    p {
                        "Do you need to "
                        Link {
                            class: r#"
                            underline
                            theme1:text-amber-300
                            theme1:hover:text-amber-200
                            theme2:text-green-200
                            theme2:hover:text-green-100
                            theme3:text-slate-700
                            theme3:hover:text-slate-500
                            "#,
                            to: Routes::AccountsPage {},
                            "login or signup?"
                        }
                    }
                }
            }
        },
        QueryState::Settled(Err(e)) => {
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
        QueryState::Loading(_) => {
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
        _ => { rsx! {} }
    }
}

#[component]
fn GameStats(identifier: String) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.unwrap_or_default();

    let game_query = use_get_query(
        [QueryKey::DisplayGame(identifier.clone()), QueryKey::Games],
        move |keys: Vec<QueryKey>| { fetch_display_game(keys, token.clone()) },
    );

    match game_query.result().value() {
        QueryState::Settled(Ok(QueryValue::DisplayGame(game))) => {
            let game_day = game.day.unwrap_or(0);
            let tribute_count = game.living_count;

            let game_status = match game.status {
                GameStatus::NotStarted => "Not started".to_string(),
                GameStatus::InProgress => "In progress".to_string(),
                GameStatus::Finished => "Finished".to_string(),
            };

            rsx! {
                div {
                    class: "flex flex-col gap-2 mt-4",
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
        QueryState::Settled(Err(_)) => { rsx! {} },
        QueryState::Loading(_) => {
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
        _ => { rsx! {} }
    }
}

#[component]
fn GameDetails(identifier: String) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let display_token = storage.get().jwt.unwrap_or_default();

    let display_game_query = use_get_query(
        [QueryKey::DisplayGame(identifier.clone()), QueryKey::Games],
        move |keys: Vec<QueryKey>| { fetch_display_game(keys, display_token.clone()) },
    );

    match display_game_query.result().value() {
        QueryState::Settled(Ok(QueryValue::DisplayGame(game))) => {
            let display_game = *game.clone();
            let day = display_game.clone().day.unwrap_or(0);

            let xl_display = match day {
                0 => "xl:grid-cols-[1fr_1fr]".to_string(),
                _ => "xl:grid-cols-[1fr_1fr_22rem]".to_string(),
            };

            let class: String = format!(r#"
            grid
            gap-4
            grid-cols-1
            lg:grid-cols-2
            {}
            "#, xl_display);

            rsx! {
                div {
                    class,

                    InfoDetail {
                        title: "Areas",
                        open: false,
                        GameAreaList { game: display_game.clone() }
                    }

                    InfoDetail {
                        title: "Tributes",
                        open: false,
                        GameTributes { game: display_game.clone() }
                    }

                    if day > 0 {
                        InfoDetail {
                            title: "Day log",
                            open: false,
                            GameDayLog { game: display_game.clone(), day: day }
                        }
                    }
                }
            }
        },
        QueryState::Settled(Err(_)) => { rsx! { } },
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
