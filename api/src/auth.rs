use crate::cookies::{
    REFRESH_COOKIE, clear_auth_cookies, read_cookie, set_refresh_cookie, set_session_cookie,
};
use crate::{AppError, AppState};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use surrealdb::sql::{Datetime, Thing};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

// JWT secret from schemas/users.surql
pub const JWT_SECRET: &str =
    "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA";

pub static AUTH_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefreshToken {
    pub token: String,
    pub user_id: Thing,
    pub username: String,
    pub expires_at: Datetime,
    pub revoked: bool,
    pub created_at: Datetime,
}

#[derive(Serialize, Deserialize, Debug, Validate, Default)]
pub struct RefreshTokenRequest {
    /// Optional in the request body — preferred source is the `hg_refresh`
    /// HttpOnly cookie. Body is kept for non-browser clients (tests, scripts).
    #[serde(default)]
    #[validate(length(min = 36, max = 36, message = "Invalid refresh token format"))]
    pub refresh_token: Option<String>,
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
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        RefreshToken {
            token,
            user_id,
            username,
            expires_at: Datetime::from(expires_at),
            revoked: false,
            created_at: Datetime::from(now),
        }
    }

    /// Check if the refresh token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }

        self.expires_at.0 > Utc::now()
    }
}

/// Store a refresh token in the database. Uses the caller-provided
/// `Surreal<Any>` (typically a per-request clone, see bd hangrier_games-c3ct)
/// so the write happens under the right `$auth` for `refresh_token` table
/// permissions.
pub async fn store_refresh_token(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    refresh_token: &RefreshToken,
) -> Result<(), AppError> {
    // Bind via serde_json::Value to bypass the SurrealDB SDK's bespoke
    // type serializer (same workaround as save_game in games.rs), which
    // can collapse externally-tagged enums and Option fields.
    let body = serde_json::to_value(refresh_token)
        .map_err(|e| AppError::DbError(format!("Failed to encode refresh token: {}", e)))?;
    db.query("CREATE refresh_token CONTENT $body")
        .bind(("body", body))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to store refresh token: {}", e)))?;
    Ok(())
}

/// Retrieve a refresh token from the database by token string
pub async fn get_refresh_token(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    token: &str,
) -> Result<RefreshToken, AppError> {
    let token_owned = token.to_string();
    let mut result = db
        .query("SELECT * FROM refresh_token WHERE token = $tk LIMIT 1")
        .bind(("tk", token_owned))
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
pub async fn revoke_refresh_token(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    token: &str,
) -> Result<(), AppError> {
    let token_owned = token.to_string();
    let _: Option<RefreshToken> = db
        .query("UPDATE refresh_token SET revoked = true WHERE token = $tk")
        .bind(("tk", token_owned))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to revoke refresh token: {}", e)))?
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to parse revoked token: {}", e)))?;
    Ok(())
}

/// Generate a new access token for a user
fn generate_access_token(
    user_id: &Thing,
    username: &str,
    namespace: &str,
    database: &str,
) -> Result<String, AppError> {
    // Create JWT claims matching SurrealDB's format
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let exp = now + 3600; // 1 hour expiration

    let claims = serde_json::json!({
        "iss": "SurrealDB",
        "iat": now,
        "nbf": now,
        "exp": exp,
        "ns": namespace,
        "db": database,
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

/// Refresh endpoint: exchange a refresh token for new access + refresh tokens.
///
/// The token is read from the `hg_refresh` HttpOnly cookie (preferred) or,
/// for non-browser clients, from the JSON body. New tokens are returned in
/// the JSON body **and** as fresh `Set-Cookie` headers.
async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response, AppError> {
    let refresh = read_cookie(&headers, REFRESH_COOKIE)
        .map(|s| s.to_owned())
        .or_else(|| body.and_then(|Json(b)| b.refresh_token))
        .ok_or_else(|| AppError::Unauthorized("Missing refresh token".to_string()))?;

    if refresh.len() != 36 {
        return Err(AppError::ValidationError(
            "Invalid refresh token format".to_string(),
        ));
    }

    // refresh_token table is permission-gated by `$auth`; the request
    // is unauthenticated (no JWT yet) so use a per-request clone of the
    // shared root-authed connection. The clone keeps the original
    // session untouched. See bd hangrier_games-c3ct.
    let user_db = (*state.db).clone();
    user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to scope refresh session: {e}")))?;

    // Retrieve the refresh token from the database
    let token = get_refresh_token(&user_db, &refresh).await?;

    // Validate the token
    if !token.is_valid() {
        return Err(AppError::Unauthorized(
            "Refresh token expired or revoked".to_string(),
        ));
    }

    // Revoke the old refresh token (token rotation)
    revoke_refresh_token(&user_db, &refresh).await?;

    // Generate new refresh token
    let new_refresh_token = RefreshToken::new(token.user_id.clone(), token.username.clone());
    store_refresh_token(&user_db, &new_refresh_token).await?;

    // Generate new access token
    let access_token = generate_access_token(
        &token.user_id,
        &token.username,
        &state.namespace,
        &state.database,
    )?;

    let pair = TokenResponse {
        access_token: access_token.clone(),
        refresh_token: new_refresh_token.token.clone(),
    };
    let mut response = (StatusCode::OK, Json(pair)).into_response();
    set_session_cookie(&mut response, &access_token);
    set_refresh_cookie(&mut response, &new_refresh_token.token);
    Ok(response)
}

/// Logout endpoint: revoke the refresh token (if known) and clear cookies.
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<Response, AppError> {
    let refresh = read_cookie(&headers, REFRESH_COOKIE)
        .map(|s| s.to_owned())
        .or_else(|| body.and_then(|Json(b)| b.refresh_token));

    if let Some(token) = refresh {
        // Best-effort revoke; clearing the cookie still happens either way.
        // Use a per-request clone (see bd hangrier_games-c3ct).
        let user_db = (*state.db).clone();
        if user_db
            .use_ns(&state.namespace)
            .use_db(&state.database)
            .await
            .is_ok()
        {
            let _ = revoke_refresh_token(&user_db, &token).await;
        }
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    clear_auth_cookies(&mut response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_token_generation() {
        let user_id = Thing::from(("user".to_string(), "test123".to_string()));
        let username = "testuser".to_string();

        let token = RefreshToken::new(user_id.clone(), username.clone());

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.username, username);
        assert_eq!(token.token.len(), 36); // UUID v4 length
        assert!(!token.revoked);
        assert!(token.expires_at.0 > Utc::now());
    }

    #[test]
    fn test_expired_token_invalid() {
        let user_id = Thing::from(("user".to_string(), "test123".to_string()));
        let username = "testuser".to_string();
        let token_str = Uuid::new_v4().to_string();

        // Create a token that expired 1 day ago
        let now = Utc::now();
        let expired = now - chrono::Duration::days(1);

        let token = RefreshToken {
            token: token_str,
            user_id,
            username,
            expires_at: Datetime::from(expired),
            revoked: false,
            created_at: Datetime::from(now - chrono::Duration::days(8)),
        };

        assert!(!token.is_valid());
    }

    #[test]
    fn test_revoked_token_invalid() {
        let user_id = Thing::from(("user".to_string(), "test123".to_string()));
        let username = "testuser".to_string();
        let token_str = Uuid::new_v4().to_string();

        // Create a token that's not expired but is revoked
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        let token = RefreshToken {
            token: token_str,
            user_id,
            username,
            expires_at: Datetime::from(expires_at),
            revoked: true,
            created_at: Datetime::from(now),
        };

        assert!(!token.is_valid());
    }
}
