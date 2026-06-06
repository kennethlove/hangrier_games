use crate::{authenticate_db, extract_auth, html_with_csrf, require_auth, validate_csrf};
use api::AppState;
use api::cookies::{SESSION_COOKIE, read_cookie};
use api::templates::game_detail;
use api::templates::pages;
use axum::Form;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use shared::ListDisplayGame;
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
    html_with_csrf(pages::home_page(auth), &csrf)
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
    let pagination = shared::PaginationMetadata {
        total,
        limit,
        offset,
        has_more,
    };
    let paginated = shared::PaginatedGames { games, pagination };
    let stats = api::templates::pages::GameStats::from_games(&all_games);
    let active_filter = params.status.as_deref().unwrap_or("");

    html_with_csrf(
        pages::games_list_page(auth, &paginated, &stats, active_filter),
        &csrf,
    )
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

/// GET /games/{id} — game detail page.
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

    match game {
        Some(game) => html_with_csrf(game_detail::game_detail_page(auth, &game), &csrf),
        None => html_with_csrf(
            pages::not_found_page(auth, "The game you're looking for doesn't exist."),
            &csrf,
        ),
    }
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
            let tributes: Vec<Vec<SerdeWrapper<game::tributes::Tribute>>> =
                result.take("tributes").unwrap_or_default();
            tributes
                .into_iter()
                .next()
                .map(|inner| inner.into_iter().map(|w| w.0).collect())
                .unwrap_or_default()
        }
        Err(_) => vec![],
    };

    html_with_csrf(
        game_detail::tributes_page(auth, &identifier, &tributes),
        &csrf,
    )
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

    html_with_csrf(game_detail::areas_page(auth, &identifier, &areas), &csrf)
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

    html_with_csrf(
        game_detail::log_page(auth, &identifier, &messages, &segments),
        &csrf,
    )
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

    html_with_csrf(
        api::templates::auth::account_page(auth, &session, &games),
        &csrf,
    )
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

    html_with_csrf(
        api::templates::auth::account_settings_page(auth, &session, &current_email, avatar_url),
        &csrf,
    )
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
            .take::<Option<SerdeWrapper<game::tributes::Tribute>>>(0)
            .unwrap_or_default()
            .map(|w| w.0),
        Err(_) => None,
    };

    match tribute {
        Some(tribute) => html_with_csrf(
            api::templates::tribute_detail::tribute_detail_page(auth, &game_id, &tribute),
            &csrf,
        ),
        None => html_with_csrf(
            pages::not_found_page(auth, "The tribute you're looking for doesn't exist."),
            &csrf,
        ),
    }
}

/// GET /games/new — create game form (requires auth).
pub async fn create_game_handler(headers: axum::http::HeaderMap) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    if !auth.is_authenticated() {
        return Redirect::to("/auth?tab=login").into_response();
    }

    let body = api::templates::auth::create_game_page_with_csrf(auth, &csrf);
    html_with_csrf(body, &csrf)
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

    let new_game = Game {
        identifier: game_identifier.clone(),
        name: game_name,
        status: shared::GameStatus::NotStarted,
        day: None,
        tributes: vec![],
        areas: vec![],
        private: is_private,
        config: Default::default(),
        messages: vec![],
        alliance_events: vec![],
        ..Default::default()
    };

    use surrealdb_types::RecordId;

    let game_rid = RecordId::new("game", game_identifier.as_str());
    let body = match serde_json::to_value(&new_game) {
        Ok(b) => b,
        Err(_) => return Redirect::to("/games/new").into_response(),
    };

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
