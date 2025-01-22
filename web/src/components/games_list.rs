use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::{CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::Game;

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        match reqwest::get("http://127.0.0.1:3000/api/games").await {
            Ok(request) => {
                if let Ok(response) = request.json::<Vec<Game>>().await {
                    QueryResult::Ok(QueryValue::Games(response))
                } else {
                    QueryResult::Err(QueryError::BadJson)
                }
            },
            _ => QueryResult::Err(QueryError::NoGames)
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);

    if let QueryResult::Ok(QueryValue::Games(games)) = games_query.result().value() {
        rsx! {
            CreateGameButton {}
            CreateGameForm {}

            if games.is_empty() {
                p { "No games yet" }
            } else {
                ul {
                    for game in games {
                        GameListMember { game: game.clone() }
                    }
                }
            }

            DeleteGameModal {}
        }
    } else { rsx! {} }
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

