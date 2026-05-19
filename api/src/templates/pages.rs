use maud::html;
use shared::{GameStatus, ListDisplayGame, PaginatedGames};

use super::{AuthState, base_layout};

/// Aggregated game statistics for the dashboard.
pub struct GameStats {
    pub running: u32,
    pub waiting: u32,
    pub finished: u32,
    pub total: u32,
}

impl GameStats {
    pub fn from_games(games: &[ListDisplayGame]) -> Self {
        let mut running = 0;
        let mut waiting = 0;
        let mut finished = 0;
        for g in games {
            match g.status {
                GameStatus::InProgress => running += 1,
                GameStatus::NotStarted => waiting += 1,
                GameStatus::Finished => finished += 1,
            }
        }
        Self {
            running,
            waiting,
            finished,
            total: games.len() as u32,
        }
    }
}

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

pub fn games_list_page(
    auth: AuthState,
    paginated: &PaginatedGames,
    stats: &GameStats,
    active_filter: &str,
) -> maud::Markup {
    base_layout(
        "Games",
        auth,
        html! {
            // Page header
            section class="page-header" {
                div class="container" {
                    h1 { "Games" }
                    p class="deck" { "All simulations — past, present, and waiting to begin." }
                }
            }

            // Action row
            div class="container" {
                div class="action-row" {
                    // Left: launch block
                    div class="launch-block" {
                        // Quickstart form
                        form method="POST" action="/games/new" {
                            input type="hidden" name="csrf_token" value="";
                            input type="hidden" name="name" value="Quick Game";
                            button.quickstart-btn type="submit" { "⚡ Quickstart — New 24-Tribute Game" }
                        }
                        div class="or-divider" { "or" }
                        // Create form
                        div class="create-form" {
                            h3 { "Create a Game" }
                            form method="POST" action="/games/new" {
                                input type="hidden" name="csrf_token" value="";
                                input type="text" name="name" placeholder="Game name";
                                button class="btn btn-primary btn-sm" type="submit" { "Create" }
                            }
                        }
                    }
                    // Right: stats column
                    div class="stats-col" {
                        (summary_card("Running", stats.running, "running"))
                        (summary_card("Waiting", stats.waiting, "waiting"))
                        (summary_card("Finished", stats.finished, "finished"))
                        (summary_card("Total Games", stats.total, ""))
                    }
                }
            }

            // Game list section
            section class="game-list" {
                div class="container" {
                    div class="game-list-header" {
                        h2 { "All Games" }
                        div class="filter-pills" {
                            (filter_pill("All", "", active_filter))
                            (filter_pill("Running", "running", active_filter))
                            (filter_pill("Waiting", "waiting", active_filter))
                            (filter_pill("Finished", "finished", active_filter))
                        }
                    }

                    div id="games-list-content" {
                        @if paginated.games.is_empty() {
                            div class="empty-state" {
                                p { "No games yet" }
                                p { "Create a game to get started" }
                            }
                        } @else {
                            @let running_games: Vec<_> = paginated.games.iter()
                                .filter(|g| g.status == GameStatus::InProgress)
                                .collect();
                            @let waiting_games: Vec<_> = paginated.games.iter()
                                .filter(|g| g.status == GameStatus::NotStarted)
                                .collect();
                            @let finished_games: Vec<_> = paginated.games.iter()
                                .filter(|g| g.status == GameStatus::Finished)
                                .collect();

                            // Featured running cards (first 2)
                            @for (idx, game) in running_games.iter().enumerate() {
                                @if idx < 2 {
                                    (featured_running_card(game))
                                } @else {
                                    (running_card(game))
                                }
                            }

                            // Waiting cards
                            @for game in &waiting_games {
                                (waiting_card(game))
                            }

                            // Finished heading + grid
                            @if !finished_games.is_empty() {
                                h3 style="font-family:var(--font-display);font-size:var(--fs-h3);font-weight:600;margin:var(--gap-lg) 0 var(--gap-md);" { "Finished" }
                                div class="finished-grid" {
                                    @for game in &finished_games {
                                        (finished_card(game))
                                    }
                                }
                            }

                            // Load more
                            @if paginated.pagination.has_more {
                                div class="text-center" style="margin-top:var(--gap-lg);" {
                                    button class="btn btn-ghost"
                                        hx-get=(format!("/games?offset={}&limit={}",
                                            paginated.pagination.offset + paginated.pagination.limit,
                                            paginated.pagination.limit))
                                        hx-target="#games-list-content"
                                        hx-swap="innerHTML"
                                        hx-disabled-elt="this" {
                                        "Load More"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

fn summary_card(label: &str, value: u32, modifier: &str) -> maud::Markup {
    let classes = format!("summary-card {}", modifier);
    html! {
        div class=(classes) {
            div class="s-val num" { (value) }
            div class="s-label" { (label) }
        }
    }
}

fn filter_pill(label: &str, status: &str, active: &str) -> maud::Markup {
    let is_active = if status.is_empty() {
        active.is_empty() || active == "all"
    } else {
        active == status
    };
    let classes = if is_active {
        "filter-pill active"
    } else {
        "filter-pill"
    };
    let href = if status.is_empty() {
        "/games".to_string()
    } else {
        format!("/games?status={}", status)
    };
    html! {
        a href=(href) class=(classes) { (label) }
    }
}

fn featured_running_card(game: &ListDisplayGame) -> maud::Markup {
    let progress = compute_progress(game.day);
    html! {
        div class="game-card featured running" data-status="running" {
            div class="game-info" {
                div class="live-pulse" { "Live" }
                h3 class="game-name" {
                    a href=(format!("/games/{}", game.identifier)) { (game.name) }
                }
                div class="game-meta" {
                    span { span class="num" { (game.tribute_count) } " tributes" }
                    span { span class="num" { (game.living_count) } " alive" }
                    @if let Some(day) = game.day {
                        span { span class="num" { "Day " (day) } }
                    }
                }
                div class="progress-bar" {
                    div class="fill" style=(format!("width:{}%", progress)) {}
                }
                div class="progress-label" {
                    @if let Some(day) = game.day {
                        span class="num" { "Day " (day) } " · "
                    }
                    span class="num" { (game.living_count) } " tributes alive"
                }
                div class="reveal-actions" {
                    a href=(format!("/games/{}", game.identifier)) class="reveal-btn" { "Broadcast" }
                    a href=(format!("/games/{}/tributes", game.identifier)) class="reveal-btn" { "Tributes" }
                    a href=(format!("/games/{}/areas", game.identifier)) class="reveal-btn" { "Arena Map" }
                    a href=(format!("/games/{}/odds", game.identifier)) class="reveal-btn" { "Odds" }
                }
            }
            div class="game-status" {
                span class="status-pill running" { "Running" }
            }
        }
    }
}

fn running_card(game: &ListDisplayGame) -> maud::Markup {
    let progress = compute_progress(game.day);
    html! {
        div class="game-card running" data-status="running" {
            div class="game-info" {
                h3 class="game-name" {
                    a href=(format!("/games/{}", game.identifier)) { (game.name) }
                }
                div class="game-meta" {
                    span { span class="num" { (game.tribute_count) } " tributes" }
                    span { span class="num" { (game.living_count) } " alive" }
                    @if let Some(day) = game.day {
                        span { span class="num" { "Day " (day) } }
                    }
                }
                div class="progress-bar" {
                    div class="fill" style=(format!("width:{}%", progress)) {}
                }
                div class="progress-label" {
                    @if let Some(day) = game.day {
                        span class="num" { "Day " (day) } " · "
                    }
                    span class="num" { (game.living_count) } " tributes alive"
                }
                div class="reveal-actions" {
                    a href=(format!("/games/{}", game.identifier)) class="reveal-btn" { "Broadcast" }
                    a href=(format!("/games/{}/tributes", game.identifier)) class="reveal-btn" { "Tributes" }
                    a href=(format!("/games/{}/areas", game.identifier)) class="reveal-btn" { "Arena Map" }
                }
            }
            div class="game-status" {
                span class="status-pill running" { "Running" }
            }
        }
    }
}

fn waiting_card(game: &ListDisplayGame) -> maud::Markup {
    html! {
        div class="game-card waiting" data-status="waiting" {
            div class="game-info" {
                h3 class="game-name" {
                    a href=(format!("/games/{}", game.identifier)) { (game.name) }
                }
                div class="game-meta" {
                    span { span class="num" { (game.tribute_count) } " tributes" }
                }
                div class="reveal-actions" {
                    a href=(format!("/games/{}/launch", game.identifier)) class="reveal-btn" { "Launch" }
                    a href=(format!("/games/{}/configure", game.identifier)) class="reveal-btn" { "Configure" }
                    button class="reveal-btn" type="button" { "Delete" }
                }
            }
            div class="game-status" {
                span class="status-pill waiting" { "Waiting" }
            }
        }
    }
}

fn finished_card(game: &ListDisplayGame) -> maud::Markup {
    html! {
        div class="game-card finished" data-status="finished" {
            div class="game-info" {
                h3 class="game-name" {
                    a href=(format!("/games/{}", game.identifier)) { (game.name) }
                }
                div class="game-meta" {
                    span { span class="num" { (game.tribute_count) } " tributes" }
                    @if let Some(day) = game.day {
                        span { span class="num" { (day) } " days" }
                    }
                }
                div class="reveal-actions" {
                    a href=(format!("/games/{}/recap", game.identifier)) class="reveal-btn" { "Recap" }
                    a href=(format!("/games/{}/tributes", game.identifier)) class="reveal-btn" { "Tributes" }
                    a href=(format!("/games/{}/areas", game.identifier)) class="reveal-btn" { "Arena" }
                }
            }
            div class="game-status" {
                span class="status-pill finished" { "Finished" }
            }
        }
    }
}

fn compute_progress(day: Option<u32>) -> f64 {
    let d = day.unwrap_or(1) as f64;
    (d / 10.0 * 100.0).min(100.0)
}

pub fn status_color(status: &str) -> &'static str {
    match status {
        "NotStarted" => "band-warn",
        "InProgress" => "band-good",
        "Finished" => "band-none",
        _ => "band-none",
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
