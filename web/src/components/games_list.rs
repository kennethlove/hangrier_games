use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::{CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::Game;
use crate::components::game_edit::GameEdit;

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        match reqwest::get(format!("{}/api/games", API_HOST.clone())).await {
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
pub fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);

    rsx! {
        div {
            class: "flex flex-row gap-2 place-content-center py-2 mb-4 bg-green-100 dark:bg-green-100/50",
            CreateGameButton {}
            CreateGameForm {}
        }

        match games_query.result().value() {
            QueryResult::Ok(QueryValue::Games(games)) => {
                rsx! {
                    if games.is_empty() {
                        p {
                            class: "pb-4",
                            "No games yet"
                        }
                    } else {
                        ul {
                            for game in games {
                                GameListMember { game: game.clone() }
                            }
                        }
                    }
                }
            },
            QueryResult::Loading(_) => rsx! { p { "Loading..." } },
            QueryResult::Err(QueryError::NoGames) => rsx! { p { "No games yet" } },
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
        button {
            class: "border px-2 py-1",
            onclick: onclick,
            "Refresh"
        }
    }
}

#[component]
pub fn GameListMember(game: Game) -> Element {
    let living_count = game.living_tributes().len();
    rsx! {
        li {
            class: "block w-full border p-2 mb-4 bg-green-100 dark:bg-green-100/50",
            div {
                class: "flex place-content-between",
                h2 {
                    class: "text-xl cinzel-font text-orange-700 dark:text-amber-500",
                    Link {
                        to: Routes::GamePage {
                            identifier: game.identifier.clone()
                        },
                        "{game.name}"
                    }
                }
                div {
                    class: "flex flex-row gap-2",
                    GameEdit { identifier: game.identifier.clone(), name: game.name.clone() }
                    GameDelete {
                        game_name: game.name.clone(),
                        game_identifier: game.identifier.clone()
                    }
                }
            }
            div {
                class: "flex flex-row place-content-between",
                p { "{living_count} / {game.tribute_count} tributes left" }
                p { "Day {game.day.unwrap_or_default()}" }
                p { "Status: {game.status}" }
            }
        }
    }
}
