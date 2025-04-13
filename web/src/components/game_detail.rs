use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::game_areas::GameAreaList;
use crate::components::game_day_log::GameDayLog;
use crate::components::game_day_summary::GameDaySummary;
use crate::components::game_edit::GameEdit;
use crate::components::game_tributes::GameTributes;
use crate::components::Button;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{
    use_get_query, use_mutation, use_query_client, MutationResult, QueryResult,
};
use game::games::Game;
use game::games::GameStatus;
use game::tributes::Tribute;
use reqwest::StatusCode;
use std::ops::Deref;

pub(crate) async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}", API_HOST, identifier))
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
    let url: String = format!("{}/api/games/{}/next", API_HOST, identifier);

    let response = client
        .put(url)
        .send()
        .await
        .expect("Failed to advance game");

    dioxus_logger::tracing::info!("{:?}", &response);

    match response.status() {
        StatusCode::NO_CONTENT => MutationResult::Ok(MutationValue::GameFinished(identifier)),
        StatusCode::CREATED => MutationResult::Ok(MutationValue::GameStarted(identifier)),
        StatusCode::OK => MutationResult::Ok(MutationValue::GameAdvanced(identifier)),
        _ => MutationResult::Err(MutationError::UnableToAdvanceGame),
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
        let tribute_count = game
            .clone()
            .tributes
            .into_iter()
            .filter(|t| t.is_alive())
            .collect::<Vec<Tribute>>()
            .len();
        let winner_name = {
            if game.winner().is_some() {
                game.winner().unwrap().name
            } else {
                String::new()
            }
        };
        let g = game.clone();

        let next_step_handler = move |_| {
            let game_id = game_id.clone();
            let mut game = game.clone();

            let client = use_query_client::<QueryValue, QueryError, QueryKey>();

            spawn(async move {
                mutate.manual_mutate(game_id.clone()).await;

                match mutate.result().deref() {
                    MutationResult::Ok(mutation_result) => match mutation_result {
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
                    },
                    MutationResult::Err(MutationError::UnableToAdvanceGame) => {
                        dioxus_logger::tracing::error!("Failed to advance game");
                    }
                    _ => {}
                }
            });
        };

        rsx! {
            div {
                class: "flex flex-col gap-2 mt-4",
                div {
                    class: "flex flex-row flex-wrap gap-2 place-content-between",
                    h2 {
                        class: r#"
                        flex
                        flex-row

                        theme1:text-2xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-300

                        theme2:font-[Forum]
                        theme2:text-3xl
                        theme2:text-green-200

                        theme3:font-[Orbitron]
                        theme3:text-2xl
                        theme3:text-stone-700
                        "#,

                        "{game_name}"

                        span {
                            class: "pl-2",
                            GameEdit {
                                identifier: g.identifier,
                                name: g.name,
                                icon_class: r#"
                                size-4

                                theme1:fill-amber-500
                                theme1:hover:fill-amber-200

                                theme2:fill-green-200/50
                                theme2:hover:fill-green-200

                                theme3:fill-amber-600/50
                                theme3:hover:fill-amber-600
                                "#
                            }
                        }
                    }
                    div {
                        class: "flex flex-row flex-grow gap-2 place-content-center sm:place-content-end",
                        Button {
                            extra_classes: Some(r#"
                            theme1:bg-radial
                            theme1:from-amber-300
                            theme1:to-red-500
                            theme1:border-red-500
                            theme1:text-red-900
                            theme1:hover:text-stone-200
                            theme1:hover:from-amber-500
                            theme1:hover:to-red-700

                            theme2:bg-linear-to-b
                            theme2:from-green-400
                            theme2:to-teal-500
                            theme2:border-none
                            theme2:hover:text-green-200
                            theme2:hover:from-green-500
                            theme2:hover:to-teal-600

                            theme3:border-none
                            theme3:bg-gold-rich
                            theme3:hover:bg-gold-rich-reverse
                            theme3:text-stone-700
                            theme3:hover:text-stone-50
                            "#.into()),

                            onclick: next_step_handler,
                            disabled: game_finished,
                            "{game_next_step}"
                        }
                    }
                }

                if !winner_name.is_empty() {
                    h1 {
                        class: "text-xl",
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
            rsx! {
                p {
                    class: r#"
                    text-center

                    theme2:text-green-200
                    "#,
                    "Failed to load"
                }
            }
        }
        _ => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    theme2:text-green-200
                    "#,
                    "Loading..."
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct InfoDetailProps {
    pub title: String,
    pub open: bool,
    pub children: Element
}

#[component]
pub fn InfoDetail(props: InfoDetailProps) -> Element {
    rsx! {
        details {
            open: props.open,
            class: r#"
            px-2
            pt-1
            open:pb-2
            group
            transition
            duration-500
            self-start

            theme1:bg-stone-800/50
            theme1:hover:bg-stone-800
            theme1:open:bg-stone-800/50

            theme2:bg-green-900
            theme2:rounded-md
            theme2:border
            theme2:border-green-800
            theme2:hover:border-green-400
            theme2:open:border-green-400

            theme3:bg-stone-50/80
            theme3:border-4
            theme3:border-gold-rich
            "#,

            summary {
                class: r#"
                flex
                items-center
                justify-between
                cursor-pointer
                "#,

                h3 {
                    class: r#"
                    mb-2
                    transition

                    theme1:text-xl
                    theme1:font-[Cinzel]
                    theme1:text-amber-300/75
                    theme1:group-open:text-amber-300
                    theme1:hover:text-amber-300

                    theme2:font-[Forum]
                    theme2:text-2xl
                    theme2:text-green-200
                    theme2:group-open:text-green-400

                    theme3:font-[Orbitron]
                    theme3:tracking-wider
                    "#,

                    "{props.title}",
                }
                span {
                    class: "transition group-open:rotate-180",
                    svg {
                        class: r#"
                        size-4
                        fill-none
                        stroke-current

                        theme1:stroke-amber-300
                        theme1:hover:stroke-amber-300
                        theme1:group-open:stroke-amber-300

                        theme2:group-open:stroke-green-400
                        theme2:stroke-green-200
                        theme2:hover:stroke-green-400
                        "#,
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M19 9l-7 7-7-7"
                        }
                    }
                }
            }
            {props.children}
        }
    }
}

#[component]
pub fn GameDetails(game: Game) -> Element {
    rsx! {
        div {
            class: r#"
            pr-2
            grid
            gap-8
            grid-cols-none
            sm:grid-cols-2
            lg:grid-cols-3
            2xl:grid-cols-4
            "#,

            InfoDetail {
                title: "Areas",
                open: false,
                GameAreaList { }
            }

            InfoDetail {
                title: "Tributes",
                open: false,
                GameTributes { }
            }

            if game.day.unwrap_or(0) > 0 {
                InfoDetail {
                    title: "Day log",
                    open: false,
                    GameDayLog { day: game.day.unwrap_or_default() }
                }
            }
        }
    }
}
