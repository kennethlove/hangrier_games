use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::GameDelete;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::Game;
use crate::routes::Routes;

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::AllGames) = keys.first() {
        let response = reqwest::get("http://127.0.0.1:3000/api/games")
            .await.unwrap()
            .json::<Vec<Game>>()
            .await.unwrap();
        QueryResult::Ok(QueryValue::Games(response))
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);

    match games_query.result().value() {
        QueryResult::Err(QueryError::NoGames) => {
            rsx! { p { "No games" } }
        }
        QueryResult::Ok(games) => {
            match games {
                QueryValue::Games(games) => {
                    if games.is_empty() {
                        rsx! { p { "No games yet" } }
                    } else {
                        rsx! {
                            ul {
                                for game in games {
                                    GameListMember { game: game.clone() }
                                }
                            }
                        }
                    }
                }
                _ => {
                    rsx! { p { "Wrong result type" } }
                }
            }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! { p { "No idea how you got here." } }
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

