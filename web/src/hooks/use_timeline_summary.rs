use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::env::APP_API_HOST;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::TimelineSummary;

pub(crate) async fn fetch_timeline_summary(
    keys: Vec<QueryKey>,
) -> QueryResult<QueryValue, QueryError> {
    let Some(QueryKey::TimelineSummary(id)) = keys.first() else {
        return Err(QueryError::Unknown);
    };
    let url = format!("{APP_API_HOST}/api/games/{id}/timeline-summary");
    match reqwest::get(&url).await {
        Ok(resp) => match resp.status() {
            StatusCode::OK => match resp.json::<TimelineSummary>().await {
                Ok(s) => Ok(QueryValue::TimelineSummary(s)),
                Err(_) => Err(QueryError::BadJson),
            },
            StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(id.clone())),
            _ => Err(QueryError::Unknown),
        },
        Err(_) => Err(QueryError::ServerNotFound),
    }
}

#[allow(dead_code)]
pub(crate) fn use_timeline_summary(game_id: String) -> UseQuery<QueryValue, QueryError, QueryKey> {
    use_get_query([QueryKey::TimelineSummary(game_id)], fetch_timeline_summary)
}
