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
struct JwtClaims {
    // SurrealDB-issued JWTs serialize the record id as `ID` (uppercase).
    // Our own `generate_access_token` (api/src/auth.rs) emits `id` (lowercase).
    // Accept both via rename + alias.
    #[serde(rename = "ID", alias = "id")]
    id: String,
    // SurrealDB record-auth JWTs do NOT include `sub`; only our own tokens do.
    // Optional so signup/signin tokens decode successfully.
    #[serde(default, alias = "sub")]
    sub: Option<String>,
}

/// Helper function to extract the user record id from a JWT token.
///
/// Returns the parsed `Thing` for the user record. The username is intentionally
/// not returned here because SurrealDB-issued signup/signin JWTs do not carry a
/// `sub` claim — callers already have the username in scope and pass it through.
fn extract_user_id_from_jwt(jwt: &str) -> Result<Thing, AppError> {
    // Decode JWT with validation disabled since we trust our own tokens
    let mut validation = Validation::new(Algorithm::HS512);
    validation.validate_exp = false; // We just created this token, no need to validate expiration
    // SurrealDB-issued JWTs don't set `sub`; jsonwebtoken validates `sub` only
    // when a required value is configured, so default validation is fine.

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

    Ok(user_id)
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
            namespace: &state.namespace,
            database: &state.database,
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await
    {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Extract user ID directly from JWT instead of querying the database.
            // Username is already in scope from the request payload.
            let user_id = extract_user_id_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&state, jwt, user_id, username).await?;
            Ok(Json(token_pair))
        }
        Err(e) => {
            // SurrealDB returns a generic "record access signup query failed" error
            // for any signup-block failure; in this code path the only realistic
            // failure mode is the unique_username index, so map to 409 Conflict.
            let combined = format!("{e} {e:?}").to_lowercase();
            if combined.contains("unique_username")
                || combined.contains("already")
                || combined.contains("duplicate")
                || combined.contains("signup query failed")
            {
                Err(AppError::Conflict("Username already taken".to_string()))
            } else {
                Err(AppError::DbError(format!("Failed to create user: {e}")))
            }
        }
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
            namespace: &state.namespace,
            database: &state.database,
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await
    {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Extract user ID directly from JWT instead of querying the database.
            // Username is already in scope from the request payload.
            let user_id = extract_user_id_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&state, jwt, user_id, username).await?;
            Ok(Json(token_pair))
        }
        Err(_) => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}
