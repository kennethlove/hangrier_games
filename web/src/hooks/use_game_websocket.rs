use crate::env::APP_API_HOST;
use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use shared::{GameEvent, WebSocketMessage};

/// WebSocket connection state
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// Maximum number of events retained in the in-memory ring buffer.
/// Prevents unbounded growth on long-running games.
const MAX_EVENTS: usize = 200;

/// Hook to manage WebSocket connection for real-time game updates.
///
/// Connects to `{API_HOST}/ws`, sends a `Subscribe { game_id }` frame,
/// then streams `GameEvent`s into the returned signal. The connection state
/// signal reflects the current lifecycle phase. The hook does not retry on
/// disconnect; callers can observe `ConnectionState::Disconnected` and
/// re-mount if needed.
pub fn use_game_websocket(game_id: String) -> (Signal<Vec<GameEvent>>, Signal<ConnectionState>) {
    let events = use_signal(Vec::<GameEvent>::new);
    let connection_state = use_signal(|| ConnectionState::Connecting);

    use_effect(move || {
        let game_id = game_id.clone();
        let mut events = events;
        let mut connection_state = connection_state;

        spawn(async move {
            // Convert http(s):// → ws(s):// for the WebSocket endpoint.
            let ws_url = build_ws_url(APP_API_HOST, &game_id);

            let ws = match WebSocket::open(&ws_url) {
                Ok(ws) => ws,
                Err(e) => {
                    tracing::error!("Failed to open WebSocket {}: {}", ws_url, e);
                    connection_state.set(ConnectionState::Error(e.to_string()));
                    return;
                }
            };

            // Split so we can hold the writer briefly to send the subscribe
            // frame and then poll the reader for the lifetime of the effect.
            let (mut writer, mut reader) = ws.split();

            // Send subscription frame.
            let subscribe = WebSocketMessage::Subscribe {
                game_id: game_id.clone(),
            };
            match serde_json::to_string(&subscribe) {
                Ok(payload) => {
                    if let Err(e) = writer.send(Message::Text(payload)).await {
                        tracing::error!("WebSocket subscribe send failed: {}", e);
                        connection_state.set(ConnectionState::Error(e.to_string()));
                        return;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize Subscribe: {}", e);
                    connection_state.set(ConnectionState::Error(e.to_string()));
                    return;
                }
            }

            connection_state.set(ConnectionState::Connected);

            // Read loop.
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(WebSocketMessage::GameEvent {
                                game_id: gid,
                                event,
                            }) if gid == game_id => {
                                events.with_mut(|list| {
                                    list.push(event);
                                    if list.len() > MAX_EVENTS {
                                        let drop_count = list.len() - MAX_EVENTS;
                                        list.drain(0..drop_count);
                                    }
                                });
                            }
                            Ok(WebSocketMessage::Error { message }) => {
                                tracing::warn!("WebSocket server error: {}", message);
                            }
                            Ok(_) => { /* ignore Subscribe/Unsubscribe echoes and other game IDs */
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to decode WebSocket message: {} ({})",
                                    text,
                                    e
                                );
                            }
                        }
                    }
                    Ok(Message::Bytes(_)) => {
                        // Server only emits text frames; ignore binary.
                    }
                    Err(e) => {
                        tracing::error!("WebSocket read error: {}", e);
                        connection_state.set(ConnectionState::Error(e.to_string()));
                        return;
                    }
                }
            }

            connection_state.set(ConnectionState::Disconnected);
        });
    });

    (events, connection_state)
}

/// Convert an `http(s)://host[/path]` API host into a `ws(s)://host/ws` URL.
fn build_ws_url(api_host: &str, _game_id: &str) -> String {
    let base = api_host.trim_end_matches('/');
    let ws_base = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else {
        // Assume the caller already supplied a ws scheme.
        base.to_string()
    };
    format!("{}/ws", ws_base)
}
