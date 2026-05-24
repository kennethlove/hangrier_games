use crate::cookies::{
    CSRF_COOKIE, REFRESH_COOKIE, clear_auth_cookies, generate_csrf_token, read_cookie,
    set_refresh_cookie, set_session_cookie,
};
use crate::email::{
    generate_verification_token, send_password_reset_email, validate_verification_token,
};
use crate::templates::AuthState;
use crate::templates::auth::reset_form_page;
use crate::{AppError, AppState};
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
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
        .route(
            "/reset-password",
            get(show_reset_form).post(request_password_reset),
        )
        .route("/reset-password/complete", post(complete_password_reset))
});

#[derive(Deserialize)]
struct ResetPasswordRequest {
    email: String,
    #[serde(default)]
    #[allow(dead_code)]
    csrf_token: String,
}

#[derive(Deserialize)]
struct CompleteResetRequest {
    token: String,
    password: String,
    confirm_password: String,
    #[serde(default)]
    #[allow(dead_code)]
    csrf_token: String,
}

#[derive(Deserialize)]
struct ResetTokenQuery {
    token: String,
}

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
    // Build the row inline so SurrealDB parses the `record<user>` /
    // `datetime` literals natively, sidestepping the SDK serializer
    // (which collapses `Thing` and `Datetime` to opaque enum payloads
    // and triggers `FieldCheck` errors that the old `.await?` chain
    // silently swallowed). See bd hangrier_games-lkxg.
    let user_id_str = refresh_token.user_id.to_string();
    let mut response = db
        .query(
            "CREATE refresh_token CONTENT {
                token: $tk,
                user_id: type::thing($user_id),
                username: $username,
                expires_at: type::datetime($expires_at),
                revoked: $revoked,
                created_at: type::datetime($created_at),
            }",
        )
        .bind(("tk", refresh_token.token.clone()))
        .bind(("user_id", user_id_str))
        .bind(("username", refresh_token.username.clone()))
        .bind(("expires_at", refresh_token.expires_at.to_raw()))
        .bind(("revoked", refresh_token.revoked))
        .bind(("created_at", refresh_token.created_at.to_raw()))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to store refresh token: {}", e)))?;
    let _: Vec<RefreshToken> = response
        .take(0)
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

/// Generate a new access token for a user.
///
/// Produces a JWT whose claims mirror SurrealDB's own format (same `iss`,
/// `ns`, `db`, `ac`, `id`, HS512 + shared secret) so the SurrealDB SDK
/// accepts it for record-level authentication, *plus* a `sub` claim
/// carrying the username so display paths (`extract_auth_state`) can read
/// it without a DB round-trip.
pub fn generate_access_token(
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

/// Handle POST /auth/reset-password — request a password reset email.
///
/// Always redirects to `/auth?tab=login` to avoid disclosing whether
/// the email exists. If user exists and verified, generates a token
/// and sends the reset email asynchronously.
async fn request_password_reset(
    State(state): State<AppState>,
    Form(req): Form<ResetPasswordRequest>,
) -> Redirect {
    let email = req.email.trim().to_lowercase();
    let base = "/auth?tab=login";

    if email.is_empty() {
        return Redirect::to(base);
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to(base);
    }

    // Check user exists and is verified without revealing result
    let email_bind = email.clone();
    #[allow(clippy::result_large_err)]
    let result: Result<Vec<serde_json::Value>, _> = user_db
        .query("SELECT email, email_verified FROM user WHERE email = $email")
        .bind(("email", email_bind))
        .await
        .and_then(|mut r| r.take(0));

    if let Ok(rows) = result
        && let Some(row) = rows.into_iter().next()
    {
        let verified = row
            .get("email_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if verified {
            let token = match generate_verification_token(&email) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to generate reset token: {}", e);
                    return Redirect::to(base);
                }
            };

            let to = email.clone();
            tokio::spawn(async move {
                if let Err(e) = send_password_reset_email(&to, &token).await {
                    tracing::error!("Failed to send password reset email: {}", e);
                }
            });
        }
    }

    Redirect::to(base)
}

/// Handle GET /auth/reset-password?token=... — show the new-password form.
async fn show_reset_form(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ResetTokenQuery>,
) -> Response {
    match validate_verification_token(&query.token) {
        Ok(_email) => {
            let csrf = read_cookie(&headers, CSRF_COOKIE)
                .map(|s| s.to_owned())
                .unwrap_or_else(generate_csrf_token);
            Html(reset_form_page(
                AuthState::guest(csrf.clone()),
                &query.token,
                &csrf,
            ))
            .into_response()
        }
        Err(_) => Redirect::to("/auth?tab=login").into_response(),
    }
}

/// Handle POST /auth/reset-password/complete — update password.
async fn complete_password_reset(
    State(state): State<AppState>,
    Form(req): Form<CompleteResetRequest>,
) -> Redirect {
    // Validate passwords match
    if req.password != req.confirm_password {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("tab", "login")
            .append_pair("error", "Passwords do not match")
            .finish();
        return Redirect::to(&format!("/auth?{}", query));
    }

    // Validate password length
    if req.password.len() < 8 || req.password.len() > 72 {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("tab", "login")
            .append_pair("error", "Password must be 8-72 characters")
            .finish();
        return Redirect::to(&format!("/auth?{}", query));
    }

    // Validate token and get email
    let email = match validate_verification_token(&req.token) {
        Ok(email) => email,
        Err(_) => {
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("tab", "login")
                .append_pair("error", "Invalid or expired reset link")
                .finish();
            return Redirect::to(&format!("/auth?{}", query));
        }
    };

    // Update password using SurrealDB crypto function
    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("tab", "login")
            .append_pair("error", "Database error")
            .finish();
        return Redirect::to(&format!("/auth?{}", query));
    }

    let password_bind = req.password.clone();
    let email_bind = email.clone();
    #[allow(clippy::result_large_err)]
    let update_result: Result<Vec<serde_json::Value>, _> = user_db
        .query(
            "UPDATE user SET password = crypto::argon2::generate($password) WHERE email = $email",
        )
        .bind(("password", password_bind))
        .bind(("email", email_bind))
        .await
        .and_then(|mut r| r.take(0));

    match update_result {
        Ok(rows) if !rows.is_empty() => {
            Redirect::to("/auth?tab=login") // Password reset success
        }
        _ => {
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("tab", "login")
                .append_pair("error", "Failed to reset password")
                .finish();
            Redirect::to(&format!("/auth?{}", query))
        }
    }
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
