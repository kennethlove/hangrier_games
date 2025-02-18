use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_tributes::GameTributes;
use crate::components::tribute_delete::{DeleteTributeModal, TributeDelete};
use crate::components::tribute_edit::EditTributeModal;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, use_query_client, QueryResult};
use game::games::GameStatus;
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;
use std::collections::HashMap;

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
    let game: Signal<Option<Game>> = use_context();
    let game = game.unwrap();

    let game_next_step: String;

    let game_ready = game_is_ready(&game.tributes);

    let game_status= match game.status {
        GameStatus::NotStarted => {
            if game_ready {
                game_next_step = "Start".to_string();
            } else {
                game_next_step = "Wait".to_string();
            }
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

fn game_is_ready(tributes: &Vec<Tribute>) -> bool {
    if tributes.len() == 0 { return false; }
    
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

    for (_, count) in &tribute_spread {
        if *count != 2 { valid = false; }
    }

    valid
}

#[component]
pub fn GameDetailPage(name: String) -> Element {
    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    let mut game_signal: Signal<Option<Game>> = use_signal(|| None);
    use_context_provider(|| game_signal);

    let game_query = use_get_query([QueryKey::Game(name.clone()), QueryKey::Games], fetch_game);
    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            game_signal.set(Some(game.clone()));
            rsx! {
                GameDetails { game: game.clone() }
            }
        }
        _ => { rsx! { p { "Loading...outer" }} }
    }
}

#[derive(Debug, Clone, PartialEq, Props)]
struct GDP {
    game: Game,
}

pub fn GameDetails(gdp: GDP) -> Element {
    // let game_signal: Signal<Option<Game>> = use_context();
    // dioxus_logger::tracing::debug!("{:?}", &game_signal.read());
    let game = gdp.game;
    let game_name = game.name.clone();

    rsx! {
        div {
            h1 { "{game.name}" }

            GameStatusState {}

            h3 { "Tributes" }

            GameTributes { game_name: game.name.clone() }

            EditTributeModal {}

            RefreshButton { game_name: game.name.clone() }

            h3 { "Areas" }
            ul {
                for (area, details) in game.areas.iter() {
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
}

#[component]
fn RefreshButton(game_name: String) -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let onclick = move |_| {
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
