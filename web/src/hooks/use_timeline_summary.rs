use crate::cache::QueryError;
use crate::http::WithCredentials;
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
        let url = crate::api_url::api_url(&format!("/api/games/{id}/timeline-summary"));
        let resp = reqwest::Client::new()
            .get(&url)
            .with_credentials()
            .send()
            .await;
        match resp {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<TimelineSummary>().await {
                    Ok(s) => Ok(s),
                    Err(_) => Err(QueryError::BadJson),
                },
                StatusCode::UNAUTHORIZED => Err(QueryError::Unauthorized),
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
