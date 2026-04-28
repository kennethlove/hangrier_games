use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum MessageSource {
    #[serde(rename = "Game")]
    Game(String), // Game identifier
    #[serde(rename = "Area")]
    Area(String), // Area name
    #[serde(rename = "Tribute")]
    Tribute(String), // Tribute identifier
}

/// Typed category for a `GameMessage`. Initial set covers alliance lifecycle
/// events; future categories (combat, area, sponsor) will be added as those
/// emit sites get refactored. `None` on `GameMessage.kind` means the message
/// has not yet been categorised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    AllianceFormed,
    BetrayalTriggered,
    TrustShockBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
    /// Optional typed category. `None` for legacy/uncategorised messages so
    /// existing serialized rows hydrate cleanly. Skipped on serialize when
    /// absent to keep JSON compact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<MessageKind>,
    /// Optional structured payload — the externally-tagged JSON form of
    /// the legacy `GameEvent`, serialized to a `String`. Stored as a string
    /// (not a structured `serde_json::Value`) so it survives SurrealDB's
    /// bespoke (de)serializer, which collapses externally-tagged Rust
    /// enums to `{}` when bound into an `object` column. `None` for
    /// legacy rows (and plain `log` calls that have no structured
    /// event). Skipped on serialize when absent to keep JSON compact.
    /// **Temporary**: this field is removed in Task 3 of the timeline-pr1
    /// plan; constructors that populate it live in the `game` crate shim
    /// (see `game/src/messages.rs`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Stable per-row identifier for the structured event payload, distinct
    /// from `identifier` on the message itself. `Some` when `event` is
    /// `Some`; `None` for legacy rows. Stored as a string-form UUID to
    /// match the repo's existing on-the-wire UUID convention.
    /// **Temporary**: removed in Task 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

impl GameMessage {
    /// Create a new game message without a typed kind.
    pub fn new(source: MessageSource, game_day: u32, subject: String, content: String) -> Self {
        GameMessage {
            identifier: Uuid::new_v4().to_string(),
            source,
            game_day,
            subject,
            timestamp: Utc::now(),
            content,
            kind: None,
            event: None,
            event_id: None,
        }
    }

    /// Create a new game message with a typed `MessageKind`.
    pub fn with_kind(
        source: MessageSource,
        game_day: u32,
        subject: String,
        content: String,
        kind: MessageKind,
    ) -> Self {
        GameMessage {
            identifier: Uuid::new_v4().to_string(),
            source,
            game_day,
            subject,
            timestamp: Utc::now(),
            content,
            kind: Some(kind),
            event: None,
            event_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn message_kind_serde_roundtrip() {
        for kind in [
            MessageKind::AllianceFormed,
            MessageKind::BetrayalTriggered,
            MessageKind::TrustShockBreak,
        ] {
            let s = serde_json::to_string(&kind).expect("serialize MessageKind");
            let back: MessageKind = serde_json::from_str(&s).expect("deserialize MessageKind");
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn game_message_kind_optional_field_default() {
        // Legacy row: no `kind` field present.
        let ts = Utc.timestamp_nanos(0);
        let raw = json!({
            "identifier": "abc",
            "source": { "type": "Game", "value": "g1" },
            "game_day": 1,
            "subject": "game:g1",
            "timestamp": ts.timestamp_nanos_opt().unwrap(),
            "content": "hello",
        });
        let msg: GameMessage = serde_json::from_value(raw).expect("deserialize without kind");
        assert!(msg.kind.is_none());
    }

    #[test]
    fn game_message_with_kind_constructor_sets_kind() {
        let msg = GameMessage::with_kind(
            MessageSource::Game("g".into()),
            2,
            "game:g".into(),
            "content".into(),
            MessageKind::AllianceFormed,
        );
        assert_eq!(msg.kind, Some(MessageKind::AllianceFormed));
    }

    #[test]
    fn game_message_default_constructor_has_no_kind() {
        let msg = GameMessage::new(
            MessageSource::Game("g".into()),
            2,
            "game:g".into(),
            "content".into(),
        );
        assert!(msg.kind.is_none());
    }

    #[test]
    fn game_message_skips_kind_when_none() {
        let msg = GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            "subj".into(),
            "c".into(),
        );
        let s = serde_json::to_string(&msg).expect("serialize");
        assert!(!s.contains("\"kind\""), "kind field should be skipped: {s}");
    }

    #[test]
    fn game_message_skips_event_fields_when_none() {
        let msg = GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            "subj".into(),
            "c".into(),
        );
        let s = serde_json::to_string(&msg).expect("serialize");
        assert!(!s.contains("\"event\""), "event should be skipped: {s}");
        assert!(
            !s.contains("\"event_id\""),
            "event_id should be skipped: {s}"
        );
    }

    #[test]
    fn game_message_legacy_row_without_event_fields_deserializes() {
        // Legacy row: no `event` / `event_id` / `kind` fields present.
        let ts = Utc.timestamp_nanos(0);
        let raw = json!({
            "identifier": "abc",
            "source": { "type": "Game", "value": "g1" },
            "game_day": 1,
            "subject": "game:g1",
            "timestamp": ts.timestamp_nanos_opt().unwrap(),
            "content": "hello",
        });
        let msg: GameMessage = serde_json::from_value(raw).expect("deserialize legacy row");
        assert!(msg.kind.is_none());
        assert!(msg.event.is_none());
        assert!(msg.event_id.is_none());
    }
}
