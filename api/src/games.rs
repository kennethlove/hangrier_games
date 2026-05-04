use crate::tributes::{TRIBUTES_ROUTER, create_tribute};
use crate::{AppError, AppState, AuthDb};
use axum::Json;
use axum::Router;
use axum::extract::{Extension, Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header::LOCATION};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use chrono::{DateTime, Utc};
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::items::Item;
use game::messages::{GameMessage, MessageSource};
use game::terrain::BaseTerrain;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::{
    CreateGame, DisplayGame, EditGame, GameArea, GameStatus, ListDisplayGame, PaginatedGames,
    PaginationMetadata,
};
use validator::Validate;

// Local type for paginated tributes - can't use shared since it would require game crate dependency
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginatedTributes {
    pub tributes: Vec<Tribute>,
    pub pagination: PaginationMetadata,
}
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::{RecordId, Surreal};
use uuid::Uuid;

/// Maximum number of messages to retain per game to prevent OOM
const MAX_MESSAGES: usize = 10000;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    limit: u32,
    #[serde(default = "default_offset")]
    #[validate(range(max = 10000))]
    offset: u32,
}

fn default_limit() -> u32 {
    10
}

fn default_offset() -> u32 {
    0
}

pub static GAMES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(create_game))
        .route(
            "/{game_identifier}",
            get(game_detail).delete(game_delete).put(game_update),
        )
        .route("/{game_identifier}/areas", get(game_areas))
        .route(
            "/{game_identifier}/areas/{area_identifier}",
            get(area_detail),
        )
        .route(
            "/{game_identifier}/items/{item_identifier}",
            get(item_detail),
        )
        .route("/{game_identifier}/display", get(game_display))
        .route("/{game_identifier}/log/{day}", get(game_day_logs))
        .route(
            "/{game_identifier}/log/{day}/{tribute_identifier}",
            get(tribute_logs),
        )
        .route("/{game_identifier}/next", put(next_step))
        .route("/{game_identifier}/timeline-summary", get(timeline_summary))
        .route("/{game_identifier}/publish", put(publish_game))
        .route("/{game_identifier}/unpublish", put(unpublish_game))
        .nest("/{game_identifier}/tributes", TRIBUTES_ROUTER.clone())
});

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameAreaEdge {
    #[serde(rename = "in")]
    game: RecordId,
    #[serde(rename = "out")]
    area: RecordId,
}

async fn create_game_area(area: Area, db: &Surreal<Any>) -> Result<GameArea, AppError> {
    let identifier = Uuid::new_v4().to_string();
    let area_id: RecordId = RecordId::from(("area", identifier.to_string()));

    // create the `area` record. Bind via serde_json::Value + a raw
    // UPDATE...CONTENT query so the SDK's bespoke serializer can't drop
    // optional fields or collapse externally-tagged enums (see save_game).
    let game_area = GameArea {
        identifier: identifier.to_string(),
        name: area.to_string(),
        area: area.to_string(),
    };
    let body = serde_json::to_value(&game_area)
        .map_err(|e| AppError::InternalServerError(format!("Failed to encode area: {}", e)))?;
    db.query("UPSERT $rid CONTENT $body")
        .bind(("rid", area_id.clone()))
        .bind(("body", body))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create area: {}", e)))?;
    crate::verify_record_persisted(db, &area_id, "create_game_area").await?;

    Ok(game_area)
}

async fn create_game_area_edge(
    area: Area,
    game_identifier: Uuid,
    db: &Surreal<Any>,
) -> Result<GameAreaEdge, AppError> {
    let game_identifier_str = game_identifier.to_string();
    let game_id = RecordId::from(("game", &game_identifier_str));

    // Does the `area` exist for the game?
    let mut resp = db
        .query(
            r#"
        SELECT identifier
        FROM area
        WHERE original_name = '$name'
        AND <-areas<-game.identifier = '$game_id'"#,
        )
        .bind(("name", area))
        .bind(("game_id", game_identifier_str.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to find area: {}", e)))?;
    let existing_area: Option<Area> = resp
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to find area: {}", e)))?;

    let area_uuid = if let Some(identifier) = existing_area {
        Uuid::from_str(&identifier.to_string())
            .map_err(|e| AppError::BadRequest(format!("Invalid area UUID: {}", e)))?
    } else {
        match create_game_area(area, db).await {
            Ok(game_area) => Uuid::from_str(game_area.identifier.as_str())
                .map_err(|e| AppError::BadRequest(format!("Invalid game area UUID: {}", e)))?,
            Err(_) => {
                return Err(AppError::InternalServerError(
                    "Failed to create game area".into(),
                ));
            }
        }
    };

    let edge = GameAreaEdge {
        game: game_id.clone(),
        area: RecordId::from(("area", &area_uuid.to_string())),
    };

    // RELATE always returns an array; the SDK's typed Option<T> path fails to
    // deserialize that into a single struct. Use a raw query so the response
    // shape doesn't matter. The (game, area) unique index keeps duplicates out.
    db.query("RELATE $game->areas->$area")
        .bind(("game", edge.game.clone()))
        .bind(("area", edge.area.clone()))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to link game and area: {}", e))
        })?;
    Ok(edge)
}

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
    let tribute_futures = (0..24).map(|idx| create_tribute(None, &game_identifier, &db, idx % 12));
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
    let area_futures =
        Area::iter().map(|area| create_area(game_identifier.as_str(), area, base_item_count, &db));
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

    let location = HeaderValue::from_str(&format!("/api/games/{}", game_identifier))
        .map_err(|e| AppError::InternalServerError(format!("Invalid Location header: {}", e)))?;
    let mut response = (StatusCode::CREATED, Json(created_game)).into_response();
    response.headers_mut().insert(LOCATION, location);
    Ok(response)
}

pub async fn create_area(
    game_identifier: &str,
    area: Area,
    num_items: u32,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    let game_uuid = Uuid::from_str(game_identifier)
        .map_err(|e| AppError::BadRequest(format!("Invalid game UUID: {}", e)))?;
    let game_area = create_game_area_edge(area, game_uuid, db)
        .await
        .map_err(|e| {
            let detail = match &e {
                AppError::InternalServerError(s)
                | AppError::BadRequest(s)
                | AppError::DbError(s) => s.clone(),
                other => other.to_string(),
            };
            AppError::InternalServerError(format!("Failed to create game area: {}", detail))
        })?;
    let item_futures = (0..num_items).map(|_| add_item_to_area(&game_area, None, db));
    let item_results = futures::future::join_all(item_futures).await;
    if let Some(err) = item_results.into_iter().find_map(Result::err) {
        let detail = match &err {
            AppError::InternalServerError(s) | AppError::BadRequest(s) | AppError::DbError(s) => {
                s.clone()
            }
            other => other.to_string(),
        };
        return Err(AppError::InternalServerError(format!(
            "Failed to create items: {}",
            detail
        )));
    }

    Ok(())
}

pub async fn add_item_to_area(
    game_area_edge: &GameAreaEdge,
    terrain: Option<BaseTerrain>,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    // Insert an item using terrain-based weights if terrain is provided
    let new_item: Item = match terrain {
        Some(t) => Item::new_random_with_terrain(t, None),
        None => Item::new_random(None),
    };
    let new_item_id: RecordId = RecordId::from(("item", &new_item.identifier));
    let body = serde_json::to_value(&new_item)
        .map_err(|e| AppError::InternalServerError(format!("Failed to encode item: {}", e)))?;
    if let Err(e) = db
        .query("UPSERT $rid CONTENT $body")
        .bind(("rid", new_item_id.clone()))
        .bind(("body", body))
        .await
    {
        return Err(AppError::InternalServerError(format!(
            "Failed to create item: {}",
            e
        )));
    }
    crate::verify_record_persisted(db, &new_item_id, "add_item_to_area").await?;

    // Insert an area-item relationship via a raw RELATE query. RELATE always
    // returns an array, so the SDK's typed insert::<Vec<_>>().relation() path
    // is fragile; the raw query with bound params sidesteps that.
    if let Err(e) = db
        .query("RELATE $area->items->$item")
        .bind(("area", game_area_edge.area.clone()))
        .bind(("item", new_item_id.clone()))
        .await
    {
        Err(AppError::InternalServerError(format!(
            "Failed to create area-item relationship: {}",
            e
        )))
    } else {
        Ok(())
    }
}

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
        delete_pieces(pieces, &db).await?
    };
    if let Some(pieces) = area_pieces {
        delete_pieces(pieces, &db).await?
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

async fn delete_pieces(
    pieces: HashMap<String, Vec<Thing>>,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    for (table, ids) in pieces {
        let db = db
            .query("DELETE $table WHERE id IN [$ids]".to_string())
            .bind(("table", table.clone()))
            .bind((
                "ids",
                ids.iter()
                    .map(|i| format!(r#"{table}:{}"#, i.id))
                    .collect::<Vec<String>>()
                    .join(","),
            ))
            .await;
        if db.is_err() {
            return Err(AppError::InternalServerError(format!(
                "Failed to delete {} pieces.",
                table
            )));
        }
    }
    Ok(())
}

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

pub async fn game_detail(
    game_identifier: Path<Uuid>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<DisplayGame>, AppError> {
    // let identifier = game_identifier.0;
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

async fn get_game_status(db: &Surreal<Any>, identifier: &str) -> Result<GameStatus, AppError> {
    let result = db
        .query("SELECT status FROM game WHERE identifier = $identifier")
        .bind(("identifier", identifier.to_string()))
        .await;
    match result {
        Ok(mut result) => match result.take::<Option<String>>("status") {
            Ok(Some(game_status)) => match GameStatus::from_str(game_status.as_str()) {
                Ok(status) => Ok(status),
                Err(_) => Err(AppError::InternalServerError("Invalid status".into())),
            },
            Err(e) => Err(AppError::NotFound(format!(
                "Failed to find game status: {}",
                e
            ))),
            _ => Err(AppError::NotFound("Failed to find game status".into())),
        },
        _ => Err(AppError::NotFound("Failed to find game".into())),
    }
}

async fn update_game_status(
    db: &Surreal<Any>,
    record_id: &RecordId,
    status: GameStatus,
) -> Result<(), AppError> {
    match db
        .query("UPDATE $record_id SET status = $status")
        .bind(("record_id", record_id.clone()))
        .bind(("status", status.to_string()))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to update game status: {}",
            e
        ))),
    }
}

async fn get_dead_tribute_count(db: &Surreal<Any>, identifier: &str) -> Result<u32, AppError> {
    let result = db
        .query(
            r#"
        RETURN count(
            SELECT in.identifier
            FROM playing_in
            WHERE out.identifier = $identifier
            AND in.status IN ["RecentlyDead", "Dead"]
        );"#,
        )
        .bind(("identifier", identifier.to_string()))
        .await;

    match result {
        Ok(mut result) => match result.take::<Option<u32>>(0) {
            Ok(Some(dead_tributes)) => Ok(dead_tributes),
            _ => Err(AppError::NotFound("Failed to find game".into())),
        },
        _ => Err(AppError::NotFound("Failed to find game".into())),
    }
}

async fn run_game_cycles(
    game: &mut Game,
    db: &Surreal<Any>,
    broadcaster: &crate::websocket::GameBroadcaster,
) -> Result<(), AppError> {
    game.run_day_night_cycle(true)
        .map_err(|e| AppError::InternalServerError(format!("Failed to run day cycle: {}", e)))?;
    game.run_day_night_cycle(false)
        .map_err(|e| AppError::InternalServerError(format!("Failed to run night cycle: {}", e)))?;
    let _ = save_game(game, db, broadcaster).await?;
    // Persistence has happened; the cycle's `CycleStart`/`CycleEnd`
    // GameMessages were broadcast by `save_game` as it drained
    // `game.messages`, so the frontend's `GamePage` already invalidates
    // the per-period timeline summary on `MessagePayload::CycleStart`/
    // `CycleEnd`/`GameEnded` without any extra synthesised broadcasts here.
    Ok(())
}

pub async fn next_step(
    Path(identifier): Path<Uuid>,
    state: State<AppState>,
    Extension(AuthDb(db)): Extension<AuthDb>,
) -> Result<Json<Option<Game>>, AppError> {
    let id = identifier.to_string();
    let id_str = id.as_str();
    let record_id = RecordId::from(("game", id_str));
    let game_status = get_game_status(&db, id_str).await?;

    match game_status {
        GameStatus::NotStarted => {
            update_game_status(&db, &record_id, GameStatus::InProgress).await?;
            let mut game = get_full_game(identifier, &db).await?.0;
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
            let dead_tribute_count = get_dead_tribute_count(&db, id_str).await?;

            if dead_tribute_count >= 24 {
                update_game_status(&db, &record_id, GameStatus::Finished).await?;

                // Find and broadcast winner
                let game = get_full_game(identifier, &db).await?.0;
                let winner = game
                    .tributes
                    .iter()
                    .find(|t| t.is_alive())
                    .map(|t| t.name.clone());
                crate::websocket::broadcast_game_finished(&state.broadcaster, &id, winner);

                Ok(Json(None))
            } else {
                let mut game = get_full_game(identifier, &db).await?.0;
                run_game_cycles(&mut game, &db, &state.broadcaster).await?;

                Ok(Json(Some(game)))
            }
        }
        GameStatus::Finished => Ok(Json(None)),
    }
}

async fn get_full_game(identifier: Uuid, db: &Surreal<Any>) -> Result<Json<Game>, AppError> {
    let identifier = identifier.to_string();
    let mut result = db
        .query("SELECT * FROM fn::get_full_game($identifier)")
        .bind(("identifier", identifier.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch game: {}", e)))?;
    if let Some(game) = result
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take game: {}", e)))?
    {
        Ok(Json::<Game>(game))
    } else {
        Err(AppError::NotFound("Failed to find game".into()))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct GameLog {
    pub id: RecordId,
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
    /// Period phase (Day or Night) — see `shared::messages::Phase`.
    pub phase: shared::messages::Phase,
    /// Per-period monotonic action counter; resets each phase boundary.
    pub tick: u32,
    /// Within-period emit ordinal; resets each phase boundary.
    pub emit_index: u32,
    /// Structured `MessagePayload` stored as a JSON-encoded `String`.
    /// SurrealDB's bespoke serializer collapses internally-tagged Rust
    /// enums and arbitrary `serde_json::Value::Object` payloads to `{}`
    /// when bound into an `object` column, so we transit the wire as a
    /// plain string and decode on the read side.
    pub payload: String,
}

impl From<GameLog> for GameMessage {
    fn from(row: GameLog) -> Self {
        let payload = serde_json::from_str(&row.payload).unwrap_or_else(|err| {
            eprintln!(
                "Warning: failed to decode message payload for {}: {err:?}",
                row.identifier
            );
            shared::messages::MessagePayload::AreaEvent {
                area: shared::messages::AreaRef {
                    identifier: String::new(),
                    name: String::new(),
                },
                kind: shared::messages::AreaEventKind::Other,
                description: row.content.clone(),
            }
        });
        GameMessage {
            identifier: row.identifier,
            source: row.source,
            game_day: row.game_day,
            phase: row.phase,
            tick: row.tick,
            emit_index: row.emit_index,
            subject: row.subject,
            timestamp: row.timestamp,
            content: row.content,
            payload,
        }
    }
}

async fn save_game(
    game: &mut Game,
    db: &Surreal<Any>,
    broadcaster: &crate::websocket::GameBroadcaster,
) -> Result<Json<Game>, AppError> {
    let game_identifier = RecordId::from(("game", game.identifier.clone()));

    // Start transaction
    db.query("BEGIN TRANSACTION").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to start transaction: {}", e))
    })?;

    // Drain events accumulated during the most recent run_day_night_cycle.
    // Persist them to the message table and broadcast to subscribed WS clients.
    let logs: Vec<GameMessage> = std::mem::take(&mut game.messages);
    if !logs.is_empty() {
        let game_day = game.day.unwrap_or_default();

        // Broadcast first so clients see updates even if persistence is slow.
        for log in &logs {
            crate::websocket::broadcast_game_message(broadcaster, &game.identifier, log.clone());
        }

        let game_logs: Vec<GameLog> = logs
            .into_iter()
            .map(|log| GameLog {
                id: RecordId::from(("message", &log.identifier)),
                identifier: log.identifier,
                source: log.source,
                game_day,
                subject: log.subject,
                timestamp: log.timestamp,
                content: log.content,
                phase: log.phase,
                tick: log.tick,
                emit_index: log.emit_index,
                payload: serde_json::to_string(&log.payload).unwrap_or_else(|_| "null".to_string()),
            })
            .collect();

        // Check current message count for this game
        let current_count: u32 = db
            .query(
                "RETURN count(SELECT id FROM message WHERE string::starts_with(subject, $game_id))",
            )
            .bind(("game_id", game.identifier.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok().flatten())
            .unwrap_or(0);

        let new_messages_count = game_logs.len();
        let total_after_insert = current_count as usize + new_messages_count;

        // Log warning if approaching limit
        if current_count >= 9000 && current_count < MAX_MESSAGES as u32 {
            tracing::warn!(
                game_id = %game.identifier,
                current_count = %current_count,
                "Game message count approaching limit (9000+)"
            );
        }

        // Rotate messages if total would exceed MAX_MESSAGES
        if total_after_insert > MAX_MESSAGES {
            let messages_to_delete = total_after_insert - MAX_MESSAGES;
            tracing::info!(
                game_id = %game.identifier,
                current_count = %current_count,
                new_count = %new_messages_count,
                deleting = %messages_to_delete,
                "Rotating old messages to maintain {} message limit", MAX_MESSAGES
            );

            if let Err(e) = db
                .query(
                    r#"
                    DELETE message
                    WHERE string::starts_with(subject, $game_id)
                    ORDER BY timestamp ASC
                    LIMIT $delete_count
                    "#,
                )
                .bind(("game_id", game.identifier.clone()))
                .bind(("delete_count", messages_to_delete))
                .await
            {
                tracing::error!(
                    game_id = %game.identifier,
                    error = %e,
                    "Failed to rotate old messages"
                );
                // Continue anyway - this is not critical enough to fail the save
            }
        }

        // The structured `event` payload rides as a plain JSON string
        // on the `GameLog.event` field, sidestepping the SurrealDB SDK
        // serializer's habit of collapsing externally-tagged enums and
        // `serde_json::Value::Object` payloads to `{}` when bound into
        // an `object` column. mqi.3.
        if let Err(e) = db.insert::<Vec<GameLog>>(()).content(game_logs).await {
            let _ = db.query("ROLLBACK").await;
            return Err(AppError::InternalServerError(format!(
                "Failed to save game logs: {}",
                e
            )));
        }
    }

    let area_results = futures::future::join_all(game.areas.iter().map(|area| async {
        let id = RecordId::from(("area", area.identifier.clone()));
        save_area_items(&area.items, id.clone(), db).await?;

        let mut area_without_items = area.clone();
        area_without_items.items = vec![];
        // Bind via serde_json::Value to bypass the SurrealDB SDK's bespoke
        // type serializer, which collapses externally-tagged enums and Option
        // fields. The generic JSON bind path round-trips cleanly.
        let body = serde_json::to_value(&area_without_items)
            .map_err(|e| AppError::InternalServerError(format!("Failed to encode area: {}", e)))?;
        db.query("UPSERT $rid CONTENT $body")
            .bind(("rid", id.clone()))
            .bind(("body", body))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to update area: {}", e)))
            .map(|_| ())
    }))
    .await;

    if let Some(err) = area_results.iter().find_map(|r| r.as_ref().err()) {
        let msg = format!("{}", err);
        let _ = db.query("ROLLBACK").await;
        return Err(AppError::InternalServerError(format!(
            "Failed to save area items: {}",
            msg
        )));
    }

    let tribute_results = futures::future::join_all(game.tributes.iter().map(|tribute| async {
        let id = RecordId::from(("tribute", tribute.identifier.clone()));
        if tribute.is_alive() {
            save_tribute_items(&tribute.items, id.clone(), db).await?;
        }

        let mut tribute_without_items = tribute.clone();
        tribute_without_items.items = vec![];
        // Same workaround as the area UPDATE above: serde_json::Value bypasses
        // the SDK's enum-collapsing serializer.
        let body = serde_json::to_value(&tribute_without_items).map_err(|e| {
            AppError::InternalServerError(format!("Failed to encode tribute: {}", e))
        })?;
        db.query("UPSERT $rid CONTENT $body")
            .bind(("rid", id.clone()))
            .bind(("body", body))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to update tribute: {}", e)))
            .map(|_| ())
    }))
    .await;

    if let Some(err) = tribute_results.iter().find_map(|r| r.as_ref().err()) {
        let msg = format!("{}", err);
        let _ = db.query("ROLLBACK").await;
        return Err(AppError::InternalServerError(format!(
            "Failed to save tribute items: {}",
            msg
        )));
    }

    // Persist mutable game fields explicitly. `db.update().content()` with the
    // SurrealDB SDK's bespoke serializer drops `Option<u32>` fields like `day`
    // (same family of bugs as the externally-tagged-enum collapse called out
    // around the message payload above), so we use a plain UPDATE query that
    // names the fields we want written.
    if let Err(e) = db
        .query("UPDATE $record_id SET day = $day, status = $status")
        .bind(("record_id", game_identifier.clone()))
        .bind(("day", game.day))
        .bind(("status", game.status.to_string()))
        .await
    {
        let _ = db.query("ROLLBACK").await;
        return Err(AppError::InternalServerError(format!(
            "Failed to update game: {}",
            e
        )));
    }

    db.query("COMMIT").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to commit transaction: {}", e))
    })?;
    Ok(Json(game.clone()))
}

async fn save_area_items(
    items: &Vec<Item>,
    owner: RecordId,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    // Get existing items
    let existing_items: Vec<Item> = db
        .query("SELECT * FROM items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch items: {}", e)))?
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take items: {}", e)))?;

    // Create lookups for efficient comparison
    let mut existing_map = HashMap::new();
    for item in existing_items {
        existing_map.insert(item.identifier.clone(), item.clone());
    }

    let mut new_map = HashMap::new();
    for item in items {
        new_map.insert(item.identifier.clone(), item.clone());
    }

    // Find items to delete (in DB but not in new items or durability is 0)
    let mut items_to_delete = Vec::new();
    for id in existing_map.keys() {
        if let Some(item) = new_map.get(id) {
            if item.current_durability == 0 {
                items_to_delete.push(id.clone());
            }
        } else {
            items_to_delete.push(id.clone());
        }
    }

    // Find items to update (in DB and in new items with different values)
    let mut items_to_update = Vec::new();
    for (id, item) in &new_map {
        if item.current_durability > 0 {
            if let Some(existing) = existing_map.get(id) {
                if existing != item {
                    items_to_update.push(item.clone());
                }
            } else {
                items_to_update.push(item.clone());
            }
        }
    }

    // Update relations - first delete existing relations
    db.query("DELETE FROM items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete items: {}", e)))?;

    // Batch delete operations
    if !items_to_delete.is_empty() {
        let delete_ids: Vec<String> = items_to_delete
            .iter()
            .map(|id| format!("item:{}", id))
            .collect();

        db.query("DELETE item WHERE id IN $ids")
            .bind(("ids", delete_ids))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to batch delete items: {}", e))
            })?;
    }

    // Batch update operations
    if !items_to_update.is_empty() {
        // Use serde_json::to_value + bound CONTENT so ALL Item fields round-trip
        // (the previous hand-rolled CONTENT block silently dropped `rarity`,
        // and string-interpolating fields is fragile around quoting).
        for item in &items_to_update {
            let rid = RecordId::from(("item", item.identifier.to_string().as_str()));
            let body = serde_json::to_value(item).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode item: {}", e))
            })?;
            db.query("UPSERT $rid CONTENT $body")
                .bind(("rid", rid))
                .bind(("body", body))
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to update item: {}", e))
                })?;
        }

        // Batch insert relations. Hyphenated UUIDs must be wrapped in
        // ⟨angle brackets⟩ or Surreal's SQL parser splits them on `-`.
        let mut relation_parts = Vec::new();
        for item in &items_to_update {
            relation_parts.push(format!(
                "RELATE {}->items->item:⟨{}⟩",
                owner, item.identifier
            ));
        }

        let bulk_relations = relation_parts.join(";\n");
        db.query(&bulk_relations).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to batch create relations: {}", e))
        })?;
    }

    Ok(())
}

async fn save_tribute_items(
    items: &Vec<Item>,
    owner: RecordId,
    db: &Surreal<Any>,
) -> Result<(), AppError> {
    // Get existing items
    let existing_items: Vec<Item> = db
        .query("SELECT * from owns->items WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch items: {}", e)))?
        .take(0)
        .map_err(|e| AppError::InternalServerError(format!("Failed to take items: {}", e)))?;

    // Create lookups for efficient comparison
    let mut existing_map = HashMap::new();
    for item in existing_items {
        existing_map.insert(item.identifier.clone(), item);
    }

    let mut new_map = HashMap::new();
    for item in items {
        new_map.insert(item.identifier.clone(), item.clone());
    }

    // Find items to delete (in DB but not in new items or durability is 0)
    let mut items_to_delete = Vec::new();
    for id in existing_map.keys() {
        if let Some(item) = new_map.get(id) {
            if item.current_durability == 0 {
                items_to_delete.push(id.clone());
            }
        } else {
            items_to_delete.push(id.clone());
        }
    }

    // Find items to update (in DB and in new items with different values)
    let mut items_to_update = Vec::new();
    for (id, item) in &new_map {
        if item.current_durability > 0 {
            if let Some(existing) = existing_map.get(id) {
                if existing != item {
                    items_to_update.push(item.clone());
                }
            } else {
                items_to_update.push(item.clone());
            }
        }
    }

    // Delete existing relations - do this once for all items
    db.query("DELETE FROM owns WHERE in = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete items: {}", e)))?;

    // Batch delete operations
    if !items_to_delete.is_empty() {
        let delete_ids: Vec<String> = items_to_delete
            .iter()
            .map(|id| format!("item:{}", id))
            .collect();

        db.query("DELETE item WHERE id IN $ids")
            .bind(("ids", delete_ids))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to batch delete items: {}", e))
            })?;
    }

    // Batch update operations
    if !items_to_update.is_empty() {
        // Use serde_json::to_value + bound CONTENT so ALL Item fields round-trip
        // (the previous hand-rolled CONTENT block silently dropped `rarity`,
        // and string-interpolating fields is fragile around quoting).
        for item in &items_to_update {
            let rid = RecordId::from(("item", item.identifier.to_string().as_str()));
            let body = serde_json::to_value(item).map_err(|e| {
                AppError::InternalServerError(format!("Failed to encode item: {}", e))
            })?;
            db.query("UPSERT $rid CONTENT $body")
                .bind(("rid", rid))
                .bind(("body", body))
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to update item: {}", e))
                })?;
        }

        // Batch insert relations. Hyphenated UUIDs must be wrapped in
        // ⟨angle brackets⟩ or Surreal's SQL parser splits them on `-`.
        let mut relation_parts = Vec::new();
        for item in &items_to_update {
            relation_parts.push(format!(
                "RELATE {}->owns->item:⟨{}⟩",
                owner, item.identifier
            ));
        }

        let bulk_relations = relation_parts.join(";\n");
        db.query(&bulk_relations).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to batch create relations: {}", e))
        })?;
    }

    Ok(())
}

async fn game_day_logs(
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

async fn tribute_logs(
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

async fn timeline_summary(
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

async fn publish_game(
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

async fn unpublish_game(
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
