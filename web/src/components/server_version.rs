use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::env::APP_API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult, QueryState};

async fn fetch_server_version(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::ServerVersion) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}", APP_API_HOST));

        match request.send().await {
            Ok(response) => {
                match response.json::<String>().await {
                    Ok(version) => {
                        Ok(QueryValue::ServerVersion(version.to_string()))
                    }
                    Err(_) => Err(QueryError::ServerVersionNotFound),
                }
            }
            Err(_) => {
                Err(QueryError::ServerNotFound)
            }
        }
    } else {
        Err(QueryError::Unknown)
    }
}

#[component]
pub fn ServerVersion() -> Element {
    let version_query = use_get_query([QueryKey::ServerVersion],
                                      move |keys: Vec<QueryKey>| { fetch_server_version(keys) },
    );

    match version_query.result().value() {
        QueryState::Settled(Ok(QueryValue::ServerVersion(version))) => {
            rsx! { "Server: v{version}" }
        }
        _ => { rsx! { "Server unavailable" } }
    }
}
