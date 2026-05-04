use crate::auth::{JWT_SECRET, RefreshToken, TokenResponse, store_refresh_token};
use crate::cookies::{set_refresh_cookie, set_session_cookie};
use crate::{AppError, AppState, AuthDb};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use shared::{RegistrationUser, UserSession};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    Json(payload): Json<RegistrationUser>,
) -> Result<Response, AppError> {
    // Validate the request
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let username = payload.username;
    let password = payload.password;

    // `db.signup` mutates connection-level session state (sets `$auth`
    // to the new user). Run it on a local clone of the shared
    // connection so the original root-authed handle is untouched and
    // concurrent requests can't observe the swapped `$auth`. The
    // refresh-token write below uses the same clone (the new user has
    // permission to insert into `refresh_token` per its schema). See
    // bd hangrier_games-c3ct (replaces the previous `auth_lock` guard).
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
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await;
    match result {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Extract user ID directly from JWT instead of querying the database.
            // Username is already in scope from the request payload.
            let user_id = extract_user_id_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&user_db, jwt, user_id, username).await?;
            Ok(token_response(token_pair))
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
) -> Result<Response, AppError> {
    // Validate the request - use generic error to not leak validation details
    payload
        .validate()
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let username = payload.username;
    let password = payload.password;

    // Same race protection as `user_create`: signin also mutates
    // connection-level `$auth`, so run it on a local clone. See
    // bd hangrier_games-c3ct.
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
            params: RegistrationUser {
                username: username.clone(),
                password: password.clone(),
            },
        })
        .await;
    match result {
        Ok(auth_result) => {
            let jwt = auth_result.into_insecure_token();

            // Extract user ID directly from JWT instead of querying the database.
            // Username is already in scope from the request payload.
            let user_id = extract_user_id_from_jwt(&jwt)?;

            let token_pair = create_token_pair(&user_db, jwt, user_id, username).await?;
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
