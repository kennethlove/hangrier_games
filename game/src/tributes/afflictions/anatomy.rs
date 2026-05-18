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
    // Trauma is single-instance per tribute (spec §5.1). If a Trauma already
    // exists, signal Upgrade so the caller (try_acquire_trauma) can perform
    // the source-merge, severity-floor, and counter-reset in place. The new
    // affliction's severity is not compared here — even a "weaker" producer
    // event still counts as a fire that resets the counter.
    if new.kind == shared::afflictions::AfflictionKind::Trauma {
        let key = (shared::afflictions::AfflictionKind::Trauma, None);
        if existing.contains_key(&key) {
            return AcquireResolution::Upgrade(key);
        }
        return AcquireResolution::Insert;
    }

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
                .cloned()
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, Severity};

    fn affl(kind: AfflictionKind, part: Option<BodyPart>, sev: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: part,
            severity: sev,
            source: AfflictionSource::Spawn,
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
        }
    }

    fn map(items: Vec<Affliction>) -> BTreeMap<AfflictionKey, Affliction> {
        items.into_iter().map(|a| (a.key(), a)).collect()
    }

    #[test]
    fn empty_state_inserts_anything() {
        let r = can_acquire(
            &map(vec![]),
            &affl(AfflictionKind::Wounded, Some(BodyPart::Arm), Severity::Mild),
        );
        assert_eq!(r, AcquireResolution::Insert);
    }

    #[test]
    fn missing_arm_supersedes_wound_state_on_same_part() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Arm), Severity::Mild),
            affl(
                AfflictionKind::BrokenBone,
                Some(BodyPart::Arm),
                Severity::Moderate,
            ),
        ]);
        let new = affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        );
        match can_acquire(&existing, &new) {
            AcquireResolution::Supersede(keys) => {
                assert_eq!(keys.len(), 2);
                assert!(keys.contains(&(AfflictionKind::Wounded, Some(BodyPart::Arm))));
                assert!(keys.contains(&(AfflictionKind::BrokenBone, Some(BodyPart::Arm))));
            }
            other => panic!("expected Supersede, got {:?}", other),
        }
    }

    #[test]
    fn missing_arm_does_not_affect_other_parts() {
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Leg),
            Severity::Mild,
        )]);
        let new = affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        );
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn breaking_a_missing_bone_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(
            AfflictionKind::BrokenBone,
            Some(BodyPart::Arm),
            Severity::Mild,
        );
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[test]
    fn re_missing_already_missing_limb_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        );
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[test]
    fn infection_without_wound_ancestor_is_rejected() {
        let new = affl(
            AfflictionKind::Infected,
            Some(BodyPart::Arm),
            Severity::Mild,
        );
        assert_eq!(
            can_acquire(&map(vec![]), &new),
            AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor)
        );
    }

    #[test]
    fn infection_with_wound_ancestor_inserts() {
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(
            AfflictionKind::Infected,
            Some(BodyPart::Arm),
            Severity::Mild,
        );
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn same_key_higher_severity_upgrades() {
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Mild,
        )]);
        let new = affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Moderate,
        );
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Upgrade((AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
    }

    #[test]
    fn same_key_equal_severity_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Moderate,
        )]);
        let new = affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Moderate,
        );
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        );
    }

    #[test]
    fn same_key_lower_severity_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Arm), Severity::Mild);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        );
    }

    #[test]
    fn multiple_body_parts_dont_interfere() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Arm), Severity::Mild),
            affl(
                AfflictionKind::BrokenBone,
                Some(BodyPart::Leg),
                Severity::Moderate,
            ),
        ]);
        // Wounded on leg should insert independently
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Leg), Severity::Mild);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn missing_leg_supersedes_wound_state_on_leg() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Leg), Severity::Mild),
            affl(
                AfflictionKind::Infected,
                Some(BodyPart::Leg),
                Severity::Mild,
            ),
        ]);
        let new = affl(
            AfflictionKind::MissingLeg,
            Some(BodyPart::Leg),
            Severity::Severe,
        );
        match can_acquire(&existing, &new) {
            AcquireResolution::Supersede(keys) => {
                assert_eq!(keys.len(), 2);
                assert!(keys.contains(&(AfflictionKind::Wounded, Some(BodyPart::Leg))));
                assert!(keys.contains(&(AfflictionKind::Infected, Some(BodyPart::Leg))));
            }
            other => panic!("expected Supersede, got {:?}", other),
        }
    }

    #[test]
    fn wound_on_missing_leg_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::MissingLeg,
            Some(BodyPart::Leg),
            Severity::Severe,
        )]);
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Leg), Severity::Mild);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[test]
    fn infected_on_missing_arm_is_rejected() {
        let existing = map(vec![affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(
            AfflictionKind::Infected,
            Some(BodyPart::Arm),
            Severity::Mild,
        );
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[rstest]
    #[case(BodyPart::Arm)]
    #[case(BodyPart::Leg)]
    fn missing_limb_supersedes_on_each_limb_part(#[case] part: BodyPart) {
        let missing_kind = match part {
            BodyPart::Arm => AfflictionKind::MissingArm,
            BodyPart::Leg => AfflictionKind::MissingLeg,
            _ => panic!("non-limb part"),
        };
        let existing = map(vec![affl(
            AfflictionKind::Wounded,
            Some(part),
            Severity::Mild,
        )]);
        let new = affl(missing_kind, Some(part), Severity::Severe);
        match can_acquire(&existing, &new) {
            AcquireResolution::Supersede(keys) => {
                assert!(keys.contains(&(AfflictionKind::Wounded, Some(part))));
            }
            other => panic!("expected Supersede for {:?}, got {:?}", part, other),
        }
    }

    #[rstest]
    #[case(BodyPart::Arm)]
    #[case(BodyPart::Leg)]
    fn breaking_missing_limb_rejected(#[case] part: BodyPart) {
        let missing_kind = match part {
            BodyPart::Arm => AfflictionKind::MissingArm,
            BodyPart::Leg => AfflictionKind::MissingLeg,
            _ => panic!("non-limb part"),
        };
        let existing = map(vec![affl(missing_kind, Some(part), Severity::Severe)]);
        let new = affl(AfflictionKind::BrokenBone, Some(part), Severity::Mild);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[test]
    fn blind_is_unique_slot() {
        let existing = map(vec![affl(
            AfflictionKind::Blind,
            Some(BodyPart::Eye),
            Severity::Severe,
        )]);
        // Same key collision → equal severity → reject
        let new = affl(AfflictionKind::Blind, Some(BodyPart::Eye), Severity::Severe);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        );
    }

    #[test]
    fn non_limb_parts_not_affected_by_missing_limb() {
        // MissingArm should not block Wounded on Torso
        let existing = map(vec![affl(
            AfflictionKind::MissingArm,
            Some(BodyPart::Arm),
            Severity::Severe,
        )]);
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Rib), Severity::Mild);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn body_part_none_afflictions_work() {
        // Afflictions without body parts (Starving, Poisoned, etc.) should insert freely
        let existing = map(vec![affl(AfflictionKind::Starving, None, Severity::Mild)]);
        let new = affl(AfflictionKind::Poisoned, None, Severity::Mild);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn body_part_none_same_kind_upgrades() {
        let existing = map(vec![affl(AfflictionKind::Starving, None, Severity::Mild)]);
        let new = affl(AfflictionKind::Starving, None, Severity::Moderate);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Upgrade((AfflictionKind::Starving, None))
        );
    }

    // ── Proptest properties ──────────────────────────────────────────────

    use proptest::prelude::*;

    fn arb_kind() -> impl Strategy<Value = AfflictionKind> {
        prop_oneof![
            Just(AfflictionKind::Wounded),
            Just(AfflictionKind::Infected),
            Just(AfflictionKind::MissingArm),
            Just(AfflictionKind::MissingLeg),
            Just(AfflictionKind::Blind),
            Just(AfflictionKind::Deaf),
            Just(AfflictionKind::BrokenBone),
            Just(AfflictionKind::Poisoned),
            Just(AfflictionKind::Starving),
            Just(AfflictionKind::Dehydrated),
            Just(AfflictionKind::Frozen),
            Just(AfflictionKind::Overheated),
            Just(AfflictionKind::Burned),
        ]
    }

    fn arb_body_part() -> impl Strategy<Value = Option<BodyPart>> {
        prop_oneof![
            Just(None),
            Just(Some(BodyPart::Arm)),
            Just(Some(BodyPart::Leg)),
            Just(Some(BodyPart::Eye)),
            Just(Some(BodyPart::Ear)),
            Just(Some(BodyPart::Skull)),
            Just(Some(BodyPart::Rib)),
            Just(Some(BodyPart::Hand)),
            Just(Some(BodyPart::Foot)),
        ]
    }

    fn arb_severity() -> impl Strategy<Value = Severity> {
        prop_oneof![
            Just(Severity::Mild),
            Just(Severity::Moderate),
            Just(Severity::Severe),
        ]
    }

    fn arb_source() -> impl Strategy<Value = AfflictionSource> {
        prop_oneof![
            Just(AfflictionSource::Spawn),
            ("[a-z]{4}").prop_map(|id| AfflictionSource::Combat {
                attacker_id: format!("tributes:{}", id)
            }),
            Just(AfflictionSource::Environmental),
            (arb_kind(), arb_body_part())
                .prop_map(|(k, bp)| AfflictionSource::Cascade { from: (k, bp) }),
            Just(AfflictionSource::Sponsor),
            Just(AfflictionSource::Gamemaker),
        ]
    }

    fn arb_limb() -> impl Strategy<Value = BodyPart> {
        prop_oneof![Just(BodyPart::Arm), Just(BodyPart::Leg)]
    }

    fn arb_existing() -> impl Strategy<Value = BTreeMap<AfflictionKey, Affliction>> {
        proptest::collection::vec(
            (arb_kind(), arb_body_part(), arb_severity(), arb_source()),
            0..=5,
        )
        .prop_map(|items| {
            let mut m = BTreeMap::new();
            for (kind, body_part, severity, source) in items {
                let a = Affliction {
                    kind,
                    body_part,
                    severity,
                    source,
                    acquired_cycle: 1,
                    last_progressed_cycle: 1,
                    trauma_metadata: None,
                };
                m.insert(a.key(), a);
            }
            m
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn prop_can_acquire_deterministic(
            existing in arb_existing(),
            kind in arb_kind(),
            body_part in arb_body_part(),
            severity in arb_severity(),
        ) {
            let new = Affliction {
                kind,
                body_part,
                severity,
                source: AfflictionSource::Environmental,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            };
            let result1 = can_acquire(&existing, &new);
            let result2 = can_acquire(&existing, &new);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn prop_missing_limb_and_broken_never_coexist(
            existing in arb_existing(),
            bp in arb_limb(),
        ) {
            let missing_kind = match bp {
                BodyPart::Arm => AfflictionKind::MissingArm,
                BodyPart::Leg => AfflictionKind::MissingLeg,
                _ => unreachable!("arb_limb only yields Arm or Leg"),
            };

            let mut m = existing.clone();
            let key = (missing_kind, Some(bp));
            if !m.contains_key(&key) {
                let a = Affliction {
                    kind: missing_kind,
                    body_part: Some(bp),
                    severity: Severity::Severe,
                    source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                    acquired_cycle: 1,
                    last_progressed_cycle: 1,
                    trauma_metadata: None,
                };
                m.insert(a.key(), a);
            }

            let new = Affliction {
                kind: AfflictionKind::BrokenBone,
                body_part: Some(bp),
                severity: Severity::Moderate,
                source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            };
            let result = can_acquire(&m, &new);

            if let AcquireResolution::Reject(reason) = result {
                prop_assert_eq!(reason, RejectReason::LimbAlreadyMissing);
            } else {
                prop_assert!(false, "BrokenBone on missing limb should be rejected, got {:?}", result);
            }
        }

        #[test]
        fn prop_missing_limb_and_wound_states_never_coexist(
            existing in arb_existing(),
            bp in arb_limb(),
        ) {
            let missing_kind = match bp {
                BodyPart::Arm => AfflictionKind::MissingArm,
                BodyPart::Leg => AfflictionKind::MissingLeg,
                _ => unreachable!("arb_limb only yields Arm or Leg"),
            };

            let mut m = existing.clone();
            let key = (missing_kind, Some(bp));
            if !m.contains_key(&key) {
                let a = Affliction {
                    kind: missing_kind,
                    body_part: Some(bp),
                    severity: Severity::Severe,
                    source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                    acquired_cycle: 1,
                    last_progressed_cycle: 1,
                    trauma_metadata: None,
                };
                m.insert(a.key(), a);
            }

            // Try Wounded
            let new_wounded = Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(bp),
                severity: Severity::Mild,
                source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            };
            let result_wounded = can_acquire(&m, &new_wounded);
            if let AcquireResolution::Reject(reason) = result_wounded {
                prop_assert_eq!(reason, RejectReason::LimbAlreadyMissing);
            } else {
                prop_assert!(false, "Wounded on missing limb should be rejected, got {:?}", result_wounded);
            }

            // Try Infected
            let new_infected = Affliction {
                kind: AfflictionKind::Infected,
                body_part: Some(bp),
                severity: Severity::Moderate,
                source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            };
            let result_infected = can_acquire(&m, &new_infected);
            // Infected also requires Wounded ancestor; either rejection reason is acceptable
            prop_assert!(matches!(result_infected, AcquireResolution::Reject(_)),
                "Infected on missing limb should be rejected, got {:?}", result_infected);
        }
    }

    #[cfg(test)]
    mod trauma_resolution_tests {
        use super::*;
        use shared::afflictions::*;
        use std::collections::BTreeMap;

        fn trauma(severity: Severity) -> Affliction {
            Affliction {
                kind: AfflictionKind::Trauma,
                body_part: None,
                severity,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                source: AfflictionSource::Spawn,
                trauma_metadata: Some(TraumaMetadata::default()),
            }
        }

        #[test]
        fn trauma_into_empty_inserts() {
            let existing: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
            let new = trauma(Severity::Mild);
            let res = can_acquire(&existing, &new);
            assert_eq!(res, AcquireResolution::Insert);
        }

        #[test]
        fn trauma_with_existing_returns_upgrade_pointing_at_existing() {
            let mut existing = BTreeMap::new();
            let cur = trauma(Severity::Mild);
            existing.insert(cur.key(), cur);
            let new = trauma(Severity::Moderate);
            let res = can_acquire(&existing, &new);
            assert_eq!(
                res,
                AcquireResolution::Upgrade((AfflictionKind::Trauma, None))
            );
        }

        #[test]
        fn trauma_with_existing_upgrade_does_not_check_severity() {
            let mut existing = BTreeMap::new();
            existing.insert(trauma(Severity::Severe).key(), trauma(Severity::Severe));
            let new = trauma(Severity::Mild);
            let res = can_acquire(&existing, &new);
            assert_eq!(
                res,
                AcquireResolution::Upgrade((AfflictionKind::Trauma, None))
            );
        }

        #[test]
        fn trauma_does_not_collide_with_non_trauma() {
            let mut existing = BTreeMap::new();
            let wounded = Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(BodyPart::Arm),
                severity: Severity::Mild,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                source: AfflictionSource::Spawn,
                trauma_metadata: None,
            };
            existing.insert(wounded.key(), wounded);
            let new = trauma(Severity::Mild);
            let res = can_acquire(&existing, &new);
            assert_eq!(res, AcquireResolution::Insert);
        }
    }
}
