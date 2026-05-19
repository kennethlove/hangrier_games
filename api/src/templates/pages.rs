use maud::html;
use shared::{ListDisplayGame, PaginatedGames};

use super::base_layout;

pub fn home_page() -> maud::Markup {
    base_layout(
        "Home",
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

pub fn games_list_page(paginated: &PaginatedGames) -> maud::Markup {
    base_layout(
        "Games",
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

fn status_color(status: &str) -> &'static str {
    match status {
        "NotStarted" => "text-gray-400",
        "InProgress" => "text-green-400",
        "Finished" => "text-amber-400",
        _ => "text-gray-500",
    }
}
