use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use thiserror::Error;

pub mod games;
pub mod tributes;
pub mod logging;
pub mod messages;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub db: Surreal<Any>,
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
        };
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}

