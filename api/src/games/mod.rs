pub mod handlers;
pub use handlers::*;
pub(crate) mod items;
pub(crate) mod persist;

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
use shared::messages::MessagePayload;
use game::terrain::BaseTerrain;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::{GameArea, GameStatus, PaginationMetadata};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, LazyLock};
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
    commentator: Option<Arc<dyn announcers::Commentator>>,
) -> Result<(), AppError> {
    game.run_full_day()
        .map_err(|e| AppError::InternalServerError(format!("Failed to run game day: {}", e)))?;

    // Clone messages before save_game drains them for commentary.
    let phase_events: Vec<GameMessage> = game.messages.clone();

    let _ = persist::save_game(game, db, broadcaster).await?;

    // Spawn commentary generation as a non-blocking background task.
    if let (Some(commentator), Some(day)) = (commentator, game.day)
        && !phase_events.is_empty()
    {
            let game_id = game.identifier.clone();
            let db = db.clone(); // Surreal<Any> is Clone
            let broadcaster = broadcaster.clone();
            let phase_label = format!("{}", game.current_phase);

            // Build kill leaders from phase events.
            let mut kill_counts: HashMap<String, u32> = HashMap::new();
            for msg in &phase_events {
                if let MessagePayload::TributeKilled { killer: Some(k), .. } = &msg.payload {
                    *kill_counts.entry(k.name.clone()).or_insert(0) += 1;
                }
            }
            let mut kill_leaders: Vec<announcers::KillLeader> = kill_counts
                .into_iter()
                .map(|(name, count)| {
                    let district = game
                        .tributes
                        .iter()
                        .find(|t| t.name == name)
                        .map(|t| t.district as u8)
                        .unwrap_or(0);
                    announcers::KillLeader {
                        name,
                        district,
                        kill_count: count,
                    }
                })
                .collect();
            kill_leaders.sort_by_key(|k| std::cmp::Reverse(k.kill_count));

            // Try to load persisted tribute histories from SurrealDB, or build fresh.
            let mut history_tracker: announcers::TributeHistories = {
                let persisted: Option<serde_json::Value> = db
                    .query("SELECT * FROM tribute_histories WHERE game_id = $game_id LIMIT 1")
                    .bind(("game_id", game_id.clone()))
                    .await
                    .ok()
                    .and_then(|mut r| r.take::<Option<serde_json::Value>>(0).ok().flatten());

                if let Some(val) = persisted {
                    // Try to deserialize persisted digests.
                    if let Ok(digests) =
                        serde_json::from_value::<Vec<announcers::TributeDigest>>(
                            val.get("digests").cloned().unwrap_or(serde_json::Value::Null),
                        )
                    {
                        announcers::TributeHistories::new(digests)
                    } else {
                        // Fall back to fresh build from roster.
                        let digests: Vec<announcers::TributeDigest> = game
                            .tributes
                            .iter()
                            .map(build_tribute_digest)
                            .collect();
                        announcers::TributeHistories::new(digests)
                    }
                } else {
                    let digests: Vec<announcers::TributeDigest> = game
                        .tributes
                        .iter()
                        .map(build_tribute_digest)
                        .collect();
                    announcers::TributeHistories::new(digests)
                }
            };
            history_tracker.update(&phase_events);
            let digests = history_tracker.digests();

            // Build state snapshot from the game's current state.
            let alive_count = game.tributes.iter().filter(|t| t.is_alive()).count() as u32;

            // Build killing sprees from active streaks in the updated digests.
            let killing_sprees: Vec<announcers::KillingSpree> = digests
                .iter()
                .filter(|d| d.kill_streak >= 2)
                .map(|d| announcers::KillingSpree {
                    name: d.name.clone(),
                    district: d.district,
                    streak: d.kill_streak,
                    label: announcers::spree_label(d.kill_streak).to_string(),
                })
                .collect();

            // Build hot zones — areas with the most activity this phase.
            let mut area_counts: HashMap<String, u32> = HashMap::new();
            for msg in &phase_events {
                use shared::messages::MessagePayload::*;
                let area_name = match &msg.payload {
                    TributeMoved { to, .. } => Some(&to.name),
                    TributeHidden { area, .. } => Some(&area.name),
                    AreaEvent { area, .. } => Some(&area.name),
                    AreaClosed { area } => Some(&area.name),
                    ItemFound { area, .. } => Some(&area.name),
                    _ => None,
                };
                if let Some(name) = area_name {
                    *area_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
            let mut sorted: Vec<(String, u32, &str)> = area_counts
                .into_iter()
                .map(|(name, count)| {
                    let level = announcers::severity::describe_area_activity(count);
                    (name, count, level)
                })
                .collect();
            // Sort by count descending so the hottest areas come first.
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            let hot_zones: Vec<announcers::AreaActivity> = sorted
                .into_iter()
                .map(|(name, _count, level)| announcers::AreaActivity {
                    name,
                    activity_level: level.to_string(),
                })
                .collect();

            // Serialize digests for persistence (clone before moving into spawn).
            let digests_json = serde_json::to_value(&digests).ok();

            let header = announcers::GameStateSnapshot {
                day,
                phase: phase_label.clone(),
                alive_count,
                kill_leaders,
                alliances: vec![],
                hot_zones,
                killing_sprees,
            };

            tokio::spawn(async move {
                match announcers::generate_commentary(
                    &*commentator,
                    &game_id,
                    day,
                    &phase_label,
                    header,
                    &phase_events,
                    digests,
                )
                .await
                {
                    Ok(segment) => {
                        if segment.lines.is_empty() {
                            tracing::warn!(
                                game_id = %game_id,
                                "Commentary generated 0 lines — skipping save and broadcast"
                            );
                        } else {
                            tracing::debug!(
                                game_id = %game_id,
                                lines = %segment.lines.len(),
                                "Commentary generated"
                            );

                            // Persist commentary segment to SurrealDB (with retry).
                            let body = match serde_json::to_value(&segment) {
                                Ok(v) => v,
                                Err(e) => {
                                    tracing::error!(error = %e, "Failed to serialize commentary");
                                    return;
                                }
                            };
                            let body_clone = body.clone();
                            let dbc = db.clone();
                            if let Err(e) = retry(move || {
                                let b = body_clone.clone();
                                let db2 = dbc.clone();
                                async move {
                                    db2.query("CREATE commentary_segments CONTENT $b")
                                        .bind(("b", b))
                                        .await
                                        .map_err(|e| e.to_string())
                                }
                            })
                            .await
                            {
                                tracing::error!(error = %e, "Failed to save commentary segment after retries");
                            }

                            // Broadcast via WebSocket/SSE (only when content exists).
                            crate::websocket::broadcast_commentary(
                                &broadcaster,
                                &game_id,
                                &segment,
                            );
                        }

                        // Always persist updated tribute histories so digests
                        // accumulate across cycles, even when the LLM returns
                        // an empty response.
                        if let Some(ref digests_json) = digests_json
                            && let Err(e) = {
                                let dj = digests_json.clone();
                                let gid = game_id.clone();
                                let dbc = db.clone();
                                retry(move || {
                                    let d = dj.clone();
                                    let g = gid.clone();
                                    let db2 = dbc.clone();
                                    async move {
                                        db2.query(
                                            "UPSERT tribute_histories CONTENT { game_id: $g, digests: $d }",
                                        )
                                        .bind(("g", g))
                                        .bind(("d", d))
                                        .await
                                        .map_err(|e| e.to_string())
                                    }
                                })
                                .await
                            }
                        {
                            tracing::error!(error = %e, "Failed to save tribute histories after retries");
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Commentary generation skipped");
                    }
                }
            });
    }

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

/// Build a fresh [`announcers::TributeDigest`] from a game Tribute.
fn build_tribute_digest(t: &game::tributes::Tribute) -> announcers::TributeDigest {
    announcers::TributeDigest {
        identifier: t.identifier.clone(),
        name: t.name.clone(),
        district: t.district as u8,
        status: if t.is_alive() {
            "alive".into()
        } else {
            "deceased".into()
        },
        injury_level: "unknown".into(),
        location: t.area.to_string(),
        allies: vec![],
        kill_streak: 0,
        highlights: vec![],
        notable_events: vec![],
    }
}

/// Retry a fallible async operation up to `MAX_RETRIES` times with
/// exponential backoff (100ms, 200ms, 400ms). Returns `Ok(value)` on the
/// first success, or the last error after all retries are exhausted.
const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 100;

async fn retry<F, Fut, T>(op: F) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for attempt in 0..=MAX_RETRIES {
        match op().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt < MAX_RETRIES {
                    let delay = BASE_DELAY_MS * 2u64.pow(attempt);
                    tracing::warn!(
                        attempt = attempt + 1,
                        max = MAX_RETRIES + 1,
                        delay_ms = delay,
                        error = %last_err,
                        "Retrying DB operation"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }
    Err(last_err)
}
