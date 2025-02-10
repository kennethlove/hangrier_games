use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::create_tribute::{CreateTributeButton, CreateTributeForm};
use crate::components::tribute_delete::TributeDelete;
use crate::components::tribute_edit::TributeEdit;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;

async fn fetch_game_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(name)) = keys.first() {
        let response = reqwest::get(
            format!("{}/api/games/{}/tributes", API_HOST.clone(), name)
        ).await.unwrap();

        match response.json::<Vec<Tribute>>().await {
            Ok(tributes) => {
                QueryResult::Ok(QueryValue::Tributes(tributes))
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
pub fn GameTributes(name: String) -> Element {
    let tributes_query = use_get_query(
        [QueryKey::Tributes(name.clone())],
        fetch_game_tributes
    );

    match tributes_query.result().value() {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            let tribute_count = &tributes.len();

            rsx! {
                ul {
                    for tribute in tributes {
                        li {
                            "{tribute.name} - {tribute.district}",
                            TributeEdit {
                                name: tribute.clone().name,
                                district: tribute.district as u8,
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
