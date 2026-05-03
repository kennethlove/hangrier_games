//! Cookie helpers for HttpOnly session + refresh-token cookies.
//!
//! Auth tokens are delivered to the browser via `Set-Cookie` so they live
//! outside of JavaScript reach (HttpOnly), are scoped to the origin, and are
//! sent automatically on every request (including the WebSocket upgrade
//! handshake) without the frontend having to manage `Authorization` headers
//! or `localStorage`.
//!
//! - `hg_session` carries the SurrealDB-issued JWT (1h lifetime, mirrors
//!   `generate_access_token`).
//! - `hg_refresh` carries the rotation token (7d lifetime, mirrors
//!   `RefreshToken::new`). Path is scoped to `/api/auth` so it isn't sent on
//!   every request — only the refresh / logout endpoints need it.
//!
//! In production (`ENV != "development"`) cookies are emitted as `Secure` and
//! `SameSite=Lax`. In development we omit `Secure` so they work over plain
//! `http://127.0.0.1`.

use axum::http::HeaderValue;
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::response::Response;

pub const SESSION_COOKIE: &str = "hg_session";
pub const REFRESH_COOKIE: &str = "hg_refresh";

const SESSION_MAX_AGE: i64 = 3600; // 1 hour, matches generate_access_token
const REFRESH_MAX_AGE: i64 = 7 * 24 * 3600; // 7 days, matches RefreshToken::new

fn is_secure() -> bool {
    std::env::var("ENV")
        .map(|v| v != "development")
        .unwrap_or(true)
}

fn build_cookie(name: &str, value: &str, path: &str, max_age: i64) -> String {
    let secure = if is_secure() { "; Secure" } else { "" };
    format!("{name}={value}; HttpOnly; SameSite=Lax; Path={path}; Max-Age={max_age}{secure}",)
}

fn build_clear_cookie(name: &str, path: &str) -> String {
    let secure = if is_secure() { "; Secure" } else { "" };
    format!("{name}=; HttpOnly; SameSite=Lax; Path={path}; Max-Age=0{secure}")
}

/// Attach a session cookie carrying the access JWT.
pub fn set_session_cookie(response: &mut Response, jwt: &str) {
    append_cookie(
        response,
        &build_cookie(SESSION_COOKIE, jwt, "/", SESSION_MAX_AGE),
    );
}

/// Attach a refresh cookie scoped to `/api/auth`.
pub fn set_refresh_cookie(response: &mut Response, token: &str) {
    append_cookie(
        response,
        &build_cookie(REFRESH_COOKIE, token, "/api/auth", REFRESH_MAX_AGE),
    );
}

/// Clear both auth cookies.
pub fn clear_auth_cookies(response: &mut Response) {
    append_cookie(response, &build_clear_cookie(SESSION_COOKIE, "/"));
    append_cookie(response, &build_clear_cookie(REFRESH_COOKIE, "/api/auth"));
}

fn append_cookie(response: &mut Response, cookie: &str) {
    if let Ok(value) = HeaderValue::from_str(cookie) {
        response.headers_mut().append(SET_COOKIE, value);
    }
}

/// Extract a cookie value from the `Cookie` request header.
pub fn read_cookie<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    let raw = headers.get(COOKIE)?.to_str().ok()?;
    for pair in raw.split(';') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once('=')
            && k == name
        {
            return Some(v);
        }
    }
    None
}
