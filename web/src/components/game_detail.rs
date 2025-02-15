use std::collections::HashMap;
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_tributes::GameTributes;
use crate::components::tribute_delete::{DeleteTributeModal, TributeDelete};
use crate::components::tribute_edit::EditTributeModal;
use crate::API_HOST;
use game::games::GameStatus;
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
fn GameStatusState() -> Element {
    let game = GAME.with_borrow(|g| { g.clone() });
    let game_ready_signal: Signal<HashMap<u32, u32>> = use_context();
    let game_next_step: String;

    let game_status= match game.status {
        GameStatus::NotStarted => {
            // if *game_ready_signal.read() {
            //     game_next_step = "Start".to_string();
            // } else {
            //     game_next_step = "Wait".to_string();
            // }
            game_next_step = "Start".to_string();
            "Not started".to_string()
        },
        GameStatus::InProgress => {
            game_next_step = "Finish".to_string();
            "In progress".to_string()
        },
        GameStatus::Finished => {
            game_next_step = "Clone".to_string();
            "Finished".to_string()
        }
    };

    rsx! {
        h2 {
            class: "game-status",
            "Game Status: {game_status}"
            button {
                class: "button",
                onclick: move |_| {
                },
                "{game_next_step}"
            }
        }
    }
}

fn game_is_ready() -> HashMap<u32, u32> {
    let game = GAME.with_borrow(|g| { g.clone() });
    let tributes = game.tributes.clone();
    
    if tributes.len() == 0 {
        return HashMap::new();
    }
    
    let mut tribute_spread: HashMap<u32, u32> = HashMap::new();
    for tribute in tributes {
        if tribute_spread.get(&tribute.district).is_some() {
            let count = tribute_spread.get(&tribute.district).unwrap();
            tribute_spread.insert(tribute.district, count + 1);
        } else {
            tribute_spread.insert(tribute.district, 1);
        }
    }

    let mut valid = true;

    for (district, count) in &tribute_spread {
        if *count != 2 { valid = false; }
    }

    // valid
    tribute_spread

    // tribute_spread.values().all(|c| *c == 2)
}

#[component]
pub fn GameDetail(name: String) -> Element {
    let game_query = use_get_query([QueryKey::Game(name.clone()), QueryKey::Games], fetch_game);

    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);
    
    let mut game_ready_signal: Signal<HashMap<u32, u32>> = use_signal(|| game_is_ready());
    dioxus_logger::tracing::debug!("{:?}", &game_ready_signal.peek());
    use_context_provider(|| game_ready_signal);

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game_result)) => {
            rsx! {
                h1 { "{game_result.name}" }

                GameStatusState {}

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
