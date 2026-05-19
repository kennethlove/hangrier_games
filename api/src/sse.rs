use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use shared::WebSocketMessage;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::debug;

use crate::AppState;

/// SSE event handler: streams game events filtered by `game_id`.
pub async fn sse_handler(
    Path(game_id): Path<String>,
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = game_event_stream(app_state.broadcaster, game_id);
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// Subscribe to the broadcast channel, filter by `game_id`, and convert
/// each matching `GameEvent` into an SSE [`Event`].
fn game_event_stream(
    broadcaster: Arc<crate::websocket::GameBroadcaster>,
    game_id: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let rx = broadcaster.subscribe();
    BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(WebSocketMessage::GameEvent {
            game_id: msg_game_id,
            message,
        }) if msg_game_id == game_id => Some(Ok(message_to_sse_event(&message))),
        Ok(_) => None,
        Err(e) => {
            debug!("SSE broadcast lag: {e}");
            None
        }
    })
}

/// Convert a [`shared::messages::GameMessage`] into an SSE [`Event`].
///
/// - `event:` field = the `MessagePayload` variant name (e.g. "TributeKilled")
/// - `data:` field = JSON-serialized `GameMessage`
/// - `id:` field = the message's `emit_index` for reconnection support
fn message_to_sse_event(message: &shared::messages::GameMessage) -> Event {
    let event_name = payload_event_name(&message.payload);
    let data = serde_json::to_string(&message).unwrap_or_default();
    let id = message.emit_index.to_string();

    Event::default().event(&event_name).data(&data).id(&id)
}

/// Extract the serde tag name from a `MessagePayload` variant.
///
/// `MessagePayload` uses `#[serde(tag = "type")]`, so we serialize to a
/// `serde_json::Value` and read the `"type"` key. This avoids a massive
/// match arm list and stays correct when new variants are added.
fn payload_event_name(payload: &shared::messages::MessagePayload) -> String {
    serde_json::to_value(payload)
        .ok()
        .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(String::from))
        .unwrap_or_else(|| "Unknown".to_string())
}
