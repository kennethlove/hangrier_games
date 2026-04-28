use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::Arc;
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
