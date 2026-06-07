use maud::html;
use shared::messages::TributeRef;
use shared::messages::{GameMessage, MessageKind, PeriodSummary, TimelineSummary};

use super::{AuthState, base_layout, icon};

pub mod cards;

/// Context for rendering the timeline page.
pub struct TimelineContext<'a> {
    pub auth: AuthState,
    pub game_id: &'a str,
    pub game_name: &'a str,
    pub periods: &'a TimelineSummary,
    pub current_day: u32,
    pub current_phase: &'a shared::messages::Phase,
    pub filter: &'a str,
    pub tribute_filter: &'a str,
    pub tributes: &'a [TributeRef],
    pub events: &'a [GameMessage],
    pub selected_day: Option<u32>,
    pub selected_phase: Option<shared::messages::Phase>,
}

/// Full timeline page with period grid, filters, and event cards.
pub fn timeline_page(ctx: &TimelineContext<'_>) -> maud::Markup {
    base_layout(
        &format!("Timeline — {}", ctx.game_name),
        ctx.auth.clone(),
        html! {
            div {
                // Back link
                a href=(format!("/games/{}", ctx.game_id))
                    class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Game"
                }

                // Header
                div class="mb-4" {
                    h1 class="text-xl font-bold text-amber-400" { "Timeline" }
                    p class="text-sm text-gray-500" {
                        "Day " (ctx.current_day) " · " (ctx.current_phase)
                    }
                }

                // Period timeline strip
                @if !ctx.periods.periods.is_empty() {
                    div class="mb-4" {
                        div class="flex gap-1.5 overflow-x-auto pb-2" {
                            @for period in &ctx.periods.periods {
                                (period_chip(ctx.game_id, period, ctx.filter, ctx.tribute_filter))
                            }
                        }
                    }
                }

                // Filter chips row
                (filter_chips(ctx.game_id, ctx.selected_day, ctx.selected_phase, ctx.filter, ctx.tribute_filter))

                // Tribute filter chips row
                @if !ctx.tributes.is_empty() {
                    (tribute_chips(ctx.game_id, ctx.selected_day, ctx.selected_phase, ctx.filter, ctx.tribute_filter, ctx.tributes))
                }

                // Event timeline or period grid
                @if let (Some(_day), Some(_phase)) = (ctx.selected_day, ctx.selected_phase) {
                    // Show filtered events for selected period
                    @if ctx.events.is_empty() {
                        (empty_state("No events for this period."))
                    } @else {
                        div class="space-y-2" {
                            @for card in render_condensed_events(ctx.events) {
                                (card)
                            }
                        }
                    }
                } @else {
                    // Show period grid overview
                    (period_grid(&ctx.periods.periods))
                }
            }
        },
    )
}

/// Single period chip in the timeline strip.
fn period_chip(
    game_id: &str,
    period: &PeriodSummary,
    filter: &str,
    tribute_filter: &str,
) -> maud::Markup {
    let is_current = period.is_current;
    let border_class = if is_current {
        "border-amber-500 ring-1 ring-amber-500/30"
    } else {
        "border-gray-700 hover:border-gray-500"
    };
    let bg_class = if is_current {
        "bg-amber-900/30"
    } else {
        "bg-gray-900"
    };

    let deaths_indicator = if period.deaths > 0 {
        html! {
            span class="text-red-400 text-xs" {
                (icon("skull"))
                " " (period.deaths)
            }
        }
    } else {
        html! {}
    };

    let filter_param = if !filter.is_empty() {
        format!("&filter={filter}")
    } else {
        String::new()
    };
    let tribute_param = if !tribute_filter.is_empty() {
        format!("&tribute={tribute_filter}")
    } else {
        String::new()
    };

    html! {
        a href=(format!(
            "/games/{}/timeline?day={}&phase={}{}{}",
            game_id, period.day, period.phase, filter_param, tribute_param
        ))
            class=(format!("flex-shrink-0 flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs transition-colors {} {}", bg_class, border_class)) {
            span class="text-gray-300 font-medium" {
                "D" (period.day) " " (period.phase)
            }
            span class="text-gray-500" { (period.event_count) }
            (deaths_indicator)
        }
    }
}

/// Category filter chips row.
fn filter_chips(
    game_id: &str,
    selected_day: Option<u32>,
    selected_phase: Option<shared::messages::Phase>,
    active_filter: &str,
    tribute_filter: &str,
) -> maud::Markup {
    let filters = [
        ("", "All", "list"),
        ("Deaths", "Deaths", "skull"),
        ("Combat", "Combat", "sword"),
        ("Alliances", "Alliances", "users"),
        ("Movement", "Movement", "map-pin"),
        ("Items", "Items", "backpack"),
    ];

    let day_param = selected_day
        .map(|d| format!("&day={d}"))
        .unwrap_or_default();
    let phase_param = selected_phase
        .map(|p| format!("&phase={p}"))
        .unwrap_or_default();
    let tribute_param = if !tribute_filter.is_empty() {
        format!("&tribute={tribute_filter}")
    } else {
        String::new()
    };

    html! {
        div class="flex flex-wrap gap-1.5 mb-3" {
            @for (value, label, icon_name) in &filters {
                @let is_active = active_filter == *value;
                @let chip_class = if is_active {
                    "bg-amber-600 text-white"
                } else {
                    "bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200"
                };
                a href=(format!(
                    "/games/{}/timeline?filter={}{}{}{}",
                    game_id, value, day_param, phase_param, tribute_param
                ))
                    class=(format!("px-2.5 py-1 rounded-full text-xs font-medium transition-colors {}", chip_class)) {
                    (icon(icon_name))
                    " " (label)
                }
            }
        }
    }
}

/// Tribute filter chips row.
fn tribute_chips(
    game_id: &str,
    selected_day: Option<u32>,
    selected_phase: Option<shared::messages::Phase>,
    filter: &str,
    tribute_filter: &str,
    tributes: &[TributeRef],
) -> maud::Markup {
    let day_param = selected_day
        .map(|d| format!("&day={d}"))
        .unwrap_or_default();
    let phase_param = selected_phase
        .map(|p| format!("&phase={p}"))
        .unwrap_or_default();
    let filter_param = if !filter.is_empty() {
        format!("&filter={filter}")
    } else {
        String::new()
    };

    // Show first 12 tributes to avoid overwhelming the UI
    let max_tributes = tributes.len().min(12);

    html! {
        div class="flex flex-wrap gap-1.5 mb-3" {
            // "All tributes" chip
            @let is_all = tribute_filter.is_empty();
            @let all_class = if is_all {
                "bg-blue-600 text-white"
            } else {
                "bg-gray-800 text-gray-500 hover:bg-gray-700"
            };
            a href=(format!(
                "/games/{}/timeline?tribute={}{}{}{}",
                game_id, "", filter_param, day_param, phase_param
            ))
                class=(format!("px-2 py-0.5 rounded-full text-xs transition-colors {}", all_class)) {
                "All"
            }
            @for tribute in &tributes[..max_tributes] {
                @let is_active = tribute_filter == tribute.name;
                @let chip_class = if is_active {
                    "bg-blue-600 text-white"
                } else {
                    "bg-gray-800 text-gray-400 hover:bg-gray-700"
                };
                a href=(format!(
                    "/games/{}/timeline?tribute={}{}{}{}",
                    game_id, tribute.name, filter_param, day_param, phase_param
                ))
                    class=(format!("px-2 py-0.5 rounded-full text-xs transition-colors {}", chip_class)) {
                    (tribute.name)
                }
            }
        }
    }
}

/// Overview grid showing all periods at a glance.
fn period_grid(periods: &[PeriodSummary]) -> maud::Markup {
    html! {
        div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2" {
            @for period in periods {
                @let border_class = if period.is_current {
                    "border-amber-500"
                } else {
                    "border-gray-800"
                };
                @let bg_class = if period.is_current {
                    "bg-amber-900/20"
                } else {
                    "bg-gray-900"
                };
                div class=(format!("p-3 rounded-lg border {} {}", bg_class, border_class)) {
                    div class="flex items-center justify-between mb-1" {
                        span class="text-sm font-medium text-gray-300" {
                            "Day " (period.day) " " (period.phase)
                        }
                        @if period.is_current {
                            span class="text-xs text-amber-400" { "Now" }
                        }
                    }
                    div class="flex items-center gap-3 text-xs text-gray-500" {
                        span { (period.event_count) " events" }
                        @if period.deaths > 0 {
                            span class="text-red-400" {
                                (icon("skull"))
                                " " (period.deaths)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render events with FixationFired condensation: consecutive FixationFired
/// events for the same target collapse into a single "ongoing fixation" card.
pub fn render_condensed_events(events: &[GameMessage]) -> Vec<maud::Markup> {
    use shared::messages::MessagePayload;
    let mut result: Vec<maud::Markup> = Vec::new();
    let mut i = 0;
    while i < events.len() {
        let msg = &events[i];
        if matches!(msg.payload, MessagePayload::FixationFired { .. }) {
            // Count consecutive FixationFired for the same target
            let target = match &msg.payload {
                MessagePayload::FixationFired { target, .. } => target.clone(),
                _ => unreachable!(),
            };
            let mut count = 1;
            while i + count < events.len() {
                let next = &events[i + count];
                match &next.payload {
                    MessagePayload::FixationFired { target: t, .. } if *t == target => {
                        count += 1;
                    }
                    _ => break,
                }
            }
            if count > 1 {
                result.push(condensed_fixation_card(msg, count));
            } else {
                result.push(event_card(msg));
            }
            i += count;
        } else {
            result.push(event_card(msg));
            i += 1;
        }
    }
    result
}

/// Condensed card for repeated FixationFired events on the same target.
fn condensed_fixation_card(msg: &GameMessage, count: usize) -> maud::Markup {
    use shared::messages::MessagePayload;
    let target = match &msg.payload {
        MessagePayload::FixationFired { target, .. } => target.as_str(),
        _ => "",
    };
    let severity = match &msg.payload {
        MessagePayload::FixationFired { severity, .. } => severity.as_str(),
        _ => "",
    };

    let border_color = match severity {
        "severe" | "Severe" => "border-red-900/50",
        "moderate" | "Moderate" => "border-orange-900/50",
        _ => "border-yellow-900/50",
    };

    let badge_color = match severity {
        "severe" | "Severe" => "bg-red-500/20 text-red-300",
        "moderate" | "Moderate" => "bg-orange-500/20 text-orange-300",
        _ => "bg-yellow-500/20 text-yellow-300",
    };

    html! {
        div class=(format!("bg-gray-900 border rounded-lg p-3 {}", border_color)) {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("eye"))
                span class="text-rose-400 font-medium" { "Fixation" }
                span { "\u{b7}" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200 flex items-center gap-2" {
                "Ongoing fixation on " (target)
                span class=(format!("text-xs px-1.5 py-0.5 rounded {}", badge_color)) { (severity) }
                span class="text-xs text-gray-500" { "(\u{d7}" (count) ")" }
            }
        }
    }
}

/// Empty state placeholder.
fn empty_state(message: &str) -> maud::Markup {
    html! {
        div class="text-center py-12" {
            (icon("ghost"))
            p class="text-gray-500 mt-2" { (message) }
        }
    }
}

/// Dispatch to kind-specific event card.
pub fn event_card(msg: &GameMessage) -> maud::Markup {
    use MessageKind::*;
    match msg.payload.kind() {
        Death => cards::death_card(msg),
        Combat => cards::combat_card(msg),
        CombatSwing => cards::combat_swing_card(msg),
        Alliance => cards::alliance_card(msg),
        Movement => cards::movement_card(msg),
        Item | SponsorGift => cards::item_card(msg),
        State => cards::state_card(msg),
        Trauma => cards::trauma_card(msg),
        Affliction => cards::affliction_card(msg),
        Phobia => cards::phobia_card(msg),
        Fixation => cards::fixation_card(msg),
        Trapped => cards::trapped_card(msg),
        Sleep => cards::sleep_card(msg),
    }
}
