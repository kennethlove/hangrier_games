use crate::auth::{RefreshToken, TokenResponse, store_refresh_token};
use crate::cookies::{set_refresh_cookie, set_session_cookie};
use crate::{AppError, AppState, AuthDb};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use shared::{EmailRegistrationUser, UserSession};
use std::sync::LazyLock;
use surrealdb::opt::auth::Record;
use surrealdb::sql::Thing;
use validator::Validate;

/// Public users routes. Mounted in `main.rs` *outside* the `surreal_jwt`
/// middleware because both endpoints establish auth from credentials, not
/// from an existing session.
pub static USERS_PUBLIC_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", post(user_create))
        .route("/authenticate", post(user_authenticate))
});

/// Authenticated users routes. Must be mounted *behind* `surreal_jwt` so
/// `session` reads `$auth` from the per-request authed `AuthDb` clone
/// instead of the root-authed shared connection. See bd hangrier_games-p9p0.
pub static USERS_PROTECTED_ROUTER: LazyLock<Router<AppState>> =
    LazyLock::new(|| Router::new().route("/session", get(session)));

/// Helper function to create both access and refresh tokens.
///
/// Takes the per-request `Surreal<Any>` clone (already authenticated as
/// the new/returning user via `signup`/`signin`) so the refresh-token
/// write happens under the user's own `$auth`. See bd hangrier_games-c3ct.
async fn create_token_pair(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    jwt: String,
    user_id: Thing,
    username: String,
) -> Result<TokenResponse, AppError> {
    // Create and store refresh token
    let refresh_token = RefreshToken::new(user_id, username);
    store_refresh_token(db, &refresh_token).await?;

    Ok(TokenResponse {
        access_token: jwt,
        refresh_token: refresh_token.token,
    })
}

/// Returns the authenticated caller's user record (`id`, `username`).
///
/// Mounted behind `surreal_jwt`, so the per-request `AuthDb` clone has
/// already been `authenticate`d with the caller's JWT — `$auth` resolves
/// to their `user:` record. Raw `$session` is not exposed; we project to
/// the small `UserSession` shape the frontend actually needs.
async fn session(Extension(AuthDb(db)): Extension<AuthDb>) -> Result<Json<UserSession>, AppError> {
    #[derive(Deserialize)]
    struct AuthRow {
        id: Thing,
        username: String,
    }

    let mut response = db
        .query("SELECT id, username FROM $auth")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query session: {e}")))?;
    let row: Option<AuthRow> = response
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to read session result: {e}")))?;
    let row = row.ok_or_else(|| AppError::Unauthorized("No authenticated session".into()))?;
    Ok(Json(UserSession {
        id: row.id.to_string(),
        username: row.username,
    }))
}

async fn user_create(
    state: State<AppState>,
    Json(payload): Json<EmailRegistrationUser>,
) -> Result<Response, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user_db = (*state.db).clone();
    user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to scope signup session: {e}")))?;
    let result = user_db
        .signup(Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: &payload,
        })
        .await;
    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "created",
                "message": "Account created. Check your email for verification link."
            })),
        )
            .into_response()),
        Err(e) => {
            let combined = format!("{e} {e:?}").to_lowercase();
            if combined.contains("unique_email") {
                Err(AppError::Conflict("Email already taken".to_string()))
            } else {
                Err(AppError::DbError(format!("Failed to create user: {e}")))
            }
        }
    }
}

async fn user_authenticate(
    state: State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Email is required".to_string()))?;
    let password = payload
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Password is required".to_string()))?;

    let user_db = (*state.db).clone();
    user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to scope signin session: {e}")))?;
    let result = user_db
        .signin(Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: serde_json::json!({
                "email": email,
                "password": password,
            }),
        })
        .await;
    match result {
        Ok(_auth_result) => {
            let mut resp = user_db
                .query("SELECT id, username, email_verified FROM $auth")
                .await
                .map_err(|e| AppError::DbError(format!("Failed to query auth: {e}")))?;
            #[derive(Deserialize)]
            struct AuthRow {
                id: Thing,
                username: String,
                email_verified: Option<bool>,
            }
            let row: Option<AuthRow> = resp
                .take(0)
                .map_err(|e| AppError::DbError(format!("Failed to read auth result: {e}")))?;
            let AuthRow {
                id: user_id,
                username: display_name,
                email_verified,
            } = row.ok_or_else(|| AppError::Unauthorized("Authentication error".to_string()))?;

            if !email_verified.unwrap_or(false) {
                return Err(AppError::Unauthorized(
                    "Please verify your email before signing in".to_string(),
                ));
            }

            let jwt = _auth_result.into_insecure_token();
            let token_pair = create_token_pair(&user_db, jwt, user_id, display_name).await?;
            Ok(token_response(token_pair))
        }
        Err(_) => Err(AppError::Unauthorized("Invalid credentials".to_string())),
    }
}

/// Build the auth response: still returns the token JSON body for non-browser
/// clients (tests, scripts), but also sets HttpOnly `hg_session` and
/// `hg_refresh` cookies so browsers don't have to manage tokens themselves.
fn token_response(pair: TokenResponse) -> Response {
    let access = pair.access_token.clone();
    let refresh = pair.refresh_token.clone();
    let mut response = (StatusCode::OK, Json(pair)).into_response();
    set_session_cookie(&mut response, &access);
    set_refresh_cookie(&mut response, &refresh);
    response
}
