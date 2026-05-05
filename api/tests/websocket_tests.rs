mod common;

use common::TestDb;

fn sample_message(
    payload: shared::messages::MessagePayload,
    content: &str,
) -> shared::messages::GameMessage {
    shared::messages::GameMessage::new(
        shared::messages::MessageSource::Game("g".into()),
        1,
        shared::messages::Phase::Day,
        0,
        0,
        "subj".into(),
        content.into(),
        payload,
    )
}

/// Test GameBroadcaster functionality directly
#[tokio::test]
async fn test_game_broadcaster_basic() {
    use api::websocket::GameBroadcaster;
    use shared::WebSocketMessage;
    use shared::messages::{MessagePayload, Phase};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);
    let mut rx = broadcaster.subscribe();

    let msg = sample_message(
        MessagePayload::CycleStart {
            day: 1,
            phase: Phase::Day,
        },
        "started",
    );
    let test_event = WebSocketMessage::GameEvent {
        game_id: "test-game".to_string(),
        message: Box::new(msg),
    };
    broadcaster.broadcast(test_event.clone());

    let received = timeout(Duration::from_secs(1), rx.recv()).await;
    assert!(received.is_ok(), "Should receive broadcast message");

    let got = received.unwrap().unwrap();
    match got {
        WebSocketMessage::GameEvent { game_id, message } => {
            assert_eq!(game_id, "test-game");
            assert!(matches!(
                message.payload,
                MessagePayload::CycleStart {
                    day: 1,
                    phase: Phase::Day
                }
            ));
        }
        _ => panic!("Expected GameEvent"),
    }
}

/// Test GameBroadcaster with multiple subscribers
#[tokio::test]
async fn test_game_broadcaster_multi_subscriber() {
    use api::websocket::GameBroadcaster;
    use shared::WebSocketMessage;
    use shared::messages::{MessagePayload, Phase};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);

    let mut rx1 = broadcaster.subscribe();
    let mut rx2 = broadcaster.subscribe();
    let mut rx3 = broadcaster.subscribe();

    let msg = sample_message(
        MessagePayload::CycleStart {
            day: 2,
            phase: Phase::Day,
        },
        "day 2",
    );
    let test_event = WebSocketMessage::GameEvent {
        game_id: "test-game".to_string(),
        message: Box::new(msg),
    };
    broadcaster.broadcast(test_event.clone());

    let m1 = timeout(Duration::from_secs(1), rx1.recv())
        .await
        .unwrap()
        .unwrap();
    let m2 = timeout(Duration::from_secs(1), rx2.recv())
        .await
        .unwrap()
        .unwrap();
    let m3 = timeout(Duration::from_secs(1), rx3.recv())
        .await
        .unwrap()
        .unwrap();

    for got in [m1, m2, m3] {
        match got {
            WebSocketMessage::GameEvent { game_id, message } => {
                assert_eq!(game_id, "test-game");
                assert!(matches!(
                    message.payload,
                    MessagePayload::CycleStart {
                        day: 2,
                        phase: Phase::Day
                    }
                ));
            }
            _ => panic!("Expected GameEvent"),
        }
    }
}

/// Test broadcast helper functions
#[tokio::test]
async fn test_broadcast_helper_functions() {
    use api::websocket::{GameBroadcaster, broadcast_game_finished, broadcast_game_started};
    use shared::WebSocketMessage;
    use shared::messages::{MessagePayload, Phase};
    use tokio::time::{Duration, timeout};

    let broadcaster = GameBroadcaster::new(10);
    let mut rx = broadcaster.subscribe();

    broadcast_game_started(&broadcaster, "game1", 1);
    let got = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match got {
        WebSocketMessage::GameEvent { game_id, message } => {
            assert_eq!(game_id, "game1");
            assert!(matches!(
                message.payload,
                MessagePayload::CycleStart {
                    day: 1,
                    phase: Phase::Day
                }
            ));
        }
        _ => panic!("Expected GameEvent"),
    }

    broadcast_game_finished(&broadcaster, "game1", Some("Winner".to_string()));
    let got = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match got {
        WebSocketMessage::GameEvent { game_id, message } => {
            assert_eq!(game_id, "game1");
            match &message.payload {
                MessagePayload::GameEnded { winner: Some(w) } => assert_eq!(w.name, "Winner"),
                other => panic!("Expected GameEnded with winner, got {other:?}"),
            }
        }
        _ => panic!("Expected GameEvent"),
    }

    broadcast_game_finished(&broadcaster, "game1", None);
    let got = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    match got {
        WebSocketMessage::GameEvent { message, .. } => {
            assert!(matches!(
                message.payload,
                MessagePayload::GameEnded { winner: None }
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
