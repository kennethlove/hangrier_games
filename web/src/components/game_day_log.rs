use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::env::APP_API_HOST;
use crate::storage::{AppState, use_persistent};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::events::GameEvent;
use game::messages::{GameMessage, MessageKind};
use shared::DisplayGame;

/// Returns (container_classes, leading_glyph) for a categorised message.
/// `None` keeps the legacy plain rendering.
fn kind_styles(kind: &MessageKind) -> (&'static str, &'static str) {
    match kind {
        MessageKind::AllianceFormed => {
            ("border-l-4 border-emerald-500 pl-2 bg-emerald-500/10", "🤝")
        }
        MessageKind::BetrayalTriggered => ("border-l-4 border-rose-500 pl-2 bg-rose-500/10", "🗡️"),
        MessageKind::TrustShockBreak => ("border-l-4 border-amber-500 pl-2 bg-amber-500/10", "💔"),
    }
}

/// Returns (container_classes, leading_glyph) for a structured event variant.
/// Returns `None` for variants without bespoke styling (caller falls back to
/// the legacy `kind`/`content` path).
///
/// Palette pattern matches PR #126: `border-l-4 border-<hue>-500 pl-2
/// bg-<hue>-500/10`. Hues are theme-friendly because the semi-transparent
/// `/10` overlay layers cleanly atop all three theme backgrounds (theme1
/// stone, theme2 green, theme3 stone-dark).
fn event_styles(event: &GameEvent) -> Option<(&'static str, &'static str)> {
    match event {
        // ---- Alliance / social (preserve PR #126 palette + glyph) ----
        GameEvent::AllianceFormed { .. } => {
            Some(("border-l-4 border-emerald-500 pl-2 bg-emerald-500/10", "🤝"))
        }
        GameEvent::BetrayalTriggered { .. } => {
            Some(("border-l-4 border-rose-500 pl-2 bg-rose-500/10", "🗡️"))
        }
        GameEvent::TrustShockBreak { .. } => {
            Some(("border-l-4 border-amber-500 pl-2 bg-amber-500/10", "💔"))
        }

        // ---- Combat outcomes ----
        GameEvent::TributeAttackSuccessKill { .. } => {
            Some(("border-l-4 border-red-600 pl-2 bg-red-600/10", "⚔️"))
        }

        // ---- Deaths ----
        GameEvent::TributeDiesFromStatus { .. } => {
            Some(("border-l-4 border-slate-500 pl-2 bg-slate-500/10", "☠️"))
        }
        GameEvent::TributeDeath { .. } => {
            Some(("border-l-4 border-slate-500 pl-2 bg-slate-500/10", "⚰️"))
        }

        // ---- Area events ----
        GameEvent::AreaEvent { .. } => {
            Some(("border-l-4 border-orange-500 pl-2 bg-orange-500/10", "🌪️"))
        }
        GameEvent::AreaClose { .. } => {
            Some(("border-l-4 border-orange-500 pl-2 bg-orange-500/10", "🚧"))
        }

        _ => None,
    }
}

async fn fetch_game_day_log(
    keys: Vec<QueryKey>,
    token: String,
) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::GameDayLog(identifier, day)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client
            .request(
                reqwest::Method::GET,
                format!("{}/api/games/{}/log/{}", APP_API_HOST, identifier, day),
            )
            .bearer_auth(token);

        match request.send().await {
            Ok(response) => {
                if let Ok(logs) = response.json::<Vec<GameMessage>>().await {
                    QueryResult::Ok(QueryValue::Logs(logs))
                } else {
                    QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
                }
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameDayLog(game: DisplayGame, day: u32) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let identifier = game.identifier.clone();

    let log_query = use_get_query(
        [
            QueryKey::GameDayLog(identifier.clone(), day),
            QueryKey::DisplayGame(identifier.clone()),
            QueryKey::Games,
        ],
        move |keys: Vec<QueryKey>| fetch_game_day_log(keys, token.clone()),
    );

    match log_query.result().value() {
        QueryState::Settled(Ok(QueryValue::Logs(logs))) => {
            rsx! {
                ul {
                    class: r#"
                    theme1:text-stone-200
                    theme2:text-green-200
                    theme3:text-stone-800
                    "#,
                    for log in logs {
                        {
                            // 1. Prefer the structured event payload when
                            //    present and decodable.
                            let structured = game::messages::structured_event(log)
                                .and_then(Result::ok)
                                .and_then(|event| {
                                    event_styles(&event).map(|(classes, glyph)| {
                                        (classes, glyph, event.to_string())
                                    })
                                });

                            match structured {
                                Some((classes, glyph, body)) => rsx! {
                                    li {
                                        class: "{classes}",
                                        span { class: "mr-2", "{glyph}" }
                                        "{body}"
                                    }
                                },
                                // 2. Fall back to the PR #126 kind-based path
                                //    (legacy rows have `event = None`; rows
                                //    whose variant lacks bespoke styling also
                                //    land here).
                                None => match log.kind.as_ref() {
                                    Some(kind) => {
                                        let (classes, glyph) = kind_styles(kind);
                                        rsx! {
                                            li {
                                                class: "{classes}",
                                                span { class: "mr-2", "{glyph}" }
                                                "{log.content}"
                                            }
                                        }
                                    }
                                    _ => rsx! {
                                        li { "{log.content}" }
                                    },
                                },
                            }
                        }
                    }
                }
            }
        }
        QueryState::Settled(Err(_)) => {
            rsx! { p { class: "theme1:text-green-200 theme2:text-green-200", "Failed to load." } }
        }
        QueryState::Loading(_) => {
            rsx! { p { class: "theme1:text-green-200 theme2:text-green-200", "Loading..." } }
        }
        _ => {
            rsx! {}
        }
    }
}
