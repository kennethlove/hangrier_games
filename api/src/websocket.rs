use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use shared::{GameEvent, WebSocketMessage};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::AppState;

/// Global broadcaster for game events
#[derive(Clone)]
pub struct GameBroadcaster {
    tx: broadcast::Sender<WebSocketMessage>,
}

impl GameBroadcaster {
    /// Create a new broadcaster with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Broadcast a message to all subscribed clients
    pub fn broadcast(&self, msg: WebSocketMessage) {
        match self.tx.send(msg.clone()) {
            Ok(count) => {
                debug!("Broadcast message to {} subscribers: {:?}", count, msg);
            }
            Err(_) => {
                debug!("No active subscribers for broadcast");
            }
        }
    }

    /// Subscribe to the broadcast channel
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.tx.subscribe()
    }
}

impl Default for GameBroadcaster {
    fn default() -> Self {
        Self::new(1000) // Default capacity: 1000 messages
    }
}

/// WebSocket upgrade handler
pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state.broadcaster))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, broadcaster: Arc<GameBroadcaster>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcaster.subscribe();
    let subscribed_games = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

    info!("WebSocket client connected");

    // Spawn task to forward broadcast messages to this client
    let subscribed_for_send = subscribed_games.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Only send messages for games this client is subscribed to
            let games = subscribed_for_send.lock().await;
            let should_send = match &msg {
                WebSocketMessage::GameEvent { game_id, .. } => games.contains(game_id),
                _ => true, // Send errors and other messages unconditionally
            };
            drop(games); // Release lock before sending

            if should_send {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize WebSocket message: {}", e);
                        continue;
                    }
                };

                if sender.send(Message::Text(json.into())).await.is_err() {
                    debug!("Client disconnected during send");
                    break;
                }
            }
        }
    });

    // Handle incoming client messages (subscriptions)
    let subscribed_for_recv = subscribed_games.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            match serde_json::from_str::<WebSocketMessage>(&text) {
                Ok(WebSocketMessage::Subscribe { game_id }) => {
                    info!("Client subscribed to game {}", game_id);
                    let mut games = subscribed_for_recv.lock().await;
                    if !games.contains(&game_id) {
                        games.push(game_id);
                    }
                }
                Ok(WebSocketMessage::Unsubscribe { game_id }) => {
                    info!("Client unsubscribed from game {}", game_id);
                    let mut games = subscribed_for_recv.lock().await;
                    games.retain(|g| g != &game_id);
                }
                Ok(msg) => {
                    warn!("Unexpected message from client: {:?}", msg);
                }
                Err(e) => {
                    error!("Failed to parse WebSocket message: {}", e);
                }
            }
        }
    });

    // Wait for either task to complete (disconnect or error)
    tokio::select! {
        _ = (&mut recv_task) => {
            send_task.abort();
        }
        _ = (&mut send_task) => {
            recv_task.abort();
        }
    }

    info!("WebSocket client disconnected");
}

/// Convert game message to broadcast event
pub fn broadcast_game_message(
    broadcaster: &GameBroadcaster,
    game_id: &str,
    source: &str,
    content: &str,
    game_day: u32,
) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        event: GameEvent::Message {
            source: source.to_string(),
            content: content.to_string(),
            game_day,
        },
    });
}

/// Broadcast game started event
pub fn broadcast_game_started(broadcaster: &GameBroadcaster, game_id: &str, day: u32) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        event: GameEvent::GameStarted { day },
    });
}

/// Broadcast day started event
pub fn broadcast_day_started(broadcaster: &GameBroadcaster, game_id: &str, day: u32) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        event: GameEvent::DayStarted { day },
    });
}

/// Broadcast night started event
pub fn broadcast_night_started(broadcaster: &GameBroadcaster, game_id: &str, day: u32) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        event: GameEvent::NightStarted { day },
    });
}

/// Broadcast game finished event
pub fn broadcast_game_finished(
    broadcaster: &GameBroadcaster,
    game_id: &str,
    winner: Option<String>,
) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        event: GameEvent::GameFinished { winner },
    });
}
