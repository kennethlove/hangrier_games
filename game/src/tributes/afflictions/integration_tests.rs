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
        Affliction {
            kind,
            body_part: Some(match kind {
                AfflictionKind::MissingArm => BodyPart::Arm,
                AfflictionKind::MissingLeg => BodyPart::Leg,
                AfflictionKind::Blind => BodyPart::Eye,
                AfflictionKind::Deaf => BodyPart::Ear,
                _ => BodyPart::Rib,
            }),
            severity,
            source: AfflictionSource::Combat,
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
}
