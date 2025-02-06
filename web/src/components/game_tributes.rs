use crate::cache::{QueryError, QueryKey, QueryValue};
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
        [QueryKey::Tributes(name)],
        fetch_game_tributes
    );

    match tributes_query.result().value() {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            rsx! {
                ul {
                    for tribute in tributes {
                        li { "{tribute.name}" }
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
