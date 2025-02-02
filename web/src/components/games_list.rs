use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::{CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::Game;

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        match reqwest::get(format!("{}/api/games", API_HOST.clone())).await {
            Ok(request) => {
                if let Ok(response) = request.json::<Vec<Game>>().await {
                    dioxus_logger::tracing::info!("Got {} games", response.len());
                    QueryResult::Ok(QueryValue::Games(response))
                } else {
                    dioxus_logger::tracing::error!("Failed to parse JSON response");
                    QueryResult::Err(QueryError::BadJson)
                }
            },
            Err(e) => {
                dioxus_logger::tracing::error!("Failed to fetch games: {:?}", e);
                QueryResult::Err(QueryError::NoGames)
            }
        }
    } else {
        dioxus_logger::tracing::info!("Unknown query: {:?}", keys);
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);

    rsx! {
        CreateGameButton {}
        CreateGameForm {}

        match games_query.result().value() {
            QueryResult::Ok(QueryValue::Games(games)) => {
                rsx! {
                    if games.is_empty() {
                        p { "No games yet" }
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
            onclick: onclick,
            "Refresh"
        }
    }
}

#[component]
pub fn GameListMember(game: Game) -> Element {
    rsx! {
        li {
            Link { to: Routes::GameDetail { name: game.name.clone() }, "{game.name}"}
            GameDelete { game_name: game.name.clone() }
        }
    }
}
