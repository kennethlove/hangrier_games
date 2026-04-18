use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use shared::{GameEvent, WebSocketMessage};
use std::rc::Rc;

/// WebSocket connection state
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// Hook to manage WebSocket connection for real-time game updates
pub fn use_game_websocket(game_id: String) -> (Signal<Vec<GameEvent>>, Signal<ConnectionState>) {
    let mut events = use_signal(|| vec![]);
    let mut connection_state = use_signal(|| ConnectionState::Connecting);

    let game_id_clone = game_id.clone();

    use_effect(move || {
        let game_id = game_id_clone.clone();

        spawn(async move {
            // Get WebSocket URL from environment
            let ws_url = if let Ok(api_host) = std::env::var("APP_API_HOST") {
                api_host
                    .replace("http://", "ws://")
                    .replace("https://", "wss://")
                    + "/ws"
            } else {
                "ws://localhost:3000/ws".to_string()
            };

            web_sys::console::log_1(&format!("Connecting to WebSocket: {}", ws_url).into());

            // Connect to WebSocket
            let ws = match WebSocket::open(&ws_url) {
                Ok(ws) => {
                    connection_state.set(ConnectionState::Connected);
                    ws
                }
                Err(e) => {
                    let err_msg = format!("Failed to connect to WebSocket: {:?}", e);
                    web_sys::console::error_1(&err_msg.clone().into());
                    connection_state.set(ConnectionState::Error(err_msg));
                    return;
                }
            };

            let (mut write, mut read) = ws.split();

            // Subscribe to game
            let subscribe_msg = WebSocketMessage::Subscribe {
                game_id: game_id.clone(),
            };

            if let Ok(json) = serde_json::to_string(&subscribe_msg) {
                if let Err(e) = write.send(Message::Text(json)).await {
                    web_sys::console::error_1(
                        &format!("Failed to send subscribe message: {:?}", e).into(),
                    );
                    connection_state
                        .set(ConnectionState::Error(format!("Subscribe failed: {:?}", e)));
                    return;
                }
                web_sys::console::log_1(&format!("Subscribed to game {}", game_id).into());
            }

            // Listen for messages
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(WebSocketMessage::GameEvent {
                                game_id: received_game_id,
                                event,
                            }) => {
                                if received_game_id == game_id {
                                    web_sys::console::log_1(
                                        &format!("Received game event: {:?}", event).into(),
                                    );
                                    events.write().push(event);
                                }
                            }
                            Ok(WebSocketMessage::Error { message }) => {
                                web_sys::console::error_1(
                                    &format!("WebSocket error: {}", message).into(),
                                );
                                connection_state.set(ConnectionState::Error(message));
                            }
                            Ok(_) => {
                                web_sys::console::warn_1(&"Unexpected WebSocket message".into());
                            }
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Failed to parse WebSocket message: {:?}", e).into(),
                                );
                            }
                        }
                    }
                    Ok(Message::Bytes(_)) => {
                        web_sys::console::warn_1(
                            &"Received binary WebSocket message (unexpected)".into(),
                        );
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("WebSocket error: {:?}", e).into());
                        connection_state.set(ConnectionState::Error(format!("{:?}", e)));
                        break;
                    }
                }
            }

            // Connection closed
            web_sys::console::log_1(&"WebSocket disconnected".into());
            connection_state.set(ConnectionState::Disconnected);
        });
    });

    (events, connection_state)
}

/// Hook to manage WebSocket connection for real-time game updates
pub fn use_game_websocket(game_id: String) -> (Signal<Vec<GameEvent>>, Signal<ConnectionState>) {
    let mut events = use_signal(|| vec![]);
    let mut connection_state = use_signal(|| ConnectionState::Connecting);

    let game_id_clone = game_id.clone();

    use_effect(move || {
        let game_id = game_id_clone.clone();

        spawn(async move {
            // Get WebSocket URL from environment
            let ws_url = if let Ok(api_host) = std::env::var("APP_API_HOST") {
                api_host
                    .replace("http://", "ws://")
                    .replace("https://", "wss://")
                    + "/ws"
            } else {
                "ws://localhost:3000/ws".to_string()
            };

            tracing::info!("Connecting to WebSocket: {}", ws_url);

            // Connect to WebSocket
            let ws = match WebSocket::open(&ws_url) {
                Ok(ws) => {
                    connection_state.set(ConnectionState::Connected);
                    ws
                }
                Err(e) => {
                    let err_msg = format!("Failed to connect to WebSocket: {:?}", e);
                    tracing::error!("{}", err_msg);
                    connection_state.set(ConnectionState::Error(err_msg));
                    return;
                }
            };

            let (mut write, mut read) = ws.split();

            // Subscribe to game
            let subscribe_msg = WebSocketMessage::Subscribe {
                game_id: game_id.clone(),
            };

            if let Ok(json) = serde_json::to_string(&subscribe_msg) {
                if let Err(e) = write.send(Message::Text(json)).await {
                    tracing::error!("Failed to send subscribe message: {:?}", e);
                    connection_state
                        .set(ConnectionState::Error(format!("Subscribe failed: {:?}", e)));
                    return;
                }
                tracing::info!("Subscribed to game {}", game_id);
            }

            // Listen for messages
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(WebSocketMessage::GameEvent {
                                game_id: received_game_id,
                                event,
                            }) => {
                                if received_game_id == game_id {
                                    tracing::debug!("Received game event: {:?}", event);
                                    events.write().push(event);
                                }
                            }
                            Ok(WebSocketMessage::Error { message }) => {
                                tracing::error!("WebSocket error: {}", message);
                                connection_state.set(ConnectionState::Error(message));
                            }
                            Ok(_) => {
                                tracing::warn!("Unexpected WebSocket message");
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse WebSocket message: {:?}", e);
                            }
                        }
                    }
                    Ok(Message::Bytes(_)) => {
                        tracing::warn!("Received binary WebSocket message (unexpected)");
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {:?}", e);
                        connection_state.set(ConnectionState::Error(format!("{:?}", e)));
                        break;
                    }
                }
            }

            // Connection closed
            tracing::info!("WebSocket disconnected");
            connection_state.set(ConnectionState::Disconnected);
        });
    });

    (events, connection_state)
}
