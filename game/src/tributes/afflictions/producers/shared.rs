//! Shared types and helpers for the trauma producer pipeline.
//!
//! Provides internal data structures (`TraumaEvent`, `TraumaMessage`),
//! the applying/forwarding helpers, and the phobia co-acquire stub.

use crate::games::Game;
use crate::messages::{MessagePayload, MessageSource, TributeRef};
use crate::tributes::Tribute;
use crate::tributes::afflictions::TraumaAcquisition;
use shared::afflictions::{DeathCause, Severity, TraumaSource};

/// Collected trauma event before application (avoids borrow conflicts).
pub(super) struct TraumaEvent {
    pub(super) tribute_id: String,
    pub(super) tribute_name: String,
    pub(super) source: TraumaSource,
    pub(super) severity: Severity,
    /// Death cause for phobia co-acquire stub.
    pub(super) cause_hint: DeathCause,
}

/// Message data collected during trauma application, pushed afterwards.
struct TraumaMessage {
    tribute_id: String,
    tribute_name: String,
    acquisition: TraumaAcquisition,
}

/// Apply collected trauma events, emitting appropriate messages.
/// When `with_phobia_stub` is true, calls the phobia co-acquire stub after
/// each acquisition.
pub(super) fn apply_trauma_events(
    game: &mut Game,
    events: Vec<TraumaEvent>,
    with_phobia_stub: bool,
) {
    let mut messages: Vec<TraumaMessage> = Vec::new();

    for event in events {
        let Some(tribute) = game
            .tributes
            .iter_mut()
            .find(|t| t.identifier == event.tribute_id)
        else {
            continue;
        };

        let acquisition = tribute.try_acquire_trauma(event.source, event.severity);
        messages.push(TraumaMessage {
            tribute_id: event.tribute_id.clone(),
            tribute_name: event.tribute_name.clone(),
            acquisition,
        });

        if with_phobia_stub {
            try_co_acquire_phobia(tribute, &event.cause_hint);
        }
    }

    // Push messages after mutable borrows end
    for msg in messages {
        push_trauma_message(game, &msg.tribute_name, &msg.tribute_id, &msg.acquisition);
    }
}

/// Push a `TraumaAcquired` or `TraumaReinforced` message onto the game log.
fn push_trauma_message(
    game: &mut Game,
    tribute_name: &str,
    tribute_id: &str,
    acquisition: &TraumaAcquisition,
) {
    let payload = match acquisition {
        TraumaAcquisition::Acquired { severity, source } => MessagePayload::TraumaAcquired {
            tribute: tribute_id.to_string(),
            severity: severity.to_string(),
            source: format!("{source:?}"),
        },
        TraumaAcquisition::Reinforced {
            from_severity,
            to_severity,
            floor_bumped,
        } => MessagePayload::TraumaReinforced {
            tribute: tribute_id.to_string(),
            from_severity: from_severity.to_string(),
            to_severity: to_severity.to_string(),
            floor_bumped: *floor_bumped,
        },
    };

    let tick = game.tick_counter.next();
    game.push_message(
        MessageSource::Tribute(tribute_id.to_string()),
        format!("tribute:{tribute_id}"),
        format_trauma_content(tribute_name, acquisition),
        payload,
        tick,
    );
}

/// Format human-readable content for a trauma message.
fn format_trauma_content(name: &str, acquisition: &TraumaAcquisition) -> String {
    match acquisition {
        TraumaAcquisition::Acquired { severity, .. } => {
            format!("{name} acquires trauma ({severity}).")
        }
        TraumaAcquisition::Reinforced {
            from_severity,
            to_severity,
            floor_bumped,
        } => {
            if *floor_bumped {
                format!("{name}'s trauma reinforced: {from_severity} → {to_severity}.")
            } else {
                format!("{name}'s trauma reinforced ({to_severity}).")
            }
        }
    }
}

/// Map a killer reference and cause to a `DeathCause`. If a killer is present,
/// their tribute identity takes priority; otherwise the original cause is returned.
pub(super) fn map_cause_to_death_cause(
    killer: Option<&TributeRef>,
    cause: &DeathCause,
) -> DeathCause {
    if let Some(k) = killer {
        return DeathCause::Tribute(k.identifier.to_string());
    }
    cause.clone()
}

/// Stub for phobia co-acquisition. No-op until phobia PR lands.
///
/// TODO(phobia-pr1): wire to `try_acquire_phobia` when the phobia affliction
/// system is implemented.
#[allow(dead_code)]
fn try_co_acquire_phobia(_tribute: &mut Tribute, _cause: &DeathCause) {
    // No-op stub. Will trigger phobia acquisition once the phobia system exists.
}
