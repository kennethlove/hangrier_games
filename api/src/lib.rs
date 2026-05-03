use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use thiserror::Error;

pub mod auth;
pub mod cleanup;
pub mod games;
// pub mod messages; // TODO: Module file missing
pub mod storage;
pub mod tributes;
pub mod users;
pub mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Surreal<Any>>,
    pub storage: Arc<dyn storage::StorageBackend>,
    pub broadcaster: Arc<websocket::GameBroadcaster>,
    pub namespace: String,
    pub database: String,
    /// Serializes JWT authentication + downstream request handling on the
    /// shared SurrealDB connection. `Surreal::authenticate` mutates
    /// connection-level session state, so concurrent requests would
    /// interleave and queries could observe a different user's `$auth`
    /// (causing the user's own private games to vanish from
    /// `fn::get_list_games` and similar). The lock is held across the
    /// entire authenticated request so the auth context cannot change
    /// until the response is built. See bd hangrier_games-c853.
    pub auth_lock: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Game is full: {0}")]
    GameFull(String),
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            AppError::GameFull(message) => (StatusCode::BAD_REQUEST, message),
            AppError::DbError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::InvalidStatus(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}

#[derive(Debug, Deserialize)]
struct VerifyRow {
    #[allow(dead_code)]
    id: RecordId,
}

/// Verify that a record exists at the given `RecordId` after a write that
/// the SDK reports as successful. SurrealDB returns an empty result set
/// (no `Err`) when a permission or schema check rejects the write, so we
/// must read back the row to be sure it landed.
///
/// Returns `Err(InternalServerError)` with a `site`-tagged message when
/// the row is missing.
pub async fn verify_record_persisted(
    db: &Surreal<Any>,
    rid: &RecordId,
    site: &'static str,
) -> Result<(), AppError> {
    let mut resp = db
        .query("SELECT id FROM $rid")
        .bind(("rid", rid.clone()))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "{}: persistence verify query failed for {}: {}",
                site, rid, e
            ))
        })?;
    let rows: Vec<VerifyRow> = resp.take(0).map_err(|e| {
        AppError::InternalServerError(format!(
            "{}: persistence verify decode failed for {}: {}",
            site, rid, e
        ))
    })?;
    if rows.is_empty() {
        return Err(AppError::InternalServerError(format!(
            "{}: persistence verification failed for {}",
            site, rid
        )));
    }
    Ok(())
}
