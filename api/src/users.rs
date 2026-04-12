use crate::auth::{RefreshToken, TokenResponse, store_refresh_token};
use crate::{AppError, AppState};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use shared::RegistrationUser;
use std::sync::LazyLock;
use surrealdb::opt::auth::Record;
use surrealdb::sql::Thing;
use validator::Validate;

pub static USERS_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(session).post(user_create))
        .route("/authenticate", post(user_authenticate))
});

#[derive(Serialize, Deserialize, Debug)]
struct JwtResponse {
    jwt: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserRecord {
    id: Thing,
    username: String,
}

/// Helper function to create both access and refresh tokens
async fn create_token_pair(
    state: &AppState,
    jwt: String,
    user_id: Thing,
    username: String,
) -> Result<TokenResponse, AppError> {
    // Create and store refresh token
    let refresh_token = RefreshToken::new(user_id, username);
    store_refresh_token(state, &refresh_token).await?;

    Ok(TokenResponse {
        access_token: jwt,
        refresh_token: refresh_token.token,
    })
}

async fn session(state: State<AppState>) -> Result<Json<String>, AppError> {
    let res: Option<String> = state
        .db
        .query("RETURN <string>$session")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    Ok(Json(res.unwrap_or("No session data found!".into())))
}

async fn user_create(
    state: State<AppState>,
    Json(payload): Json<RegistrationUser>,
) -> Result<Json<TokenResponse>, AppError> {
    // Validate the request
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let username = payload.username;
    let password = payload.password;

    match state
        .db
        .signup(Record {
            access: "user",
            namespace: "hangry-games",
            database: "games",
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await
    {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Query the user to get their ID
            let mut result = state
                .db
                .query("SELECT id, username FROM user WHERE username = $username LIMIT 1")
                .bind(("username", username.clone()))
                .await
                .map_err(|e| AppError::DbError(format!("Failed to query user: {}", e)))?;

            let users: Vec<UserRecord> = result
                .take(0)
                .map_err(|e| AppError::DbError(format!("Failed to parse user: {}", e)))?;

            let user = users
                .into_iter()
                .next()
                .ok_or_else(|| AppError::DbError("User created but not found".to_string()))?;

            let token_pair = create_token_pair(&state, jwt, user.id, user.username).await?;
            Ok(Json(token_pair))
        }
        Err(_) => Err(AppError::DbError("Failed to create user".to_string())),
    }
}

async fn user_authenticate(
    state: State<AppState>,
    Json(payload): Json<RegistrationUser>,
) -> Result<Json<TokenResponse>, AppError> {
    // Validate the request - use generic error to not leak validation details
    payload
        .validate()
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let username = payload.username;
    let password = payload.password;

    match state
        .db
        .signin(Record {
            access: "user",
            namespace: "hangry-games",
            database: "games",
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await
    {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Query the user to get their ID
            let mut result = state
                .db
                .query("SELECT id, username FROM user WHERE username = $username LIMIT 1")
                .bind(("username", username.clone()))
                .await
                .map_err(|e| AppError::DbError(format!("Failed to query user: {}", e)))?;

            let users: Vec<UserRecord> = result
                .take(0)
                .map_err(|e| AppError::DbError(format!("Failed to parse user: {}", e)))?;

            let user = users
                .into_iter()
                .next()
                .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

            let token_pair = create_token_pair(&state, jwt, user.id, user.username).await?;
            Ok(Json(token_pair))
        }
        Err(_) => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}
