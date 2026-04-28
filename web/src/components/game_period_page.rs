//! Per-period detail page: `/games/:identifier/day/:day/:phase`.
//!
//! Validates the (day, phase) pair against the timeline summary, fetches the
//! day log via `/api/games/{id}/log/{day}`, filters down to the requested
//! phase, and renders [`FilterChips`] + [`Timeline`].

use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::filter_chips::FilterChips;
use crate::components::timeline::{PeriodFilters, Timeline};
use crate::env::APP_API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::{GameMessage, Phase};

async fn fetch_day_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    let Some(QueryKey::GameDayLog(id, day)) = keys.first() else {
        return Err(QueryError::Unknown).into();
    };
    let url = format!("{APP_API_HOST}/api/games/{id}/log/{day}");
    match reqwest::get(&url).await {
        Ok(resp) => match resp.status() {
            StatusCode::OK => match resp.json::<Vec<GameMessage>>().await {
                Ok(v) => Ok(QueryValue::Logs(v)).into(),
                Err(_) => Err(QueryError::BadJson).into(),
            },
            StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(id.clone())).into(),
            _ => Err(QueryError::Unknown).into(),
        },
        Err(_) => Err(QueryError::ServerNotFound).into(),
    }
}

#[component]
pub fn GamePeriodPage(identifier: String, day: u32, phase: Phase) -> Element {
    let filters: Signal<PeriodFilters> = use_context();
    let filter = filters.read().filter_for(&identifier);

    let summary_q = use_get_query(
        [QueryKey::TimelineSummary(identifier.clone())],
        crate::hooks::use_timeline_summary::fetch_timeline_summary,
    );

    let valid = match summary_q.result().value() {
        QueryState::Settled(Ok(QueryValue::TimelineSummary(s))) => {
            s.periods.iter().any(|p| p.day == day && p.phase == phase)
        }
        QueryState::Settled(Err(_)) => false,
        _ => true,
    };

    if !valid {
        return rsx! {
            div { class: "space-y-2",
                h1 { class: "text-2xl font-semibold", "Period not found" }
                p { class: "text-gray-600", "Day {day} ({phase}) doesn't exist for this game." }
            }
        };
    }

    let log_q =
        use_get_query([QueryKey::GameDayLog(identifier.clone(), day)], fetch_day_log);

    rsx! {
        div { class: "space-y-4",
            h1 { class: "text-2xl font-semibold", "Day {day} — {phase}" }
            FilterChips { game_identifier: identifier.clone() }
            match log_q.result().value() {
                QueryState::Settled(Ok(QueryValue::Logs(msgs))) => {
                    let filtered: Vec<GameMessage> = msgs
                        .iter()
                        .filter(|m| m.phase == phase)
                        .cloned()
                        .collect();
                    rsx! {
                        Timeline {
                            game_identifier: identifier.clone(),
                            messages: filtered,
                            filter,
                        }
                    }
                }
                QueryState::Settled(Err(_)) => rsx! {
                    p { class: "text-red-600", "Failed to load events." }
                },
                _ => rsx! {
                    div { class: "animate-pulse h-32 rounded bg-gray-200" }
                },
            }
        }
    }
}
