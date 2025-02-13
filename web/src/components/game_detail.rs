use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_tributes::GameTributes;
use crate::components::tribute_delete::{DeleteTributeModal, TributeDelete};
use crate::components::tribute_edit::EditTributeModal;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;

async fn fetch_game(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(name)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}", API_HOST.clone(), name))
            .await.unwrap();

        match response.json::<Game>().await {
            Ok(game) => {
                GAME.set(game.clone());
                QueryResult::Ok(QueryValue::Game(game))
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(name.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameDetail(name: String) -> Element {
    let game_query = use_get_query([QueryKey::Game(name.clone()), QueryKey::Games], fetch_game);

    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game_result)) => {
            rsx! {
                h1 { "{game_result.name}" }
                h2 { "{game_result.status}" }
                
                h3 { "Tributes" }

                GameTributes { game_name: game_result.name.clone() }

                EditTributeModal {}

                RefreshButton { game_name: game_result.name.clone() }

                h3 { "Areas" }
                ul {
                    for (area, details) in game_result.areas.iter() {
                        li {
                            "{area}: {details.open}"
                            ul {
                                for item in &details.items {
                                    li {
                                        "{item.name}",
                                    }
                                }
                            }
                        }
                    }
                }

            }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! { p { "Game not found" } }
        }
    }
}

#[component]
fn RefreshButton(game_name: String) -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let onclick = move |e| {
        client.invalidate_queries(&[QueryKey::Tributes(game_name.clone())]);
    };

    rsx! {
        button {
            r#type: "button",
            onclick: onclick,
            "Refresh"
        }
    }
}
