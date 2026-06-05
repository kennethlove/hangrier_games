use crate::auth::{RefreshToken, TokenResponse, store_refresh_token};
use crate::cookies::{set_refresh_cookie, set_session_cookie};
use crate::{AppError, AppState, AuthDb};
use axum::extract::{Extension, Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use shared::{EmailRegistrationUser, UserSession};
use std::sync::LazyLock;
use surrealdb::opt::auth::Record;
use surrealdb::sql::Thing;
use validator::Validate;

use crate::storage::{UploadConstraints, validate_upload};

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
pub static USERS_PROTECTED_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/session", get(session))
        .route("/me", patch(update_account_settings))
        .route("/me/avatar", post(upload_avatar).delete(delete_avatar))
        .route("/me/password", post(change_password))
});

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
        avatar: Option<String>,
        account_status: String,
    }

    let mut response = db
        .query("SELECT id, username, avatar, account_status FROM $auth")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query session: {e}")))?;
    let row: Option<AuthRow> = response
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to read session result: {e}")))?;
    let row = row.ok_or_else(|| AppError::Unauthorized("No authenticated session".into()))?;
    Ok(Json(UserSession {
        id: row.id.to_string(),
        username: row.username,
        avatar: row.avatar,
        account_status: row.account_status,
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
            eprintln!("DEBUG_SIGNUP_ERROR: {e} {e:?}");
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

// ---------------------------------------------------------------------------
// Account settings
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
struct UpdateAccountRequest {
    #[validate(email)]
    email: Option<String>,
    #[validate(length(min = 3, max = 50))]
    username: Option<String>,
}

/// PATCH /api/users/me — partial update of email and/or username.
async fn update_account_settings(
    Extension(AuthDb(db)): Extension<AuthDb>,
    State(_): State<AppState>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<StatusCode, AppError> {
    if payload.email.is_none() && payload.username.is_none() {
        return Err(AppError::BadRequest(
            "At least one field (email, username) must be provided".into(),
        ));
    }

    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // If changing email, verify it isn't taken by another user
    if let Some(ref new_email) = payload.email {
        let mut resp = db
            .query("SELECT id FROM user WHERE email = $email AND id != $auth.id")
            .bind(("email", new_email.clone()))
            .await
            .map_err(|e| AppError::DbError(format!("Failed to check email: {e}")))?;
        let existing: Option<Thing> = resp
            .take(0)
            .map_err(|e| AppError::DbError(format!("Failed to read email check: {e}")))?;
        if existing.is_some() {
            return Err(AppError::Conflict("Email already taken".into()));
        }
    }

    // Build dynamic SET clause for provided fields
    let mut set_clauses: Vec<String> = Vec::new();
    if payload.email.is_some() {
        set_clauses.push("email = $email".to_string());
    }
    if payload.username.is_some() {
        set_clauses.push("username = $username".to_string());
    }

    let query_str = format!(
        "UPDATE user SET {} WHERE id = $auth.id",
        set_clauses.join(", ")
    );

    let mut q = db.query(&query_str);
    if let Some(email) = payload.email.clone() {
        q = q.bind(("email", email));
    }
    if let Some(username) = payload.username.clone() {
        q = q.bind(("username", username));
    }

    q.await
        .map_err(|e| AppError::DbError(format!("Failed to update account: {e}")))?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Avatar upload / delete
// ---------------------------------------------------------------------------

/// POST /api/users/me/avatar — upload a user avatar image.
async fn upload_avatar(
    State(state): State<AppState>,
    Extension(AuthDb(db)): Extension<AuthDb>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    // Fetch the authenticated user's ID for the storage path
    let mut auth_resp = db
        .query("SELECT id FROM $auth")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query auth: {e}")))?;
    #[derive(Deserialize)]
    struct AuthId {
        id: Thing,
    }
    let AuthId { id: user_id } = auth_resp
        .take::<Option<AuthId>>(0)
        .map_err(|e| AppError::DbError(format!("Failed to read auth id: {e}")))?
        .ok_or_else(|| AppError::Unauthorized("No authenticated session".into()))?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {e}")))?
    {
        if field.name().unwrap_or("") == "avatar" {
            filename = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {e}")))?
                    .to_vec(),
            );
            break;
        }
    }

    let file_data = file_data
        .ok_or_else(|| AppError::BadRequest("No file uploaded. Field 'avatar' required.".into()))?;
    let filename = filename.ok_or_else(|| AppError::BadRequest("No filename provided".into()))?;

    let constraints = UploadConstraints::default();
    validate_upload(&file_data, &filename, &constraints)?;

    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| AppError::ValidationError("Invalid filename".into()))?;

    // SurrealDB Thing key is opaque; use the full string with colon replaced
    let user_key = user_id.to_string().replace(':', "_");
    let storage_path = format!("avatars/user/{}.{}", user_key, extension);

    let saved_path = state.storage.save(&storage_path, &file_data).await?;
    let public_url = state.storage.public_url(&saved_path);

    db.query("UPDATE user SET avatar = $path WHERE id = $auth.id")
        .bind(("path", saved_path.clone()))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to update avatar: {e}")))?;

    Ok(Json(serde_json::json!({
        "url": public_url,
        "path": saved_path
    })))
}

/// DELETE /api/users/me/avatar — remove the user's avatar.
async fn delete_avatar(
    State(state): State<AppState>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<StatusCode, AppError> {
    let mut resp = db
        .query("SELECT avatar FROM $auth")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query avatar: {e}")))?;
    #[derive(Deserialize)]
    struct AvatarRow {
        avatar: Option<String>,
    }
    let row: Option<AvatarRow> = resp
        .take(0)
        .map_err(|e| AppError::DbError(format!("Failed to read avatar: {e}")))?;

    if let Some(AvatarRow { avatar: Some(path) }) = row {
        state.storage.delete(&path).await?;
    }

    db.query("UPDATE user SET avatar = NONE WHERE id = $auth.id")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to clear avatar: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Password change
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PasswordChangeRequest {
    current_password: String,
    new_password: String,
}

/// POST /api/users/me/password — change the user's password.
async fn change_password(
    State(state): State<AppState>,
    Extension(AuthDb(db)): Extension<AuthDb>,
    Json(payload): Json<PasswordChangeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate new password length
    let pw_len = payload.new_password.len();
    if !(8..=72).contains(&pw_len) {
        return Err(AppError::ValidationError(
            "New password must be between 8 and 72 characters".into(),
        ));
    }

    // Fetch current email for re-authentication
    let mut resp = db
        .query("SELECT email FROM $auth")
        .await
        .map_err(|e| AppError::DbError(format!("Failed to query email: {e}")))?;
    #[derive(Deserialize)]
    struct EmailRow {
        email: String,
    }
    let EmailRow { email } = resp
        .take::<Option<EmailRow>>(0)
        .map_err(|e| AppError::DbError(format!("Failed to read email: {e}")))?
        .ok_or_else(|| AppError::Unauthorized("No authenticated session".into()))?;

    // Verify current password by attempting signin
    let user_db = (*state.db).clone();
    user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .map_err(|e| AppError::DbError(format!("Failed to scope session: {e}")))?;
    user_db
        .signin(Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: serde_json::json!({
                "email": email,
                "password": payload.current_password,
            }),
        })
        .await
        .map_err(|_| AppError::Unauthorized("Current password is incorrect".to_string()))?;

    // Hash and set new password
    db.query("UPDATE user SET password = crypto::argon2::generate($password) WHERE id = $auth.id")
        .bind(("password", payload.new_password))
        .await
        .map_err(|e| AppError::DbError(format!("Failed to update password: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "updated" })))
}
