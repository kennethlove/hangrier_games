use axum::Json;
use game::games::Game;
use game::messages::GameMessage;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use super::items::{save_area_items, save_tribute_items};
use super::{GameLog, MAX_MESSAGES};
use crate::AppError;
use crate::websocket::{GameBroadcaster, broadcast_game_message};

pub(crate) async fn save_game(
    game: &mut Game,
    db: &Surreal<Any>,
    broadcaster: &GameBroadcaster,
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
            broadcast_game_message(broadcaster, &game.identifier, log.clone());
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
