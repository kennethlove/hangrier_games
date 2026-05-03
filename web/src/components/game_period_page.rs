//! Per-period detail page: `/games/:identifier/day/:day/:phase`.
//!
//! Validates the (day, phase) pair against the timeline summary, fetches the
//! day log via `/api/games/{id}/log/{day}`, filters down to the requested
//! phase, and renders [`FilterChips`] + [`Timeline`].

use crate::cache::QueryError;
use crate::components::TributeFilterChips;
use crate::components::filter_chips::FilterChips;
use crate::components::timeline::{PeriodFilters, Timeline};
use crate::env::APP_API_HOST;
use crate::hooks::use_timeline_summary::use_timeline_summary;
use crate::http::WithCredentials;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::{GameMessage, Phase};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DayLogQ;

impl QueryCapability for DayLogQ {
    type Ok = Vec<GameMessage>;
    type Err = QueryError;
    type Keys = (String, u32);

    async fn run(&self, keys: &(String, u32)) -> Result<Vec<GameMessage>, QueryError> {
        let (id, day) = keys;
        let url = format!("{APP_API_HOST}/api/games/{id}/log/{day}");
        let resp = reqwest::Client::new()
            .get(&url)
            .with_credentials()
            .send()
            .await;
        match resp {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Vec<GameMessage>>().await {
                    Ok(v) => Ok(v),
                    Err(_) => Err(QueryError::BadJson),
                },
                StatusCode::UNAUTHORIZED => Err(QueryError::Unauthorized),
                StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(id.clone())),
                _ => Err(QueryError::Unknown),
            },
            Err(_) => Err(QueryError::ServerNotFound),
        }
    }
}

#[component]
pub fn GamePeriodPage(identifier: String, day: u32, phase: Phase) -> Element {
    let filters: Signal<PeriodFilters> = use_context();
    let filter = filters.read().filter_for(&identifier);
    let tribute_filter = filters.read().tribute_filter(&identifier);

    let summary_q = use_timeline_summary(identifier.clone());

    let valid = {
        let reader = summary_q.read();
        let state = reader.state();
        match &*state {
            QueryStateData::Settled { res: Ok(s), .. } => {
                s.periods.iter().any(|p| p.day == day && p.phase == phase)
            }
            QueryStateData::Settled { res: Err(_), .. } => false,
            _ => true,
        }
    };

    if !valid {
        return rsx! {
            div { class: "space-y-2",
                h1 { class: "text-2xl font-semibold", "Period not found" }
                p { class: "text-gray-600", "Day {day} ({phase}) doesn't exist for this game." }
            }
        };
    }

    let log_q = use_query(Query::new((identifier.clone(), day), DayLogQ));
    let reader = log_q.read();
    let state = reader.state();

    rsx! {
        div { class: "space-y-4",
            h1 { class: "text-2xl font-semibold", "Day {day} — {phase}" }
            FilterChips { game_identifier: identifier.clone() }
            TributeFilterChips { game_identifier: identifier.clone() }
            match &*state {
                QueryStateData::Settled { res: Ok(msgs), .. } => {
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
                            tribute_filter: tribute_filter.clone(),
                        }
                    }
                }
                QueryStateData::Settled { res: Err(_), .. } => rsx! {
                    p { class: "text-red-600", "Failed to load events." }
                },
                _ => rsx! {
                    div { class: "animate-pulse h-32 rounded bg-gray-200" }
                },
            }
        }
    }
}
