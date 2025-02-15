use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_delete::TributeDelete;
use crate::components::tribute_edit::{TributeEdit, EditTributeModal};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;
use std::borrow::Borrow;

async fn fetch_game_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(game_name)) = keys.first() {
        let response = reqwest::get(
            format!("{}/api/games/{}/tributes", API_HOST.clone(), game_name)
        ).await.expect("failed to fetch game tributes");

        match response.json::<Vec<Tribute>>().await {
            Ok(tributes) => {
                QueryResult::Ok(QueryValue::Tributes(tributes))
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(game_name.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributes(game_name: String) -> Element {
    let tributes_query = use_get_query(
        [QueryKey::Tributes(game_name.clone())],
        fetch_game_tributes
    );
    let tribute_result = tributes_query.result();
    let tribute_response = tribute_result.value();
    
    let game = GAME.borrow();
    let mut game = game.take();

    match tribute_response {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            game.tributes = tributes.clone();
            GAME.set(game);
            
            rsx! {
                ul {
                    for tribute in tributes {
                        li {
                            "{tribute.name} - {tribute.district}",
                            TributeEdit {
                                name: tribute.clone().name,
                                district: tribute.district,
                                identifier: tribute.clone().identifier,
                            }
                        }
                    }
                }

            }
        },
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx!("")
        }
    }
}
