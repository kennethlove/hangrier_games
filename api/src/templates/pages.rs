use maud::html;
use shared::{ListDisplayGame, PaginatedGames};

use super::{AuthState, base_layout};

pub fn home_page(auth: AuthState) -> maud::Markup {
    base_layout(
        "Home",
        auth,
        html! {
            div class="text-center py-12" {
                h1 class="text-3xl font-bold text-amber-400 mb-4" { "Hangrier Games" }
                p class="text-gray-400 mb-8" { "A browser-based Hunger Games simulator" }
                a href="/games" class="inline-block bg-amber-500 hover:bg-amber-600 text-gray-900 font-semibold px-6 py-3 rounded" {
                    "View Games"
                }
            }
        },
    )
}

pub fn games_list_page(auth: AuthState, paginated: &PaginatedGames) -> maud::Markup {
    base_layout(
        "Games",
        auth,
        html! {
            h1 class="text-2xl font-bold text-amber-400 mb-6" { "Games" }
            @if paginated.games.is_empty() {
                p class="text-gray-400" { "No games yet." }
            } @else {
                div class="space-y-4" {
                    @for game in &paginated.games {
                        (game_card(game))
                    }
                }
                @if paginated.pagination.has_more {
                    div class="mt-6 text-center" {
                        button
                            class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded"
                            hx-get=(format!("/games?offset={}&limit={}",
                                paginated.pagination.offset + paginated.pagination.limit,
                                paginated.pagination.limit))
                            hx-target="#games-list"
                            hx-swap="beforeend"
                            hx-disabled-elt="this" {
                            "Load More"
                        }
                    }
                }
            }
        },
    )
}

fn game_card(game: &ListDisplayGame) -> maud::Markup {
    html! {
        a href=(format!("/games/{}", game.identifier))
            class="block bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg p-4 transition-colors" {
            div class="flex items-center justify-between" {
                div {
                    h2 class="text-lg font-semibold text-white" { (game.name) }
                    p class="text-sm text-gray-400" {
                        "Day " (game.day.unwrap_or(0))
                        " · "
                        span class=(status_color(&game.status.to_string())) { (game.status) }
                    }
                }
                @if game.private {
                    span class="text-xs text-gray-500 bg-gray-800 px-2 py-1 rounded" { "Private" }
                }
            }
        }
    }
}

pub fn status_color(status: &str) -> &'static str {
    match status {
        "NotStarted" => "text-gray-400",
        "InProgress" => "text-green-400",
        "Finished" => "text-amber-400",
        _ => "text-gray-500",
    }
}

/// 404 Not Found page.
pub fn not_found_page(message: &str) -> maud::Markup {
    base_layout(
        "Not Found",
        AuthState::guest(),
        html! {
            div class="text-center py-16" {
                div class="mb-6" {
                    span class="text-6xl font-bold text-amber-400" { "404" }
                }
                h1 class="text-2xl font-bold text-gray-100 mb-3" { "Not Found" }
                p class="text-gray-400 mb-8" { (message) }
                div class="flex items-center justify-center gap-4" {
                    a href="/" class="text-amber-400 hover:text-amber-300" { "Go Home" }
                    span class="text-gray-600" { "·" }
                    a href="/games" class="text-amber-400 hover:text-amber-300" { "View Games" }
                }
            }
        },
    )
}

/// 500 Internal Server Error page.
pub fn server_error_page(message: &str) -> maud::Markup {
    base_layout(
        "Server Error",
        AuthState::guest(),
        html! {
            div class="text-center py-16" {
                div class="mb-6" {
                    span class="text-6xl font-bold text-red-400" { "500" }
                }
                h1 class="text-2xl font-bold text-gray-100 mb-3" { "Something Went Wrong" }
                p class="text-gray-400 mb-8" { (message) }
                a href="/" class="text-amber-400 hover:text-amber-300" { "Go Home" }
            }
        },
    )
}
