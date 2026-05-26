//! Trauma affliction acquisition and reinforcement logic (spec §4, PR2).
//!
//! Trauma is a special affliction that can be acquired from witnessing traumatic
//! events and reinforced by subsequent events. Unlike regular afflictions, trauma
//! does not cascade or cure — it can only be reinforced to higher severity or
//! gradually reduced through shelter recovery.

use rand::RngExt;
use shared::afflictions::{AfflictionKind, Severity, TraumaSource};
use shared::messages::MessagePayload;

use crate::tributes::Tribute;

use super::effects::flashback_chance;

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
/// 3. Increment cycles_since_last_fire (decay counter)
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
            let source_str = format_trauma_source(&meta.source);

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
        meta.cycles_since_last_fire = meta.cycles_since_last_fire.saturating_add(1);

        // ── Check decay threshold (10 cycles) ─────────────────────
        if meta.cycles_since_last_fire >= 10 {
            let outcome =
                shared::afflictions::tick_decay(aff.severity, meta.cycles_since_last_fire, 10);
            if outcome.decayed {
                if let Some(new_sev) = outcome.new_severity {
                    result.messages.push(MessagePayload::TraumaHabituated {
                        tribute: tribute.identifier.clone(),
                        from_severity: aff.severity.to_string(),
                        to_severity: Some(new_sev.to_string()),
                    });
                    aff.severity = new_sev;
                    meta.cycles_since_last_fire = 0;
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
