use super::severity::Severity;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Sub-discriminator for `AfflictionKind::Trapped(TrapKind)`.
///
/// Initial v1 ships Drowning and Buried only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapKind {
    /// Washed ashore after flood/rapids. No escape roll — forced relocation,
    /// disorientation 1-2 cycles, stamina/sanity penalty, some HP loss.
    Drowning,
    /// Trapped under cave-in/debris. Escape via cumulative dig-out progress
    /// rolls. Progressive HP loss while trapped. Others can assist (PR2).
    Buried,
}

impl fmt::Display for TrapKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrapKind::Drowning => write!(f, "drowning"),
            TrapKind::Buried => write!(f, "buried"),
        }
    }
}

/// Runtime state for a Trapped affliction. Lives on `Affliction.trapped_metadata`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrappedMetadata {
    /// Cycles spent trapped. Drives escape-roll decay for Buried,
    /// disorientation recovery for Drowning.
    pub cycles_trapped: u8,
    /// Partial rescue accumulator. Only meaningful at Severe for Buried.
    /// Each single-rescuer cycle adds 1; reaches escape threshold at 2.
    pub escape_progress: u8,
    /// Cached terrain hazard floor for the area at acquisition time.
    /// Caps escape roll regardless of stat/rescue bonuses.
    /// `None` means no floor applies. Drowning always has floor (rapids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain_hazard_floor: Option<f32>,
    /// For Drowning: cycles remaining in disoriented state.
    /// Starts at 1-2, decrements each cycle. While > 0, stat penalty applies.
    /// Buried always has this as 0.
    #[serde(default)]
    pub disorientation_remaining: u8,
}

impl TrappedMetadata {
    pub fn fresh_for(kind: TrapKind, terrain_hazard_floor: Option<f32>) -> Self {
        let disorientation_remaining = match kind {
            TrapKind::Drowning => 2,
            TrapKind::Buried => 0,
        };
        Self {
            cycles_trapped: 0,
            escape_progress: 0,
            terrain_hazard_floor,
            disorientation_remaining,
        }
    }
}

/// Escape-roll severity bases. Used for Buried escape target computation.
/// Mild is most escapable; Severe is hardest.
pub const SEVERITY_BASE_MILD: f32 = 0.50;
pub const SEVERITY_BASE_MODERATE: f32 = 0.35;
pub const SEVERITY_BASE_SEVERE: f32 = 0.15;

/// Maximum bonus from a maxed-out escape stat.
pub const ESCAPE_STAT_BONUS_MAX: f32 = 0.30;

/// Per-cycle decay applied to the escape roll (longer stuck = harder escape).
pub const CYCLES_DECAY_PER_CYCLE: f32 = 0.02;

/// Hard cap on escape probability — never a guaranteed escape.
pub const ESCAPE_ROLL_CAP: f32 = 0.95;

/// Threshold for partial rescue at Severe — a single rescuer must contribute
/// this many cycles before their bonus applies.
pub const PARTIAL_RESCUE_THRESHOLD: u8 = 2;

/// Cap on total rescue contribution per cycle (prevents 4 rescuers from trivializing).
pub const RESCUE_BONUS_CAP: f32 = 0.80;

// Escape progress needed to free self from Buried, indexed by severity.
// Retained as doc reference — single success escapes now (threshold always 1).
// const ESCAPE_THRESHOLD_MILD: u8 = 3;
// const ESCAPE_THRESHOLD_MODERATE: u8 = 5;
// const ESCAPE_THRESHOLD_SEVERE: u8 = 8;

/// Returns the escape progress threshold for a given severity.
/// Always 1 — single success escapes.
pub fn escape_threshold(_severity: Severity) -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trap_kind_serializes_snake_case() {
        let drowning = serde_json::to_string(&TrapKind::Drowning).unwrap();
        assert_eq!(drowning, "\"drowning\"");
        let buried = serde_json::to_string(&TrapKind::Buried).unwrap();
        assert_eq!(buried, "\"buried\"");
    }

    #[test]
    fn trapped_metadata_fresh_for_drowning_has_disorientation() {
        let m = TrappedMetadata::fresh_for(TrapKind::Drowning, None);
        assert_eq!(m.cycles_trapped, 0);
        assert_eq!(m.escape_progress, 0);
        assert_eq!(m.disorientation_remaining, 2);
        assert_eq!(m.terrain_hazard_floor, None);
    }

    #[test]
    fn trapped_metadata_fresh_for_buried_no_disorientation() {
        let m = TrappedMetadata::fresh_for(TrapKind::Buried, Some(0.30));
        assert_eq!(m.disorientation_remaining, 0);
        assert_eq!(m.terrain_hazard_floor, Some(0.30));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn severity_bases_ordered() {
        assert!(SEVERITY_BASE_SEVERE < SEVERITY_BASE_MODERATE);
        assert!(SEVERITY_BASE_MODERATE < SEVERITY_BASE_MILD);
    }

    #[test]
    fn escape_threshold_is_one_for_all_severities() {
        assert_eq!(escape_threshold(Severity::Mild), 1);
        assert_eq!(escape_threshold(Severity::Moderate), 1);
        assert_eq!(escape_threshold(Severity::Severe), 1);
    }
}
