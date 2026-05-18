//! Trauma affliction acquisition and reinforcement logic (spec §4, PR2).
//!
//! Trauma is a special affliction that can be acquired from witnessing traumatic
//! events and reinforced by subsequent events. Unlike regular afflictions, trauma
//! does not cascade or cure — it can only be reinforced to higher severity or
//! gradually reduced through shelter recovery.

use shared::afflictions::{Severity, TraumaSource};

/// Result of attempting to acquire or reinforce trauma.
#[derive(Debug, Clone, PartialEq)]
pub enum TraumaAcquisition {
    /// New trauma acquired at the given severity.
    Acquired {
        severity: Severity,
        source: TraumaSource,
    },
    /// Existing trauma reinforced to a higher severity.
    Reinforced {
        from_severity: Severity,
        to_severity: Severity,
        /// True if the severity was bumped up from a floor (e.g. Mild → Moderate).
        floor_bumped: bool,
    },
}
