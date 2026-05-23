use maud::{DOCTYPE, Markup, PreEscaped, html};

pub mod auth;
pub mod game_detail;
pub mod pages;
pub mod timeline;

/// Authentication state passed to templates for conditional rendering.
///
/// Modeled as an enum so illegal combinations (authenticated user with no
/// username, or any state without a CSRF token) are unrepresentable.
#[derive(Clone)]
pub enum AuthState {
    Guest {
        csrf_token: String,
    },
    Authenticated {
        id: String,
        username: String,
        csrf_token: String,
    },
}

impl AuthState {
    pub fn guest(csrf: impl Into<String>) -> Self {
        AuthState::Guest {
            csrf_token: csrf.into(),
        }
    }

    pub fn authenticated(
        id: impl Into<String>,
        username: impl Into<String>,
        csrf: impl Into<String>,
    ) -> Self {
        AuthState::Authenticated {
            id: id.into(),
            username: username.into(),
            csrf_token: csrf.into(),
        }
    }

    pub fn csrf_token(&self) -> &str {
        match self {
            AuthState::Guest { csrf_token } | AuthState::Authenticated { csrf_token, .. } => {
                csrf_token
            }
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated { .. })
    }

    pub fn username(&self) -> Option<&str> {
        match self {
            AuthState::Authenticated { username, .. } => Some(username),
            AuthState::Guest { .. } => None,
        }
    }
}

pub fn base_layout(title: &str, auth: AuthState, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Hangry Games" }
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link href="https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,16..72,200..800;1,16..72,200..800&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/assets/main.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                script src="https://unpkg.com/htmx-ext-sse@2.2.3" {}
            }
            body {
                // SVG sprites served as static files
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-ui.svg"/></svg>"#))
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-narrative.svg"/></svg>"#))
                header class="topnav" {
                    div class="container topnav-inner" {
                        div class="logo" {
                            "Hangry "
                            span { "Games" }
                        }
                        nav {
                            a href="/games" class="active" { "Broadcast" }
                            a href="#" { "Tributes" }
                            a href="#" { "Arena" }
                            a href="#" { "Odds" }
                        }
                        (auth_links(&auth))
                    }
                }
                main {
                    (content)
                }
                footer class="pagefoot" {
                    div class="container row-between" {
                        span { "© Hangry Games" }
                        span class="num" { "Server v0.1.15" }
                    }
                }
            }
        }
    }
}

/// Render auth links based on authentication state.
fn auth_links(auth: &AuthState) -> Markup {
    html! {
        div class="auth-links" {
            @match auth {
                AuthState::Authenticated { username, csrf_token, .. } => {
                    a href="/account" { (username) }
                    form class="logout-form" method="POST" action="/auth/logout" {
                        input type="hidden" name="csrf_token" value=(csrf_token) {}
                        button type="submit" class="logout-btn" { "Logout" }
                    }
                }
                AuthState::Guest { .. } => {
                    a href="/auth" { "Login" }
                }
            }
        }
    }
}

pub fn icon(name: &str) -> Markup {
    html! {
        svg class="icon" {
            use href=(format!("#icon_ui_{}", name)) {}
        }
    }
}

pub fn narrative_icon(name: &str) -> Markup {
    html! {
        svg class="icon" {
            use href=(format!("#icon_narrative_{}", name)) {}
        }
    }
}
