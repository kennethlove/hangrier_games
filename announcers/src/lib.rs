//! # Announcers — AI-powered Hunger Games commentary.
//!
//! Transforms structured game events (from `shared::messages`) into
//! broadcast-style commentary between two Capitol commentators (Verity
//! and Rex).
//!
//! ## Architecture
//!
//! ```text
//! Game phase completes
//!         │
//!         ▼
//! BroadcastPackageBuilder::build(header, events, histories)
//!         │
//!         ▼
//! BroadcastPackage  ──→  Commentator::generate(package)
//!         │
//!         ▼
//! CommentarySegment (stored + pushed via SSE)
//! ```
//!
//! The [`Commentator`] trait abstracts over LLM backends. The default
//! implementation uses Ollama (feature-gated behind `features = ["ollama"]`).

pub mod broadcast;
pub mod history;
pub mod llm;
pub mod severity;
pub mod types;

// Re-export key types and traits at crate root.
pub use broadcast::BroadcastPackageBuilder;
pub use history::TributeHistories;
pub use llm::Commentator;
pub use types::*;

#[cfg(feature = "ollama")]
pub use llm::ollama::OllamaCommentator;

#[cfg(feature = "cloudflare")]
pub use llm::cloudflare::CloudflareCommentator;

/// One-shot convenience: build a [`BroadcastPackage`] and generate commentary.
///
/// This is the primary entry point for the API integration layer. It:
/// 1. Builds the package from the phase's game state, events, and histories
/// 2. Calls `commentator.generate()` to produce the segment
/// 3. Fills in the game context (id, day, phase) on the returned segment
pub async fn generate_commentary(
    commentator: &dyn Commentator,
    game_id: &str,
    day: u32,
    phase: &str,
    header: GameStateSnapshot,
    events: &[shared::messages::GameMessage],
    histories: Vec<TributeDigest>,
) -> Result<CommentarySegment, CommentaryError> {
    let package = BroadcastPackageBuilder::build(header, events, histories);
    let mut segment = commentator.generate(&package).await?;
    segment.game_id = game_id.to_string();
    segment.day = day;
    segment.phase = phase.to_string();
    Ok(segment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CommentaryError, CommentaryLine, CommentarySegment};
    use async_trait::async_trait;
    use futures::stream::Stream;
    use shared::messages::{GameMessage, MessagePayload, MessageSource, Phase, TributeRef};
    use std::pin::Pin;

    /// A mock commentator that returns a fixed segment.
    struct MockCommentator {
        lines: Vec<CommentaryLine>,
        fail: bool,
    }

    impl MockCommentator {
        fn ok(lines: Vec<&str>) -> Self {
            let lines = lines
                .into_iter()
                .enumerate()
                .map(|(i, text)| {
                    let speaker = if i % 2 == 0 { "Verity" } else { "Rex" };
                    CommentaryLine {
                        speaker: speaker.into(),
                        text: text.into(),
                    }
                })
                .collect();
            Self { lines, fail: false }
        }

        fn empty() -> Self {
            Self {
                lines: vec![],
                fail: false,
            }
        }

        fn error() -> Self {
            Self {
                lines: vec![],
                fail: true,
            }
        }
    }

    #[async_trait]
    impl Commentator for MockCommentator {
        async fn generate(
            &self,
            _package: &BroadcastPackage,
        ) -> Result<CommentarySegment, CommentaryError> {
            if self.fail {
                return Err(CommentaryError::Generate("mock failure".into()));
            }
            Ok(CommentarySegment {
                id: "mock-id".into(),
                game_id: String::new(),
                day: 0,
                phase: String::new(),
                lines: self.lines.clone(),
                generated_at: chrono::Utc::now(),
                model_used: "mock".into(),
            })
        }

        fn generate_stream(
            &self,
            _package: &BroadcastPackage,
        ) -> Pin<Box<dyn Stream<Item = Result<CommentaryLine, CommentaryError>> + Send>> {
            let lines: Vec<Result<CommentaryLine, CommentaryError>> =
                self.lines.clone().into_iter().map(Ok).collect();
            Box::pin(futures::stream::iter(lines))
        }
    }

    fn test_uuid(name: &str) -> uuid::Uuid {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        uuid::Uuid::from_u128(hash as u128)
    }

    fn make_event() -> GameMessage {
        GameMessage {
            identifier: "evt-1".into(),
            source: MessageSource::Tribute("id-Katniss".into()),
            game_day: 1,
            phase: Phase::Day,
            tick: 1,
            emit_index: 1,
            subject: "tribute:Katniss".into(),
            timestamp: chrono::Utc::now(),
            content: "Katniss found a bow.".into(),
            payload: MessagePayload::ItemFound {
                tribute: TributeRef {
                    identifier: test_uuid("Katniss").to_string().into(),
                    name: "Katniss".into(),
                },
                item: shared::messages::ItemRef {
                    identifier: test_uuid("bow").to_string().into(),
                    name: "bow".into(),
                },
                area: shared::messages::AreaRef {
                    identifier: test_uuid("Cornucopia").to_string().into(),
                    name: "Cornucopia".into(),
                },
            },
        }
    }

    #[tokio::test]
    async fn generate_populates_game_context() {
        let commentator = MockCommentator::ok(vec!["Hello!", "Great shot!"]);
        let header = GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count: 12,
            kill_leaders: vec![],
            alliances: vec![],
            hot_zones: vec![],
            killing_sprees: vec![],
        };
        let events = vec![make_event()];
        let histories = vec![];

        let segment = generate_commentary(
            &commentator,
            "game-abc",
            3,
            "day",
            header,
            &events,
            histories,
        )
        .await
        .unwrap();

        assert_eq!(segment.game_id, "game-abc");
        assert_eq!(segment.day, 3);
        assert_eq!(segment.phase, "day");
        assert_eq!(segment.lines.len(), 2);
        assert_eq!(segment.lines[0].speaker, "Verity");
        assert_eq!(segment.lines[0].text, "Hello!");
    }

    #[tokio::test]
    async fn generate_handles_empty_lines() {
        let commentator = MockCommentator::empty();
        let header = GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count: 24,
            kill_leaders: vec![],
            alliances: vec![],
            hot_zones: vec![],
            killing_sprees: vec![],
        };
        let segment =
            generate_commentary(&commentator, "game-xyz", 1, "night", header, &[], vec![])
                .await
                .unwrap();

        assert_eq!(segment.game_id, "game-xyz");
        assert!(segment.lines.is_empty());
    }

    #[tokio::test]
    async fn generate_propagates_commentator_error() {
        let commentator = MockCommentator::error();
        let header = GameStateSnapshot {
            day: 1,
            phase: "day".into(),
            alive_count: 12,
            kill_leaders: vec![],
            alliances: vec![],
            hot_zones: vec![],
            killing_sprees: vec![],
        };
        let result = generate_commentary(&commentator, "g", 1, "d", header, &[], vec![]).await;

        assert!(result.is_err());
        match result {
            Err(CommentaryError::Generate(msg)) => assert!(msg.contains("mock failure")),
            _ => panic!("expected Generate error"),
        }
    }
}
