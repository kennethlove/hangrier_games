use crate::{authenticate_db, extract_auth, html_with_csrf, require_auth, validate_csrf};
use api::AppState;
use api::cookies::{SESSION_COOKIE, read_cookie};
use api::templates::game_detail;
use api::templates::tera_engine;
use axum::Form;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use shared::ListDisplayGame;
use std::str::FromStr;
use surrealdb_types::SerdeWrapper;

// ── Game list types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct GamesListQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub status: Option<String>,
}

fn default_limit() -> u32 {
    10
}

#[derive(Deserialize)]
pub struct CreateGameRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub private: Option<String>,
    #[serde(default)]
    pub csrf_token: String,
}

// ── HTMX page handlers ──────────────────────────────────────────────

/// GET / — home page.
pub async fn home_handler(headers: axum::http::HeaderMap) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let mut ctx = tera_engine::base_context("Home", &auth);
    ctx.insert(
        "stats",
        &serde_json::json!({"running": 0, "waiting": 0, "finished": 0, "total": 0}),
    );
    html_with_csrf(tera_engine::render("home.html", &ctx), &csrf)
}

/// GET /games — list paginated games.
pub async fn games_list_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<GamesListQuery>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let limit = params.limit.min(100);
    let offset = params.offset.min(10000);

    let result = state
        .db
        .query("SELECT * FROM fn::get_list_games($limit, $offset)")
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await;

    let all_games: Vec<ListDisplayGame> = match result {
        Ok(mut result) => result
            .take::<Vec<SerdeWrapper<ListDisplayGame>>>(0)
            .unwrap_or_default()
            .into_iter()
            .map(|w| w.0)
            .collect(),
        Err(_) => vec![],
    };

    let games = filter_games_by_status(&all_games, params.status.as_deref());
    let total = all_games.len() as u32;
    let has_more = (offset + limit) < total;
    let active_filter = params.status.as_deref().unwrap_or("");

    // Compute stats
    let running = all_games
        .iter()
        .filter(|g| g.status == shared::GameStatus::InProgress)
        .count() as u32;
    let waiting = all_games
        .iter()
        .filter(|g| g.status == shared::GameStatus::NotStarted)
        .count() as u32;
    let finished = all_games
        .iter()
        .filter(|g| g.status == shared::GameStatus::Finished)
        .count() as u32;

    let mut ctx = tera_engine::base_context("Games", &auth);
    ctx.insert("stats", &serde_json::json!({"running": running, "waiting": waiting, "finished": finished, "total": total}));
    ctx.insert("games", &games);
    ctx.insert("active_filter", active_filter);
    ctx.insert(
        "pagination",
        &shared::PaginationMetadata {
            total,
            limit,
            offset: offset + limit,
            has_more,
        },
    );

    html_with_csrf(tera_engine::render("games_list.html", &ctx), &csrf)
}

fn filter_games_by_status(games: &[ListDisplayGame], status: Option<&str>) -> Vec<ListDisplayGame> {
    match status {
        Some("running") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::InProgress)
            .cloned()
            .collect(),
        Some("waiting") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::NotStarted)
            .cloned()
            .collect(),
        Some("finished") => games
            .iter()
            .filter(|g| g.status == shared::GameStatus::Finished)
            .cloned()
            .collect(),
        Some(_) | None => games.to_vec(),
    }
}

/// GET /games/{id} — game detail page (broadcast interface).
pub async fn game_detail_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();

    let result = state
        .db
        .query("SELECT * FROM fn::get_display_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let game = match result {
        Ok(mut result) => {
            let game: Option<SerdeWrapper<shared::DisplayGame>> =
                result.take(0).unwrap_or_default();
            game.map(|w| w.0)
        }
        Err(_) => None,
    };

    let Some(game) = game else {
        let mut ctx = tera_engine::base_context("Not Found", &auth);
        ctx.insert("message", "The game you're looking for doesn't exist.");
        return html_with_csrf(tera_engine::render("not_found.html", &ctx), &csrf);
    };

    let tributes_result = state
        .db
        .query("SELECT * FROM fn::get_tributes_by_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let tributes: Vec<game::tributes::Tribute> = match tributes_result {
        Ok(mut result) => {
            let raw_rows: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
            raw_rows
                .into_iter()
                .filter_map(|row| row["tributes"].as_array().cloned())
                .flatten()
                .filter_map(|t| serde_json::from_value(t).ok())
                .collect()
        }
        Err(_) => vec![],
    };

    // Fetch areas with terrain + tribute slot assignments
    let areas_result = state
        .db
        .query(
            r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = $identifier;
"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let areas: Vec<game::areas::AreaDetails> = match areas_result {
        Ok(mut result) => {
            let rows: Vec<Vec<SerdeWrapper<game::areas::AreaDetails>>> =
                result.take("areas").unwrap_or_default();
            rows.into_iter()
                .next()
                .map(|inner| inner.into_iter().map(|w| w.0).collect())
                .unwrap_or_default()
        }
        Err(_) => vec![],
    };

    let messages_result = state
        .db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let messages: Vec<shared::messages::GameMessage> = match messages_result {
        Ok(mut logs) => {
            let rows: Vec<SerdeWrapper<api::games::GameLog>> = logs.take(0).unwrap_or_default();
            rows.into_iter()
                .map(|w| shared::messages::GameMessage::from(w.0))
                .collect()
        }
        Err(_) => vec![],
    };

    let commentary_result = state
        .db
        .query(
            r#"SELECT * FROM commentary_segments
            WHERE game_id = $identifier
            ORDER BY day, phase;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let segments: Vec<announcers::CommentarySegment> = match commentary_result {
        Ok(mut result) => result
            .take::<Vec<SerdeWrapper<announcers::CommentarySegment>>>(0)
            .unwrap_or_default()
            .into_iter()
            .map(|w| w.0)
            .collect(),
        Err(_) => vec![],
    };

    // Pre-compute context data
    let alive = tributes.iter().filter(|t| t.is_alive()).count() as u32;
    let fallen = tributes.len() as u32 - alive;
    let total = tributes.len() as u32;

    let (phase_class, phase_label) = game_detail::current_broadcast_phase(&game, &messages);

    // Sort tributes: alive first, then alphabetically
    let mut sorted_tributes: Vec<_> = tributes.iter().collect();
    sorted_tributes.sort_by(|a, b| b.is_alive().cmp(&a.is_alive()).then(a.name.cmp(&b.name)));

    // Day numbers from messages
    let mut day_numbers: Vec<u32> = messages.iter().map(|m| m.game_day).collect();
    day_numbers.sort();
    day_numbers.dedup();
    let current_day = game.day.unwrap_or(0);

    // Pre-render event cards
    let mut event_cards = String::new();
    for msg in &messages {
        event_cards.push_str(&game_detail::render_event_card(msg));
    }
    for seg in &segments {
        event_cards.push_str(&game_detail::render_commentary_card(seg));
    }

    // Pre-render tribute rows — grouped by alliance
    let alliance_groups = game_detail::build_alliance_groups(&sorted_tributes);
    let mut tribute_rows = String::new();
    for group in &alliance_groups {
        tribute_rows.push_str(&game_detail::render_alliance_group(group, &sorted_tributes));
    }

    // Build hex arena map SVG
    let hex_map = game_detail::render_hex_map(&areas, &sorted_tributes);

    // SSE events string
    let sse_events = "death,wound,attack,combat,alliance_formed,alliance_proposed,alliance_dissolved,betrayal,trust_shock_break,sponsor_gift,movement,hidden,area_closed,area_event,item_found,item_used,item_dropped,rested,starved,dehydrated,sanity_break,hunger_band_changed,thirst_band_changed,stamina_band_changed,shelter_sought,foraged,drank,ate,cycle_start,cycle_end,phase_started,phase_ended,slept,woke,game_ended,wounded,attacked,affliction_acquired,affliction_progressed,affliction_healed,affliction_cascaded,trauma_acquired,trauma_reinforced,trauma_escalated,trauma_flashback,trauma_avoidance,trauma_observed,trauma_forgotten,trauma_habituated,phobia_acquired,phobia_triggered,phobia_escalated,phobia_habituated,phobia_observed,phobia_forgotten,fixation_acquired,fixation_escalated,fixation_fired,fixation_consummated,fixation_thwarted,fixation_faded,generic,trapped,struggling,trapped_escaped,died_while_trapped,trap_set,trap_triggered,rescue_attempted,sleep_incident,partial_rescue_progress";

    let mut ctx = tera_engine::base_context(&game.name, &auth);
    ctx.insert("body_class", "broadcast");
    ctx.insert("game", &game);
    ctx.insert("alive", &alive);
    ctx.insert("fallen", &fallen);
    ctx.insert("total", &total);
    ctx.insert("phase_class", phase_class);
    ctx.insert("phase_label", phase_label);
    ctx.insert("day_numbers", &day_numbers);
    ctx.insert("current_day", &current_day);
    ctx.insert("sse_events", sse_events);
    ctx.insert("tribute_rows", &tribute_rows);
    ctx.insert("hex_map", &hex_map);
    ctx.insert("event_cards", &event_cards);
    ctx.insert("messages", &messages);
    ctx.insert("segments", &segments);

    html_with_csrf(tera_engine::render("game_detail.html", &ctx), &csrf)
}

/// GET /games/{id}/tributes — tributes for a game.
pub async fn game_tributes_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query("SELECT * FROM fn::get_tributes_by_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let tributes = match result {
        Ok(mut result) => {
            let raw_rows: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
            raw_rows
                .into_iter()
                .filter_map(|row| row["tributes"].as_array().cloned())
                .flatten()
                .filter_map(|t| serde_json::from_value(t).ok())
                .collect()
        }
        Err(_) => vec![],
    };

    // Pre-render tribute cards grouped by district
    let mut tribute_cards = String::new();
    for tribute in &tributes {
        tribute_cards.push_str(&game_detail::render_tribute_card(tribute));
    }

    let mut ctx = tera_engine::base_context("Tributes", &auth);
    ctx.insert("game_id", &identifier);
    ctx.insert("tribute_cards", &tribute_cards);
    html_with_csrf(tera_engine::render("tributes.html", &ctx), &csrf)
}

/// GET /games/{id}/areas — areas for a game.
pub async fn game_areas_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query(
            r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = $identifier;
"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let areas = match result {
        Ok(mut result) => {
            let areas: Vec<Vec<SerdeWrapper<game::areas::AreaDetails>>> =
                result.take("areas").unwrap_or_default();
            areas
                .into_iter()
                .next()
                .map(|inner| inner.into_iter().map(|w| w.0).collect())
                .unwrap_or_default()
        }
        Err(_) => vec![],
    };

    // Pre-render area cards
    let mut area_cards = String::new();
    for area in &areas {
        area_cards.push_str(&game_detail::render_area_card(area));
    }

    let mut ctx = tera_engine::base_context("Areas", &auth);
    ctx.insert("game_id", &identifier);
    ctx.insert("area_cards", &area_cards);
    html_with_csrf(tera_engine::render("areas.html", &ctx), &csrf)
}

/// GET /games/{id}/log — event log for a game with commentary.
pub async fn game_log_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();
    let result = state
        .db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let messages = match result {
        Ok(mut logs) => {
            let rows: Vec<SerdeWrapper<api::games::GameLog>> = logs.take(0).unwrap_or_default();
            rows.into_iter()
                .map(|w| shared::messages::GameMessage::from(w.0))
                .collect()
        }
        Err(_) => vec![],
    };

    let commentary_result = state
        .db
        .query(
            r#"SELECT * FROM commentary_segments
            WHERE game_id = $identifier
            ORDER BY day, phase;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let segments = match commentary_result {
        Ok(mut result) => result
            .take::<Vec<SerdeWrapper<announcers::CommentarySegment>>>(0)
            .unwrap_or_default()
            .into_iter()
            .map(|w| w.0)
            .collect(),
        Err(_) => vec![],
    };

    // Pre-render event cards
    let mut event_cards = String::new();
    for msg in &messages {
        event_cards.push_str(&game_detail::render_event_card(msg));
    }
    for seg in &segments {
        event_cards.push_str(&game_detail::render_commentary_card(seg));
    }

    let mut ctx = tera_engine::base_context("Event Log", &auth);
    ctx.insert("game_id", &identifier);
    ctx.insert("event_cards", &event_cards);
    html_with_csrf(tera_engine::render("log.html", &ctx), &csrf)
}

#[derive(Deserialize)]
pub struct TimelineQuery {
    pub filter: Option<String>,
    pub tribute: Option<String>,
    pub day: Option<u32>,
    pub phase: Option<String>,
}

/// GET /games/{id}/timeline — timeline view with period grid, filters, and event cards.
pub async fn timeline_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(game_identifier): axum::extract::Path<uuid::Uuid>,
    Query(params): Query<TimelineQuery>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();

    // ── Fetch game ─────────────────────────────────────────────────
    let game_result = state
        .db
        .query("SELECT * FROM fn::get_display_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let game = match game_result {
        Ok(mut result) => {
            let game: Option<SerdeWrapper<shared::DisplayGame>> =
                result.take(0).unwrap_or_default();
            game.map(|w| w.0)
        }
        Err(_) => None,
    };

    let game = match game {
        Some(g) => g,
        None => {
            let mut ctx = tera_engine::base_context("Not Found", &auth);
            ctx.insert("message", "Game not found.");
            return html_with_csrf(tera_engine::render("not_found.html", &ctx), &csrf);
        }
    };

    // ── Fetch tributes ─────────────────────────────────────────────
    let tributes_result = state
        .db
        .query("SELECT * FROM fn::get_tributes_by_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await;

    let tribute_refs: Vec<shared::messages::TributeRef> = match tributes_result {
        Ok(mut result) => {
            let raw_rows: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
            raw_rows
                .into_iter()
                .filter_map(|row| row["tributes"].as_array().cloned())
                .flatten()
                .filter_map(|t| {
                    let id = t.get("identifier")?.as_str()?.to_string();
                    let name = t.get("name")?.as_str()?.to_string();
                    Some(shared::messages::TributeRef {
                        identifier: id.into(),
                        name,
                    })
                })
                .collect()
        }
        Err(_) => vec![],
    };

    // ── Fetch messages ─────────────────────────────────────────────
    let messages_result = state
        .db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", identifier.clone()))
        .await;

    let messages: Vec<shared::messages::GameMessage> = match messages_result {
        Ok(mut logs) => {
            let rows: Vec<SerdeWrapper<api::games::GameLog>> = logs.take(0).unwrap_or_default();
            rows.into_iter()
                .map(|w| shared::messages::GameMessage::from(w.0))
                .collect()
        }
        Err(_) => vec![],
    };

    let current_day = game.day.unwrap_or(0);
    let current_phase = messages
        .iter()
        .filter(|m| m.game_day == current_day)
        .max_by_key(|m| (m.phase, m.tick, m.emit_index))
        .map(|m| m.phase)
        .unwrap_or(shared::messages::Phase::Day);

    let periods = shared::messages::summarize_periods(&messages, (current_day, current_phase));

    let filter_str = params.filter.as_deref().unwrap_or("");
    let tribute_filter_str = params.tribute.as_deref().unwrap_or("");
    let selected_day = params.day;
    let selected_phase = params
        .phase
        .as_deref()
        .and_then(|p| shared::messages::Phase::from_str(p).ok());

    // ── Filter events ──────────────────────────────────────────────
    let filtered_events: Vec<shared::messages::GameMessage> = messages
        .iter()
        .filter(|m| {
            if let (Some(day), Some(phase)) = (selected_day, selected_phase)
                && (m.game_day != day || m.phase != phase)
            {
                return false;
            }
            if !filter_str.is_empty() {
                let kind = m.payload.kind();
                let kind_str = match kind {
                    shared::messages::MessageKind::TributeKilled => "death",
                    shared::messages::MessageKind::Combat
                    | shared::messages::MessageKind::CombatSwing
                    | shared::messages::MessageKind::TributeAttacked
                    | shared::messages::MessageKind::TributeWounded => "combat",
                    shared::messages::MessageKind::AllianceFormed
                    | shared::messages::MessageKind::AllianceProposed
                    | shared::messages::MessageKind::AllianceDissolved
                    | shared::messages::MessageKind::BetrayalTriggered
                    | shared::messages::MessageKind::TrustShockBreak => "alliance",
                    shared::messages::MessageKind::TributeMoved
                    | shared::messages::MessageKind::TributeHidden
                    | shared::messages::MessageKind::AreaClosed
                    | shared::messages::MessageKind::AreaEvent => "movement",
                    shared::messages::MessageKind::ItemFound
                    | shared::messages::MessageKind::ItemUsed
                    | shared::messages::MessageKind::ItemDropped
                    | shared::messages::MessageKind::SponsorGift => "items",
                    _ => "",
                };
                if kind_str != filter_str {
                    return false;
                }
            }
            if !tribute_filter_str.is_empty() {
                let tribute_id = tribute_refs
                    .iter()
                    .find(|t| t.name == tribute_filter_str)
                    .map(|t| t.identifier.as_str());
                if let Some(id) = tribute_id {
                    if !m.payload.involves(id) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    // ── Pre-render event cards ─────────────────────────────────────
    let rendered_events: Vec<String> = filtered_events
        .iter()
        .map(game_detail::render_event_card)
        .collect();

    // ── Build template context ─────────────────────────────────────
    let mut ctx = tera_engine::base_context(&format!("Timeline — {}", game.name), &auth);

    ctx.insert("game_id", &identifier);
    ctx.insert("game_name", &game.name);
    ctx.insert("current_day", &current_day);
    ctx.insert("current_phase", &current_phase.to_string());
    ctx.insert("periods", &periods);
    ctx.insert("filter", filter_str);
    ctx.insert("tribute_filter", tribute_filter_str);
    ctx.insert("tributes", &tribute_refs);
    ctx.insert("rendered_events", &rendered_events);
    ctx.insert("selected_day", &selected_day);
    ctx.insert("selected_phase", &selected_phase.map(|p| p.to_string()));

    let filter_options = vec![
        serde_json::json!({"value": "", "label": "All", "icon_name": "list"}),
        serde_json::json!({"value": "death", "label": "Deaths", "icon_name": "skull"}),
        serde_json::json!({"value": "combat", "label": "Combat", "icon_name": "sword"}),
        serde_json::json!({"value": "alliance", "label": "Alliances", "icon_name": "users"}),
        serde_json::json!({"value": "movement", "label": "Movement", "icon_name": "map-pin"}),
        serde_json::json!({"value": "items", "label": "Items", "icon_name": "backpack"}),
    ];
    ctx.insert("filter_options", &filter_options);

    let rendered = tera_engine::render("timeline.html", &ctx);
    html_with_csrf(rendered, &csrf)
}

/// GET /account — account dashboard (requires auth).
pub async fn account_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let session = match require_auth(&state, &headers).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/auth?tab=login").into_response(),
    };

    let user_db = (*state.db).clone();
    if user_db
        .use_ns(&state.namespace)
        .use_db(&state.database)
        .await
        .is_err()
    {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let games: Vec<ListDisplayGame> = user_db
        .query("SELECT * FROM fn::get_list_games(100, 0)")
        .await
        .ok()
        .and_then(|mut r| r.take::<Vec<SerdeWrapper<ListDisplayGame>>>(0).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|w| w.0)
        .collect();

    let mut ctx = tera_engine::base_context("Account", &auth);
    ctx.insert("session", &session);
    ctx.insert("games", &games);
    html_with_csrf(tera_engine::render("account.html", &ctx), &csrf)
}

/// GET /account/settings — account settings page (requires auth).
pub async fn account_settings_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let session = match require_auth(&state, &headers).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/auth?tab=login").into_response(),
    };

    // Clone and authenticate DB to query email
    let token = match read_cookie(&headers, SESSION_COOKIE) {
        Some(t) => t.to_owned(),
        None => return Redirect::to("/auth?tab=login").into_response(),
    };
    let user_db = match authenticate_db(&state, &token).await {
        Ok(db) => db,
        Err(_) => return Redirect::to("/auth?tab=login").into_response(),
    };

    #[derive(serde::Deserialize, serde::Serialize)]
    struct EmailRow {
        email: String,
    }
    let current_email: String = user_db
        .query("SELECT email FROM $auth")
        .await
        .ok()
        .and_then(|mut r| r.take::<Option<SerdeWrapper<EmailRow>>>(0).ok())
        .flatten()
        .map(|w| w.0.email)
        .unwrap_or_default();

    // Convert avatar storage path to public URL
    let avatar_url: Option<String> = session
        .avatar
        .as_ref()
        .map(|path| state.storage.public_url(path));

    let mut ctx = tera_engine::base_context("Account Settings", &auth);
    ctx.insert("session", &session);
    ctx.insert("current_email", &current_email);
    if let Some(ref url) = avatar_url {
        ctx.insert("avatar_url", url);
    }
    html_with_csrf(tera_engine::render("account_settings.html", &ctx), &csrf)
}

/// GET /games/{id}/tributes/{tribute_id} — tribute detail page.
pub async fn game_tribute_detail_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Path((game_identifier, tribute_identifier)): axum::extract::Path<(
        uuid::Uuid,
        uuid::Uuid,
    )>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let game_id = game_identifier.to_string();
    let tribute_id = tribute_identifier.to_string();

    let result = state
        .db
        .query("SELECT * FROM fn::get_full_tribute($identifier);")
        .bind(("identifier", tribute_id.clone()))
        .await;

    let tribute = match result {
        Ok(mut result) => result
            .take::<Vec<serde_json::Value>>(0)
            .unwrap_or_default()
            .into_iter()
            .next()
            .and_then(|v| serde_json::from_value(v).ok()),
        Err(_) => None,
    };

    match tribute {
        Some(tribute) => {
            // Pre-render tribute detail content
            let tribute_html = game_detail::render_tribute_detail(&tribute, &game_id);
            let mut ctx = tera_engine::base_context(&tribute.name, &auth);
            ctx.insert("game_id", &game_id);
            ctx.insert("tribute_detail_html", &tribute_html);
            html_with_csrf(tera_engine::render("tribute_detail.html", &ctx), &csrf)
        }
        None => {
            let mut ctx = tera_engine::base_context("Not Found", &auth);
            ctx.insert("message", "The tribute you're looking for doesn't exist.");
            html_with_csrf(tera_engine::render("not_found.html", &ctx), &csrf)
        }
    }
}

/// GET /games/new — create game form (requires auth).
pub async fn create_game_handler(headers: axum::http::HeaderMap) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    if !auth.is_authenticated() {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let ctx = tera_engine::base_context("Create Game", &auth);
    html_with_csrf(tera_engine::render("create_game.html", &ctx), &csrf)
}

/// POST /games/new — create game, redirect to /games/{id}.
pub async fn create_game_post_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<CreateGameRequest>,
) -> Response {
    if !validate_csrf(&headers, &form.csrf_token) {
        return Redirect::to("/auth").into_response();
    }

    let token = match read_cookie(&headers, SESSION_COOKIE) {
        Some(t) => t.to_owned(),
        None => return Redirect::to("/auth").into_response(),
    };

    let user_db = match authenticate_db(&state, &token).await {
        Ok(db) => db,
        Err(redirect) => return redirect.into_response(),
    };

    use game::games::Game;
    let game = Game::default();
    let game_identifier = uuid::Uuid::new_v4().to_string();
    let game_name = form.name.filter(|n| !n.is_empty()).unwrap_or(game.name);
    let is_private = form.private.is_some_and(|v| v == "true");

    use surrealdb_types::RecordId;

    let game_rid = RecordId::new("game", game_identifier.as_str());
    let body = serde_json::json!({
        "identifier": &game_identifier,
        "name": &game_name,
        "status": "NotStarted",
        "day": null,
        "private": is_private,
    });

    if user_db
        .query("UPSERT $rid CONTENT $body")
        .bind(("rid", game_rid.clone()))
        .bind(("body", body))
        .await
        .is_err()
    {
        return Redirect::to("/games/new").into_response();
    }

    let tribute_futures = (0..24)
        .map(|idx| api::tributes::create_tribute(None, &game_identifier, &user_db, idx % 12));
    let tribute_results = futures::future::join_all(tribute_futures).await;
    if tribute_results.into_iter().any(|r| r.is_err()) {
        return Redirect::to("/games/new").into_response();
    }

    use game::areas::Area;
    use strum::IntoEnumIterator;
    let base_item_count = shared::ItemQuantity::default().base_item_count();
    let area_futures = Area::iter()
        .map(|area| api::games::create_area(&game_identifier, area, base_item_count, &user_db));
    let area_results = futures::future::join_all(area_futures).await;
    if area_results.into_iter().any(|r| r.is_err()) {
        return Redirect::to("/games/new").into_response();
    }

    Redirect::to(&format!("/games/{game_identifier}")).into_response()
}
