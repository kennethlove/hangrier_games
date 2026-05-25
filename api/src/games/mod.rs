pub mod handlers;
pub use handlers::*;

use crate::tributes::TRIBUTES_ROUTER;
use crate::{AppError, AppState};
use axum::Json;
use axum::Router;
use axum::routing::{get, put};
use chrono::{DateTime, Utc};
use game::areas::{Area, AreaDetails};
use game::games::Game;
use game::items::Item;
use game::messages::{GameMessage, MessageSource};
use game::terrain::BaseTerrain;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::{GameArea, GameStatus, PaginationMetadata};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::{RecordId, Surreal};
use uuid::Uuid;
use validator::Validate;

/// Maximum number of messages to retain per game to prevent OOM
const MAX_MESSAGES: usize = 10000;

// Local type for paginated tributes - can't use shared since it would require game crate dependency
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginatedTributes {
    pub tributes: Vec<Tribute>,
    pub pagination: PaginationMetadata,
}

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
        .route("/{game_identifier}/log", get(game_logs))
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

/// Create areas for a game, including spawning items within them.
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

async fn add_item_to_area(
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
    game.run_full_day()
        .map_err(|e| AppError::InternalServerError(format!("Failed to run game day: {}", e)))?;
    let _ = save_game(game, db, broadcaster).await?;
    // Persistence has happened; the cycle's `CycleStart`/`CycleEnd`
    // GameMessages were broadcast by `save_game` as it drained
    // `game.messages`, so the frontend's `GamePage` already invalidates
    // the per-period timeline summary on `MessagePayload::CycleStart`/
    // `CycleEnd`/`GameEnded` without any extra synthesised broadcasts here.
    Ok(())
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
pub struct GameLog {
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
