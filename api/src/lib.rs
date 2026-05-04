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
use tracing::error;

pub mod auth;
pub mod cleanup;
pub mod cookies;
pub mod games;
// pub mod messages; // TODO: Module file missing
pub mod storage;
pub mod tributes;
pub mod users;
pub mod websocket;

#[derive(Clone)]
pub struct AppState {
    /// Root-authenticated, shared SurrealDB connection. NEVER call
    /// `.signup()`, `.signin()`, or `.authenticate()` on this handle
    /// directly — those mutate connection-level session state and would
    /// race across concurrent requests (see bd hangrier_games-c853).
    /// Per-request user auth is done on a `clone()` (see `surreal_jwt`
    /// middleware in `main.rs` and the `AuthDb` extractor below); the
    /// SurrealDB Rust SDK's documented multi-tenancy model gives each
    /// clone its own independent session (auth + variables) while sharing
    /// the underlying socket.
    pub db: Arc<Surreal<Any>>,
    pub storage: Arc<dyn storage::StorageBackend>,
    pub broadcaster: Arc<websocket::GameBroadcaster>,
    pub namespace: String,
    pub database: String,
}

/// Per-request, JWT-authenticated SurrealDB session injected by the
/// `surreal_jwt` middleware. Handlers behind that middleware should
/// extract this instead of touching `AppState::db` so `$auth`-gated
/// queries see the calling user's identity. The wrapped `Surreal<Any>`
/// is a clone of the shared connection — independent session state, same
/// underlying socket. See bd hangrier_games-c3ct.
#[derive(Clone)]
pub struct AuthDb(pub Surreal<Any>);

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
            AppError::InternalServerError(message) => {
                error!(error = %message, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            AppError::GameFull(message) => (StatusCode::CONFLICT, message),
            AppError::DbError(message) => {
                error!(error = %message, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::InvalidStatus(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
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
