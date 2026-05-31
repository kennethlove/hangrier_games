//! Integration tests for the announcers commentary pipeline.
//!
//! Validates the pipeline end-to-end by calling into the API's game cycle
//! logic directly, avoiding the `tokio::spawn` race that makes
//! HTTP-level testing of the background task flaky.

use announcers::{CommentaryError, CommentaryLine, CommentarySegment};
use async_trait::async_trait;
use futures::stream::Stream;
use shared::messages::{MessagePayload, MessageSource, Phase, TributeRef};
use std::pin::Pin;
use std::sync::Arc;

/// A mock commentator that records invocations.
struct MockCommentator {
    invocation_count: std::sync::atomic::AtomicU32,
}

impl MockCommentator {
    fn new() -> Self {
        Self {
            invocation_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn invoked(&self) -> u32 {
        self.invocation_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl announcers::Commentator for MockCommentator {
    async fn generate(
        &self,
        _package: &announcers::BroadcastPackage,
    ) -> Result<CommentarySegment, CommentaryError> {
        self.invocation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(CommentarySegment {
            id: "mock-test-id".into(),
            game_id: String::new(),
            day: 0,
            phase: String::new(),
            lines: vec![
                announcers::CommentaryLine {
                    speaker: "Verity".into(),
                    text: "What a day in the arena!".into(),
                },
                announcers::CommentaryLine {
                    speaker: "Rex".into(),
                    text: "Absolutely brutal, Verity.".into(),
                },
            ],
            generated_at: chrono::Utc::now(),
            model_used: "test-mock".into(),
        })
    }

    fn generate_stream(
        &self,
        _package: &announcers::BroadcastPackage,
    ) -> Pin<Box<dyn Stream<Item = Result<CommentaryLine, CommentaryError>> + Send>> {
        self.invocation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let lines = vec![
            CommentaryLine {
                speaker: "Verity".into(),
                text: "What a day in the arena!".into(),
            },
            CommentaryLine {
                speaker: "Rex".into(),
                text: "Absolutely brutal, Verity.".into(),
            },
        ];
        let items: Vec<_> = lines.into_iter().map(Ok).collect();
        Box::pin(futures::stream::iter(items))
    }
}

/// The full commentary pipeline generates a segment with correct metadata.
#[tokio::test]
async fn generate_commentary_pipeline() {
    let mock = MockCommentator::new();

    // Build a realistic game snapshot.
    let header = announcers::GameStateSnapshot {
        day: 1,
        phase: "day".into(),
        alive_count: 22,
        kill_leaders: vec![announcers::KillLeader {
            name: "Cato".into(),
            district: 2,
            kill_count: 2,
        }],
        alliances: vec![],
        hot_zones: vec![],
        killing_sprees: vec![],
    };

    let events = vec![make_msg(MessagePayload::TributeKilled {
        victim: tr("Peeta"),
        killer: Some(tr("Cato")),
        cause: "combat".into(),
    })];

    let histories = vec![
        tribute_digest("Cato", 2, "alive", "unharmed", "Cornucopia"),
        tribute_digest("Peeta", 12, "deceased", "deceased", "Cornucopia"),
        tribute_digest("Katniss", 12, "alive", "unharmed", "Forest"),
    ];

    let segment = announcers::generate_commentary(
        &mock,
        "game-test-123",
        3,
        "day",
        header,
        &events,
        histories,
    )
    .await
    .unwrap();

    assert_eq!(segment.game_id, "game-test-123");
    assert_eq!(segment.day, 3);
    assert_eq!(segment.phase, "day");
    assert_eq!(segment.lines.len(), 2);
    assert_eq!(segment.model_used, "test-mock");
    assert_eq!(mock.invoked(), 1, "Commentator should be called exactly once");
}

/// Broadcast package building with kill leaders and sprees produces
/// correctly structured output.
#[test]
fn broadcast_package_with_leaders_and_sprees() {
    let header = announcers::GameStateSnapshot {
        day: 1,
        phase: "day".into(),
        alive_count: 20,
        kill_leaders: vec![announcers::KillLeader {
            name: "Cato".into(),
            district: 2,
            kill_count: 2,
        }],
        alliances: vec![],
        hot_zones: vec![announcers::AreaActivity {
            name: "Cornucopia".into(),
            activity_level: "hot".into(),
        }],
        killing_sprees: vec![announcers::KillingSpree {
            name: "Cato".into(),
            district: 2,
            streak: 4,
            label: "on fire".into(),
        }],
    };

    let events = vec![make_msg(MessagePayload::TributeKilled {
            victim: tr("Marvel"),
            killer: Some(tr("Cato")),
            cause: "combat".into(),
        },
    )];

    let package = announcers::BroadcastPackageBuilder::build(header, &events, vec![]);
    assert_eq!(package.header.kill_leaders.len(), 1);
    assert_eq!(package.header.killing_sprees.len(), 1);
    assert_eq!(package.header.killing_sprees[0].label, "on fire");
    assert_eq!(package.header.hot_zones.len(), 1);
    assert_eq!(package.events.len(), 1);
    assert_eq!(package.events[0].kind, announcers::EventKind::Death);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tr(name: &str) -> TributeRef {
    TributeRef {
        identifier: format!("id-{name}"),
        name: name.into(),
    }
}

fn make_msg(payload: MessagePayload) -> shared::messages::GameMessage {
    shared::messages::GameMessage {
        identifier: format!("msg-{}", uuid::Uuid::new_v4()),
        source: MessageSource::Game("test-game".into()),
        game_day: 1,
        phase: Phase::Day,
        tick: 0,
        emit_index: 1,
        subject: String::new(),
        timestamp: chrono::Utc::now(),
        content: String::new(),
        payload,
    }
}

fn tribute_digest(
    name: &str,
    district: u8,
    status: &str,
    injury_level: &str,
    location: &str,
) -> announcers::TributeDigest {
    announcers::TributeDigest {
        identifier: format!("id-{name}"),
        name: name.into(),
        district,
        status: status.into(),
        injury_level: injury_level.into(),
        location: location.into(),
        allies: vec![],
        kill_streak: 0,
        notable_events: vec![],
        highlights: vec![],
    }
}
