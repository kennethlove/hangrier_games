//! Trauma affliction acquisition and reinforcement logic (spec §4).
//!
//! This module provides:
//! - `Tribute::try_acquire_trauma` — single-instance acquisition/reinforcement (PR1)
//! - `process_traumas` — per-cycle flashback/observer/decay processing (PR3)
//!
//! Producer pipeline (PR2) calls `try_acquire_trauma` for each traumatic event.
//! PR3 wires the decay tick and flashback roll.
//!
//! Trauma is a special affliction that does not cascade or cure — it can only
//! be reinforced to higher severity or gradually reduced through decay.

use rand::RngExt;
use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, AfflictionSource, Severity, TraumaMetadata,
    TraumaSource,
};
use shared::messages::MessagePayload;

use crate::tributes::Tribute;

use super::effects::flashback_chance;

// ────────────────────────────────────────────────────────────────────────────
// Per-cycle processing (flashback roll, observer tracking, decay tick)
// ────────────────────────────────────────────────────────────────────────────

/// Result of per-cycle trauma processing for a single tribute.
#[derive(Debug, Clone, Default)]
pub struct TraumaCycleResult {
    /// Messages produced (flashbacks, observations, habituations).
    pub messages: Vec<MessagePayload>,
}

/// Process trauma afflictions for a tribute for one cycle.
///
/// For each trauma on the tribute:
/// 1. Roll flashback chance (0.05/0.10/0.20 per severity tier)
/// 2. If flashback hits at Moderate+: track observers
/// 3. Increment cycles_since_last_event (decay counter)
/// 4. If decay threshold (10 cycles) met: step severity down
///
/// Returns messages for events that occurred.
///
/// Gated on `config.trauma_enabled` — caller must check.
pub fn process_traumas(
    tribute: &mut Tribute,
    other_tributes_in_area: &[Tribute],
    cycle: u32,
    rng: &mut impl rand::Rng,
) -> TraumaCycleResult {
    let mut result = TraumaCycleResult::default();

    // Early exit: no trauma afflictions on this tribute.
    let has_trauma = tribute
        .afflictions
        .values()
        .any(|a| matches!(a.kind, AfflictionKind::Trauma));
    if !has_trauma {
        return result;
    }

    // Collect trauma keys first to avoid borrow conflicts when mutating.
    let trauma_keys: Vec<_> = tribute
        .afflictions
        .iter()
        .filter(|(_, a)| matches!(a.kind, AfflictionKind::Trauma))
        .map(|(key, _)| key.clone())
        .collect();

    for key in &trauma_keys {
        let Some(aff) = tribute.afflictions.get_mut(key) else {
            continue;
        };
        let Some(meta) = &mut aff.trauma_metadata else {
            continue;
        };

        // ── Flashback roll ────────────────────────────────────────
        let chance = flashback_chance(aff.severity);
        if rng.random_bool(chance) {
            let source_str = meta
                .sources
                .iter()
                .next()
                .map(|s| format_trauma_source(s))
                .unwrap_or_default();

            result.messages.push(MessagePayload::TraumaFlashback {
                tribute: tribute.identifier.clone(),
                severity: aff.severity.to_string(),
                source: source_str.clone(),
            });

            // Observer tracking for Moderate+ flashbacks.
            if aff.severity > Severity::Mild {
                for other in other_tributes_in_area {
                    let other_id = &other.identifier;
                    if other_id == &tribute.identifier {
                        continue;
                    }
                    if !meta.observed_by.contains(other_id) {
                        meta.observed_by.insert(other_id.clone());
                        result.messages.push(MessagePayload::TraumaObserved {
                            observer: other_id.clone(),
                            subject: tribute.identifier.clone(),
                            source: source_str.clone(),
                        });
                    }
                    meta.observer_seen_cycle.insert(other_id.clone(), cycle);
                }
            }
        }

        // ── Decay (increment idle counter) ────────────────────────
        meta.cycles_since_last_event = meta.cycles_since_last_event.saturating_add(1);

        // ── Check decay threshold (10 cycles) ─────────────────────
        if meta.cycles_since_last_event >= 10 {
            let outcome =
                shared::afflictions::tick_decay(aff.severity, meta.cycles_since_last_event, 10);
            if outcome.decayed {
                if let Some(new_sev) = outcome.new_severity {
                    result.messages.push(MessagePayload::TraumaHabituated {
                        tribute: tribute.identifier.clone(),
                        from_severity: aff.severity.to_string(),
                        to_severity: Some(new_sev.to_string()),
                    });
                    aff.severity = new_sev;
                    meta.cycles_since_last_event = 0;
                } else {
                    // Cured — Mild decayed off entirely.
                    result.messages.push(MessagePayload::TraumaHabituated {
                        tribute: tribute.identifier.clone(),
                        from_severity: aff.severity.to_string(),
                        to_severity: None,
                    });
                    tribute.afflictions.remove(key);
                }
            }
        }
    }

    result
}

/// Format a trauma source into a human-readable fragment for messages.
fn format_trauma_source(source: &TraumaSource) -> String {
    match source {
        TraumaSource::WitnessedAllyDeath { ally, .. } => {
            format!("witnessing the death of {ally}")
        }
        TraumaSource::NearDeath { .. } => "a near-death experience".to_string(),
        TraumaSource::Betrayal { by } => format!("betrayal by {by}"),
        TraumaSource::MassCasualty { .. } => "mass casualties".to_string(),
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Acquisition outcome (shared by both the PR1 entry-point and the PR3
// escalation helper; PR2's producer pipeline maps this to MessagePayload).
// ────────────────────────────────────────────────────────────────────────────

/// Outcome of `Tribute::try_acquire_trauma`. Distinguishes fresh acquisition
/// from reinforcement (counter reset) so the producer pipeline (PR2) can emit
/// the correct `MessagePayload::TraumaAcquired` vs `MessagePayload::TraumaReinforced`
/// later. PR1 returns the outcome but emits nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraumaAcquisition {
    /// First Trauma on this tribute. Severity = producer's first-occurrence severity.
    Acquired {
        severity: Severity,
        source: TraumaSource,
    },
    /// Existing Trauma was reinforced. Counter reset; source merged into the set;
    /// severity floor-bumped if producer severity exceeded current; metadata preserved.
    /// `floor_bumped` indicates whether the bump actually changed severity.
    Reinforced {
        from_severity: Severity,
        to_severity: Severity,
        floor_bumped: bool,
    },
}

// ────────────────────────────────────────────────────────────────────────────
// Tribute method — single-instance Trauma acquisition / reinforcement (spec §5.1)
// ────────────────────────────────────────────────────────────────────────────

impl Tribute {
    /// Acquire or reinforce the tribute's Trauma affliction (spec §5.1).
    ///
    /// `producer_severity` is the producer's first-occurrence severity from the
    /// table in spec §5: Mild for (a)/(b), Moderate for (c)/(d), Moderate/Severe
    /// for (f) depending on death count.
    ///
    /// Single-instance semantics: at most one Trauma per tribute. Subsequent
    /// calls reinforce the existing one (counter reset, source merge,
    /// severity floor bump if applicable). Escalation rolls (spec §6.1 step 4)
    /// happen elsewhere — PR3 will call `try_acquire_trauma` then run the
    /// shared reinforcement helper.
    pub fn try_acquire_trauma(
        &mut self,
        source: TraumaSource,
        producer_severity: Severity,
    ) -> TraumaAcquisition {
        let key: AfflictionKey = (AfflictionKind::Trauma, None);

        if let Some(existing) = self.afflictions.get_mut(&key) {
            // Reinforcement path: mutate existing affliction in place.
            let from_severity = existing.severity;
            let floor_bumped = producer_severity > from_severity;
            if floor_bumped {
                existing.severity = producer_severity;
                existing.last_progressed_cycle = self.game_day.unwrap_or(0) as u32;
            }

            let metadata = existing
                .trauma_metadata
                .get_or_insert_with(TraumaMetadata::default);
            metadata.sources.insert(source);
            metadata.cycles_since_last_event = 0;

            TraumaAcquisition::Reinforced {
                from_severity,
                to_severity: existing.severity,
                floor_bumped,
            }
        } else {
            // Fresh acquisition path: build the affliction directly so we can
            // attach trauma_metadata, then insert. We bypass try_acquire_affliction
            // here because that helper does not know how to construct trauma_metadata.
            let cycle = self.game_day.unwrap_or(0) as u32;
            let mut metadata = TraumaMetadata::default();
            metadata.sources.insert(source.clone());
            metadata.cycles_since_last_event = 0;

            let affliction = Affliction {
                kind: AfflictionKind::Trauma,
                body_part: None,
                severity: producer_severity,
                acquired_cycle: cycle,
                last_progressed_cycle: cycle,
                source: trauma_source_to_affliction_source(&source),
                trauma_metadata: Some(metadata),
                phobia_metadata: None,
                fixation_metadata: None,
            };
            self.afflictions.insert(key, affliction);

            TraumaAcquisition::Acquired {
                severity: producer_severity,
                source,
            }
        }
    }
}

/// Map a `TraumaSource` to the generic `AfflictionSource` field that lives on
/// every `Affliction`. The detailed trauma source list is preserved in
/// `TraumaMetadata.sources`; the generic source on the `Affliction` itself is
/// the coarsest-fit `AfflictionSource` variant for cascade tracking.
fn trauma_source_to_affliction_source(s: &TraumaSource) -> AfflictionSource {
    match s {
        // Trauma from witnessing/surviving combat keys to the canonical attacker
        // when one is identifiable; otherwise falls through to Cascade::Unknown.
        TraumaSource::Betrayal { by } => AfflictionSource::Combat {
            attacker_id: by.clone(),
        },
        TraumaSource::NearDeath {
            cause: shared::afflictions::DeathCause::Tribute(id),
        } => AfflictionSource::Combat {
            attacker_id: id.clone(),
        },
        // Witnessed deaths and mass casualties are environmental cascades from
        // the originating event chain. PR2's producer will populate the precise
        // event reference when it has access to the message stream; PR1 stamps
        // the coarsest-fit source so the wire field is well-formed.
        _ => AfflictionSource::Cascade {
            from: (AfflictionKind::Trauma, None),
        },
    }
}

#[cfg(test)]
#[path = "trauma_tests.rs"]
mod trauma_tests;
