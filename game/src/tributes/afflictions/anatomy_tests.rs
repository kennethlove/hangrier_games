#[cfg(test)]
mod tests {
    use crate::tributes::afflictions::anatomy::{AcquireResolution, RejectReason, can_acquire};
    use proptest::prelude::*;
    use rstest::rstest;
    use shared::afflictions::{
        Affliction, AfflictionKey, AfflictionKind, AfflictionSource, BodyPart, Severity,
    };
    use std::collections::BTreeMap;

    fn affl(kind: AfflictionKind, part: Option<BodyPart>, sev: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: part,
            severity: sev,
            source: AfflictionSource::Spawn,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
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
            Just(AfflictionSource::Combat {
                attacker_id: String::new()
            }),
            Just(AfflictionSource::Environmental),
            Just(AfflictionSource::Cascade {
                from: (AfflictionKind::Wounded, None)
            }),
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
                    acquired_cycle: 0,
                    last_progressed_cycle: 0,
                    trauma_metadata: None,
                    phobia_metadata: None,
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
                acquired_cycle: 0,
                last_progressed_cycle: 0,
                trauma_metadata: None,
                phobia_metadata: None,
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
                    source: AfflictionSource::Combat { attacker_id: String::new() },
                    acquired_cycle: 0,
                    last_progressed_cycle: 0,
                    trauma_metadata: None,
                    phobia_metadata: None,
                };
                m.insert(a.key(), a);
            }

            let new = Affliction {
                kind: AfflictionKind::BrokenBone,
                body_part: Some(bp),
                severity: Severity::Moderate,
                source: AfflictionSource::Combat { attacker_id: String::new() },
                acquired_cycle: 0,
                last_progressed_cycle: 0,
                trauma_metadata: None,
                phobia_metadata: None,
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
                    source: AfflictionSource::Combat { attacker_id: String::new() },
                    acquired_cycle: 0,
                    last_progressed_cycle: 0,
                    trauma_metadata: None,
                    phobia_metadata: None,
                };
                m.insert(a.key(), a);
            }

            // Try Wounded
            let new_wounded = Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(bp),
                severity: Severity::Mild,
                source: AfflictionSource::Combat { attacker_id: String::new() },
                acquired_cycle: 0,
                last_progressed_cycle: 0,
                trauma_metadata: None,
                phobia_metadata: None,
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
                source: AfflictionSource::Combat { attacker_id: String::new() },
                acquired_cycle: 0,
                last_progressed_cycle: 0,
                trauma_metadata: None,
                phobia_metadata: None,
            };
            let result_infected = can_acquire(&m, &new_infected);
            // Infected also requires Wounded ancestor; either rejection reason is acceptable
            prop_assert!(matches!(result_infected, AcquireResolution::Reject(_)),
                "Infected on missing limb should be rejected, got {:?}", result_infected);
        }
    }
}
