//! PeriodGrid — renders the timeline summary as a responsive grid of PeriodCards.
//!
//! Loads via `use_timeline_summary`. Surfaces empty/error states through
//! `PeriodGridEmpty`. Distinguishes 404 (game not found) from other failures.

use crate::cache::{QueryError, QueryValue};
use crate::components::period_card::PeriodCard;
use crate::components::period_grid_empty::{EmptyKind, PeriodGridEmpty};
use crate::hooks::use_timeline_summary;
use dioxus::prelude::*;
use dioxus_query::prelude::QueryState;

#[derive(Props, PartialEq, Clone)]
pub struct PeriodGridProps {
    pub game_identifier: String,
}

#[component]
pub fn PeriodGrid(props: PeriodGridProps) -> Element {
    let query = use_timeline_summary(props.game_identifier.clone());
    match query.result().value() {
        QueryState::Settled(Ok(QueryValue::TimelineSummary(s))) => {
            if s.periods.is_empty() {
                rsx! { PeriodGridEmpty { kind: EmptyKind::NotStarted } }
            } else {
                rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        for p in s.periods.iter() {
                            PeriodCard {
                                key: "{p.day}-{p.phase:?}",
                                game_identifier: props.game_identifier.clone(),
                                day: p.day,
                                phase: p.phase,
                                deaths: p.deaths,
                                event_count: p.event_count,
                                is_current: p.is_current,
                            }
                        }
                    }
                }
            }
        }
        QueryState::Settled(Ok(_)) => {
            rsx! { PeriodGridEmpty { kind: EmptyKind::LoadFailed } }
        }
        QueryState::Settled(Err(QueryError::GameNotFound(_))) => {
            rsx! { PeriodGridEmpty { kind: EmptyKind::NotFound } }
        }
        QueryState::Settled(Err(_)) => {
            rsx! { PeriodGridEmpty { kind: EmptyKind::LoadFailed } }
        }
        _ => rsx! { div { class: "animate-pulse h-32 rounded-lg bg-gray-200" } },
    }
}
