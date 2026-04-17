use crate::auth::{JWT_SECRET, RefreshToken, TokenResponse, store_refresh_token};
use crate::{AppError, AppState};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
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
struct JwtClaims {
    id: String,
    #[serde(rename = "sub")]
    username: String,
}

/// Helper function to extract user ID and username from JWT token
fn extract_user_from_jwt(jwt: &str) -> Result<(Thing, String), AppError> {
    // Decode JWT with validation disabled since we trust our own tokens
    let mut validation = Validation::new(Algorithm::HS512);
    validation.validate_exp = false; // We just created this token, no need to validate expiration

    let token_data = decode::<JwtClaims>(
        jwt,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to decode JWT: {}", e)))?;

    let claims = token_data.claims;

    // Parse the id claim into a Thing
    let user_id: Thing = surrealdb::sql::thing(&claims.id).map_err(|e| {
        AppError::InternalServerError(format!("Failed to parse user ID from JWT: {}", e))
    })?;

    Ok((user_id, claims.username))
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

            // Extract user ID directly from JWT instead of querying the database
            let (user_id, username) = extract_user_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&state, jwt, user_id, username).await?;
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

            // Extract user ID directly from JWT instead of querying the database
            let (user_id, username) = extract_user_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&state, jwt, user_id, username).await?;
            Ok(Json(token_pair))
        }
        Err(_) => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}
