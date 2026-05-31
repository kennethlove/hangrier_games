use super::*;
use crate::{AppError, AppState, AuthDb};
use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use game::games::Game;
use game::items::Item;
use game::messages::GameMessage;
use game::tributes::Tribute;
use shared::{
    CreateGame, DisplayGame, EditGame, GameStatus, ListDisplayGame, PaginatedGames,
    PaginationMetadata,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use surrealdb::RecordId;
use surrealdb::sql::Thing;
use uuid::Uuid;
use validator::Validate;

/// Creates a new game with fully initialized tributes, areas, and items.
/// The request body is validated before any database writes begin.
pub async fn create_game(
    Extension(AuthDb(db)): Extension<AuthDb>,
    Json(payload): Json<CreateGame>,
) -> Result<Response, AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(format!("{}", e)))?;

    // Generate server-controlled fields. Game::default() runs WPGen to
    // produce a three-word "clever" name; use it as the fallback when
    // the client didn't supply one (e.g. Quickstart).
    let default_game = Game::default();
    let game_identifier = Uuid::new_v4().to_string();
    let game_name = payload.name.unwrap_or(default_game.name);

    // Construct Game with server-controlled fields
    let game = Game {
        identifier: game_identifier.clone(),
        name: game_name,
        status: GameStatus::NotStarted,
        day: None,
        tributes: vec![],
        areas: vec![],
        private: true, // Default to private
        config: Default::default(),
        messages: vec![],
        alliance_events: vec![],
        ..Default::default()
    };

    // Use serde_json::Value + bound CONTENT to bypass the SurrealDB SDK's
    // bespoke serializer (which collapses externally-tagged enums and drops
    // Option fields like `day`). Same pattern as save_game.
    let game_rid = RecordId::from(("game", game_identifier.as_str()));
    let body = serde_json::to_value(&game)
        .map_err(|e| AppError::InternalServerError(format!("Failed to encode game: {}", e)))?;
    db.query("UPSERT $rid CONTENT $body")
        .bind(("rid", game_rid.clone()))
        .bind(("body", body))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create game: {}", e)))?;
    crate::verify_record_persisted(&db, &game_rid, "create_game").await?;

    let created_game = game;

    // Create tributes concurrently
    let tribute_futures =
        (0..24).map(|idx| crate::tributes::create_tribute(None, &game_identifier, &db, idx % 12));
    let tribute_results = futures::future::join_all(tribute_futures).await;

    if let Some(err) = tribute_results.into_iter().find_map(Result::err) {
        return Err(AppError::InternalServerError(format!(
            "Failed to create tributes: {}",
            err
        )));
    }

    // Apply game customization settings
    let base_item_count = payload.item_quantity.base_item_count();

    // Create areas concurrently with customized item count
    let area_futures = Area::iter()
        .map(|area| super::create_area(game_identifier.as_str(), area, base_item_count, &db));
    let area_results = futures::future::join_all(area_futures).await;

    if let Some(err) = area_results.into_iter().find_map(Result::err) {
        let detail = match &err {
            AppError::InternalServerError(s) | AppError::BadRequest(s) | AppError::DbError(s) => {
                s.clone()
            }
            other => other.to_string(),
        };
        return Err(AppError::InternalServerError(format!(
            "Failed to create areas: {}",
            detail
        )));
    }

    let location: axum::http::HeaderValue =
        axum::http::HeaderValue::from_str(&format!("/api/games/{}", game_identifier)).map_err(
            |e| AppError::InternalServerError(format!("Invalid Location header: {}", e)),
        )?;
    let mut response = (StatusCode::CREATED, Json(created_game)).into_response();
    response
        .headers_mut()
        .insert(axum::http::header::LOCATION, location);
    Ok(response)
}

/// Delete a game and all its associated pieces (tributes, items, areas).
pub async fn game_delete(
    game_identifier: Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<StatusCode, AppError> {
    let game_identifier = game_identifier.to_string();
    let mut result = db
        .query(
            r#"
        SELECT * FROM fn::get_tributes_items_by_game($game_id);
        SELECT * FROM fn::get_areas_items_by_game($game_id);
    "#,
        )
        .bind(("game_id", game_identifier.clone()))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to fetch game pieces: {}", e))
        })?;

    let game_pieces: Option<HashMap<String, Vec<Thing>>> = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take game pieces: {}", e)))?;
    let area_pieces: Option<HashMap<String, Vec<Thing>>> = result
        .take(1)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take area pieces: {}", e)))?;
    if let Some(pieces) = game_pieces {
        super::delete_pieces(pieces, &db).await?
    };
    if let Some(pieces) = area_pieces {
        super::delete_pieces(pieces, &db).await?
    };

    let game: Option<Game> = db
        .delete(("game", &game_identifier))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete game: {}", e)))?;
    match game {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(AppError::InternalServerError(
            "Failed to delete game".into(),
        )),
    }
}

/// List games with pagination.
pub async fn game_list(
    Extension(AuthDb(db)): Extension<AuthDb>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedGames>, AppError> {
    // Validate pagination parameters
    params
        .validate()
        .map_err(|e| AppError::ValidationError(format!("{}", e)))?;

    let result = db
        .query("SELECT * FROM fn::get_list_games($limit, $offset)")
        .bind(("limit", params.limit))
        .bind(("offset", params.offset))
        .await;

    match result {
        Ok(mut result) => {
            let games: Vec<ListDisplayGame> = match result.take(0) {
                Ok(games) => games,
                Err(e) => {
                    return Err(AppError::InternalServerError(format!(
                        "Failed to parse games: {}",
                        e
                    )));
                }
            };

            let total = games.len() as u32;
            let has_more = (params.offset + params.limit) < total;
            let pagination = PaginationMetadata {
                total,
                limit: params.limit,
                offset: params.offset,
                has_more,
            };

            Ok(Json(PaginatedGames { games, pagination }))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to fetch games: {}",
            e
        ))),
    }
}

/// Get detailed game info.
pub async fn game_detail(
    game_identifier: Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<DisplayGame>, AppError> {
    let identifier = game_identifier.to_string();

    let mut result = db
        .query("SELECT * FROM fn::get_detail_game($identifier)")
        .bind(("identifier", identifier.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch game: {}", e)))?;

    let game: Option<DisplayGame> = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take game: {}", e)))?;

    if let Some(game) = game {
        Ok(Json(game))
    } else {
        Err(AppError::NotFound("Failed to find game".into()))
    }
}

/// Update game metadata (name, private flag).
pub async fn game_update(
    Path(game_identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
    Json(payload): Json<EditGame>,
) -> Result<Json<Game>, AppError> {
    // Validate input
    if let Err(e) = validator::Validate::validate(&payload) {
        return Err(AppError::ValidationError(format!("{}", e)));
    }

    let response = db
        .query(
            r#"
        UPDATE game
        SET name = $name, private = $private
        WHERE identifier = $identifier;
        "#,
        )
        .bind(("identifier", game_identifier.to_string()))
        .bind(("name", payload.name.clone()))
        .bind(("private", payload.private))
        .await;

    match response {
        Ok(mut response) => {
            let game: Option<Game> = response.take(0).map_err(|e| {
                AppError::InternalServerError(format!("Failed to take game: {}", e))
            })?;
            if let Some(game) = game {
                Ok(Json::<Game>(game))
            } else if game.is_none() {
                Err(AppError::NotFound("Failed to find game".into()))
            } else {
                unreachable!()
            }
        }
        Err(_) => Err(AppError::InternalServerError(
            "Failed to update game".into(),
        )),
    }
}

/// Get areas belonging to a game.
pub async fn game_areas(
    Path(identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Vec<AreaDetails>>, AppError> {
    let response = db
        .query(
            r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
) AS areas FROM game WHERE identifier = $identifier;
"#,
        )
        .bind(("identifier", identifier.to_string()))
        .await;

    match response {
        Ok(mut response) => {
            let areas: Vec<Vec<AreaDetails>> = response.take("areas").map_err(|e| {
                AppError::InternalServerError(format!("Failed to take areas: {}", e))
            })?;
            Ok(Json::<Vec<AreaDetails>>(areas[0].clone()))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to fetch areas: {}",
            e
        ))),
    }
}

/// Fetch a single area belonging to a specific game, including its items.
/// Returns 404 if the area is not bound to the game.
pub async fn area_detail(
    Path((game_identifier, area_identifier)): Path<(Uuid, Uuid)>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<AreaDetails>, AppError> {
    let mut response = db
        .query(
            r#"
SELECT (
    SELECT *, ->items->item[*] AS items
    FROM ->areas->area
    WHERE identifier = $area_identifier
) AS areas FROM game WHERE identifier = $game_identifier;
"#,
        )
        .bind(("game_identifier", game_identifier.to_string()))
        .bind(("area_identifier", area_identifier.to_string()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch area: {}", e)))?;

    let areas: Vec<Vec<AreaDetails>> = response
        .take("areas")
        .map_err(|e| AppError::InternalServerError(format!("Failed to take area: {}", e)))?;
    match areas
        .into_iter()
        .next()
        .and_then(|inner| inner.into_iter().next())
    {
        Some(area) => Ok(Json(area)),
        None => Err(AppError::NotFound("Area not found".to_string())),
    }
}

/// Fetch a single item by identifier scoped to a game. The item must either
/// be sitting in one of the game's areas (`game->areas->area->items->item`)
/// or owned by one of the game's tributes
/// (`game<-playing_in<-tribute->owns->item`). Returns 404 otherwise.
pub async fn item_detail(
    Path((game_identifier, item_identifier)): Path<(Uuid, Uuid)>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Item>, AppError> {
    let mut response = db
        .query(
            r#"
LET $game = (SELECT * FROM ONLY game WHERE identifier = $game_identifier LIMIT 1);
LET $area_items = $game.->areas->area->items->item;
LET $tribute_items = $game<-playing_in<-tribute->owns->item;
LET $candidates = array::union($area_items, $tribute_items);
SELECT * FROM $candidates WHERE identifier = $item_identifier LIMIT 1;
"#,
        )
        .bind(("game_identifier", game_identifier.to_string()))
        .bind(("item_identifier", item_identifier.to_string()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch item: {}", e)))?;

    // The four LET statements occupy result indexes 0..=3; the SELECT lives at
    // index 4.
    let items: Vec<Item> = response
        .take(4)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take item: {}", e)))?;
    match items.into_iter().next() {
        Some(item) => Ok(Json(item)),
        None => Err(AppError::NotFound("Item not found".to_string())),
    }
}

/// Get tributes for a game with pagination.
pub async fn game_tributes(
    Path(identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedTributes>, AppError> {
    // Validate pagination parameters
    params
        .validate()
        .map_err(|e| AppError::ValidationError(format!("{}", e)))?;

    let game_identifier = identifier.to_string();

    // Get total count
    let total: Option<u32> = db
        .query("RETURN count(SELECT id FROM playing_in WHERE out.identifier=$game)")
        .bind(("game", game_identifier.clone()))
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok().flatten());

    let total = total.unwrap_or(0);

    let response = db
        .query("SELECT * FROM fn::get_tributes_by_game($identifier);")
        .bind(("identifier", game_identifier.clone()))
        .await;

    match response {
        Ok(mut response) => {
            let tributes: Vec<Vec<Tribute>> = response.take("tributes").map_err(|e| {
                AppError::InternalServerError(format!("Failed to take tributes: {}", e))
            })?;
            let all_tributes = tributes[0].clone();

            // Apply pagination to the results
            let paginated_tributes: Vec<Tribute> = all_tributes
                .into_iter()
                .skip(params.offset as usize)
                .take(params.limit as usize)
                .collect();

            let has_more = (params.offset + params.limit) < total;
            let pagination = PaginationMetadata {
                total,
                limit: params.limit,
                offset: params.offset,
                has_more,
            };

            Ok(Json(PaginatedTributes {
                tributes: paginated_tributes,
                pagination,
            }))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to fetch tributes: {}",
            e
        ))),
    }
}

/// Advance the game to the next step (start, run day, or finish).
pub async fn next_step(
    Path(identifier): Path<Uuid>,
    state: State<AppState>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Option<Game>>, AppError> {
    let id = identifier.to_string();
    let id_str = id.as_str();
    let record_id = RecordId::from(("game", id_str));
    let game_status = super::get_game_status(&db, id_str).await?;

    match game_status {
        GameStatus::NotStarted => {
            super::update_game_status(&db, &record_id, GameStatus::InProgress).await?;
            let mut game = super::get_full_game(identifier, &db).await?.0;
            game.status = GameStatus::InProgress;

            // Broadcast game started
            crate::websocket::broadcast_game_started(
                &state.broadcaster,
                &game.identifier,
                game.day.unwrap_or(1),
            );

            Ok(Json(Some(game)))
        }
        GameStatus::InProgress => {
            let dead_tribute_count = super::get_dead_tribute_count(&db, id_str).await?;

            if dead_tribute_count >= 24 {
                super::update_game_status(&db, &record_id, GameStatus::Finished).await?;

                // Find and broadcast winner
                let game = super::get_full_game(identifier, &db).await?.0;
                let winner = game
                    .tributes
                    .iter()
                    .find(|t| t.is_alive())
                    .map(|t| t.name.clone());
                crate::websocket::broadcast_game_finished(&state.broadcaster, &id, winner);

                Ok(Json(None))
            } else {
                let mut game = super::get_full_game(identifier, &db).await?.0;
                super::run_game_cycles(
                    &mut game,
                    &db,
                    &state.broadcaster,
                    state.commentator.clone(),
                )
                .await?;

                Ok(Json(Some(game)))
            }
        }
        GameStatus::Finished => Ok(Json(None)),
    }
}

/// Get logs for a specific game day.
pub(crate) async fn game_day_logs(
    Path((game_identifier, day)): Path<(Uuid, u32)>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Vec<GameMessage>>, AppError> {
    let game_identifier = game_identifier.to_string();
    match db
        .query(
            r#"SELECT * FROM message
        WHERE string::starts_with(subject, $identifier) AND
        game_day = $day ORDER BY phase, emit_index;"#,
        )
        .bind(("identifier", game_identifier))
        .bind(("day", day))
        .await
    {
        Ok(mut logs) => {
            let rows: Vec<GameLog> = logs.take(0).unwrap_or_else(|err| {
                eprintln!("Error taking logs: {err:?}");
                vec![]
            });
            let logs: Vec<GameMessage> = rows.into_iter().map(GameMessage::from).collect();
            Ok(Json(logs))
        }
        Err(err) => Err(AppError::NotFound(format!("Failed to get logs: {err:?}"))),
    }
}

/// Get all logs for a game.
pub(crate) async fn game_logs(
    Path(game_identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Vec<GameMessage>>, AppError> {
    let game_identifier = game_identifier.to_string();
    match db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", game_identifier))
        .await
    {
        Ok(mut logs) => {
            let rows: Vec<GameLog> = logs.take(0).unwrap_or_else(|err| {
                eprintln!("Error taking logs: {err:?}");
                vec![]
            });
            let logs: Vec<GameMessage> = rows.into_iter().map(GameMessage::from).collect();
            Ok(Json(logs))
        }
        Err(err) => Err(AppError::NotFound(format!("Failed to get logs: {err:?}"))),
    }
}

/// Get logs for a specific tribute on a specific day.
pub(crate) async fn tribute_logs(
    Path((game_identifier, day, tribute_identifier)): Path<(Uuid, u32, Uuid)>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Vec<GameMessage>>, AppError> {
    let game_identifier = game_identifier.to_string();
    let tribute_identifier = tribute_identifier.to_string();
    match db
        .query(
            r#"SELECT *
        FROM message
        WHERE string::starts_with(subject, $game_identifier)
        AND game_day = $day
        AND source.value = $tribute_identifier
        ORDER BY timestamp;"#
                .to_string(),
        )
        .bind(("game_identifier", game_identifier))
        .bind(("day", day))
        .bind(("tribute_identifier", tribute_identifier))
        .await
    {
        Ok(mut logs) => {
            let rows: Vec<GameLog> = logs.take(0).map_err(|e| {
                AppError::InternalServerError(format!("Failed to take logs: {}", e))
            })?;
            let logs: Vec<GameMessage> = rows.into_iter().map(GameMessage::from).collect();
            Ok(Json(logs))
        }
        Err(err) => Err(AppError::NotFound(format!("Failed to get logs: {err:?}"))),
    }
}

/// Get timeline summary for a game.
pub(crate) async fn timeline_summary(
    Path(game_identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<shared::messages::TimelineSummary>, AppError> {
    // `Game.current_phase` is `#[serde(skip)]` and there is no
    // `current_phase` column on the `game` table, so we derive the live
    // phase from the most recent persisted message for this game. Picking
    // the newest `(game_day, phase, tick, emit_index)` tuple matches the
    // engine's per-cycle emission order and stays correct across the
    // Day -> Night transition without needing a schema migration.
    #[derive(serde::Deserialize)]
    struct GameDay {
        day: Option<u32>,
    }

    let game_identifier_str = game_identifier.to_string();

    let mut game_resp = db
        .query("SELECT day FROM game WHERE identifier = $identifier;")
        .bind(("identifier", game_identifier_str.clone()))
        .await
        .map_err(|err| AppError::NotFound(format!("Failed to query game: {err:?}")))?;

    let game_rows: Vec<GameDay> = game_resp.take(0).unwrap_or_default();
    let game_row = game_rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("Game {game_identifier} not found")))?;

    let current_day = game_row.day.unwrap_or(0);

    let mut msg_resp = db
        .query(
            r#"SELECT * FROM message
            WHERE string::starts_with(subject, $identifier)
            ORDER BY game_day, phase, tick, emit_index;"#,
        )
        .bind(("identifier", game_identifier_str))
        .await
        .map_err(|err| AppError::NotFound(format!("Failed to query timeline messages: {err:?}")))?;

    let rows: Vec<GameLog> = msg_resp.take(0).unwrap_or_else(|err| {
        eprintln!("Error taking timeline logs: {err:?}");
        vec![]
    });
    let messages: Vec<GameMessage> = rows.into_iter().map(GameMessage::from).collect();

    // Derive current phase from the latest message in the current day.
    // Falls back to Day if no messages exist yet for this day (e.g. the
    // game has just been started and the day has not run a cycle).
    let current_phase = messages
        .iter()
        .filter(|m| m.game_day == current_day)
        .max_by_key(|m| (m.phase, m.tick, m.emit_index))
        .map(|m| m.phase)
        .unwrap_or(shared::messages::Phase::Day);

    let summaries = shared::messages::summarize_periods(&messages, (current_day, current_phase));
    Ok(Json(shared::messages::TimelineSummary {
        periods: summaries,
    }))
}

/// Publish a game (set private = false).
pub(crate) async fn publish_game(
    Path(game_identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<serde_json::Value>, AppError> {
    let game_identifier = game_identifier.to_string();
    let response = db
        .query("UPDATE game SET private = false WHERE identifier = $identifier")
        .bind(("identifier", game_identifier))
        .await;

    match response {
        Ok(_) => Ok(Json(serde_json::json!({ "published": true }))),
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to publish game: {e:?}"
        ))),
    }
}

/// Unpublish a game (set private = true).
pub(crate) async fn unpublish_game(
    Path(game_identifier): Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<serde_json::Value>, AppError> {
    let game_identifier = game_identifier.to_string();
    let response = db
        .query("UPDATE game SET private = true WHERE identifier = $identifier")
        .bind(("identifier", game_identifier))
        .await;

    match response {
        Ok(_) => Ok(Json(serde_json::json!({ "published": false }))),
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to unpublish game: {e:?}"
        ))),
    }
}

/// Get display game info.
pub async fn game_display(
    game_identifier: Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<DisplayGame>, AppError> {
    let identifier = game_identifier.to_string();
    let mut result = db
        .query("SELECT * FROM fn::get_display_game($identifier);")
        .bind(("identifier", identifier.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch game: {}", e)))?;

    let game: Option<DisplayGame> = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take game: {}", e)))?;

    if let Some(game) = game {
        Ok(Json(game))
    } else {
        Err(AppError::NotFound(format!(
            "No game found with identifier: {identifier}"
        )))
    }
}
