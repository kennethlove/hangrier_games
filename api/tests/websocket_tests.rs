mod common;

use common::TestDb;

/// Test GameBroadcaster functionality directly
#[tokio::test]
async fn test_game_broadcaster_basic() {
    use api::websocket::GameBroadcaster;
    use shared::{GameEvent, WebSocketMessage};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);

    // Subscribe to broadcasts
    let mut rx = broadcaster.subscribe();

    // Broadcast a message
    let test_event = WebSocketMessage::GameEvent {
        game_id: "test-game".to_string(),
        event: GameEvent::GameStarted { day: 1 },
    };
    broadcaster.broadcast(test_event.clone());

    // Receive the message
    let received = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(received.is_ok(), "Should receive broadcast message");

    let msg = received.unwrap().unwrap();
    match msg {
        WebSocketMessage::GameEvent { game_id, event } => {
            assert_eq!(game_id, "test-game");
            assert!(matches!(event, GameEvent::GameStarted { day: 1 }));
        }
        _ => panic!("Expected GameEvent"),
    }
}

/// Test GameBroadcaster with multiple subscribers
#[tokio::test]
async fn test_game_broadcaster_multi_subscriber() {
    use api::websocket::GameBroadcaster;
    use shared::{GameEvent, WebSocketMessage};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);

    // Create 3 subscribers
    let mut rx1 = broadcaster.subscribe();
    let mut rx2 = broadcaster.subscribe();
    let mut rx3 = broadcaster.subscribe();

    // Broadcast a message
    let test_event = WebSocketMessage::GameEvent {
        game_id: "test-game".to_string(),
        event: GameEvent::DayStarted { day: 2 },
    };
    broadcaster.broadcast(test_event.clone());

    // All subscribers should receive the message
    let msg1 = timeout(Duration::from_secs(1), rx1.recv())
        .await
        .unwrap()
        .unwrap();
    let msg2 = timeout(Duration::from_secs(1), rx2.recv())
        .await
        .unwrap()
        .unwrap();
    let msg3 = timeout(Duration::from_secs(1), rx3.recv())
        .await
        .unwrap()
        .unwrap();

    // Verify all received the same message
    for msg in [msg1, msg2, msg3] {
        match msg {
            WebSocketMessage::GameEvent { game_id, event } => {
                assert_eq!(game_id, "test-game");
                assert!(matches!(event, GameEvent::DayStarted { day: 2 }));
            }
            _ => panic!("Expected GameEvent"),
        }
    }
}

/// Test broadcast helper functions
#[tokio::test]
async fn test_broadcast_helper_functions() {
    use api::websocket::{
        GameBroadcaster, broadcast_day_started, broadcast_game_finished, broadcast_game_started,
        broadcast_night_started,
    };
    use shared::{GameEvent, WebSocketMessage};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);
    let mut rx = broadcaster.subscribe();

    // Test broadcast_game_started
    broadcast_game_started(&broadcaster, "game1", 1);
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match msg {
        WebSocketMessage::GameEvent { game_id, event } => {
            assert_eq!(game_id, "game1");
            assert!(matches!(event, GameEvent::GameStarted { day: 1 }));
        }
        _ => panic!("Expected GameEvent"),
    }

    // Test broadcast_day_started
    broadcast_day_started(&broadcaster, "game1", 2);
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match msg {
        WebSocketMessage::GameEvent { game_id, event } => {
            assert_eq!(game_id, "game1");
            assert!(matches!(event, GameEvent::DayStarted { day: 2 }));
        }
        _ => panic!("Expected GameEvent"),
    }

    // Test broadcast_night_started
    broadcast_night_started(&broadcaster, "game1", 2);
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match msg {
        WebSocketMessage::GameEvent { game_id, event } => {
            assert_eq!(game_id, "game1");
            assert!(matches!(event, GameEvent::NightStarted { day: 2 }));
        }
        _ => panic!("Expected GameEvent"),
    }

    // Test broadcast_game_finished
    broadcast_game_finished(&broadcaster, "game1", Some("Winner".to_string()));
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match msg {
        WebSocketMessage::GameEvent { game_id, event } => {
            assert_eq!(game_id, "game1");
            assert!(matches!(
                event,
                GameEvent::GameFinished {
                    winner: Some(ref w)
                } if w == "Winner"
            ));
        }
        _ => panic!("Expected GameEvent"),
    }
}

/// Test database cleanup (ensures TestDb works)
#[tokio::test]
async fn test_database_setup() {
    let test_db = TestDb::new().await;
    let _state = test_db.app_state();
    test_db.cleanup().await;
}
