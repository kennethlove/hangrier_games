//! Client-side auth token helpers: expiry checking and refresh-token rotation.
//!
//! The frontend stores both an access token (`jwt`) and a `refresh_token` in
//! `LocalStorage` (see `storage::AppState`). This module exposes:
//!
//!   * [`token_seconds_remaining`] — read the access token's `exp` claim
//!     (without verifying the signature; the API verifies it on the server)
//!     and return how many seconds until it expires.
//!   * [`refresh_access_token`] — call `POST /api/auth/refresh` with the
//!     stored refresh token and return the new `(access_token, refresh_token)`
//!     pair. Callers are responsible for writing the new pair into storage.
//!   * [`maybe_refresh_token`] — combine the two: if the stored access token
//!     expires in ≤ `REFRESH_THRESHOLD_SECONDS` and a refresh token exists,
//!     rotate. Updates storage in place. Returns the (possibly refreshed)
//!     access token.

use crate::env::APP_API_HOST;
use crate::storage::{AppState, UsePersistent};
use serde::Deserialize;

/// Refresh once the access token has this many seconds (or fewer) remaining.
/// 60s gives a safe buffer for clock skew and request latency.
const REFRESH_THRESHOLD_SECONDS: i64 = 60;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

/// Return seconds remaining on a JWT's `exp` claim. Negative when expired.
/// Returns `None` if the token can't be decoded or has no `exp`.
pub fn token_seconds_remaining(jwt: &str) -> Option<i64> {
    let decoded = jwt_rustcrypto::decode_only(jwt).ok()?;
    let exp = decoded.payload.get("exp")?.as_i64()?;
    Some(exp - chrono::Utc::now().timestamp())
}

/// Exchange a refresh token for a fresh access+refresh pair via the API.
/// Returns the new `(access_token, refresh_token)` pair.
pub async fn refresh_access_token(refresh_token: &str) -> Result<(String, String), reqwest::Error> {
    let resp = reqwest::Client::new()
        .post(format!("{}/api/auth/refresh", APP_API_HOST))
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;
    Ok((resp.access_token, resp.refresh_token))
}

/// If the access token in `storage` is expired or near-expiry AND a refresh
/// token is available, rotate the pair via the API and persist the new tokens.
///
/// On any failure (no tokens, network error, server rejected), the stored
/// state is **cleared** because the session is no longer recoverable — the
/// user will be redirected to login on the next protected route render.
///
/// Returns the current (possibly refreshed) access token, or `None` if there
/// is no usable session.
pub async fn maybe_refresh_token(storage: &mut UsePersistent<AppState>) -> Option<String> {
    let state = storage.get();
    let jwt = state.jwt.clone()?;

    // If we can't decode the token at all, treat it as a broken session.
    let remaining = match token_seconds_remaining(&jwt) {
        Some(r) => r,
        None => {
            tracing::warn!("could not decode stored JWT; clearing session");
            clear_session(storage);
            return None;
        }
    };

    if remaining > REFRESH_THRESHOLD_SECONDS {
        return Some(jwt);
    }

    let Some(refresh) = state.refresh_token.clone() else {
        tracing::info!(
            "access token expiring (remaining={}s) but no refresh token; clearing session",
            remaining
        );
        clear_session(storage);
        return None;
    };

    match refresh_access_token(&refresh).await {
        Ok((new_access, new_refresh)) => {
            let mut new_state = storage.get();
            new_state.jwt = Some(new_access.clone());
            new_state.refresh_token = Some(new_refresh);
            storage.set(new_state);
            tracing::debug!("rotated access token");
            Some(new_access)
        }
        Err(e) => {
            tracing::warn!("refresh failed: {e}; clearing session");
            clear_session(storage);
            None
        }
    }
}

fn clear_session(storage: &mut UsePersistent<AppState>) {
    let mut s = storage.get();
    s.jwt = None;
    s.refresh_token = None;
    s.username = None;
    storage.set(s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    /// Build an unsigned JWT-shaped string with a given `exp` claim.
    /// `decode_only` does not verify signatures, so a placeholder is fine.
    fn make_jwt(exp: i64) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
        let payload =
            URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{},"sub":"test"}}"#, exp).as_bytes());
        format!("{}.{}.sig", header, payload)
    }

    #[test]
    fn future_exp_is_positive() {
        let exp = chrono::Utc::now().timestamp() + 600;
        let remaining = token_seconds_remaining(&make_jwt(exp)).unwrap();
        assert!(remaining > 590 && remaining <= 600, "got {remaining}");
    }

    #[test]
    fn past_exp_is_negative() {
        let exp = chrono::Utc::now().timestamp() - 60;
        let remaining = token_seconds_remaining(&make_jwt(exp)).unwrap();
        assert!(remaining < 0, "got {remaining}");
    }

    #[test]
    fn garbage_returns_none() {
        assert!(token_seconds_remaining("not.a.jwt").is_none());
        assert!(token_seconds_remaining("").is_none());
    }

    #[test]
    fn missing_exp_returns_none() {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"test"}"#);
        let jwt = format!("{}.{}.sig", header, payload);
        assert!(token_seconds_remaining(&jwt).is_none());
    }
}
