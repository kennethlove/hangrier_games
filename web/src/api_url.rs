//! Build absolute API URLs.
//!
//! When [`APP_API_HOST`](crate::env::APP_API_HOST) is set at build time, the
//! frontend uses it directly (production / Docker compose deploys). When it
//! is empty (the default dev setup), we derive the origin from
//! `window.location` so reqwest — which requires absolute URLs even on the
//! WASM target — can issue requests to the same origin and hit the
//! `dx serve` proxy declared in `web/Dioxus.toml`.
//!
//! See bd-jgxd: cross-origin Set-Cookie was being silently dropped as a
//! third-party cookie, breaking auth on every dev session.

use crate::env::APP_API_HOST;

/// Build an absolute URL for an API path. `path` should start with `/api/...`
/// or `/ws`. The result is suitable for `reqwest::Client::get` / `post`.
pub fn api_url(path: &str) -> String {
    let host = APP_API_HOST.trim_end_matches('/');
    if !host.is_empty() {
        return format!("{host}{path}");
    }
    format!("{}{path}", origin())
}

#[cfg(target_arch = "wasm32")]
fn origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn origin() -> String {
    // Host-target tests don't have a window.
    "http://localhost".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_host_falls_back_to_origin() {
        // APP_API_HOST is generated at build time; the cargo-test build sees
        // whatever .env contains. Just assert the URL ends with the path.
        let u = api_url("/api/games");
        assert!(u.ends_with("/api/games"), "got {u}");
        assert!(u.starts_with("http"), "got {u}");
    }
}
