use crate::cache::QueryError;
use crate::env::APP_API_HOST;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::TimelineSummary;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TimelineSummaryQ;

impl QueryCapability for TimelineSummaryQ {
    type Ok = TimelineSummary;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, id: &String) -> Result<TimelineSummary, QueryError> {
        let url = format!("{APP_API_HOST}/api/games/{id}/timeline-summary");
        match reqwest::get(&url).await {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<TimelineSummary>().await {
                    Ok(s) => Ok(s),
                    Err(_) => Err(QueryError::BadJson),
                },
                StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(id.clone())),
                _ => Err(QueryError::Unknown),
            },
            Err(_) => Err(QueryError::ServerNotFound),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn use_timeline_summary(game_id: String) -> UseQuery<TimelineSummaryQ> {
    use_query(Query::new(game_id, TimelineSummaryQ))
}
