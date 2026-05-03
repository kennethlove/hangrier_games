//! Per-period detail page: `/games/:identifier/day/:day/:phase`.
//!
//! Validates the (day, phase) pair against the timeline summary, fetches the
//! day log via `/api/games/{id}/log/{day}`, filters down to the requested
//! phase, and renders [`FilterChips`] + [`Timeline`].

use crate::cache::QueryError;
use crate::components::TributeFilterChips;
use crate::components::filter_chips::FilterChips;
use crate::components::timeline::{FilterMode, PeriodFilters, Timeline};
use crate::env::APP_API_HOST;
use crate::hooks::use_timeline_summary::use_timeline_summary;
use crate::http::WithCredentials;
use crate::routes::Routes;
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
pub fn GamePeriodPage(
    identifier: String,
    day: u32,
    phase: Phase,
    filter: String,
    tribute: String,
) -> Element {
    let mut filters: Signal<PeriodFilters> = use_context();

    // One-shot URL → context seeding. If the URL carries `?filter=`/`?tribute=`,
    // it wins over any persisted gloo-storage state on first render so that
    // shared links land on exactly the slice they describe (hangrier_games-nil).
    let seed_id = identifier.clone();
    let seed_filter = filter.clone();
    let seed_tribute = tribute.clone();
    use_hook(move || {
        let mut f = filters.write();
        f.hydrate(&seed_id);
        if !seed_filter.is_empty() {
            f.set_filter(&seed_id, FilterMode::from_query_value(&seed_filter));
        }
        if !seed_tribute.is_empty() {
            f.set_tribute_filter(&seed_id, Some(seed_tribute));
        }
    });

    let filter_mode = filters.read().filter_for(&identifier);
    let tribute_filter = filters.read().tribute_filter(&identifier);

    // context → URL: whenever the filter or tribute selection changes, replace
    // the current history entry so the URL always reflects the visible slice.
    {
        let nav_id = identifier.clone();
        let nav_filter = filter_mode.to_query_value();
        let nav_tribute = tribute_filter.clone().unwrap_or_default();
        let url_filter = filter.clone();
        let url_tribute = tribute.clone();
        let navigator = use_navigator();
        use_effect(move || {
            if nav_filter == url_filter && nav_tribute == url_tribute {
                return;
            }
            navigator.replace(Routes::GamePeriodPage {
                identifier: nav_id.clone(),
                day,
                phase,
                filter: nav_filter.clone(),
                tribute: nav_tribute.clone(),
            });
        });
    }

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
                            filter: filter_mode.clone(),
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
