use maud::{PreEscaped, html};
use shared::{ListDisplayGame, UserSession};

use super::pages::status_color;
use super::{AuthState, base_layout, icon};

/// Which tab to activate by default on the auth page.
#[derive(Clone, Copy, Default)]
pub enum AuthTab {
    #[default]
    Login,
    Register,
    Reset,
}

impl AuthTab {
    pub fn from_query(tab: Option<&str>) -> Self {
        match tab {
            Some("login") => AuthTab::Login,
            Some("register") => AuthTab::Register,
            Some("reset") => AuthTab::Reset,
            _ => AuthTab::default(),
        }
    }
}

/// Unified tabbed auth page with login, register, and reset panels.
pub fn auth_page(auth: AuthState, error: Option<&str>, default_tab: AuthTab) -> maud::Markup {
    base_layout(
        "Sign In",
        auth,
        html! {
            div class="auth-card" {
                div class="auth-logo" { "Hangry " span { "Games" } }
                div class="tab-bar" {
                    @let login_btn_active = matches!(default_tab, AuthTab::Login);
                    @let register_btn_active = matches!(default_tab, AuthTab::Register);
                    @let reset_btn_active = matches!(default_tab, AuthTab::Reset);
                    button class=(if login_btn_active { "tab-btn active" } else { "tab-btn" }) data-tab="login" { "Sign In" }
                    button class=(if register_btn_active { "tab-btn active" } else { "tab-btn" }) data-tab="register" { "Register" }
                    button class=(if reset_btn_active { "tab-btn active" } else { "tab-btn" }) data-tab="reset" { "Reset Password" }
                }

                @let login_active = matches!(default_tab, AuthTab::Login);
                @let register_active = matches!(default_tab, AuthTab::Register);
                @let reset_active = matches!(default_tab, AuthTab::Reset);

                // ── Login tab ──
                div class=(if login_active { "tab-panel active" } else { "tab-panel" }) id="login" {
                    h2 class="auth-title" { "Welcome back" }
                    p class="auth-subtitle" { "Sign in to your account to continue." }

                    @if login_active {
                        @if let Some(err) = error {
                            div class="error-banner" { (err) }
                        }
                    }

                    form method="POST" action="/auth/login" {
                        input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                        div class="form-group" {
                            label for="username" { "Username" }
                            input type="text" id="username" name="username"
                                required minlength="3" maxlength="50"
                                placeholder="Your username";
                        }

                        div class="form-group" {
                            label for="password" { "Password" }
                            input type="password" id="password" name="password"
                                required minlength="8" maxlength="72"
                                placeholder="Your password";
                        }

                        div class="form-row" {
                            label {
                                input type="checkbox" name="remember" value="true";
                                " Remember me"
                            }
                            a href="#" onclick="switchTab('reset');return false;" {
                                "Forgot password?"
                            }
                        }

                        button type="submit" class="btn btn-primary"
                            hx-indicator="#login-spinner" {
                            "Sign In"
                            span id="login-spinner" class="htmx-indicator" {
                                (spinner_icon())
                            }
                        }
                    }

                    div class="auth-footer" {
                        "Don't have an account? "
                        a href="#" onclick="switchTab('register');return false;" {
                            "Create one"
                        }
                    }
                }

                // ── Register tab ──
                div class=(if register_active { "tab-panel active" } else { "tab-panel" }) id="register" {
                    h2 class="auth-title" { "Create account" }
                    p class="auth-subtitle" { "Join the broadcast. No credit card required." }

                    @if register_active {
                        @if let Some(err) = error {
                            div class="error-banner" { (err) }
                        }
                    }

                    form method="POST" action="/auth/register" {
                        input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                        div class="form-group" {
                            label for="reg-username" { "Username" }
                            input type="text" id="reg-username" name="username"
                                required minlength="3" maxlength="50"
                                placeholder="3-50 characters";
                        }

                        div class="form-group" {
                            label for="reg-password" { "Password" }
                            input type="password" id="reg-password" name="password"
                                required minlength="8" maxlength="72"
                                placeholder="8-72 characters";
                        }

                        div class="form-group" {
                            label for="reg-confirm" { "Confirm Password" }
                            input type="password" id="reg-confirm" name="confirm_password"
                                required minlength="8" maxlength="72"
                                placeholder="Repeat your password";
                        }

                        button type="submit" class="btn btn-primary"
                            hx-indicator="#register-spinner" {
                            "Create Account"
                            span id="register-spinner" class="htmx-indicator" {
                                (spinner_icon())
                            }
                        }
                    }

                    div class="auth-footer" {
                        "Already have an account? "
                        a href="#" onclick="switchTab('login');return false;" {
                            "Sign in"
                        }
                    }
                }

                // ── Reset tab ──
                div class=(if reset_active { "tab-panel active" } else { "tab-panel" }) id="reset" {
                    h2 class="auth-title" { "Reset password" }
                    p class="auth-subtitle" { "Enter your username and we'll send you a reset link." }

                    form method="POST" action="/auth/reset-password" {
                        input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                        div class="form-group" {
                            label for="reset-username" { "Username" }
                            input type="text" id="reset-username" name="username"
                                required placeholder="Your username";
                        }

                        button type="submit" class="btn btn-primary" {
                            "Send Reset Link"
                        }
                    }

                    div class="divider" { "or" }

                    button type="button" class="btn btn-ghost"
                        onclick="switchTab('login')" {
                        "Back to Sign In"
                    }
                }
            }

            script {
                (PreEscaped(r#"
function switchTab(id) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
  const btn = document.querySelector('[data-tab="'+id+'"]');
  if (btn) btn.classList.add('active');
  const panel = document.getElementById(id);
  if (panel) panel.classList.add('active');
}
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => switchTab(btn.dataset.tab));
});
                "#))
            }
        },
    )
}

/// Account dashboard page.
pub fn account_page(
    auth: AuthState,
    session: &UserSession,
    games: &[ListDisplayGame],
) -> maud::Markup {
    base_layout(
        &format!("Account — {}", session.username),
        auth,
        html! {
            div {
                // Account header
                div class="mb-8 p-4 bg-gray-900 border border-gray-800 rounded-lg" {
                    div class="flex items-center justify-between" {
                        div {
                            h1 class="text-xl font-bold text-amber-400" { (icon("user")) " " (session.username) }
                            p class="text-sm text-gray-400 mt-1" { "Account Dashboard" }
                        }
                    }
                }

                // Quick actions
                div class="mb-6" {
                    a
                        href="/games/new"
                        class="inline-block bg-green-600 hover:bg-green-700 text-white font-semibold px-4 py-2 rounded" {
                        "Create New Game"
                    }
                }

                // User's games
                h2 class="text-lg font-semibold text-amber-400 mb-4" { "Your Games" }

                @if games.is_empty() {
                    p class="text-gray-400" { "You haven't created any games yet." }
                } @else {
                    div class="space-y-3" {
                        @for game in games {
                            (user_game_card(game))
                        }
                    }
                }
            }
        },
    )
}

/// Single game card for account page.
fn user_game_card(game: &ListDisplayGame) -> maud::Markup {
    html! {
        a href=(format!("/games/{}", game.identifier))
            class="block bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg p-4 transition-colors" {
            div class="flex items-center justify-between" {
                div {
                    h3 class="text-lg font-semibold text-white" { (game.name) }
                    p class="text-sm text-gray-400" {
                        "Day " (game.day.unwrap_or(0))
                        " · "
                        span class=(status_color(&game.status.to_string())) { (game.status) }
                        " · " (game.living_count) "/" (game.tribute_count) " alive"
                    }
                }
                div class="flex items-center gap-2" {
                    @if game.private {
                        span class="text-xs text-gray-500 bg-gray-800 px-2 py-1 rounded" { "Private" }
                    }
                    @if game.ready {
                        span class="text-xs text-green-400 bg-green-900/30 px-2 py-1 rounded" { "Ready" }
                    }
                }
            }
        }
    }
}

/// Create game form page.
pub fn create_game_page(auth: AuthState, error: Option<&str>) -> maud::Markup {
    base_layout(
        "Create Game",
        auth,
        html! {
            div class="max-w-lg mx-auto py-8" {
                a href="/account" class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Account"
                }

                h1 class="text-2xl font-bold text-amber-400 mb-6" { "Create New Game" }

                @if let Some(err) = error {
                    div class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-red-300 text-sm" {
                        (err)
                    }
                }

                form method="POST" action="/games/new" class="space-y-4" {
                    input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                    div {
                        label for="name" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Game Name"
                        }
                        input
                            type="text"
                            id="name"
                            name="name"
                            required
                            minlength="1"
                            maxlength="100"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="Enter game name";
                    }

                    div {
                        label for="description" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Description (optional)"
                        }
                        textarea
                            id="description"
                            name="description"
                            rows="3"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="Describe your game";
                    }

                    div class="flex items-center gap-2" {
                        input
                            type="checkbox"
                            id="private"
                            name="private"
                            value="true"
                            class="rounded bg-gray-800 border-gray-700 text-amber-500 focus:ring-amber-500";
                        label for="private" class="text-sm text-gray-300" {
                            "Private game"
                        }
                    }

                    button
                        type="submit"
                        class="w-full bg-green-600 hover:bg-green-700 text-white font-semibold px-4 py-2 rounded relative"
                        hx-indicator="#create-game-spinner" {
                        "Create Game"
                        span id="create-game-spinner" class="htmx-indicator absolute right-3 top-1/2 -translate-y-1/2" {
                            (spinner_icon())
                        }
                    }
                }
            }
        },
    )
}

/// Generic error page.
pub fn error_page(auth: AuthState, title: &str, message: &str) -> maud::Markup {
    base_layout(
        title,
        auth,
        html! {
            div class="text-center py-12" {
                h1 class="text-2xl font-bold text-red-400 mb-4" { (title) }
                p class="text-gray-400 mb-6" { (message) }
                a href="/" class="text-amber-400 hover:text-amber-300" { "Go Home" }
            }
        },
    )
}

/// Inline SVG spinner for HTMX loading indicators.
fn spinner_icon() -> maud::Markup {
    maud::html! {
        svg class="inline w-4 h-4 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" {
            circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" {}
            path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" {}
        }
    }
}

/// Placeholder for CSRF token value in templates.
/// The real token is injected by the handler via a form value override.
fn csrf_placeholder() -> &'static str {
    "__CSRF_TOKEN__"
}

/// Auth page with CSRF token injected (used by GET /auth).
pub fn auth_page_with_csrf(
    auth: AuthState,
    csrf: &str,
    error: Option<&str>,
    default_tab: AuthTab,
) -> maud::Markup {
    let rendered: String = auth_page(auth, error, default_tab).into();
    maud::PreEscaped(rendered.replace(csrf_placeholder(), csrf))
}

/// Create game form page with CSRF token injected.
pub fn create_game_page_with_csrf(auth: AuthState, csrf: &str) -> maud::Markup {
    let rendered: String = create_game_page(auth, None).into();
    maud::PreEscaped(rendered.replace(csrf_placeholder(), csrf))
}
