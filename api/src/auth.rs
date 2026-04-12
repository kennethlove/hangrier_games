use crate::{AppError, AppState};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use surrealdb::sql::Thing;
use time::OffsetDateTime;
use uuid::Uuid;

// JWT secret from schemas/users.surql
const JWT_SECRET: &str =
    "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA";

pub static AUTH_ROUTER: LazyLock<Router<AppState>> =
    LazyLock::new(|| Router::new().route("/refresh", post(refresh_token)));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefreshToken {
    pub token: String,
    pub user_id: Thing,
    pub username: String,
    pub expires_at: String,
    pub revoked: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl RefreshToken {
    /// Generate a new refresh token for a user
    pub fn new(user_id: Thing, username: String) -> Self {
        let token = Uuid::new_v4().to_string();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + time::Duration::days(7);

        RefreshToken {
            token,
            user_id,
            username,
            expires_at: expires_at.to_string(),
            revoked: false,
            created_at: now.to_string(),
        }
    }

    /// Check if the refresh token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }

        let now = OffsetDateTime::now_utc();
        if let Ok(expires_at) = OffsetDateTime::parse(
            &self.expires_at,
            &time::format_description::well_known::Rfc3339,
        ) {
            expires_at > now
        } else {
            false
        }
    }
}

/// Store a refresh token in the database
pub async fn store_refresh_token(
    state: &AppState,
    refresh_token: &RefreshToken,
) -> Result<(), AppError> {
    let _: Option<RefreshToken> = state
        .db
        .create("refresh_token")
        .content(refresh_token)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to store refresh token: {}", e)))?;
    Ok(())
}

/// Retrieve a refresh token from the database by token string
pub async fn get_refresh_token(state: &AppState, token: &str) -> Result<RefreshToken, AppError> {
    let mut result = state
        .db
        .query("SELECT * FROM refresh_token WHERE token = $token LIMIT 1")
        .bind(("token", token))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query refresh token: {}", e)))?;

    let tokens: Vec<RefreshToken> = result
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to parse refresh token: {}", e)))?;

    tokens
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))
}

/// Revoke a refresh token in the database
pub async fn revoke_refresh_token(state: &AppState, token: &str) -> Result<(), AppError> {
    let _: Option<RefreshToken> = state
        .db
        .query("UPDATE refresh_token SET revoked = true WHERE token = $token")
        .bind(("token", token))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to revoke refresh token: {}", e)))?
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to parse revoked token: {}", e)))?;
    Ok(())
}

/// Generate a new access token for a user
async fn generate_access_token(
    _state: &AppState,
    user_id: &Thing,
    username: &str,
) -> Result<String, AppError> {
    // Create JWT claims matching SurrealDB's format
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let exp = now + 3600; // 1 hour expiration

    let claims = serde_json::json!({
        "iss": "SurrealDB",
        "iat": now,
        "nbf": now,
        "exp": exp,
        "ns": "hangry-games",
        "db": "games",
        "ac": "user",
        "id": user_id.to_string(),
        "sub": username,
    });

    // Sign the JWT with HS512 algorithm
    let header = Header::new(Algorithm::HS512);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to generate JWT: {}", e)))?;

    Ok(token)
}

/// Refresh endpoint: exchange a refresh token for new access + refresh tokens
async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    // Retrieve the refresh token from the database
    let token = get_refresh_token(&state, &payload.refresh_token).await?;

    // Validate the token
    if !token.is_valid() {
        return Err(AppError::Unauthorized(
            "Refresh token expired or revoked".to_string(),
        ));
    }

    // Revoke the old refresh token (token rotation)
    revoke_refresh_token(&state, &payload.refresh_token).await?;

    // Generate new refresh token
    let new_refresh_token = RefreshToken::new(token.user_id.clone(), token.username.clone());
    store_refresh_token(&state, &new_refresh_token).await?;

    // Generate new access token
    let access_token = generate_access_token(&state, &token.user_id, &token.username).await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token.token,
    }))
}
