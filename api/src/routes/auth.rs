use crate::{
    RESEND_COOLDOWN, extract_auth, html_with_csrf, redirect_with_error, urlencoding, validate_csrf,
};
use api::AppState;
use api::auth::{
    RefreshToken, TokenResponse, generate_access_token, revoke_refresh_token, store_refresh_token,
};
use api::cookies::{REFRESH_COOKIE, clear_auth_cookies, clear_csrf_cookie, read_cookie};
use api::email::{
    generate_verification_token, send_verification_email, validate_verification_token,
};
use api::templates::auth::{self, AuthTab};
use axum::extract::{Form, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use validator::Validate;

// ── Auth form types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AuthTabQuery {
    pub tab: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub csrf_token: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub display_name: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    #[serde(default)]
    pub csrf_token: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CheckEmailQuery {
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
    #[serde(default)]
    pub csrf_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    pub csrf_token: String,
}

// ── Auth page handlers ──────────────────────────────────────────────

/// GET /auth — render unified auth form with CSRF token.
pub async fn auth_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<AuthTabQuery>,
) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    if auth.is_authenticated() {
        return Redirect::to("/games").into_response();
    }
    let body = auth::auth_page_with_csrf(
        auth,
        &csrf,
        params.error.as_deref(),
        AuthTab::from_query(params.tab.as_deref()),
    );
    html_with_csrf(body, &csrf)
}

/// POST /login — authenticate user.
pub async fn login_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LoginRequest>,
) -> impl IntoResponse {
    handle_login_post(&state, &headers, form.email, form.password, form.csrf_token).await
}

/// POST /register — create new user account.
pub async fn register_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<RegisterRequest>,
) -> impl IntoResponse {
    handle_register_post(
        &state,
        &headers,
        form.display_name,
        form.email,
        form.password,
        form.confirm_password,
        form.csrf_token,
    )
    .await
}

/// Handle login POST logic.
pub async fn handle_login_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    login: String,
    password: String,
    csrf_token: String,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return redirect_with_error("/auth", "login", "Invalid form submission");
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return redirect_with_error("/auth", "login", "Database error");
    }

    let result = user_db
        .signin(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: serde_json::json!({
                "email": login.clone(),
                "password": password,
            }),
        })
        .await;

    match result {
        Ok(_auth_result) => {
            use serde::Deserialize as SerdeDeserialize;

            #[derive(SerdeDeserialize)]
            struct AuthRow {
                id: surrealdb::sql::Thing,
                username: String,
                email_verified: Option<bool>,
            }

            let mut resp = match user_db
                .query("SELECT id, username, email_verified FROM $auth")
                .await
            {
                Ok(r) => r,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };
            let row: Option<AuthRow> = match resp.take(0) {
                Ok(r) => r,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };
            let AuthRow {
                id: user_id,
                username: display_name,
                email_verified,
            } = match row {
                Some(r) => r,
                None => return redirect_with_error("/auth", "login", "Authentication error"),
            };

            if !email_verified.unwrap_or(false) {
                return redirect_with_error(
                    "/auth",
                    "login",
                    "Please verify your email before signing in",
                );
            }

            let access_token = match generate_access_token(
                &user_id,
                &display_name,
                &state.namespace,
                &state.database,
            ) {
                Ok(t) => t,
                Err(_) => return redirect_with_error("/auth", "login", "Authentication error"),
            };

            let refresh_token = RefreshToken::new(user_id, display_name);
            if store_refresh_token(&user_db, &refresh_token).await.is_err() {
                return redirect_with_error("/auth", "login", "Session error");
            }

            let pair = TokenResponse {
                access_token,
                refresh_token: refresh_token.token,
            };

            let mut response = Redirect::to("/account").into_response();
            api::cookies::set_session_cookie(&mut response, &pair.access_token);
            api::cookies::set_refresh_cookie(&mut response, &pair.refresh_token);
            response
        }
        Err(_) => redirect_with_error("/auth", "login", "Invalid email or username"),
    }
}

/// Handle register POST logic.
pub async fn handle_register_post(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    display_name: String,
    email: String,
    password: String,
    confirm_password: String,
    csrf_token: String,
) -> Response {
    if !validate_csrf(headers, &csrf_token) {
        return redirect_with_error("/auth", "register", "Invalid form submission");
    }

    if password != confirm_password {
        return redirect_with_error("/auth", "register", "Passwords do not match");
    }

    let reg_user = shared::EmailRegistrationUser {
        display_name: display_name.clone(),
        email: email.clone(),
        password: password.clone(),
    };
    if let Err(e) = reg_user.validate() {
        return redirect_with_error("/auth", "register", &e.to_string());
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return redirect_with_error("/auth", "register", "Database error");
    }

    let result = user_db
        .signup(surrealdb::opt::auth::Record {
            access: "user",
            namespace: &state.namespace,
            database: &state.database,
            params: reg_user,
        })
        .await;

    match result {
        Ok(_auth_result) => {
            let email_for_token = email.clone();
            tokio::spawn(async move {
                match generate_verification_token(&email_for_token) {
                    Ok(token) => {
                        if let Err(e) = send_verification_email(&email_for_token, &token).await {
                            tracing::error!(
                                "Failed to send verification email to {}: {}",
                                email_for_token,
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to generate verification token for {}: {}",
                            email_for_token,
                            e
                        );
                    }
                }
            });

            Redirect::to(&format!(
                "/auth/check-email?address={}",
                urlencoding(&email)
            ))
            .into_response()
        }
        Err(e) => {
            let combined = format!("{e} {e:?}").to_lowercase();
            if combined.contains("unique_email") || combined.contains("already exists") {
                return redirect_with_error(
                    "/auth",
                    "register",
                    "An account with this email already exists",
                );
            }
            tracing::warn!("Registration failed with unrecognized error: {}", e);
            redirect_with_error("/auth", "register", "Registration failed")
        }
    }
}

/// GET /auth/check-email — interstitial page shown after registration.
pub async fn check_email_handler(
    headers: axum::http::HeaderMap,
    Query(params): Query<CheckEmailQuery>,
) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    let body = auth::check_email_page(auth, params.address.as_deref(), &csrf);
    html_with_csrf(body, &csrf)
}

/// GET /auth/verify-email?token=... — verify email address.
pub async fn verify_email_handler(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> Response {
    let email = match validate_verification_token(&params.token) {
        Ok(email) => email,
        Err(_) => {
            return Redirect::to("/auth?tab=login&error=Invalid+or+expired+verification+link")
                .into_response();
        }
    };

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth?tab=login&error=Database+error").into_response();
    }

    let result = user_db
        .query("UPDATE user SET email_verified = true WHERE email = $email")
        .bind(("email", email.clone()))
        .await;

    match result {
        Ok(_) => Redirect::to("/auth/email-verified").into_response(),
        Err(e) => {
            tracing::error!("Failed to verify email {}: {}", email, e);
            Redirect::to("/auth?tab=login&error=Verification+failed.+Please+try+again.")
                .into_response()
        }
    }
}

/// POST /auth/resend-verification — resend verification email.
pub async fn resend_verification_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<ResendVerificationRequest>,
) -> impl IntoResponse {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let now = std::time::Instant::now();
    let cooldown_key = format!("resend:{}", form.email.to_lowercase());
    if let Some(last_sent) = RESEND_COOLDOWN.lock().unwrap().get(&cooldown_key)
        && now.duration_since(*last_sent).as_secs() < 60
    {
        return Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Please wait 60 seconds before requesting another email.")
        ))
        .into_response();
    }

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Something went wrong. Please try again.")
        ))
        .into_response();
    }

    let result = user_db
        .query("SELECT email_verified FROM user WHERE email = $email LIMIT 1")
        .bind(("email", form.email.clone()))
        .await;

    match result {
        Ok(mut resp) => {
            #[derive(serde::Deserialize)]
            struct EmailRow {
                email_verified: Option<bool>,
            }
            let row: Option<EmailRow> = resp.take(0).unwrap_or(None);
            match row {
                Some(r) if r.email_verified.unwrap_or(false) => {
                    Redirect::to("/auth?tab=login&error=Email+already+verified").into_response()
                }
                Some(_) => {
                    let email_for_token = form.email.clone();
                    tokio::spawn(async move {
                        match api::email::generate_verification_token(&email_for_token) {
                            Ok(token) => {
                                if let Err(e) =
                                    api::email::send_verification_email(&email_for_token, &token)
                                        .await
                                {
                                    tracing::error!("Resend failed for {}: {}", email_for_token, e);
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Token generation failed for {}: {}",
                                    email_for_token,
                                    e
                                );
                            }
                        }
                    });

                    RESEND_COOLDOWN.lock().unwrap().insert(cooldown_key, now);

                    Redirect::to(&format!(
                        "/auth/check-email?address={}",
                        urlencoding(&form.email)
                    ))
                    .into_response()
                }
                None => Redirect::to(&format!(
                    "/auth/check-email?address={}",
                    urlencoding(&form.email)
                ))
                .into_response(),
            }
        }
        Err(_) => Redirect::to(&format!(
            "/auth/check-email?address={}&error={}",
            urlencoding(&form.email),
            urlencoding("Something went wrong. Please try again.")
        ))
        .into_response(),
    }
}

/// GET /auth/email-verified — confirmation page after email verification.
pub async fn email_verified_handler(headers: axum::http::HeaderMap) -> impl IntoResponse {
    let (auth, csrf) = extract_auth(&headers);
    let body = auth::email_verified_page(auth);
    html_with_csrf(body, &csrf)
}

/// POST /logout — revoke refresh token, clear cookies, redirect to /auth.
pub async fn logout_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<LogoutRequest>,
) -> impl IntoResponse {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let refresh = read_cookie(&headers, REFRESH_COOKIE).map(|s| s.to_owned());

    if let Some(token) = refresh {
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

    let mut response = Redirect::to("/auth?tab=login").into_response();
    clear_auth_cookies(&mut response);
    clear_csrf_cookie(&mut response);
    response
}
