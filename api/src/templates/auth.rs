use maud::html;
use shared::{ListDisplayGame, UserSession};

use super::pages::status_color;
use super::{AuthState, base_layout, icon};

/// Login form page.
pub fn login_page(auth: AuthState, error: Option<&str>) -> maud::Markup {
    base_layout(
        "Login",
        auth,
        html! {
            div class="max-w-md mx-auto py-12" {
                h1 class="text-2xl font-bold text-amber-400 mb-6 text-center" { "Login" }

                @if let Some(err) = error {
                    div class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-red-300 text-sm" {
                        (err)
                    }
                }

                form method="POST" action="/login" class="space-y-4" {
                    input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                    div {
                        label for="username" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Username"
                        }
                        input
                            type="text"
                            id="username"
                            name="username"
                            required
                            minlength="3"
                            maxlength="50"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="Your username";
                    }

                    div {
                        label for="password" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Password"
                        }
                        input
                            type="password"
                            id="password"
                            name="password"
                            required
                            minlength="8"
                            maxlength="72"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="Your password";
                    }

                    button
                        type="submit"
                        class="w-full bg-amber-500 hover:bg-amber-600 text-gray-900 font-semibold px-4 py-2 rounded relative"
                        hx-indicator="#login-spinner" {
                        "Login"
                        span id="login-spinner" class="htmx-indicator absolute right-3 top-1/2 -translate-y-1/2" {
                            (spinner_icon())
                        }
                    }
                }

                p class="mt-4 text-center text-sm text-gray-400" {
                    "Don't have an account? "
                    a href="/register" class="text-amber-400 hover:text-amber-300" { "Register" }
                }
            }
        },
    )
}

/// Registration form page.
pub fn register_page(auth: AuthState, error: Option<&str>) -> maud::Markup {
    base_layout(
        "Register",
        auth,
        html! {
            div class="max-w-md mx-auto py-12" {
                h1 class="text-2xl font-bold text-amber-400 mb-6 text-center" { "Register" }

                @if let Some(err) = error {
                    div class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-red-300 text-sm" {
                        (err)
                    }
                }

                form method="POST" action="/register" class="space-y-4" {
                    input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}

                    div {
                        label for="username" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Username"
                        }
                        input
                            type="text"
                            id="username"
                            name="username"
                            required
                            minlength="3"
                            maxlength="50"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="3-50 characters";
                    }

                    div {
                        label for="password" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Password"
                        }
                        input
                            type="password"
                            id="password"
                            name="password"
                            required
                            minlength="8"
                            maxlength="72"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="8-72 characters";
                    }

                    div {
                        label for="confirm_password" class="block text-sm font-medium text-gray-300 mb-1" {
                            "Confirm Password"
                        }
                        input
                            type="password"
                            id="confirm_password"
                            name="confirm_password"
                            required
                            minlength="8"
                            maxlength="72"
                            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-amber-500"
                            placeholder="Repeat your password";
                    }

                    button
                        type="submit"
                        class="w-full bg-amber-500 hover:bg-amber-600 text-gray-900 font-semibold px-4 py-2 rounded relative"
                        hx-indicator="#register-spinner" {
                        "Create Account"
                        span id="register-spinner" class="htmx-indicator absolute right-3 top-1/2 -translate-y-1/2" {
                            (spinner_icon())
                        }
                    }
                }

                p class="mt-4 text-center text-sm text-gray-400" {
                    "Already have an account? "
                    a href="/login" class="text-amber-400 hover:text-amber-300" { "Login" }
                }
            }
        },
    )
}

/// Account dashboard page.
pub fn account_page(session: &UserSession, games: &[ListDisplayGame]) -> maud::Markup {
    let auth = AuthState::authenticated(&session.username);
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
                        form method="POST" action="/logout" {
                            input type="hidden" name="csrf_token" value=(csrf_placeholder()) {}
                            button
                                type="submit"
                                class="text-sm text-gray-400 hover:text-red-400" {
                                "Logout"
                            }
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
    ""
}

/// Login form page with CSRF token injected.
pub fn login_page_with_csrf(csrf: &str) -> String {
    let markup: String = login_page(AuthState::guest(), None).into();
    markup.replace(csrf_placeholder(), csrf)
}

/// Registration form page with CSRF token injected.
pub fn register_page_with_csrf(csrf: &str) -> String {
    let markup: String = register_page(AuthState::guest(), None).into();
    markup.replace(csrf_placeholder(), csrf)
}

/// Create game form page with CSRF token injected.
pub fn create_game_page_with_csrf(csrf: &str) -> String {
    let markup: String = create_game_page(AuthState::guest(), None).into();
    markup.replace(csrf_placeholder(), csrf)
}
