mod common;

use chrono::Utc;
use common::TestDb;
use game::events::GameEvent;
use game::messages::{GameMessage, MessageSource};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use uuid::Uuid;

/// API persistence shape — mirrors the private `GameLog` struct in
/// `api::games`. Kept here so the test exercises the same on-the-wire
/// representation that production code writes to the `message` table.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct GameLog {
    pub id: RecordId,
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: chrono::DateTime<Utc>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<game::messages::MessageKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// mqi.3: a structured `GameEvent` written to the `message` table must
/// roundtrip back through a normal `SELECT * FROM message` and deserialize
/// cleanly into `GameMessage` with `event` and `event_id` populated.
#[tokio::test]
async fn test_game_event_persistence_roundtrip() {
    let test_db = TestDb::new().await;
    let db = test_db.db.clone();

    let game_id = Uuid::new_v4().to_string();
    let tribute_id = Uuid::new_v4();
    let event = GameEvent::TributeRest {
        tribute_id,
        tribute_name: "Alice".into(),
    };
    let msg = GameMessage::with_event(
        MessageSource::Tribute(tribute_id.to_string()),
        1,
        format!("game:{}/tribute:{}", game_id, tribute_id),
        event.to_string(),
        event.clone(),
    );

    let row = GameLog {
        id: RecordId::from(("message", &msg.identifier)),
        identifier: msg.identifier.clone(),
        source: msg.source.clone(),
        game_day: msg.game_day,
        subject: msg.subject.clone(),
        timestamp: msg.timestamp,
        content: msg.content.clone(),
        kind: msg.kind,
        event: msg.event.clone(),
        event_id: msg.event_id.clone(),
    };

    db.insert::<Vec<GameMessage>>(())
        .content(vec![row])
        .await
        .expect("insert structured event row");

    let mut response = db
        .query("SELECT * FROM message WHERE identifier = $id")
        .bind(("id", msg.identifier.clone()))
        .await
        .expect("select roundtripped row");

    let rows: Vec<GameMessage> = response.take(0).expect("deserialize as Vec<GameMessage>");
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one row, got {}",
        rows.len()
    );
    let back = &rows[0];
    let decoded = back
        .structured_event()
        .expect("event payload present")
        .expect("structured event decodes cleanly");
    assert_eq!(decoded, event, "event payload must roundtrip");
    assert_eq!(
        back.event_id, msg.event_id,
        "event_id must roundtrip unchanged"
    );
    assert_eq!(back.content, msg.content);
    assert!(back.kind.is_none(), "with_event must not set kind");

    test_db.cleanup().await;
}

/// mqi.3: legacy rows written without `event` / `event_id` must still
/// deserialize cleanly into `GameMessage` with those fields `None`. This
/// guards the schema migration from breaking existing message data.
#[tokio::test]
async fn test_legacy_message_row_without_event_fields_deserializes() {
    let test_db = TestDb::new().await;
    let db = test_db.db.clone();

    let identifier = Uuid::new_v4().to_string();
    let game_id = Uuid::new_v4().to_string();

    // Insert via raw SurrealQL so we omit `event` / `event_id` / `kind`
    // entirely — the legacy shape from before mqi.1.
    db.query(
        r#"
        CREATE type::thing("message", $id) CONTENT {
            identifier: $id,
            source: { type: "Game", value: $game_id },
            game_day: 1,
            subject: $subject,
            timestamp: 0,
            content: "legacy line"
        }
        "#,
    )
    .bind(("id", identifier.clone()))
    .bind(("game_id", game_id.clone()))
    .bind(("subject", format!("game:{}", game_id)))
    .await
    .expect("create legacy row");

    let mut response = db
        .query("SELECT * FROM message WHERE identifier = $id")
        .bind(("id", identifier.clone()))
        .await
        .expect("select legacy row");

    let rows: Vec<GameMessage> = response
        .take(0)
        .expect("deserialize legacy row as Vec<GameMessage>");
    assert_eq!(rows.len(), 1);
    let back = &rows[0];
    assert!(back.event.is_none(), "legacy row must hydrate event=None");
    assert!(
        back.event_id.is_none(),
        "legacy row must hydrate event_id=None"
    );
    assert!(back.kind.is_none(), "legacy row must hydrate kind=None");
    assert_eq!(back.content, "legacy line");

    test_db.cleanup().await;
}
