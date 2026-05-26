//! Phobia fire and idle outcome handlers.
//!
//! Called by the scan loop when a phobia either fires (trigger present) or
//! sits idle (trigger absent). Pure reaction — no scan logic.
//!
//! See spec §7.

use shared::afflictions::{
    PhobiaMetadata, PhobiaOrigin, PhobiaTrigger, Severity, apply_traumatic_reinforcement,
    tick_decay,
};
use shared::messages::MessagePayload;

use super::triggers::PhobiaContext;

/// Called when a phobia fires this cycle.
///
/// Resets the decay counter. Rolls escalation for all phobias (~12% chance
/// per spec amendment). For Traumatic phobias, also rolls the original
/// reinforcement escalation (spec §7.1). For visible firings (Moderate+),
/// adds co-located tributes as observers.
pub(super) fn on_phobia_fire(
    meta: &mut PhobiaMetadata,
    cycle: u32,
    severity: &mut Severity,
    trigger: &PhobiaTrigger,
    tribute_id: &str,
    ctx: &PhobiaContext<'_>,
    rng: &mut impl rand::Rng,
) -> Vec<MessagePayload> {
    let mut messages = Vec::new();

    // Reset the decay counter.
    meta.cycles_since_last_fire = 0;

    // Clean stale observer entries (keep only those seen within 5 cycles).
    meta.observer_seen_cycle
        .retain(|_observer_id, last_seen| cycle.saturating_sub(*last_seen) <= 5);

    // Add observer if this is a visible firing (Moderate+) and there are
    // other tributes in the area who haven't already observed.
    if *severity > Severity::Mild {
        let trigger_str = trigger.to_string();
        for other in ctx.other_tributes_in_area {
            let other_id = &other.identifier;
            if other_id == tribute_id {
                continue;
            }
            if !meta.observed_by.contains(other_id) {
                meta.observed_by.insert(other_id.clone());
                messages.push(MessagePayload::PhobiaObserved {
                    observer: other_id.clone(),
                    subject: tribute_id.to_string(),
                    trigger: trigger_str.clone(),
                });
            }
            meta.observer_seen_cycle.insert(other_id.clone(), cycle);
        }
    }

    // Spec amendment: all phobia firings roll ~12% escalation chance.
    let outcome = apply_traumatic_reinforcement(*severity, 0.12, rng);
    if outcome.escalated {
        messages.push(MessagePayload::PhobiaEscalated {
            tribute: tribute_id.to_string(),
            trigger: trigger.to_string(),
            from_severity: severity.to_string(),
            to_severity: outcome.new_severity.to_string(),
        });
        *severity = outcome.new_severity;
    }

    messages
}

/// Called when a phobia does not fire this cycle.
///
/// Increments the decay counter. For Traumatic phobias at threshold (5 cycles),
/// decays one tier. Returns PhobiaHabituated message if decay occurred.
pub(super) fn on_phobia_idle(
    meta: &mut PhobiaMetadata,
    severity: &mut Severity,
    trigger: &PhobiaTrigger,
    tribute_id: &str,
) -> (Vec<MessagePayload>, bool) {
    meta.cycles_since_last_fire = meta.cycles_since_last_fire.saturating_add(1);
    let mut should_remove = false;
    let mut messages = Vec::new();

    // Traumatic habituation (spec §7.2).
    if matches!(meta.origin, PhobiaOrigin::Traumatic { .. }) {
        let outcome = tick_decay(*severity, meta.cycles_since_last_fire, 5);
        if outcome.decayed {
            if let Some(new_sev) = outcome.new_severity {
                messages.push(MessagePayload::PhobiaHabituated {
                    tribute: tribute_id.to_string(),
                    trigger: trigger.to_string(),
                    from_severity: severity.to_string(),
                    to_severity: Some(new_sev.to_string()),
                });
                *severity = new_sev;
            } else {
                // Cured — Mild decayed off.
                messages.push(MessagePayload::PhobiaHabituated {
                    tribute: tribute_id.to_string(),
                    trigger: trigger.to_string(),
                    from_severity: severity.to_string(),
                    to_severity: None,
                });
                should_remove = true;
            }
        }
    }

    (messages, should_remove)
}
