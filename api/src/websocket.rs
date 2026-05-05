use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use shared::WebSocketMessage;
use shared::messages::{GameMessage, MessagePayload, MessageSource, Phase, TributeRef};
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

/// Broadcast a typed [`GameMessage`] (drained from `Game::messages`) to
/// subscribed websocket clients. The message rides as-is so frontends can
/// dispatch on `MessagePayload` directly without parsing a parallel event
/// hierarchy.
pub fn broadcast_game_message(broadcaster: &GameBroadcaster, game_id: &str, message: GameMessage) {
    broadcaster.broadcast(WebSocketMessage::GameEvent {
        game_id: game_id.to_string(),
        message: Box::new(message),
    });
}

/// Synthesize and broadcast a lifecycle [`MessagePayload::CycleStart`] for a
/// game whose status just transitioned to `InProgress` but where the engine
/// has not yet run a cycle (so it has not had a chance to emit a
/// `CycleStart` itself).
pub fn broadcast_game_started(broadcaster: &GameBroadcaster, game_id: &str, day: u32) {
    let payload = MessagePayload::CycleStart {
        day,
        phase: Phase::Day,
    };
    let msg = GameMessage::new(
        MessageSource::Game(game_id.to_string()),
        day,
        Phase::Day,
        0,
        0,
        format!("game:{}", game_id),
        format!("Day {} dawns over the arena.", day),
        payload,
    );
    broadcast_game_message(broadcaster, game_id, msg);
}

/// Synthesize and broadcast a lifecycle [`MessagePayload::GameEnded`] for a
/// game that finished without an additional cycle being run (e.g. the
/// 24-deaths-already early-finish path in `next_step`).
pub fn broadcast_game_finished(
    broadcaster: &GameBroadcaster,
    game_id: &str,
    winner: Option<String>,
) {
    let winner_ref = winner.map(|name| TributeRef {
        identifier: String::new(),
        name,
    });
    let payload = MessagePayload::GameEnded {
        winner: winner_ref.clone(),
    };
    let content = match &winner_ref {
        Some(w) => format!("{} has won the game!", w.name),
        None => "The game has ended with no survivors.".to_string(),
    };
    let msg = GameMessage::new(
        MessageSource::Game(game_id.to_string()),
        0,
        Phase::Day,
        0,
        0,
        format!("game:{}", game_id),
        content,
        payload,
    );
    broadcast_game_message(broadcaster, game_id, msg);
}
