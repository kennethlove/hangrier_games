use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::GameEdit;
use crate::components::{Button, CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::Game;
use crate::components::icons::eye_closed::EyeClosedIcon;
use crate::components::icons::eye_open::EyeOpenIcon;
use crate::storage::{use_persistent, AppState};

async fn fetch_games(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        let client = reqwest::Client::new();
        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games", API_HOST),
        ).bearer_auth(token);

        match request.send().await{
            Ok(request) => {
                if let Ok(response) = request.json::<Vec<Game>>().await {
                    QueryResult::Ok(QueryValue::Games(response))
                } else {
                    QueryResult::Err(QueryError::BadJson)
                }
            },
            Err(_) => {
                QueryResult::Err(QueryError::NoGames)
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
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
    let mut storage = use_persistent("hangry-games", AppState::default);
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
            QueryResult::Ok(QueryValue::Games(games)) => {
                rsx! {
                    if games.is_empty() {
                        NoGames {}
                    } else {
                        ul {
                            for game in games {
                                GameListMember { game: game.clone() }
                            }
                        }
                    }
                }
            },
            QueryResult::Loading(_) => rsx! { p {
                class: "pb-4 text-center theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-700",
                "Loading..."
            } },
            QueryResult::Err(QueryError::NoGames) => rsx! { NoGames {} },
            _ => rsx! { p { "Something went wrong" } },
        }

        RefreshButton {}

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
pub fn GameListMember(game: Game) -> Element {
    let living_count = game.living_tributes().len();
    rsx! {
        li {
            class: r#"
            block
            w-full
            border
            p-2
            mb-4
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
            theme3:bg-stone-50/50
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
                if game.is_mine {
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
                        game::games::GameStatus::InProgress => "In progress",
                        game::games::GameStatus::Finished => "Finished",
                        game::games::GameStatus::NotStarted => "Not started",
                    }
                }
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
