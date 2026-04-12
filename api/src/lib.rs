use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use thiserror::Error;

pub mod auth;
pub mod games;
pub mod logging;
// pub mod messages; // TODO: Module file missing
pub mod tributes;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Surreal<Any>>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound(String),
    #[error("Internal Server Error")]
    InternalServerError(String),
    #[error("Bad Request")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized(String),
    #[error("Game is full")]
    GameFull(String),
    #[error("Database error")]
    DbError(String),
    #[error("Invalid status")]
    InvalidStatus(String),
    #[error("Validation error")]
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
            AppError::InvalidStatus(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
