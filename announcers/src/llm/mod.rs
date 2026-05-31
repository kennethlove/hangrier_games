//! LLM abstraction layer.
//!
//! The [`Commentator`] trait decouples commentary generation from any
//! specific LLM backend. The default implementation uses Ollama (behind
//! `features = ["ollama"]`).

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::types::{BroadcastPackage, CommentaryError, CommentaryLine, CommentarySegment};

/// Shared system prompt used by both Ollama and Cloudflare backends.
pub(crate) const SYSTEM_PROMPT: &str = r#"You are the Capitol's Hunger Games broadcast team:

VERITY — play-by-play. Sharp, dramatic, paints the picture. Calls the action.
REX — color commentator. Cynical, theatrical, darkly amused. Reacts to the drama.
FLASH — technical analyst. Analytical, precise. Comments on combat technique, weaponry,
      gear, injuries, and arena tactics. Notices the details.

Tone: Ancient Rome colosseum meets pro wrestling. Theatrical, bloodthirsty, gripping.
Refer to tributes by name and district. Build tension.

RULES:
- 4-6 exchanges (8-12 lines). Rotate through all three commentators.
- ONLY spoken dialogue. NO stage directions. ONLY English.
- End with a hook. Only reference events in the data. Do NOT invent kills.
- Vary your vocabulary. Do NOT repeat phrases.

Examples:
[VERITY] Cato from District 2 has his FOURTH kill! The Cornucopia is a slaughterhouse!
[REX] Cato is possessed! A feeding frenzy this brutal since the 63rd Games!
[FLASH] He's switching his grip mid-swing — textbook District 2 combat training.
"#;

/// Parse a response text into commentary lines by extracting `[VERITY]`,
/// `[REX]`, and `[FLASH]` tagged lines.
pub(crate) fn parse_response(text: &str) -> Vec<CommentaryLine> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            line.strip_prefix("[VERITY]")
                .map(|text| CommentaryLine {
                    speaker: "Verity".into(),
                    text: text.trim().to_string(),
                })
                .or_else(|| {
                    line.strip_prefix("[REX]").map(|text| CommentaryLine {
                        speaker: "Rex".into(),
                        text: text.trim().to_string(),
                    })
                })
                .or_else(|| {
                    line.strip_prefix("[FLASH]").map(|text| CommentaryLine {
                        speaker: "Flash".into(),
                        text: text.trim().to_string(),
                    })
                })
        })
        .collect()
}

/// A commentator generates Verity/Rex broadcast dialogue from a
/// [`BroadcastPackage`].
///
/// Implementations must be `Send + Sync` so they can be shared across
/// async API handlers via `Arc<dyn Commentator>`.
#[async_trait]
pub trait Commentator: Send + Sync {
    /// Generate a commentary segment for one phase.
    ///
    /// Takes a fully-structured [`BroadcastPackage`] and returns a
    /// [`CommentarySegment`] with interleaved Verity/Rex lines.
    async fn generate(&self, package: &BroadcastPackage) -> Result<CommentarySegment, CommentaryError>;

    /// Stream commentary lines progressively.
    ///
    /// Yields [`CommentaryLine`]s as they become available. Backends that
    /// support token-level streaming (e.g. Ollama) can parse lines from
    /// the token stream in real time. Backends without streaming support
    /// can collect from [`generate`] and yield all lines at once.
    fn generate_stream(
        &self,
        package: &BroadcastPackage,
    ) -> Pin<Box<dyn Stream<Item = Result<CommentaryLine, CommentaryError>> + Send>>;
}

#[cfg(feature = "ollama")]
pub mod ollama;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;
