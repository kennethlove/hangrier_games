use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::tributes::Tribute;

async fn fetch_game_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Game(name)) = keys.first() {
        if let Some(QueryKey::Tributes) = keys.last() {
            let response = reqwest::get(
                format!("{}/api/game/{}/tributes", API_HOST.clone(), name)
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
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

// pub async fn get_game_tributes()

// pub fn GameTributes() -> Element {
//
// }
