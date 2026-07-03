use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb_types::{RecordId, RecordIdKey};
use thiserror::Error;
use tracing::error;

pub mod auth;
pub mod cleanup;
pub mod cookies;
pub mod email;
pub mod games;
// pub mod messages; // TODO: Module file missing
pub mod sse;
pub mod storage;
pub mod templates;
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
    pub commentator: Option<Arc<dyn announcers::Commentator>>,
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
                    format!("Internal server error: {message}"),
                )
            }
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::InvalidStatus(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}

/// Format a RecordId as a human-readable string (`table:key`).
/// surrealdb_types::RecordId does not implement Display.
pub fn rid_to_string(rid: &RecordId) -> String {
    match &rid.key {
        RecordIdKey::String(s) => format!("{}:{}", rid.table, s),
        RecordIdKey::Number(n) => format!("{}:{n}", rid.table),
        RecordIdKey::Uuid(u) => format!("{}:{u}", rid.table),
        k => format!("{}:{k:?}", rid.table),
    }
}

/// Extract the key part of a RecordId as a string.
pub fn rid_key_to_string(rid: &RecordId) -> String {
    match &rid.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        k => format!("{k:?}"),
    }
}
