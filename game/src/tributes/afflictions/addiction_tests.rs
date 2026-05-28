#[cfg(test)]
mod tests {
    use crate::tributes::Tribute;
    use crate::tributes::afflictions::addiction::{
        AddictionAcquisition, MAX_ACTIVE_ADDICTIONS, acquisition_probability, high_duration,
    };
    use rand::RngExt;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::{AddictionResistReason, AfflictionKind, Severity, Substance};

    fn fresh_tribute() -> Tribute {
        Tribute {
            identifier: "tributes:test".into(),
            game_day: Some(5),
            ..Default::default()
        }
    }

    /// Find a SmallRng seed that makes `random_bool(p)` return `true`.
    fn seed_hits(p: f64) -> u64 {
        for seed in 0..10000u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            if rng.random_bool(p) {
                return seed;
            }
        }
        panic!("no seed found where random_bool({p}) hits");
    }

    /// Find a SmallRng seed that makes `random_bool(p)` return `false`.
    fn seed_misses(p: f64) -> u64 {
        for seed in 0..10000u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            if !rng.random_bool(p) {
                return seed;
            }
        }
        panic!("no seed found where random_bool({p}) misses");
    }

    // ── Acquisition curve tests ─────────────────────────────────────

    #[test]
    fn acquisition_curve_use_count_1() {
        assert_eq!(acquisition_probability(1, Substance::Stimulant), 0.05);
    }

    #[test]
    fn acquisition_curve_use_count_2() {
        assert_eq!(acquisition_probability(2, Substance::Stimulant), 0.15);
    }

    #[test]
    fn acquisition_curve_use_count_3() {
        assert_eq!(acquisition_probability(3, Substance::Stimulant), 0.30);
    }

    #[test]
    fn acquisition_curve_use_count_4() {
        assert_eq!(acquisition_probability(4, Substance::Stimulant), 0.50);
    }

    #[test]
    fn acquisition_curve_use_count_5_plus() {
        assert_eq!(acquisition_probability(5, Substance::Stimulant), 0.75);
        assert_eq!(acquisition_probability(100, Substance::Stimulant), 0.75);
    }

    // ── Substance multiplier tests (spec §5.2 table) ────────────────

    #[test]
    fn morphling_multiplier_at_use_1() {
        let p = acquisition_probability(1, Substance::Morphling);
        assert!((p - 0.075).abs() < f64::EPSILON);
    }

    #[test]
    fn morphling_multiplier_caps_at_95_percent() {
        let p = acquisition_probability(5, Substance::Morphling);
        assert!((p - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn alcohol_multiplier_reduces_chance() {
        let p = acquisition_probability(4, Substance::Alcohol);
        assert!((p - 0.35).abs() < f64::EPSILON);
    }

    #[test]
    fn painkiller_multiplier_is_1() {
        let p1 = acquisition_probability(3, Substance::Painkiller);
        let p2 = acquisition_probability(3, Substance::Stimulant);
        assert!((p1 - p2).abs() < f64::EPSILON);
    }

    // ── High duration table tests (spec §7.2) ───────────────────────

    #[test]
    fn high_duration_stimulant_mild() {
        assert_eq!(high_duration(Substance::Stimulant, Severity::Mild), 2);
    }

    #[test]
    fn high_duration_stimulant_moderate_severe() {
        assert_eq!(high_duration(Substance::Stimulant, Severity::Moderate), 1);
        assert_eq!(high_duration(Substance::Stimulant, Severity::Severe), 1);
    }

    #[test]
    fn high_duration_painkiller_mild() {
        assert_eq!(high_duration(Substance::Painkiller, Severity::Mild), 3);
    }

    #[test]
    fn high_duration_painkiller_severe() {
        assert_eq!(high_duration(Substance::Painkiller, Severity::Severe), 1);
    }

    #[test]
    fn high_duration_morphling_mild() {
        assert_eq!(high_duration(Substance::Morphling, Severity::Mild), 4);
    }

    #[test]
    fn high_duration_morphling_severe() {
        assert_eq!(high_duration(Substance::Morphling, Severity::Severe), 1);
    }

    #[test]
    fn high_duration_alcohol_always_1() {
        assert_eq!(high_duration(Substance::Alcohol, Severity::Mild), 1);
        assert_eq!(high_duration(Substance::Alcohol, Severity::Moderate), 1);
        assert_eq!(high_duration(Substance::Alcohol, Severity::Severe), 1);
    }

    // ── Acquisition: first use, roll hits ───────────────────────────

    #[test]
    fn first_use_acquires_on_lucky_roll() {
        let mut t = fresh_tribute();
        let seed = seed_hits(0.75); // use_count=100 → p=0.75
        let mut rng = SmallRng::seed_from_u64(seed);
        t.addiction_use_count.insert(Substance::Stimulant, 100);

        let outcome = t.try_acquire_addiction(Substance::Stimulant, &mut rng);

        match outcome {
            AddictionAcquisition::Acquired {
                substance,
                use_count,
            } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(use_count, 100);
            }
            other => panic!("expected Acquired, got {other:?}"),
        }

        // Verify the affliction was created.
        let key = (AfflictionKind::Addiction(Substance::Stimulant), None);
        let aff = t.afflictions.get(&key).expect("addiction should exist");
        assert_eq!(aff.severity, Severity::Mild);
        let meta = aff.addiction_metadata.as_ref().expect("metadata");
        assert_eq!(meta.substance, Substance::Stimulant);
        assert_eq!(meta.high_cycles_remaining, 2); // Stimulant Mild

        // Verify ever_addicted_to is populated.
        assert!(t.ever_addicted_to.contains(&Substance::Stimulant));
    }

    // ── Acquisition: first use, roll misses ─────────────────────────

    #[test]
    fn first_use_misses_on_unlucky_roll() {
        let mut t = fresh_tribute();
        let seed = seed_misses(0.05); // use_count=1 → p=0.05
        let mut rng = SmallRng::seed_from_u64(seed);
        t.addiction_use_count.insert(Substance::Stimulant, 1);

        let outcome = t.try_acquire_addiction(Substance::Stimulant, &mut rng);

        match outcome {
            AddictionAcquisition::Resisted { substance, reason } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(reason, AddictionResistReason::AtCap);
            }
            other => panic!("expected Resisted, got {other:?}"),
        }

        // No addiction created.
        let key = (AfflictionKind::Addiction(Substance::Stimulant), None);
        assert!(!t.afflictions.contains_key(&key));
    }

    // ── Single-instance reinforcement ───────────────────────────────

    #[test]
    fn second_use_reinforces_existing_addiction() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);

        // First call: relapse (ever_addicted_to set, no current addiction).
        let first = t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));
        assert!(
            matches!(first, AddictionAcquisition::Relapse { .. }),
            "expected relapse, got {first:?}"
        );

        // Second call: should reinforce.
        let outcome = t.try_acquire_addiction(
            Substance::Stimulant,
            &mut SmallRng::seed_from_u64(seed_hits(1.0)),
        );
        match outcome {
            AddictionAcquisition::Reinforced {
                substance,
                severity,
                escalated,
            } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(severity, Severity::Mild);
                assert!(!escalated);
            }
            other => panic!("expected Reinforced, got {other:?}"),
        }

        // Still only one addiction.
        let count = t
            .afflictions
            .values()
            .filter(|a| matches!(a.kind, AfflictionKind::Addiction(_)))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn reinforcement_does_not_create_second_addiction() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);

        // Relapse.
        t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));

        // Reinforce 10 times.
        for _ in 0..10 {
            t.try_acquire_addiction(
                Substance::Stimulant,
                &mut SmallRng::seed_from_u64(seed_hits(0.12)),
            );
        }

        let count = t
            .afflictions
            .values()
            .filter(|a| matches!(a.kind, AfflictionKind::Addiction(_)))
            .count();
        assert_eq!(count, 1, "reinforcement must not create second addiction");
    }

    // ── Cap-at-2 enforcement (spec §5.3) ────────────────────────────

    #[test]
    fn cap_at_two_prevents_third_addiction() {
        let mut t = fresh_tribute();
        let hit_seed = seed_hits(0.75);

        // Acquire first addiction (Stimulant).
        t.addiction_use_count.insert(Substance::Stimulant, 100);
        let r1 =
            t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(hit_seed));
        assert!(matches!(r1, AddictionAcquisition::Acquired { .. }));

        // Acquire second addiction (Alcohol).
        t.addiction_use_count.insert(Substance::Alcohol, 100);
        let r2 =
            t.try_acquire_addiction(Substance::Alcohol, &mut SmallRng::seed_from_u64(hit_seed));
        assert!(matches!(r2, AddictionAcquisition::Acquired { .. }));

        // Third (Painkiller) should be resisted.
        t.addiction_use_count.insert(Substance::Painkiller, 100);
        let r3 = t.try_acquire_addiction(
            Substance::Painkiller,
            &mut SmallRng::seed_from_u64(hit_seed),
        );
        match r3 {
            AddictionAcquisition::Resisted { substance, reason } => {
                assert_eq!(substance, Substance::Painkiller);
                assert_eq!(reason, AddictionResistReason::AtCap);
            }
            other => panic!("expected Resisted(AtCap), got {other:?}"),
        }

        // Still exactly 2 addictions.
        let count = t
            .afflictions
            .values()
            .filter(|a| matches!(a.kind, AfflictionKind::Addiction(_)))
            .count();
        assert_eq!(count, MAX_ACTIVE_ADDICTIONS);
    }

    #[test]
    fn same_substance_skips_cap_check() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);
        t.ever_addicted_to.insert(Substance::Alcohol);

        // Relapse into Stimulant.
        t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));
        // Relapse into Alcohol.
        t.try_acquire_addiction(Substance::Alcohol, &mut SmallRng::seed_from_u64(0));

        // Now at cap. Reinforcing existing Stimulant should still work.
        let r = t.try_acquire_addiction(
            Substance::Stimulant,
            &mut SmallRng::seed_from_u64(seed_hits(1.0)),
        );
        assert!(
            matches!(r, AddictionAcquisition::Reinforced { .. }),
            "reinforcing at-cap-existing should not be resisted: {r:?}"
        );
    }

    // ── Relapse path (spec §5.1 step 5c) ────────────────────────────

    #[test]
    fn relapse_auto_acquires_when_ever_addicted() {
        let mut t = fresh_tribute();
        // Simulate prior cured addiction.
        t.ever_addicted_to.insert(Substance::Stimulant);
        t.addiction_use_count.insert(Substance::Stimulant, 3);

        let outcome =
            t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));

        match outcome {
            AddictionAcquisition::Relapse {
                substance,
                prior_uses,
            } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(prior_uses, 3);
            }
            other => panic!("expected Relapse, got {other:?}"),
        }

        // Relapse creates the addiction at Mild.
        let key = (AfflictionKind::Addiction(Substance::Stimulant), None);
        let aff = t.afflictions.get(&key).expect("relapse should create");
        assert_eq!(aff.severity, Severity::Mild);
        assert_eq!(
            aff.addiction_metadata
                .as_ref()
                .unwrap()
                .use_count_at_acquisition,
            3
        );
    }

    #[test]
    fn relapse_skips_probabilistic_roll() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);
        t.addiction_use_count.insert(Substance::Stimulant, 10);

        // Relapse should fire regardless of RNG state.
        let outcome =
            t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));
        assert!(
            matches!(outcome, AddictionAcquisition::Relapse { .. }),
            "relapse should bypass roll, got {outcome:?}"
        );
    }

    #[test]
    fn no_relapse_when_already_addicted() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);

        // Relapse.
        t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));

        // Next use: should reinforce, not relapse.
        let outcome = t.try_acquire_addiction(
            Substance::Stimulant,
            &mut SmallRng::seed_from_u64(seed_hits(1.0)),
        );
        assert!(
            matches!(outcome, AddictionAcquisition::Reinforced { .. }),
            "existing addiction should reinforce, got {outcome:?}"
        );
    }

    // ── Severity floor: escalation roll (spec §6.1) ─────────────────

    #[test]
    fn escalation_roll_steps_up_severity() {
        let mut t = fresh_tribute();
        t.ever_addicted_to.insert(Substance::Stimulant);

        // Relapse at Mild.
        t.try_acquire_addiction(Substance::Stimulant, &mut SmallRng::seed_from_u64(0));

        // Force escalation: find a seed where random_bool(0.12) hits.
        let escalate_seed = seed_hits(0.12);
        let outcome = t.try_acquire_addiction(
            Substance::Stimulant,
            &mut SmallRng::seed_from_u64(escalate_seed),
        );
        assert!(
            matches!(
                outcome,
                AddictionAcquisition::Reinforced {
                    escalated: true,
                    ..
                }
            ),
            "expected escalated reinforcement, got {outcome:?}"
        );

        // Verify severity stepped up.
        let key = (AfflictionKind::Addiction(Substance::Stimulant), None);
        let aff = t.afflictions.get(&key).expect("addiction should exist");
        assert_eq!(aff.severity, Severity::Moderate);
    }

    // ── Constant tests ──────────────────────────────────────────────

    #[test]
    fn max_active_addictions_is_2() {
        assert_eq!(MAX_ACTIVE_ADDICTIONS, 2);
    }
}
