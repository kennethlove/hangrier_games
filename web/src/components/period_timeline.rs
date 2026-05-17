//! PeriodTimeline — horizontally-scrolling strip of period chips.
//!
//! Renders every (day, phase) pair as a compact clickable chip. The current
//! period is highlighted and auto-scrolled into view on mount. Future periods
//! are dimmed.

use crate::cache::QueryError;
use crate::hooks::use_timeline_summary;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use shared::messages::Phase;

#[component]
pub fn PeriodTimeline(identifier: String) -> Element {
    let query = use_timeline_summary(identifier.clone());
    let reader = query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled { res: Ok(s), .. } => {
            rsx! {
                TimelineScroll {
                    identifier: identifier.clone(),
                    periods: s.periods.clone(),
                }
            }
        }
        QueryStateData::Settled {
            res: Err(QueryError::GameNotFound(_)),
            ..
        }
        | QueryStateData::Settled { res: Err(_), .. } => {
            rsx! {
                div { class: "flex overflow-x-auto gap-1 pb-2 [&::-webkit-scrollbar]:h-1 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-gray-300 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:dark:bg-gray-600",
                    span { class: "text-sm text-muted", "Timeline unavailable" }
                }
            }
        }
        _ => rsx! {
            div { class: "flex overflow-x-auto gap-1 pb-2 [&::-webkit-scrollbar]:h-1 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-gray-300 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:dark:bg-gray-600",
                div { class: "animate-pulse h-8 w-48 rounded bg-gray-200" }
            }
        },
    }
}

#[component]
fn TimelineScroll(identifier: String, periods: Vec<shared::messages::PeriodSummary>) -> Element {
    let current = periods.iter().find(|p| p.is_current);
    let (current_day, current_phase) = current
        .map(|p| (p.day, p.phase))
        .unwrap_or((0, Phase::Dawn));

    // Run scroll exactly once, decoupled from render cycle
    use_hook(move || {
        scroll_to_current(current_day, current_phase);
    });

    let periods_with_keys: Vec<_> = periods
        .iter()
        .map(|p| (format!("{}-{}", p.day, phase_slug(p.phase)), p))
        .collect();

    rsx! {
        div {
            class: "flex overflow-x-auto gap-1 pb-2 scroll-smooth [&::-webkit-scrollbar]:h-1 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-gray-300 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:dark:bg-gray-600",
            for (key, period) in &periods_with_keys {
                PeriodChip {
                    key: "{key}",
                    identifier: identifier.clone(),
                    period: (*period).clone(),
                    current_day,
                    current_phase,
                }
                if period.phase == Phase::Night {
                    div { key: "divider-{period.day}", class: "flex-shrink-0 w-px h-8 bg-border self-center mx-0.5" }
                }
            }
        }
    }
}

/// Scroll the current-period chip into the visible area of the timeline.
fn scroll_to_current(day: u32, phase: Phase) {
    let chip_id = format!("period-chip-{day}-{}", phase_slug(phase));
    if let Some(doc) = web_sys::window().and_then(|w| w.document())
        && let Some(el) = doc.get_element_by_id(&chip_id)
    {
        el.scroll_into_view();
    }
}

/// Compact slug for a phase, used in element IDs.
fn phase_slug(phase: Phase) -> &'static str {
    match phase {
        Phase::Dawn => "dawn",
        Phase::Day => "day",
        Phase::Dusk => "dusk",
        Phase::Night => "night",
    }
}

#[component]
fn PeriodChip(
    identifier: String,
    period: shared::messages::PeriodSummary,
    current_day: u32,
    current_phase: Phase,
) -> Element {
    let is_future = period.day > current_day
        || (period.day == current_day && period.phase.ord() > current_phase.ord());

    let (bg, text, border) = phase_colors(period.phase);
    let icon = phase_icon(period.phase);
    let day_label = format!("D{}", period.day);

    let chip_id = format!("period-chip-{}-{}", period.day, phase_slug(period.phase));

    let current_class = if period.is_current {
        "ring-2 ring-gold "
    } else {
        ""
    };

    let future_class = if is_future {
        "opacity-40 pointer-events-none "
    } else {
        ""
    };

    let route = Routes::GamePeriodPage {
        identifier: identifier.clone(),
        day: period.day,
        phase: period.phase,
        filter: None,
        tribute: None,
    };

    rsx! {
        Link {
            to: route,
            id: chip_id,
            class: "flex flex-col items-center justify-center min-w-[2.5rem] sm:min-w-[3rem] px-1.5 py-0.5 sm:px-2 sm:py-1 rounded-md border text-[0.65rem] sm:text-xs font-medium transition hover:shadow-md {bg} {text} {border} {current_class} {future_class}",
            span { class: "text-xs sm:text-sm leading-none", "{icon}" }
            span { class: "mt-0.5 font-bold", "{day_label}" }
            if period.deaths > 0 {
                span {
                    class: "mt-0.5 inline-flex items-center justify-center min-w-[1rem] h-4 px-1 rounded-full bg-red-600 text-white text-[0.6rem] font-bold leading-none",
                    "{period.deaths}"
                }
            }
        }
    }
}

/// Returns (bg, text, border) Tailwind classes for a phase.
fn phase_colors(phase: Phase) -> (&'static str, &'static str, &'static str) {
    match phase {
        Phase::Dawn => (
            "bg-amber-100 dark:bg-amber-900/30",
            "text-amber-800 dark:text-amber-200",
            "border-amber-300 dark:border-amber-700",
        ),
        Phase::Day => (
            "bg-yellow-100 dark:bg-yellow-900/30",
            "text-yellow-800 dark:text-yellow-200",
            "border-yellow-300 dark:border-yellow-700",
        ),
        Phase::Dusk => (
            "bg-orange-100 dark:bg-orange-900/30",
            "text-orange-800 dark:text-orange-200",
            "border-orange-300 dark:border-orange-700",
        ),
        Phase::Night => (
            "bg-slate-200 dark:bg-slate-700/50",
            "text-slate-800 dark:text-slate-200",
            "border-slate-400 dark:border-slate-600",
        ),
    }
}

/// Phase emoji icon.
fn phase_icon(phase: Phase) -> &'static str {
    match phase {
        Phase::Dawn => "🌄",
        Phase::Day => "☀️",
        Phase::Dusk => "🌆",
        Phase::Night => "🌙",
    }
}
