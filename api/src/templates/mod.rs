use maud::{DOCTYPE, Markup, PreEscaped, html};

pub mod auth;
pub mod game_detail;
pub mod pages;
pub mod timeline;

/// Authentication state passed to templates for conditional rendering.
#[derive(Clone, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
}

impl AuthState {
    pub fn authenticated(username: impl Into<String>) -> Self {
        Self {
            is_authenticated: true,
            username: Some(username.into()),
        }
    }

    pub fn guest() -> Self {
        Self {
            is_authenticated: false,
            username: None,
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
            @if auth.is_authenticated {
                a href="/account" { (auth.username.as_deref().unwrap_or("Account")) }
            } @else {
                a href="/login" { "Login" }
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
