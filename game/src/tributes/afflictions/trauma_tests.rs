#[cfg(test)]
mod tests {
    use crate::tributes::AfflictionDraft;
    use crate::tributes::Tribute;
    use crate::tributes::afflictions::trauma::TraumaAcquisition;
    use shared::afflictions::*;

    fn fresh_tribute() -> Tribute {
        Tribute {
            identifier: "tributes:test".into(),
            game_day: Some(5),
            ..Default::default()
        }
    }

    fn first_trauma_severity_floor(producer_severity: Severity) -> Tribute {
        let mut t = fresh_tribute();
        let outcome = t.try_acquire_trauma(
            TraumaSource::WitnessedAllyDeath {
                ally: "tributes:glimmer".into(),
                cause: Some(DeathCause::Fire),
            },
            producer_severity,
        );
        assert!(matches!(outcome, TraumaAcquisition::Acquired { .. }));
        t
    }

    #[test]
    fn first_acquisition_creates_trauma_at_producer_severity() {
        let t = first_trauma_severity_floor(Severity::Moderate);
        let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
        assert_eq!(trauma.severity, Severity::Moderate);
        assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 1);
        assert_eq!(
            trauma
                .trauma_metadata
                .as_ref()
                .unwrap()
                .cycles_since_last_event,
            0
        );
    }

    #[test]
    fn second_acquisition_reinforces_and_merges_source() {
        let mut t = first_trauma_severity_floor(Severity::Mild);
        // Tick the counter to simulate cycles passing.
        let trauma = t
            .afflictions
            .get_mut(&(AfflictionKind::Trauma, None))
            .unwrap();
        trauma
            .trauma_metadata
            .as_mut()
            .unwrap()
            .cycles_since_last_event = 4;

        let outcome = t.try_acquire_trauma(
            TraumaSource::Betrayal {
                by: "tributes:marvel".into(),
            },
            Severity::Moderate,
        );

        match outcome {
            TraumaAcquisition::Reinforced {
                from_severity,
                to_severity,
                floor_bumped,
            } => {
                assert_eq!(from_severity, Severity::Mild);
                // Severity floor of Moderate exceeded current Mild, so floor bumped.
                assert_eq!(to_severity, Severity::Moderate);
                assert!(floor_bumped);
            }
            other => panic!("expected Reinforced, got {:?}", other),
        }

        let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
        assert_eq!(trauma.severity, Severity::Moderate);
        assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 2);
        // Counter reset.
        assert_eq!(
            trauma
                .trauma_metadata
                .as_ref()
                .unwrap()
                .cycles_since_last_event,
            0
        );
    }

    #[test]
    fn weaker_producer_reinforces_without_floor_bump() {
        // Existing Severe + new Mild producer: counter resets, source merges,
        // severity stays Severe, floor_bumped = false.
        let mut t = first_trauma_severity_floor(Severity::Severe);
        let trauma = t
            .afflictions
            .get_mut(&(AfflictionKind::Trauma, None))
            .unwrap();
        trauma
            .trauma_metadata
            .as_mut()
            .unwrap()
            .cycles_since_last_event = 7;

        let outcome = t.try_acquire_trauma(
            TraumaSource::NearDeath {
                cause: DeathCause::Drowning,
            },
            Severity::Mild,
        );

        match outcome {
            TraumaAcquisition::Reinforced {
                from_severity,
                to_severity,
                floor_bumped,
            } => {
                assert_eq!(from_severity, Severity::Severe);
                assert_eq!(to_severity, Severity::Severe);
                assert!(!floor_bumped);
            }
            other => panic!("expected Reinforced, got {:?}", other),
        }
        let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
        assert_eq!(trauma.severity, Severity::Severe);
        assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 2);
        assert_eq!(
            trauma
                .trauma_metadata
                .as_ref()
                .unwrap()
                .cycles_since_last_event,
            0
        );
    }

    #[test]
    fn duplicate_source_does_not_grow_set_but_still_resets_counter() {
        // Same producer event firing twice (e.g. two separate scenes with the
        // same Betrayal source): set stays size 1, counter still resets.
        let mut t = fresh_tribute();
        let src = TraumaSource::Betrayal {
            by: "tributes:x".into(),
        };
        t.try_acquire_trauma(src.clone(), Severity::Moderate);
        let trauma = t
            .afflictions
            .get_mut(&(AfflictionKind::Trauma, None))
            .unwrap();
        trauma
            .trauma_metadata
            .as_mut()
            .unwrap()
            .cycles_since_last_event = 3;

        let outcome = t.try_acquire_trauma(src.clone(), Severity::Moderate);
        assert!(matches!(outcome, TraumaAcquisition::Reinforced { .. }));
        let trauma = t.afflictions.get(&(AfflictionKind::Trauma, None)).unwrap();
        assert_eq!(trauma.trauma_metadata.as_ref().unwrap().sources.len(), 1);
        assert_eq!(
            trauma
                .trauma_metadata
                .as_ref()
                .unwrap()
                .cycles_since_last_event,
            0
        );
    }

    #[test]
    fn floor_bump_only_works_when_producer_severity_strictly_greater() {
        // Existing Moderate + new Moderate: no floor bump (equal, not greater).
        let mut t = first_trauma_severity_floor(Severity::Moderate);
        let outcome = t.try_acquire_trauma(
            TraumaSource::NearDeath {
                cause: DeathCause::Fire,
            },
            Severity::Moderate,
        );
        match outcome {
            TraumaAcquisition::Reinforced {
                from_severity,
                to_severity,
                floor_bumped,
            } => {
                assert_eq!(from_severity, Severity::Moderate);
                assert_eq!(to_severity, Severity::Moderate);
                assert!(!floor_bumped);
            }
            other => panic!("expected Reinforced, got {:?}", other),
        }
    }

    #[test]
    fn trauma_acquisition_does_not_disturb_other_afflictions() {
        let mut t = fresh_tribute();
        t.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Spawn,
        });
        assert_eq!(t.afflictions.len(), 1);

        t.try_acquire_trauma(
            TraumaSource::NearDeath {
                cause: DeathCause::Fire,
            },
            Severity::Moderate,
        );
        assert_eq!(t.afflictions.len(), 2);
        assert!(
            t.afflictions
                .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
        assert!(t.afflictions.contains_key(&(AfflictionKind::Trauma, None)));
    }

    use proptest::prelude::*;

    fn arb_severity() -> impl Strategy<Value = Severity> {
        prop_oneof![
            Just(Severity::Mild),
            Just(Severity::Moderate),
            Just(Severity::Severe),
        ]
    }

    fn arb_trauma_source() -> impl Strategy<Value = TraumaSource> {
        prop_oneof![
            ("[a-z]{4}").prop_map(|by| TraumaSource::Betrayal {
                by: format!("tributes:{}", by)
            }),
            (
                Just(DeathCause::Fire),
                Just(DeathCause::Drowning),
                Just(DeathCause::Starvation)
            )
                .prop_map(|(_, _, _)| TraumaSource::NearDeath {
                    cause: DeathCause::Fire
                }),
            (1u32..10u32).prop_map(|n| TraumaSource::MassCasualty {
                cause_class: CauseClass::Combat,
                deaths_this_cycle: n,
            }),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Spec §13.4: across any sequence of producer events, a tribute has at
        /// most one Trauma affliction.
        #[test]
        fn single_trauma_invariant(events in proptest::collection::vec(
            (arb_trauma_source(), arb_severity()),
            1..32,
        )) {
            let mut t = fresh_tribute();
            for (src, sev) in events {
                t.try_acquire_trauma(src, sev);
            }
            let trauma_count = t.afflictions
                .keys()
                .filter(|(k, _)| *k == AfflictionKind::Trauma)
                .count();
            prop_assert!(trauma_count <= 1);
        }

        /// Spec §13.4: within the producer pass of a single cycle, sources only
        /// grow (PR1 has no decay; this trivially holds, but the test pins it
        /// for when PR3 adds the decay tick).
        #[test]
        fn sources_monotonic_within_cycle(events in proptest::collection::vec(
            (arb_trauma_source(), arb_severity()),
            1..16,
        )) {
            let mut t = fresh_tribute();
            let mut last_size = 0usize;
            for (src, sev) in events {
                t.try_acquire_trauma(src, sev);
                let cur_size = t.afflictions
                    .get(&(AfflictionKind::Trauma, None))
                    .and_then(|a| a.trauma_metadata.as_ref())
                    .map(|m| m.sources.len())
                    .unwrap_or(0);
                prop_assert!(cur_size >= last_size);
                last_size = cur_size;
            }
        }

        /// Spec §13.4: severity never decreases as a result of a producer event
        /// (it can only stay or rise via floor + escalation; PR1 has no escalation
        /// rolls so only the floor mechanic is exercised here).
        #[test]
        fn severity_floor_monotonic(events in proptest::collection::vec(
            (arb_trauma_source(), arb_severity()),
            1..16,
        )) {
            let mut t = fresh_tribute();
            let mut last_severity: Option<Severity> = None;
            for (src, sev) in events {
                t.try_acquire_trauma(src, sev);
                let cur = t.afflictions
                    .get(&(AfflictionKind::Trauma, None))
                    .map(|a| a.severity);
                if let (Some(prev), Some(now)) = (last_severity, cur) {
                    prop_assert!(now >= prev, "severity decreased: {:?} -> {:?}", prev, now);
                }
                last_severity = cur;
            }
        }
    }
}
