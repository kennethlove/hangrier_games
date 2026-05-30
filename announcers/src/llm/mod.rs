//! LLM abstraction layer.
//!
//! The [`Commentator`] trait decouples commentary generation from any
//! specific LLM backend. The default implementation uses Ollama (behind
//! `features = ["ollama"]`).

use async_trait::async_trait;
use crate::types::{BroadcastPackage, CommentaryError, CommentarySegment};

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
}

#[cfg(feature = "ollama")]
pub mod ollama;
