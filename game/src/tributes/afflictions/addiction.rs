//! Addiction affliction acquisition and reinforcement logic (spec §4-5).
//!
//! This module provides:
//! - `try_acquire_addiction` — probabilistic acquisition/relapse/reinforcement
//! - `high_duration` — tolerance-driven High mode duration table
//! - `AddictionAcquisition` — result enum
//!
//! Called from the consumable-use hook (PR2). PR3 wires decay, observer tracking,
//! and the brain layer.

use rand::RngExt;
use shared::afflictions::{
    AddictionMetadata, AddictionResistReason, Affliction, AfflictionKind, AfflictionSource,
    Severity, Substance,
};

use crate::tributes::Tribute;
use std::collections::{BTreeMap, BTreeSet};

/// Number of active addictions allowed per tribute (spec §5.3).
pub const MAX_ACTIVE_ADDICTIONS: usize = 2;

/// Result of attempting to acquire or reinforce an addiction.
#[derive(Debug, Clone, PartialEq)]
pub enum AddictionAcquisition {
    /// New addiction acquired (probabilistic roll succeeded).
    Acquired {
        substance: Substance,
        use_count: u32,
    },
    /// Relapse — tribute was previously cured, auto-acquires at Mild.
    Relapse {
        substance: Substance,
        prior_uses: u32,
    },
    /// Existing addiction reinforced (used while already addicted).
    Reinforced {
        substance: Substance,
        severity: Severity,
        /// True if the severity escalated (12% sensitization roll).
        escalated: bool,
    },
    /// Acquisition prevented by cap (already at MAX_ACTIVE_ADDICTIONS).
    Resisted {
        substance: Substance,
        reason: AddictionResistReason,
    },
}

/// High duration (cycles) per (substance, severity) — spec §7.2 table.
///
/// As addiction worsens, tolerance shortens the High window.
pub fn high_duration(substance: Substance, severity: Severity) -> u32 {
    use Severity::*;
    match (substance, severity) {
        (Substance::Stimulant, Mild) => 2,
        (Substance::Stimulant, Moderate | Severe) => 1,
        (Substance::Painkiller, Mild) => 3,
        (Substance::Painkiller, Moderate) => 2,
        (Substance::Painkiller, Severe) => 1,
        (Substance::Morphling, Mild) => 4,
        (Substance::Morphling, Moderate) => 2,
        (Substance::Morphling, Severe) => 1,
        (Substance::Alcohol, _) => 1,
    }
}

/// Per-use acquisition probability base table — spec §5.2.
///
/// Returns the base chance (0.0–1.0) for a given lifetime use count.
fn acquisition_base_chance(use_count: u32) -> f64 {
    match use_count {
        1 => 0.05,
        2 => 0.15,
        3 => 0.30,
        4 => 0.50,
        _ => 0.75,
    }
}

/// Substance multiplier on acquisition probability — spec §5.2 table.
fn substance_multiplier(substance: Substance) -> f64 {
    match substance {
        Substance::Morphling => 1.5,
        Substance::Alcohol => 0.7,
        Substance::Stimulant | Substance::Painkiller => 1.0,
    }
}

/// Compute effective acquisition probability for a (use_count, substance) pair.
///
/// `p = min(0.95, base_chance × substance_multiplier)`
pub fn acquisition_probability(use_count: u32, substance: Substance) -> f64 {
    let base = acquisition_base_chance(use_count);
    let mult = substance_multiplier(substance);
    (base * mult).min(0.95)
}

/// Attempt to acquire or reinforce an addiction on a tribute (spec §5.1).
///
/// Called from `try_use_consumable` (PR2) after the substance's immediate effect
/// resolves and `addiction_use_count` is incremented.
///
/// Flow:
/// 1. If tribute already has Addiction(substance) → reinforce (§6.1)
/// 2. If tribute had it before (cured) → relapse (auto-acquire at Mild)
/// 3. If at MAX_ACTIVE_ADDICTIONS cap → resist
/// 4. Otherwise → probabilistic roll (§5.2)
impl Tribute {
    pub fn try_acquire_addiction(
        &mut self,
        substance: Substance,
        rng: &mut impl rand::Rng,
    ) -> AddictionAcquisition {
        let key = (AfflictionKind::Addiction(substance), None);

        // ── Step 1: Check existing addiction → reinforce ────────────
        if let Some(existing) = self.afflictions.get_mut(&key) {
            // Refresh metadata
            let meta = existing
                .addiction_metadata
                .get_or_insert_with(|| AddictionMetadata {
                    substance,
                    cycles_since_last_use: 0,
                    high_cycles_remaining: high_duration(substance, existing.severity),
                    use_count_at_acquisition: 0,
                    observed_by: BTreeSet::new(),
                    observer_seen_cycle: BTreeMap::new(),
                });
            meta.cycles_since_last_use = 0;
            meta.high_cycles_remaining = high_duration(substance, existing.severity);

            // Escalation roll (12% sensitization, spec §6.1 step 3)
            let escalated = rng.random_bool(0.12) && existing.severity < Severity::Severe;
            if escalated {
                existing.severity = existing.severity.next_tier();
                existing.last_progressed_cycle = self.game_day.unwrap_or(0) as u32;
            }

            return AddictionAcquisition::Reinforced {
                substance,
                severity: existing.severity,
                escalated,
            };
        }

        // ── Step 2: Relapse check (spec §5.1 step 5c) ───────────────
        // If `ever_addicted_to` contains the substance but no current addiction
        // exists, the tribute was cured. Next use auto-acquires at Mild.
        if self.ever_addicted_to.contains(&substance) && !self.has_addiction(substance) {
            let use_count = *self.addiction_use_count.get(&substance).unwrap_or(&0);
            let cycle = self.game_day.unwrap_or(0) as u32;
            let meta = AddictionMetadata {
                substance,
                cycles_since_last_use: 0,
                high_cycles_remaining: high_duration(substance, Severity::Mild),
                use_count_at_acquisition: use_count,
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
            };
            let aff = Affliction {
                kind: AfflictionKind::Addiction(substance),
                body_part: None,
                severity: Severity::Mild,
                source: AfflictionSource::Environmental,
                acquired_cycle: cycle,
                last_progressed_cycle: cycle,
                trauma_metadata: None,
                phobia_metadata: None,
                fixation_metadata: None,
                addiction_metadata: Some(meta),
            };
            self.ever_addicted_to.insert(substance);
            self.afflictions.insert(key, aff);
            return AddictionAcquisition::Relapse {
                substance,
                prior_uses: use_count,
            };
        }

        // ── Step 3: Count current active addictions ─────────────────
        let active_count: usize = self
            .afflictions
            .values()
            .filter(|a| matches!(a.kind, AfflictionKind::Addiction(_)))
            .count();

        // ── Step 4: Cap check (spec §5.3) ───────────────────────────
        if active_count >= MAX_ACTIVE_ADDICTIONS {
            return AddictionAcquisition::Resisted {
                substance,
                reason: AddictionResistReason::AtCap,
            };
        }

        // ── Step 5: Probabilistic acquisition (spec §5.2) ───────────
        let use_count = *self.addiction_use_count.get(&substance).unwrap_or(&0);
        if rng.random_bool(acquisition_probability(use_count, substance)) {
            let cycle = self.game_day.unwrap_or(0) as u32;
            let meta = AddictionMetadata {
                substance,
                cycles_since_last_use: 0,
                high_cycles_remaining: high_duration(substance, Severity::Mild),
                use_count_at_acquisition: use_count,
                observed_by: BTreeSet::new(),
                observer_seen_cycle: BTreeMap::new(),
            };
            let aff = Affliction {
                kind: AfflictionKind::Addiction(substance),
                body_part: None,
                severity: Severity::Mild,
                source: AfflictionSource::Environmental,
                acquired_cycle: cycle,
                last_progressed_cycle: cycle,
                trauma_metadata: None,
                phobia_metadata: None,
                fixation_metadata: None,
                addiction_metadata: Some(meta),
            };
            self.ever_addicted_to.insert(substance);
            self.afflictions.insert(key, aff);
            AddictionAcquisition::Acquired {
                substance,
                use_count,
            }
        } else {
            // Failed roll — no state change (silent per spec §5.2).
            AddictionAcquisition::Resisted {
                substance,
                reason: AddictionResistReason::AtCap,
            }
        }
    }

    /// Returns true if the affliction map contains Addiction for `substance`.
    fn has_addiction(&self, substance: Substance) -> bool {
        let key = (AfflictionKind::Addiction(substance), None);
        self.afflictions.contains_key(&key)
    }
}

#[cfg(test)]
#[path = "addiction_tests.rs"]
mod addiction_tests;
