//! Anatomy resolution: how a new affliction interacts with existing slots.
//!
//! See spec §4 (full table) and §17 (testing strategy).

use shared::afflictions::{Affliction, AfflictionKey, AfflictionKind, BodyPart};
use std::collections::BTreeMap;

/// Outcome of attempting to acquire an affliction given the current tribute state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquireResolution {
    /// No conflict; insert the new affliction.
    Insert,
    /// Replace an existing slot at the same key with the new (higher) severity.
    Upgrade(AfflictionKey),
    /// Remove subordinate afflictions; insert the new one. Used when
    /// `MissingArm`/`MissingLeg` arrives and supersedes wound state on that limb.
    Supersede(Vec<AfflictionKey>),
    /// Acquisition is nonsensical (e.g. break a missing bone).
    Reject(RejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Body part is already missing; the new affliction can't apply.
    LimbAlreadyMissing,
    /// `Infected` requires a `Wounded` ancestor on the same part (no random
    /// whole-body infection in v1; only via cascade).
    InfectedRequiresWoundedAncestor,
    /// New severity is not strictly greater than existing same-key severity.
    NotStrictlyHigherSeverity,
}

/// Decide what happens when `new` is offered to a tribute who already carries
/// `existing` afflictions. Pure function; no mutation. Spec §4.
pub fn can_acquire(
    existing: &BTreeMap<AfflictionKey, Affliction>,
    new: &Affliction,
) -> AcquireResolution {
    let new_key = new.key();

    // Rule: MissingArm/MissingLeg on a part supersedes ALL wound-state slots
    // on that part and rejects subsequent same-part Broken/Wounded/Infected.
    if let Some(part) = new.body_part {
        // 1. Reject if same part is already missing and new kind is wound-state.
        let limb_already_missing = is_limb_missing(existing, part);
        if limb_already_missing
            && matches!(
                new.kind,
                AfflictionKind::BrokenBone | AfflictionKind::Wounded | AfflictionKind::Infected
            )
        {
            return AcquireResolution::Reject(RejectReason::LimbAlreadyMissing);
        }

        // 2. Reject if trying to re-miss an already-missing limb.
        if is_missing_kind(new.kind) && existing.contains_key(&(new.kind, Some(part))) {
            return AcquireResolution::Reject(RejectReason::LimbAlreadyMissing);
        }

        // 3. MissingArm/MissingLeg supersedes wound-state on the same part.
        if is_missing_kind(new.kind) {
            let supersede: Vec<AfflictionKey> = existing
                .keys()
                .filter(|(k, p)| {
                    p == &Some(part)
                        && matches!(
                            k,
                            AfflictionKind::BrokenBone
                                | AfflictionKind::Wounded
                                | AfflictionKind::Infected
                        )
                })
                .copied()
                .collect();
            if !supersede.is_empty() {
                return AcquireResolution::Supersede(supersede);
            }
            return AcquireResolution::Insert;
        }
    }

    // Rule: Infected requires Wounded ancestor on the same part.
    if new.kind == AfflictionKind::Infected
        && !existing.contains_key(&(AfflictionKind::Wounded, new.body_part))
    {
        return AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor);
    }

    // Rule: Same-key collision → upgrade if strictly higher severity.
    if let Some(prev) = existing.get(&new_key) {
        return if new.severity > prev.severity {
            AcquireResolution::Upgrade(new_key)
        } else {
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        };
    }

    // Rule: Blind/Deaf are unique (single slot regardless of body_part).
    // Caller is expected to pass body_part = Some(Eye) / Some(Ear) for these,
    // so the same-key collision rule above already handles uniqueness.

    AcquireResolution::Insert
}

/// Check if the given body part is already missing.
fn is_limb_missing(existing: &BTreeMap<AfflictionKey, Affliction>, part: BodyPart) -> bool {
    let missing_kind = match part {
        BodyPart::Arm => AfflictionKind::MissingArm,
        BodyPart::Leg => AfflictionKind::MissingLeg,
        _ => return false,
    };
    existing.contains_key(&(missing_kind, Some(part)))
}

/// Check if an affliction kind represents a missing limb.
fn is_missing_kind(kind: AfflictionKind) -> bool {
    matches!(
        kind,
        AfflictionKind::MissingArm | AfflictionKind::MissingLeg
    )
}
