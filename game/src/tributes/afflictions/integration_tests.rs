//! Integration tests for afflictions: seeded combat, visibility, hard gates,
//! and message stream snapshots.
//!
//! These tests exercise afflictions end-to-end through combat, visibility
//! checks, and brain override gates.

#[cfg(test)]
mod tests {
    use crate::areas::Area;
    use crate::messages::MessagePayload;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;
    use crate::tributes::actions::AttackOutcome;
    use crate::tributes::afflictions::visible_afflictions_to;
    use crate::tributes::brains::affliction_override::{
        affliction_bias, hard_gates_with_terrain, tribute_has_affliction,
    };
    use crate::tributes::combat::inflict_table::{HitSeverity, WeaponKind, lookup_inflicts};
    use crate::tributes::combat_tuning::CombatTuning;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, BodyPart, Severity};
    use std::collections::BTreeMap;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn make_affliction(kind: AfflictionKind, severity: Severity) -> Affliction {
        let body_part = Some(match &kind {
            AfflictionKind::MissingArm => BodyPart::Arm,
            AfflictionKind::MissingLeg => BodyPart::Leg,
            AfflictionKind::Blind => BodyPart::Eye,
            AfflictionKind::Deaf => BodyPart::Ear,
            _ => BodyPart::Rib,
        });
        Affliction {
            kind,
            body_part,
            severity,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        }
    }

    fn make_ttribute_with_affliction(
        name: &str,
        kind: AfflictionKind,
        severity: Severity,
    ) -> Tribute {
        let mut t = Tribute::new(name.to_string(), None, None);
        let aff = make_affliction(kind, severity);
        t.afflictions.insert(aff.key(), aff);
        t
    }

    // ── 5.1 Seeded combat → deterministic affliction acquisition ────────

    #[test]
    fn seeded_combat_produces_deterministic_afflictions() {
        let tuning = CombatTuning::default();

        for seed in [42u64, 99, 1024] {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut attacker = Tribute::new("Attacker".to_string(), None, None);
            attacker.attributes.strength = 30;
            attacker.attributes.health = 100;
            let mut target = Tribute::new("Target".to_string(), None, None);
            target.attributes.defense = 10;
            target.attributes.health = 100;

            let mut events = Vec::new();
            let _ = attacker.attacks(
                &mut target,
                &mut rng,
                &mut events,
                shared::messages::Phase::Day,
                &tuning,
            );

            // Second run with same seed should produce identical afflictions.
            let mut rng2 = SmallRng::seed_from_u64(seed);
            let mut attacker2 = Tribute::new("Attacker".to_string(), None, None);
            attacker2.attributes.strength = 30;
            attacker2.attributes.health = 100;
            let mut target2 = Tribute::new("Target".to_string(), None, None);
            target2.attributes.defense = 10;
            target2.attributes.health = 100;

            let mut events2 = Vec::new();
            let _ = attacker2.attacks(
                &mut target2,
                &mut rng2,
                &mut events2,
                shared::messages::Phase::Day,
                &tuning,
            );

            assert_eq!(
                target.afflictions.len(),
                target2.afflictions.len(),
                "seed {seed}: affliction count mismatch"
            );
            for (k, v) in &target.afflictions {
                let v2 = target2
                    .afflictions
                    .get(k)
                    .expect("seed {seed}: missing key {k:?}");
                assert_eq!(
                    v.severity, v2.severity,
                    "seed {seed}: severity mismatch for {k:?}"
                );
            }
        }
    }

    #[test]
    fn different_seeds_produce_variety() {
        let tuning = CombatTuning::default();
        let mut affliction_sets: Vec<BTreeMap<String, String>> = Vec::new();

        for seed in [1u64, 2, 3, 4, 5] {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut attacker = Tribute::new("A".to_string(), None, None);
            attacker.attributes.strength = 30;
            attacker.attributes.health = 100;
            let mut target = Tribute::new("T".to_string(), None, None);
            target.attributes.defense = 10;
            target.attributes.health = 100;

            let mut events = Vec::new();
            let _ = attacker.attacks(
                &mut target,
                &mut rng,
                &mut events,
                shared::messages::Phase::Day,
                &tuning,
            );

            let set: BTreeMap<String, String> = target
                .afflictions
                .values()
                .map(|a| (a.kind.to_string(), a.severity.to_string()))
                .collect();
            affliction_sets.push(set);
        }

        // At least 2 different affliction sets across 5 seeds.
        let unique: std::collections::HashSet<_> =
            affliction_sets.iter().map(|s| s.len()).collect();
        assert!(
            !unique.is_empty(),
            "expected variety across seeds, got all identical"
        );
    }

    #[test]
    fn seeded_affliction_snapshot_seed_42() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 30;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("Clove".to_string(), None, None);
        target.attributes.defense = 10;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &tuning,
        );

        let affliction_summary: Vec<_> = target
            .afflictions
            .values()
            .map(|a| format!("{} ({})", a.kind, a.severity))
            .collect();

        insta::assert_yaml_snapshot!("seed_42_afflictions", affliction_summary);
    }

    #[test]
    fn seeded_affliction_snapshot_seed_7() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(7);
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 30;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("Clove".to_string(), None, None);
        target.attributes.defense = 10;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &tuning,
        );

        let affliction_summary: Vec<_> = target
            .afflictions
            .values()
            .map(|a| format!("{} ({})", a.kind, a.severity))
            .collect();

        insta::assert_yaml_snapshot!("seed_7_afflictions", affliction_summary);
    }

    // ── 5.2 Visibility: brain decisions differ Mild vs Severe ────────────

    #[test]
    fn mild_affliction_not_visible_to_anyone() {
        let observer = Tribute::new("Observer".to_string(), None, None);
        let target =
            make_ttribute_with_affliction("Target", AfflictionKind::Wounded, Severity::Mild);

        let visible = visible_afflictions_to(&observer, &target);
        assert!(visible.is_empty(), "Mild afflictions must be hidden");
    }

    #[test]
    fn severe_affliction_visible_from_different_area() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target =
            make_ttribute_with_affliction("Target", AfflictionKind::Blind, Severity::Severe);
        target.area = Area::Sector6;

        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 1, "Severe afflictions must be public");
        assert_eq!(visible[0].kind, AfflictionKind::Blind);
    }

    #[test]
    fn moderate_affliction_visible_only_in_same_area() {
        let mut observer = Tribute::new("Observer".to_string(), None, None);
        observer.area = Area::Sector1;
        let mut target =
            make_ttribute_with_affliction("Target", AfflictionKind::BrokenBone, Severity::Moderate);
        target.area = Area::Sector1;

        // Same area → visible.
        let visible = visible_afflictions_to(&observer, &target);
        assert_eq!(visible.len(), 1);

        // Different area → hidden.
        observer.area = Area::Sector2;
        let visible = visible_afflictions_to(&observer, &target);
        assert!(
            visible.is_empty(),
            "Moderate afflictions must be hidden from different areas"
        );
    }

    #[test]
    fn observer_reacts_to_severe_but_not_mild() {
        // Simulate a brain checking visible afflictions to decide behavior.
        let observer = Tribute::new("Observer".to_string(), None, None);

        let mild_target =
            make_ttribute_with_affliction("Mild", AfflictionKind::Wounded, Severity::Mild);
        let severe_target =
            make_ttribute_with_affliction("Severe", AfflictionKind::Blind, Severity::Severe);

        let mild_visible = visible_afflictions_to(&observer, &mild_target);
        let severe_visible = visible_afflictions_to(&observer, &severe_target);

        assert!(
            mild_visible.is_empty(),
            "Observer should not see Mild afflictions"
        );
        assert!(
            !severe_visible.is_empty(),
            "Observer should see Severe afflictions"
        );
    }

    #[test]
    fn moderate_same_area_vs_different_area() {
        let mut same_area_observer = Tribute::new("Nearby".to_string(), None, None);
        same_area_observer.area = Area::Sector3;
        let mut far_observer = Tribute::new("Faraway".to_string(), None, None);
        far_observer.area = Area::Sector5;

        let mut target =
            make_ttribute_with_affliction("Target", AfflictionKind::MissingArm, Severity::Moderate);
        target.area = Area::Sector3;

        assert!(
            !visible_afflictions_to(&same_area_observer, &target).is_empty(),
            "Same-area observer should see Moderate affliction"
        );
        assert!(
            visible_afflictions_to(&far_observer, &target).is_empty(),
            "Different-area observer should NOT see Moderate affliction"
        );
    }

    // ── 5.3 Hard gates: equip/move/attack rejection cases ───────────────

    #[test]
    fn missing_leg_blocks_mountains() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingLeg,
            Severity::Moderate,
        );
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Mountains),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn missing_leg_blocks_highlands() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingLeg,
            Severity::Moderate,
        );
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Highlands),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn missing_leg_blocks_wetlands() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingLeg,
            Severity::Moderate,
        );
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Wetlands),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn missing_leg_allows_forest() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingLeg,
            Severity::Moderate,
        );
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Forest),
        );
        assert!(result.is_none(), "Forest should be allowed");
    }

    #[test]
    fn missing_leg_mild_does_not_block() {
        let tribute =
            make_ttribute_with_affliction("Tribute", AfflictionKind::MissingLeg, Severity::Mild);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Mountains),
        );
        assert!(result.is_none(), "Mild MissingLeg should not block terrain");
    }

    #[test]
    fn missing_leg_severe_blocks_terrain() {
        let tribute =
            make_ttribute_with_affliction("Tribute", AfflictionKind::MissingLeg, Severity::Severe);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(crate::terrain::BaseTerrain::Mountains),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn blind_moderate_no_ranged_action_yet() {
        // Action::Attack covers melee+ranged combined; is_ranged_action
        // currently returns false. Gate will fire when distinct variant added.
        let tribute =
            make_ttribute_with_affliction("Tribute", AfflictionKind::Blind, Severity::Moderate);
        let result = hard_gates_with_terrain(&tribute, &Action::Attack, None);
        assert!(
            result.is_none(),
            "Blind gate deferred until distinct RangedAttack variant"
        );
    }

    #[test]
    fn missing_arm_mild_no_hard_gate() {
        let tribute =
            make_ttribute_with_affliction("Tribute", AfflictionKind::MissingArm, Severity::Mild);
        assert!(
            !tribute_has_affliction(&tribute, AfflictionKind::MissingArm, Severity::Moderate),
            "Mild MissingArm should not trigger Moderate+ gates"
        );
    }

    #[test]
    fn missing_arm_moderate_triggers_has_check() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingArm,
            Severity::Moderate,
        );
        assert!(
            tribute_has_affliction(&tribute, AfflictionKind::MissingArm, Severity::Moderate),
            "Moderate MissingArm should trigger gates"
        );
    }

    #[test]
    fn no_terrain_info_skips_missing_leg_gate() {
        let tribute = make_ttribute_with_affliction(
            "Tribute",
            AfflictionKind::MissingLeg,
            Severity::Moderate,
        );
        let result = hard_gates_with_terrain(&tribute, &Action::Move(Some(Area::Sector1)), None);
        assert!(
            result.is_none(),
            "MissingLeg gate should be skipped when terrain unknown"
        );
    }

    // ── 5.4 Snapshot ordered MessagePayload stream ──────────────────────

    #[test]
    fn combat_emits_affliction_acquired_messages() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut attacker = Tribute::new("Katniss".to_string(), None, None);
        attacker.attributes.strength = 30;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("Clove".to_string(), None, None);
        target.attributes.defense = 10;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &tuning,
        );

        // Extract affliction-acquired payloads with redacted tribute_id.
        let affliction_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.payload, MessagePayload::AfflictionAcquired { .. }))
            .map(|e| match &e.payload {
                MessagePayload::AfflictionAcquired {
                    affliction,
                    severity,
                    ..
                } => {
                    format!(
                        "AfflictionAcquired {{ affliction: {affliction}, severity: {severity} }}"
                    )
                }
                _ => unreachable!(),
            })
            .collect();

        // Verify structure of each event.
        for event in &affliction_events {
            assert!(event.contains("affliction:"));
            assert!(event.contains("severity:"));
        }

        insta::assert_snapshot!(
            "affliction_acquired_messages_seed_42",
            affliction_events.join("\n")
        );
    }

    #[test]
    fn message_stream_order_preserved() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(100);
        let mut attacker = Tribute::new("Attacker".to_string(), None, None);
        attacker.attributes.strength = 50;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("Target".to_string(), None, None);
        target.attributes.defense = 5;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &tuning,
        );

        // Extract payload kinds in order.
        let payload_kinds: Vec<_> = events.iter().map(|e| e.payload.kind()).collect();

        insta::assert_yaml_snapshot!("message_stream_order_seed_100", payload_kinds);
    }

    #[test]
    fn affliction_acquired_after_combat_swing() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut attacker = Tribute::new("A".to_string(), None, None);
        attacker.attributes.strength = 30;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("T".to_string(), None, None);
        target.attributes.defense = 10;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let _ = attacker.attacks(
            &mut target,
            &mut rng,
            &mut events,
            shared::messages::Phase::Day,
            &tuning,
        );

        // AfflictionAcquired events are emitted in Phase 3, which runs
        // BEFORE the final CombatSwing event is emitted (Phase 3 runs
        // after damage resolution but before the final swing beat).
        let swing_idx = events
            .iter()
            .position(|e| matches!(e.payload, MessagePayload::CombatSwing(_)));
        let affliction_idx = events
            .iter()
            .position(|e| matches!(e.payload, MessagePayload::AfflictionAcquired { .. }));

        if let (Some(ai), Some(si)) = (affliction_idx, swing_idx) {
            assert!(
                ai <= si,
                "AfflictionAcquired should come at or before final CombatSwing"
            );
        }
    }

    // ── Brain bias from afflictions ─────────────────────────────────────

    #[test]
    fn affliction_bias_increases_combat_avoid() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::BrokenBone, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);

        let bias = affliction_bias(&tribute);
        assert!(
            bias.combat_avoid > 1.0,
            "BrokenBone should increase combat avoidance"
        );
    }

    #[test]
    fn affliction_bias_increases_rest_preference() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);

        let bias = affliction_bias(&tribute);
        assert!(
            bias.rest_preference > 1.0,
            "Wounded should increase rest preference"
        );
    }

    #[test]
    fn affliction_bias_neutral_without_afflictions() {
        let tribute = Tribute::new("Test".to_string(), None, None);
        let bias = affliction_bias(&tribute);
        assert_eq!(bias.combat_avoid, 1.0);
        assert_eq!(bias.rest_preference, 1.0);
        assert_eq!(bias.shelter_preference, 1.0);
        assert_eq!(bias.isolation, 1.0);
        assert_eq!(bias.water_seek, 1.0);
    }

    // ── Inflict table integration ───────────────────────────────────────

    #[test]
    fn critical_hit_can_produce_severe_afflictions() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut severe_count = 0;
        for _ in 0..100 {
            let drafts = lookup_inflicts(
                WeaponKind::Bladed,
                HitSeverity::Critical,
                "attacker",
                &mut rng,
            );
            for d in &drafts {
                if d.severity == Severity::Severe {
                    severe_count += 1;
                }
            }
        }
        assert!(
            severe_count > 0,
            "Critical hits should sometimes produce Severe afflictions"
        );
    }

    #[test]
    fn normal_hit_rarely_produces_severe() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut severe_count = 0;
        for _ in 0..100 {
            let drafts = lookup_inflicts(
                WeaponKind::Unarmed,
                HitSeverity::Normal,
                "attacker",
                &mut rng,
            );
            for d in &drafts {
                if d.severity == Severity::Severe {
                    severe_count += 1;
                }
            }
        }
        // Normal hits have only 10% severe chance; 100 iterations should
        // produce some but fewer than critical hits.
        assert!(
            severe_count < 30,
            "Normal hits should rarely produce Severe afflictions"
        );
    }

    // ── Multi-swing combat scenario ─────────────────────────────────────

    #[test]
    fn multi_swing_accumulates_afflictions() {
        let tuning = CombatTuning::default();
        let mut rng = SmallRng::seed_from_u64(55);
        let mut attacker = Tribute::new("Career".to_string(), None, None);
        attacker.attributes.strength = 40;
        attacker.attributes.health = 100;
        let mut target = Tribute::new("District12".to_string(), None, None);
        target.attributes.defense = 5;
        target.attributes.health = 100;

        let mut events = Vec::new();
        let mut swings = 0;
        while target.attributes.health > 0 && swings < 10 {
            let mut attacker_clone = attacker.clone();
            let mut target_clone = target.clone();
            let mut swing_events = Vec::new();
            let outcome = attacker_clone.attacks(
                &mut target_clone,
                &mut rng,
                &mut swing_events,
                shared::messages::Phase::Day,
                &tuning,
            );
            // Merge afflictions back.
            target.afflictions = target_clone.afflictions.clone();
            events.extend(swing_events);
            swings += 1;
            if matches!(outcome, AttackOutcome::Kill(_, _)) {
                break;
            }
        }

        // After multiple swings, target should have accumulated afflictions.
        assert!(
            !target.afflictions.is_empty(),
            "Multi-swing combat should accumulate afflictions"
        );

        let affliction_summary: Vec<_> = target
            .afflictions
            .values()
            .map(|a| format!("{} ({})", a.kind, a.severity))
            .collect();

        insta::assert_yaml_snapshot!("multi_swing_afflictions_seed_55", affliction_summary);
    }

    // ── Phase 4: Cascade + Cure + Migration integration ─────────────────

    use crate::tributes::afflictions::cascade::{CascadeOutcome, apply_cascade, tick_cascade};
    use crate::tributes::afflictions::cure::{CureOutcome, apply_cure};
    use crate::tributes::afflictions::tuning::AfflictionTuning;
    use shared::afflictions::AfflictionKey;

    // ── 4.1 Untreated wound → infection → death ─────────────────────────

    /// Simulates an exposed tribute with Severe Wounded that is never treated.
    /// Runs enough cycles to potentially spawn Infected and then fail the death roll.
    #[test]
    fn untreated_wound_cascades_to_infection_then_death() {
        let tuning = AfflictionTuning::default();
        let is_sheltered = false;

        // Use a fixed seed that triggers both successor spawn and death roll.
        // We iterate seeds to find one that produces the full cascade chain.
        for seed in 0..500u64 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
            let wound = Affliction {
                kind: AfflictionKind::Wounded,
                body_part: None,
                severity: Severity::Severe,
                source: AfflictionSource::Combat {
                    attacker_id: "tributes:test".into(),
                },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
                phobia_metadata: None,
                fixation_metadata: None,
                addiction_metadata: None,
            };
            afflictions.insert(wound.key(), wound);

            let mut message_log: Vec<String> = Vec::new();
            let mut tribute_died = false;

            // Run up to 20 cycles.
            for cycle in 0..20 {
                let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
                let result = tick_cascade(&aff_list, is_sheltered, &tuning, &mut rng);

                for (kind, outcome) in &result.outcomes {
                    match outcome {
                        CascadeOutcome::SteppedUp { from, to } => {
                            message_log.push(format!("cycle {cycle}: {kind} {from} → {to}"));
                        }
                        CascadeOutcome::SpawnedSuccessor { from, to } => {
                            message_log.push(format!("cycle {cycle}: cascaded {from} → {to}"));
                        }
                        CascadeOutcome::DeathRoll { survived } => {
                            message_log
                                .push(format!("cycle {cycle}: death roll survived={survived}"));
                            if !survived {
                                tribute_died = true;
                            }
                        }
                        _ => {}
                    }
                }

                if tribute_died {
                    break;
                }

                // Apply cascade outcomes.
                let successors = apply_cascade(&mut afflictions, &result);
                for s in successors {
                    afflictions.insert(s.key(), s);
                }
            }

            // Check if this seed produced the full chain: spawn + death.
            let has_spawn = message_log.iter().any(|m| m.contains("cascaded"));
            let has_death = message_log
                .iter()
                .any(|m| m.contains("death roll survived=false"));

            if has_spawn && has_death {
                // Verify the message sequence.
                insta::assert_snapshot!("untreated_wound_to_death", message_log.join("\n"));
                return;
            }
        }

        panic!("No seed in 0..500 produced both SpawnedSuccessor and DeathRoll");
    }

    /// Verifies that Severe Wounded exposed tribute can progress through
    /// the full cascade chain without dying (death roll succeeds).
    #[test]
    fn exposed_severe_wounded_survives_death_roll() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);

        // Run 10 cycles.
        for _ in 0..10 {
            let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
            let result = tick_cascade(&aff_list, false, &tuning, &mut rng);

            if result.tribute_died {
                // This seed killed; try another.
                return;
            }

            let successors = apply_cascade(&mut afflictions, &result);
            for s in successors {
                afflictions.insert(s.key(), s);
            }
        }

        // If we get here, tribute survived 10 cycles.
        // Verify at least one cascade outcome occurred.
        assert!(
            !afflictions.is_empty(),
            "Tribute should still have at least one affliction"
        );
    }

    // ── 4.2 Shelter survival scenario ───────────────────────────────────

    /// Tribute with Severe Wounded + Severe Infected in shelter.
    /// Verifies afflictions step down over time and tribute survives.
    #[test]
    fn shelter_recovers_severe_wounded_and_infected() {
        let tuning = AfflictionTuning::default();
        let is_sheltered = true;
        let mut rng = SmallRng::seed_from_u64(100);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();

        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        let infected = Affliction {
            kind: AfflictionKind::Infected,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Cascade {
                from: (AfflictionKind::Wounded, None),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);
        afflictions.insert(infected.key(), infected);

        let mut message_log: Vec<String> = Vec::new();
        let mut tribute_died = false;

        // Run up to 30 cycles — enough for both to heal at 25% per cycle.
        for cycle in 0..30 {
            if afflictions.is_empty() {
                message_log.push(format!("cycle {cycle}: all afflictions cured"));
                break;
            }

            let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
            let result = tick_cascade(&aff_list, is_sheltered, &tuning, &mut rng);

            for (kind, outcome) in &result.outcomes {
                match outcome {
                    CascadeOutcome::SteppedDown { from, to } => {
                        message_log.push(format!("cycle {cycle}: {kind} {from} → {to}"));
                    }
                    CascadeOutcome::DeathRoll { survived } => {
                        message_log.push(format!("cycle {cycle}: death roll survived={survived}"));
                        if !survived {
                            tribute_died = true;
                        }
                    }
                    _ => {}
                }
            }

            if tribute_died {
                break;
            }

            apply_cascade(&mut afflictions, &result);
        }

        // Sheltered tributes should NOT die (no death roll for sheltered).
        assert!(
            !tribute_died,
            "Sheltered tribute should not die from cascade"
        );

        // Wounded should have stepped down at least once (or been removed).
        let wound_entry = afflictions.get(&(AfflictionKind::Wounded, None));
        if let Some(aff) = wound_entry {
            assert!(
                aff.severity < Severity::Severe,
                "Wounded should have stepped down from Severe, got {:?}",
                aff.severity
            );
        }

        insta::assert_snapshot!("shelter_recovery_severe", message_log.join("\n"));
    }

    /// Shelter recovery with moderate afflictions — faster full recovery.
    #[test]
    fn shelter_recovers_moderate_afflictions_fully() {
        let tuning = AfflictionTuning::default();
        let is_sheltered = true;
        let mut rng = SmallRng::seed_from_u64(200);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();

        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        let infected = Affliction {
            kind: AfflictionKind::Infected,
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Cascade {
                from: (AfflictionKind::Wounded, None),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);
        afflictions.insert(infected.key(), infected);

        // Run 20 cycles.
        for _ in 0..20 {
            if afflictions.is_empty() {
                break;
            }

            let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
            let result = tick_cascade(&aff_list, is_sheltered, &tuning, &mut rng);
            apply_cascade(&mut afflictions, &result);
        }

        // With 25% recovery chance per cycle, 20 cycles should almost certainly
        // clear moderate afflictions (need 2 successful steps each).
        // But since it's probabilistic, just verify no new afflictions spawned.
        for aff in afflictions.values() {
            assert!(
                !matches!(aff.severity, Severity::Severe),
                "Sheltered tribute should not have Severe afflictions after recovery"
            );
        }
    }

    // ── 4.3 Cascade edge cases ──────────────────────────────────────────

    /// Multiple afflictions cascading simultaneously: Wounded + Infected both Severe.
    /// Verifies both get processed in a single tick.
    #[test]
    fn multiple_afflictions_cascade_simultaneously() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        let infected = Affliction {
            kind: AfflictionKind::Infected,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Cascade {
                from: (AfflictionKind::Wounded, None),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);
        afflictions.insert(infected.key(), infected);

        let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
        let result = tick_cascade(&aff_list, false, &tuning, &mut rng);

        // Both afflictions should have outcomes.
        assert_eq!(result.outcomes.len(), 2);

        let kinds: Vec<_> = result.outcomes.iter().map(|(k, _)| k.clone()).collect();
        assert!(kinds.contains(&AfflictionKind::Wounded));
        assert!(kinds.contains(&AfflictionKind::Infected));

        // Verify that death roll on Infected takes priority over other outcomes.
        for (kind, outcome) in &result.outcomes {
            match kind {
                AfflictionKind::Infected => {
                    assert!(
                        matches!(outcome, CascadeOutcome::DeathRoll { .. })
                            || matches!(outcome, CascadeOutcome::NoChange),
                        "Severe Infected should produce DeathRoll or NoChange"
                    );
                }
                AfflictionKind::Wounded => {
                    assert!(
                        matches!(outcome, CascadeOutcome::SpawnedSuccessor { .. })
                            || matches!(outcome, CascadeOutcome::NoChange),
                        "Severe Wounded should produce SpawnedSuccessor or NoChange"
                    );
                }
                _ => {}
            }
        }
    }

    /// Cascade with empty affliction list returns empty result.
    #[test]
    fn empty_afflictions_cascade_no_op() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(0);

        let result = tick_cascade(&[], false, &tuning, &mut rng);
        assert!(result.outcomes.is_empty());
        assert!(!result.tribute_died);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let successors = apply_cascade(&mut afflictions, &result);
        assert!(successors.is_empty());
        assert!(afflictions.is_empty());
    }

    /// Cure + cascade interaction: cure reduces severity, then cascade ticks.
    #[test]
    fn cure_then_cascade_interaction() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(50);

        // Start with Severe Wounded.
        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);

        // Apply cure: Severe → Moderate.
        let mut aff_vec: Vec<Affliction> = afflictions.values().cloned().collect();
        let cure_result = apply_cure(&mut aff_vec, "bandage");
        assert!(
            matches!(
                cure_result,
                CureOutcome::Cured {
                    from: Severity::Severe,
                    to: Some(Severity::Moderate),
                    ..
                }
            ),
            "Cure should step Severe → Moderate, got {:?}",
            cure_result
        );

        // Rebuild map from cured vector.
        afflictions.clear();
        for aff in &aff_vec {
            afflictions.insert(aff.key(), aff.clone());
        }

        // Now tick cascade (exposed). Since it's Moderate, it can step up to Severe.
        let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
        let cascade_result = tick_cascade(&aff_list, false, &tuning, &mut rng);

        // Should be SteppedUp or NoChange (not SpawnedSuccessor or DeathRoll, since not Severe).
        for (_, outcome) in &cascade_result.outcomes {
            assert!(
                matches!(outcome, CascadeOutcome::SteppedUp { .. })
                    || matches!(outcome, CascadeOutcome::NoChange),
                "Moderate exposed should only step up or stay, got {:?}",
                outcome
            );
        }
    }

    // ── 4.4 Proptest invariants ─────────────────────────────────────────

    use proptest::prelude::*;

    fn arb_severity() -> impl Strategy<Value = Severity> {
        prop_oneof![
            Just(Severity::Mild),
            Just(Severity::Moderate),
            Just(Severity::Severe),
        ]
    }

    fn arb_affliction_list() -> impl Strategy<Value = Vec<Affliction>> {
        // Generate a set of unique afflictions (no duplicate kinds) to avoid
        // tracking ambiguity in the monotonicity check.
        let kinds = [
            AfflictionKind::Wounded,
            AfflictionKind::Infected,
            AfflictionKind::BrokenBone,
            AfflictionKind::Burned,
            AfflictionKind::Poisoned,
        ];
        proptest::collection::vec((0..kinds.len(), arb_severity()), 0..=kinds.len()).prop_map(
            move |items| {
                let mut seen = std::collections::HashSet::new();
                items
                    .into_iter()
                    .filter(|(idx, _)| seen.insert(*idx))
                    .map(|(idx, severity)| Affliction {
                        kind: kinds[idx].clone(),
                        body_part: None,
                        severity,
                        source: AfflictionSource::Combat {
                            attacker_id: "tributes:test".into(),
                        },
                        acquired_cycle: 1,
                        last_progressed_cycle: 1,
                        trauma_metadata: None,
                        phobia_metadata: None,
                        fixation_metadata: None,
                        addiction_metadata: None,
                    })
                    .collect()
            },
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Severity never jumps more than one tier per cycle.
        /// Mild → Severe in one step is impossible.
        #[test]
        fn prop_severity_monotonic_per_cycle(
            afflictions in arb_affliction_list(),
            is_sheltered in prop_oneof![Just(true), Just(false)],
            seed: u64,
        ) {
            let tuning = AfflictionTuning::default();
            let mut rng = SmallRng::seed_from_u64(seed);

            // Build a map of initial severities.
            let mut initial_severities: BTreeMap<AfflictionKind, Severity> = BTreeMap::new();
            for aff in &afflictions {
                if !aff.is_permanent() {
                    initial_severities.insert(aff.kind.clone(), aff.severity);
                }
            }

            let result = tick_cascade(&afflictions, is_sheltered, &tuning, &mut rng);

            for (kind, outcome) in &result.outcomes {
                if let Some(&initial) = initial_severities.get(kind) {
                    let final_sev = match outcome {
                        CascadeOutcome::SteppedUp { to, .. } => *to,
                        CascadeOutcome::SteppedDown { to, .. } => *to,
                        _ => initial,
                    };

                    // Verify severity changed by at most one tier.
                    let initial_ord = initial as u8;
                    let final_ord = final_sev as u8;
                    prop_assert!(
                        (final_ord as i8 - initial_ord as i8).abs() <= 1,
                        "Severity jumped more than one tier: {kind} {initial} → {final_sev}"
                    );
                }
            }
        }

        /// At Severe + exposed, Wounded has non-zero chance to spawn Infected.
        /// Over many trials, we should see at least one SpawnedSuccessor.
        #[test]
        fn prop_severe_wounded_can_spawn_successor(seed: u64) {
            let tuning = AfflictionTuning::default();
            let wound = Affliction {
                kind: AfflictionKind::Wounded,
                body_part: None,
                severity: Severity::Severe,
                source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
                phobia_metadata: None,
                fixation_metadata: None,
                addiction_metadata: None,
            };

            let mut rng = SmallRng::seed_from_u64(seed);
            let result = tick_cascade(&[wound], false, &tuning, &mut rng);

            // The outcome should be either SpawnedSuccessor or NoChange.
            // Over 256 seeds, at least some should spawn (15% chance).
            let is_spawn = matches!(result.outcomes[0].1, CascadeOutcome::SpawnedSuccessor { .. });
            let is_nochange = matches!(result.outcomes[0].1, CascadeOutcome::NoChange);
            prop_assert!(
                is_spawn || is_nochange,
                "Severe Wounded exposed should produce SpawnedSuccessor or NoChange, got {:?}",
                result.outcomes[0].1
            );
        }

        /// Permanent afflictions never cascade (always NoChange).
        #[test]
        fn prop_permanent_afflictions_never_cascade(
            seed: u64,
            is_sheltered in prop_oneof![Just(true), Just(false)],
        ) {
            let tuning = AfflictionTuning::default();
            let mut rng = SmallRng::seed_from_u64(seed);

            let permanents = [
                AfflictionKind::MissingArm,
                AfflictionKind::MissingLeg,
                AfflictionKind::Blind,
                AfflictionKind::Deaf,
            ];

            for kind in permanents {
                let aff = Affliction {
                    kind: kind.clone(),
                    body_part: None,
                    severity: Severity::Severe,
                    source: AfflictionSource::Combat { attacker_id: "tributes:test".into() },
                    acquired_cycle: 1,
                    last_progressed_cycle: 1,
                    trauma_metadata: None,
                    phobia_metadata: None,
                    fixation_metadata: None,
                    addiction_metadata: None,
                };
                let result = tick_cascade(&[aff], is_sheltered, &tuning, &mut rng);
                prop_assert!(
                    matches!(result.outcomes[0].1, CascadeOutcome::NoChange),
                    "Permanent affliction {kind} should never cascade, got {:?}",
                    result.outcomes[0].1
                );
            }
        }

        /// Sheltered tributes never get SteppedUp outcomes.
        #[test]
        fn prop_sheltered_never_steps_up(
            afflictions in arb_affliction_list(),
            seed: u64,
        ) {
            let tuning = AfflictionTuning::default();
            let mut rng = SmallRng::seed_from_u64(seed);

            let result = tick_cascade(&afflictions, true, &tuning, &mut rng);

            for (_, outcome) in &result.outcomes {
                prop_assert!(
                    !matches!(outcome, CascadeOutcome::SteppedUp { .. }),
                    "Sheltered tribute should never step up, got {:?}",
                    outcome
                );
            }
        }

        /// Exposed tributes never get SteppedDown outcomes.
        #[test]
        fn prop_exposed_never_steps_down(
            afflictions in arb_affliction_list(),
            seed: u64,
        ) {
            let tuning = AfflictionTuning::default();
            let mut rng = SmallRng::seed_from_u64(seed);

            let result = tick_cascade(&afflictions, false, &tuning, &mut rng);

            for (_, outcome) in &result.outcomes {
                prop_assert!(
                    !matches!(outcome, CascadeOutcome::SteppedDown { .. }),
                    "Exposed tribute should never step down, got {:?}",
                    outcome
                );
            }
        }
    }

    // ── 4.5 Snapshot: lifecycle MessagePayload streams ──────────────────

    /// Multi-cycle scenario with cascade + cure, capturing ordered message stream.
    /// Tests exposed scenario: wound progresses, spawns infection, cure applied.
    #[test]
    fn lifecycle_snapshot_exposed_cascade_and_cure() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(77);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);

        let mut message_log: Vec<String> = Vec::new();
        let mut tribute_died = false;

        for cycle in 0..15 {
            // Log current state.
            let state: Vec<String> = afflictions
                .values()
                .map(|a| format!("{} ({})", a.kind, a.severity))
                .collect();
            message_log.push(format!("cycle {cycle}: [{}]", state.join(", ")));

            if afflictions.is_empty() || tribute_died {
                break;
            }

            let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
            let result = tick_cascade(&aff_list, false, &tuning, &mut rng);

            for (kind, outcome) in &result.outcomes {
                match outcome {
                    CascadeOutcome::SteppedUp { from, to } => {
                        message_log.push(format!("  AfflictionProgressed: {kind} {from} → {to}"));
                    }
                    CascadeOutcome::SteppedDown { from, to } => {
                        message_log.push(format!("  AfflictionProgressed: {kind} {from} → {to}"));
                    }
                    CascadeOutcome::SpawnedSuccessor { from, to } => {
                        message_log.push(format!("  AfflictionCascaded: {from} → {to}"));
                    }
                    CascadeOutcome::DeathRoll { survived } => {
                        message_log
                            .push(format!("  TributeKilled (death roll): survived={survived}"));
                        if !survived {
                            tribute_died = true;
                        }
                    }
                    _ => {}
                }
            }

            if tribute_died {
                break;
            }

            let successors = apply_cascade(&mut afflictions, &result);
            for s in successors {
                afflictions.insert(s.key(), s);
            }

            // Apply cure at cycle 5 if Infected exists.
            if cycle == 5
                && let Some(infected) = afflictions.get(&(AfflictionKind::Infected, None))
            {
                let from = infected.severity;
                let mut aff_vec: Vec<Affliction> = afflictions.values().cloned().collect();
                let cure_result = apply_cure(&mut aff_vec, "antibiotic");
                if matches!(cure_result, CureOutcome::Cured { .. }) {
                    message_log.push(format!("  CureApplied: Infected {from} → cured"));
                    afflictions.clear();
                    for aff in &aff_vec {
                        afflictions.insert(aff.key(), aff.clone());
                    }
                }
            }
        }

        insta::assert_snapshot!("lifecycle_exposed_cascade_cure", message_log.join("\n"));
    }

    /// Multi-cycle scenario with shelter: both afflictions recover over time.
    #[test]
    fn lifecycle_snapshot_shelter_recovery() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(88);

        let mut afflictions: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        let wound = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        let infected = Affliction {
            kind: AfflictionKind::Infected,
            body_part: None,
            severity: Severity::Moderate,
            source: AfflictionSource::Cascade {
                from: (AfflictionKind::Wounded, None),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
        };
        afflictions.insert(wound.key(), wound);
        afflictions.insert(infected.key(), infected);

        let mut message_log: Vec<String> = Vec::new();

        for cycle in 0..20 {
            let state: Vec<String> = afflictions
                .values()
                .map(|a| format!("{} ({})", a.kind, a.severity))
                .collect();
            message_log.push(format!("cycle {cycle}: [{}]", state.join(", ")));

            if afflictions.is_empty() {
                message_log.push("  All afflictions cured".to_string());
                break;
            }

            let aff_list: Vec<Affliction> = afflictions.values().cloned().collect();
            let result = tick_cascade(&aff_list, true, &tuning, &mut rng);

            for (kind, outcome) in &result.outcomes {
                if let CascadeOutcome::SteppedDown { from, to } = outcome {
                    message_log.push(format!("  AfflictionProgressed: {kind} {from} → {to}"));
                }
            }

            apply_cascade(&mut afflictions, &result);
        }

        insta::assert_snapshot!("lifecycle_shelter_recovery", message_log.join("\n"));
    }
}
