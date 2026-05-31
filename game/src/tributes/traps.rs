//! PlacedTrap state — traps set by tributes via Action::SetTrap.
//!
//! See `docs/superpowers/specs/2026-05-29-trap-expansion-design.md`.

use serde::{Deserialize, Serialize};
use shared::afflictions::{Severity, TrapKind};

/// A trap placed by a tribute in a specific area.
/// Lives on `AreaDetails::placed_traps`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedTrap {
    /// Unique identifier for this trap instance.
    pub id: String,
    /// Trap kind determines what happens on trigger.
    pub kind: TrapKind,
    /// Severity scales damage and escape difficulty.
    pub severity: Severity,
    /// ID of the tribute who set this trap.
    pub set_by: String,
    /// Concealment DC — Perception check threshold to spot before triggering.
    pub concealment: u32,
    /// Whether this trap has already been triggered.
    #[serde(default)]
    pub triggered: bool,
}
