use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::GameEdit;
use crate::components::icons::eye_closed::EyeClosedIcon;
use crate::components::icons::eye_open::EyeOpenIcon;
use crate::components::{Button, CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::env::APP_API_HOST;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult, QueryState};
use shared::{DisplayGame, GameStatus};

async fn fetch_games(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        let client = reqwest::Client::new();
        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games", APP_API_HOST),
        ).bearer_auth(token);

        match request.send().await{
            Ok(request) => {
                if let Ok(response) = request.json::<Vec<DisplayGame>>().await {
                    Ok(QueryValue::DisplayGames(response))
                } else {
                    Err(QueryError::BadJson)
                }
            },
            Err(_) => {
                Err(QueryError::NoGames)
            }
        }
    } else {
        Err(QueryError::Unknown)
    }
}

#[component]
fn NoGames() -> Element {
    rsx! {
        p {
            class: "pb-4 text-center theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-700",
            "No games yet"
        }
    }
}

#[component]
pub fn GamesList() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");
    let games_query = use_get_query(
        [QueryKey::AllGames, QueryKey::Games],
        move |keys: Vec<QueryKey>| { fetch_games(keys, token.clone()) });

    rsx! {
        div {
            class: r#"
            flex
            flex-col
            flex-col-reverse
            sm:flex-row
            flex-wrap
            sm:flex-nowrap
            gap-2
            place-content-center
            py-2
            mb-4
            theme1:bg-transparent
            "#,
            CreateGameButton {}
            CreateGameForm {}
        }

        match games_query.result().value() {
            QueryState::Settled(Ok(QueryValue::DisplayGames(games))) => {
                rsx! {
                    if games.is_empty() {
                        NoGames {}
                    } else {
                        ul {
                            class: r#"
                            xl:grid
                            xl:grid-cols-2
                            xl:gap-4
                            "#,
                            for game in games {
                                GameListMember { game: game.clone() }
                            }
                        }
                    }
                }
            },
            QueryState::Loading(_) => rsx! { p {
                class: "pb-4 text-center theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-700",
                "Loading..."
            } },
            QueryState::Settled(Err(QueryError::NoGames)) => rsx! { NoGames {} },
            QueryState::Settled(Err(QueryError::BadJson)) => rsx! { p {
                class: "pb-4 text-center theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-700",
                "Bad JSON response"
            } },
            _ => rsx! { p { "Something went wrong" } },
        }

        div {
            class: "mt-4",
            RefreshButton {}
        }

        DeleteGameModal {}
    }
}

#[component]
fn RefreshButton() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let onclick = move |_| {
        client.invalidate_queries(&[QueryKey::Games]);
    };

    rsx! {
        div {
            class: "text-center",
            Button {
                onclick,
                "Refresh"
            }
        }
    }
}

#[component]
pub fn GameListMember(game: DisplayGame) -> Element {
    let created_by = game.clone().created_by;
    let is_mine = game.clone().is_mine;

    let living_count = game.living_count;

    rsx! {
        li {
            class: r#"
            block
            w-full
            border
            p-2
            mb-4
            xl:mb-0
            transition

            theme1:border-stone-950
            theme1:bg-stone-800/65
            theme1:hover:bg-stone-800/75
            theme1:rounded-sm

            theme2:border-none
            theme2:bg-green-900
            theme2:rounded-md

            theme3:border-3
            theme3:border-gold-rich
            theme3:bg-stone-50/75
            "#,
            div {
                class: "flex place-content-between",
                h2 {
                    class: r#"
                    text-xl
                    theme1:font-[Cinzel]
                    theme1:text-amber-300
                    theme1:hover:underline

                    theme2:text-green-200
                    theme2:hover:underline
                    theme2:hover:decoration-wavy
                    theme2:hover:decoration-2
                    theme2:mb-2

                    theme3:text-yellow-600
                    "#,
                    Link {
                        to: Routes::GamePage {
                            identifier: game.identifier.clone()
                        },
                        title: r#"Play "{game.name}""#,
                        "{game.name}"
                    }
                }
                if is_mine {
                    div {
                        class: "flex flex-row gap-2",
                        GameEdit {
                            identifier: game.identifier.clone(),
                            name: game.name.clone(),
                            private: game.private.clone(),
                            icon_class: r#"
                            size-4
                            theme1:fill-amber-600
                            theme1:hover:fill-amber-500

                            theme2:fill-green-200/50
                            theme2:hover:fill-green-200

                            theme3:fill-yellow-600
                            theme3:hover:fill-amber-500
                            "#,
                        }
                        GameDelete {
                            game_name: game.name.clone(),
                            game_identifier: game.identifier.clone(),
                            icon_class: r#"
                            size-4
                            theme1:fill-amber-600
                            theme1:hover:fill-amber-500

                            theme2:fill-green-200/50
                            theme2:hover:fill-green-200

                            theme3:fill-yellow-600
                            theme3:hover:fill-amber-500
                            "#,
                        }
                    }
                } else {
                    div {
                        class: "flex flex-row gap-2",
                        p {
                            class: r#"
                            text-sm
                            theme1:text-stone-200/75
                            theme2:text-green-200/50
                            theme3:text-stone-700
                            "#,
                            "By {created_by.username}"
                        }
                    }
                }
            }
            div {
                class: r#"
                flex
                flex-row
                place-content-between
                text-xs
                theme1:text-stone-200/75
                theme2:text-green-200/50
                theme3:text-stone-700
                "#,
                p { class: "flex-grow", "{living_count} / {game.tribute_count} tributes" }
                p { class: "flex-grow", "Day {game.day.unwrap_or_default()}" }
                p {
                    class: "flex-grow",
                    match game.status {
                        GameStatus::InProgress => "In progress",
                        GameStatus::Finished => "Finished",
                        GameStatus::NotStarted => "Not started",
                    }
                }
                div {
                    class: "px-2",
                    if game.private {
                        EyeClosedIcon { class: r#"
                            size-4
                            theme1:fill-amber-600
                            theme2:fill-green-200/50
                            theme3:fill-yellow-600
                        "# }
                    } else {
                        EyeOpenIcon { class: r#"
                            size-4
                            theme1:fill-amber-600
                            theme2:fill-green-200/50
                            theme3:fill-yellow-600
                        "# }
                    }
                }
            }
        }
    }
}
