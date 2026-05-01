//! PeriodGrid — renders the timeline summary as a responsive grid of PeriodCards.
//!
//! Loads via `use_timeline_summary`. Surfaces empty/error states through
//! `PeriodGridEmpty`. Distinguishes 404 (game not found) from other failures.

use crate::cache::QueryError;
use crate::components::period_card::PeriodCard;
use crate::components::period_grid_empty::{EmptyKind, PeriodGridEmpty};
use crate::hooks::use_timeline_summary;
use dioxus::prelude::*;
use dioxus_query::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct PeriodGridProps {
    pub game_identifier: String,
}

#[component]
pub fn PeriodGrid(props: PeriodGridProps) -> Element {
    let query = use_timeline_summary(props.game_identifier.clone());
    let reader = query.read();
    let state = reader.state();
    match &*state {
        QueryStateData::Settled { res: Ok(s), .. } => {
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
        QueryStateData::Settled {
            res: Err(QueryError::GameNotFound(_)),
            ..
        } => {
            rsx! { PeriodGridEmpty { kind: EmptyKind::NotFound } }
        }
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! { PeriodGridEmpty { kind: EmptyKind::LoadFailed } }
        }
        _ => rsx! { div { class: "animate-pulse h-32 rounded-lg bg-gray-200" } },
    }
}
