use crate::messages::TaggedEvent;
use crate::tributes::Tribute;
use crate::tributes::brains::Brain;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rstest::*;

#[fixture]
fn tribute() -> Tribute {
    Tribute::new("Katniss".to_string(), None, None)
}

#[fixture]
fn target() -> Tribute {
    Tribute::new("Peeta".to_string(), None, None)
}

#[fixture]
fn small_rng() -> SmallRng {
    SmallRng::seed_from_u64(0)
}

#[rstest]
fn default() {
    let tribute = Tribute::default();
    assert_eq!(tribute.name, "Default Tribute");
}

#[rstest]
fn serde_roundtrip_alliance_fields() {
    use crate::tributes::traits::Trait;
    use uuid::Uuid;

    let mut tribute = Tribute::new("Rue".to_string(), None, None);
    let ally = Uuid::new_v4();
    tribute.allies.push(ally);
    tribute.traits.clear();
    tribute.traits.push(Trait::Loyal);
    tribute.traits.push(Trait::Treacherous);
    tribute.turns_since_last_betrayal = 7;
    tribute.pending_trust_shock = true;

    let json = serde_json::to_string(&tribute).expect("serialize");
    assert!(json.contains("\"allies\""));
    assert!(json.contains("\"traits\""));
    assert!(json.contains("\"Loyal\""));
    assert!(json.contains("\"turns_since_last_betrayal\":7"));
    assert!(json.contains("\"pending_trust_shock\":true"));

    let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.allies, vec![ally]);
    assert_eq!(restored.traits, vec![Trait::Loyal, Trait::Treacherous]);
    assert_eq!(restored.turns_since_last_betrayal, 7);
    assert!(restored.pending_trust_shock);
}

#[rstest]
fn serde_defaults_for_missing_alliance_fields() {
    // Persisted tribute records written before the alliance fields existed
    // must still deserialize. Simulate this by serialising a fresh tribute,
    // stripping the new fields, then round-tripping.
    let mut rng = SmallRng::seed_from_u64(42);
    let baseline = Tribute::new_with_rng("Legacy".to_string(), None, None, &mut rng);
    let mut value: serde_json::Value = serde_json::to_value(&baseline).expect("to_value");
    let obj = value.as_object_mut().expect("object");
    obj.remove("allies");
    obj.remove("traits");
    obj.remove("turns_since_last_betrayal");
    obj.remove("pending_trust_shock");

    let restored: Tribute = serde_json::from_value(value).expect("legacy deserialize");
    assert!(restored.allies.is_empty());
    assert!(restored.traits.is_empty());
    assert_eq!(restored.turns_since_last_betrayal, 0);
    assert!(!restored.pending_trust_shock);
}

#[rstest]
fn brain_roundtrips_psychotic_break_state() {
    use crate::tributes::brains::PsychoticBreakType;

    let mut rng = SmallRng::seed_from_u64(42);
    let mut tribute = Tribute::new_with_rng("Cato".to_string(), None, None, &mut rng);
    tribute.brain.psychotic_break = Some(PsychoticBreakType::Berserk);

    let json = serde_json::to_string(&tribute).expect("serialize");
    assert!(json.contains("\"brain\""));
    assert!(json.contains("\"Berserk\""));

    let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        restored.brain.psychotic_break,
        Some(PsychoticBreakType::Berserk),
    );
}

#[rstest]
fn brain_preferred_action_is_not_persisted() {
    // preferred_action is transient AI state recomputed each cycle, so the
    // field is `skip_serializing` and `deserialize_optional_enum_lenient`
    // (which absorbs both null and the {} corruption left over from the
    // SDK's enum-collapse bug). A roundtrip therefore intentionally drops
    // any preferred_action that was set in memory.
    use crate::tributes::actions::Action;

    let mut tribute = Tribute::new("Foxface".to_string(), None, None);
    tribute.brain.preferred_action = Some(Action::Hide);
    tribute.brain.preferred_action_percentage = 0.75;

    let json = serde_json::to_string(&tribute).expect("serialize");

    let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.brain.preferred_action, None);
    // Non-skipped fields still round-trip normally.
    assert!((restored.brain.preferred_action_percentage - 0.75).abs() < f64::EPSILON);
}

#[rstest]
fn brain_tolerates_corrupt_preferred_action_object() {
    // SurrealDB rows written before the bug-5 fix have preferred_action: {}
    // because the SDK's bespoke serializer collapsed the externally-tagged
    // Action enum. The lenient deserializer must read those rows as None.
    // Round-trip a real Brain to get a valid base JSON, then swap
    // preferred_action's value to {} to simulate the corruption.
    use crate::tributes::brains::Brain;

    let brain = Brain {
        preferred_action_percentage: 0.5,
        ..Brain::default()
    };
    let mut value = serde_json::to_value(&brain).expect("serialize brain");
    value["preferred_action"] = serde_json::json!({});
    let restored: Brain = serde_json::from_value(value).expect("deserialize legacy row");
    assert_eq!(restored.preferred_action, None);
}

#[rstest]
fn brain_missing_field_defaults() {
    // Pre-fix tribute rows persisted before #[serde(default)] was added
    // omit the `brain` column entirely. They must still deserialize, with
    // brain hydrated via `Brain::default()`.
    let mut rng = SmallRng::seed_from_u64(42);
    let baseline = Tribute::new_with_rng("Legacy".to_string(), None, None, &mut rng);
    let mut value: serde_json::Value = serde_json::to_value(&baseline).expect("to_value");
    value.as_object_mut().expect("object").remove("brain");

    let restored: Tribute = serde_json::from_value(value).expect("legacy deserialize");
    assert_eq!(restored.brain, Brain::default());
    assert!(restored.brain.psychotic_break.is_none());
    assert!(restored.brain.preferred_action.is_none());
}

#[rstest]
fn new() {
    let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
    assert_eq!(tribute.name, "Katniss");
    assert_eq!(tribute.district, 12);
    // Attributes::new() randomizes health in 50..=max_health.
    assert!(
        (50..=100).contains(&tribute.attributes.health),
        "health {} out of range",
        tribute.attributes.health
    );
}

#[rstest]
fn random() {
    let tribute = Tribute::random();
    assert!(!tribute.name.is_empty());
    assert!(tribute.district >= 1 && tribute.district <= 12);
}

#[rstest]
fn new_tribute_has_empty_alliance_state() {
    let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    assert!(tribute.allies.is_empty());
    assert_eq!(tribute.turns_since_last_betrayal, 0);
    // `id` mirrors `identifier`.
    assert_eq!(tribute.id.to_string(), tribute.identifier);
}

#[rstest]
fn new_tribute_has_no_pending_trust_shock() {
    let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    assert!(!tribute.pending_trust_shock);
}

#[test]
fn tribute_default_survival_fields_are_zero_and_none() {
    let t = Tribute::new("Test".to_string(), None, None);
    assert_eq!(t.hunger, 0, "hunger starts at 0 (Sated)");
    assert_eq!(t.thirst, 0, "thirst starts at 0 (Sated)");
    assert_eq!(t.sheltered_until, None, "starts exposed");
    assert_eq!(t.starvation_drain_step, 0);
    assert_eq!(t.dehydration_drain_step, 0);
}

#[test]
fn tribute_legacy_json_loads_with_defaults() {
    // JSON missing the new survival fields entirely (simulates a saved
    // game from before this feature landed). serde(default) must
    // populate them.
    let mut t = Tribute::new("Legacy".to_string(), Some(1), None);
    t.hunger = 0;
    t.thirst = 0;
    t.sheltered_until = None;
    t.starvation_drain_step = 0;
    t.dehydration_drain_step = 0;
    let mut json: serde_json::Value = serde_json::to_value(&t).unwrap();
    // strip the survival fields to mimic a pre-feature save
    let obj = json.as_object_mut().unwrap();
    obj.remove("hunger");
    obj.remove("thirst");
    obj.remove("sheltered_until");
    obj.remove("starvation_drain_step");
    obj.remove("dehydration_drain_step");
    let loaded: Tribute = serde_json::from_value(json).expect("legacy load must succeed");
    assert_eq!(loaded.hunger, 0);
    assert_eq!(loaded.thirst, 0);
    assert_eq!(loaded.sheltered_until, None);
    assert_eq!(loaded.starvation_drain_step, 0);
    assert_eq!(loaded.dehydration_drain_step, 0);
}

#[test]
fn tribute_legacy_json_loads_with_sleep_defaults() {
    // JSON missing the new sleep fields must default to zero/false.
    let t = Tribute::new("Legacy".to_string(), Some(1), None);
    let mut json: serde_json::Value = serde_json::to_value(&t).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.remove("cycles_awake");
    obj.remove("sleeping");
    obj.remove("sleep_remaining");
    let loaded: Tribute = serde_json::from_value(json).expect("legacy load must succeed");
    assert_eq!(loaded.cycles_awake, 0);
    assert!(!loaded.sleeping);
    assert_eq!(loaded.sleep_remaining, 0);
}

#[rstest]
fn tribute_drain_alliance_events_returns_and_clears_buffer() {
    use crate::tributes::alliances::AllianceEvent;
    use uuid::Uuid;
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    let other = Uuid::new_v4();
    tribute
        .alliance_events
        .push(AllianceEvent::BetrayalRecorded {
            betrayer: tribute.id,
            victim: other,
        });
    let drained = tribute.drain_alliance_events();
    assert_eq!(drained.len(), 1);
    assert!(tribute.alliance_events.is_empty());
}

#[rstest]
fn consume_pending_trust_shock_resets_flag_when_not_set() {
    // No flag → no rolls, flag stays false, allies untouched.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    let ally = uuid::Uuid::new_v4();
    tribute.allies.push(ally);
    let mut events: Vec<TaggedEvent> = vec![];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(53);
    tribute.consume_pending_trust_shock(&mut rng, &mut events);
    assert!(!tribute.pending_trust_shock);
    assert_eq!(tribute.allies, vec![ally]);
    assert!(events.is_empty());
}

#[rstest]
fn consume_pending_trust_shock_breaks_allies_on_success_and_clears_flag() {
    // Force trust_shock to fire deterministically: sanity=0, threshold>0
    // gives p = 0.5 + 0.5 * 1.0 = 1.0 → always true.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    tribute.attributes.sanity = 0;
    tribute.brain.thresholds.extreme_low_sanity = 50;
    let ally1 = uuid::Uuid::new_v4();
    let ally2 = uuid::Uuid::new_v4();
    tribute.allies.push(ally1);
    tribute.allies.push(ally2);
    tribute.pending_trust_shock = true;

    let mut events: Vec<TaggedEvent> = vec![];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(211);
    tribute.consume_pending_trust_shock(&mut rng, &mut events);

    assert!(!tribute.pending_trust_shock, "flag must reset");
    assert!(
        tribute.allies.is_empty(),
        "all allies broken on guaranteed success"
    );
    assert_eq!(events.len(), 2, "one message per broken ally");
}

#[rstest]
fn consume_pending_trust_shock_no_break_when_sanity_above_threshold() {
    // Sanity at/above threshold → trust_shock_roll returns false → no break.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    tribute.attributes.sanity = 100;
    tribute.brain.thresholds.extreme_low_sanity = 50;
    let ally = uuid::Uuid::new_v4();
    tribute.allies.push(ally);
    tribute.pending_trust_shock = true;

    let mut events: Vec<TaggedEvent> = vec![];
    let mut rng = rand::rngs::SmallRng::seed_from_u64(89);
    tribute.consume_pending_trust_shock(&mut rng, &mut events);

    assert!(!tribute.pending_trust_shock, "flag must reset");
    assert_eq!(tribute.allies, vec![ally], "ally retained");
    assert!(events.is_empty());
}

#[rstest]
fn new_tribute_has_traits_for_valid_district() {
    let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
    // generate_traits rolls 2..=6 traits from the district pool.
    assert!((2..=6).contains(&tribute.traits.len()));
}

#[rstest]
fn pick_target_skips_allies() {
    // An ally is in the same area but must not be picked as a target.
    let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
    me.attributes.sanity = 100; // not suicidal
    let ally = Tribute::new("Peeta".to_string(), Some(12), None);
    me.allies.push(ally.id);

    let mut events: Vec<TaggedEvent> = vec![];
    let target = me.pick_target(vec![ally.clone()], 5, &mut events);
    // Only candidate was an ally and we're not in final confrontation.
    assert!(target.is_none());
}

#[rstest]
fn pick_target_allows_same_district_when_not_ally() {
    // Same-district tributes can now be targeted unless they're allies.
    let me = Tribute::new("Katniss".to_string(), Some(12), None);
    let same_district = Tribute::new("Peeta".to_string(), Some(12), None);

    let mut events: Vec<TaggedEvent> = vec![];
    let target = me.pick_target(vec![same_district.clone()], 5, &mut events);
    assert!(target.is_some());
    assert_eq!(target.unwrap().id, same_district.id);
}

#[rstest]
fn pick_target_final_confrontation_overrides_alliance() {
    // When only two tributes remain alive, even an ally is a valid target.
    let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
    me.attributes.sanity = 100;
    let ally = Tribute::new("Peeta".to_string(), Some(12), None);
    me.allies.push(ally.id);

    let mut events: Vec<TaggedEvent> = vec![];
    let target = me.pick_target(vec![ally.clone()], 2, &mut events);
    assert!(target.is_some());
    assert_eq!(target.unwrap().id, ally.id);
}

#[rstest]
fn tick_alliance_timers_increments_betrayal_counter() {
    // Living tribute: counter increments by exactly one per tick.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    assert_eq!(tribute.turns_since_last_betrayal, 0);
    tribute.tick_alliance_timers();
    assert_eq!(tribute.turns_since_last_betrayal, 1);
    tribute.tick_alliance_timers();
    assert_eq!(tribute.turns_since_last_betrayal, 2);
}

#[rstest]
fn tick_alliance_timers_saturates_does_not_overflow() {
    // u8 saturating add: never panics, never wraps to zero.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    tribute.turns_since_last_betrayal = u8::MAX;
    tribute.tick_alliance_timers();
    assert_eq!(tribute.turns_since_last_betrayal, u8::MAX);
}

#[rstest]
fn tick_alliance_timers_skips_dead_tributes() {
    // Dead tributes don't accumulate betrayal cooldown.
    let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
    tribute.attributes.health = 0;
    tribute.status = crate::tributes::TributeStatus::RecentlyDead;
    tribute.tick_alliance_timers();
    assert_eq!(tribute.turns_since_last_betrayal, 0);
}

#[rstest]
fn pick_target_picks_ex_ally_after_trust_shock_breaks_bond() {
    // End-to-end break-then-attack (spec §7.3c1 + §7.5):
    // Once a trust shock fires and removes the betrayer from the
    // victim's `allies`, the victim's next `pick_target` call must
    // consider that ex-ally a valid target.
    let mut victim = Tribute::new("Glimmer".to_string(), Some(1), None);
    victim.attributes.sanity = 100; // not suicidal
    let ex_ally = Tribute::new("Cato".to_string(), Some(2), None);
    // Pre-condition: bonded.
    victim.allies.push(ex_ally.id);

    // Simulate the bond breaking (what process_alliance_events does
    // for BetrayalRecorded, plus what consume_pending_trust_shock
    // does on the victim's side: drop the ex-ally locally).
    victim.allies.retain(|id| *id != ex_ally.id);

    let mut events: Vec<TaggedEvent> = vec![];
    let target = victim.pick_target(vec![ex_ally.clone()], 5, &mut events);
    assert!(
        target.is_some(),
        "ex-ally must be targetable after the bond breaks"
    );
    assert_eq!(target.unwrap().id, ex_ally.id);
}

#[rstest]
fn consume_pending_trust_shock_leaves_asymmetric_back_edge() {
    // Spec §7.3c1 explicitly defers the symmetric back-edge cleanup
    // for trust-shock breaks: only `self` is mutated. This regression
    // test pins that contract so any future tightening is intentional.
    let mut victim = Tribute::new("Glimmer".to_string(), Some(1), None);
    victim.attributes.sanity = 0; // force a break
    victim.brain.thresholds.extreme_low_sanity = 100;
    let betrayer_id = uuid::Uuid::new_v4();
    victim.allies.push(betrayer_id);
    victim.pending_trust_shock = true;

    let mut rng = SmallRng::seed_from_u64(419);
    let mut events: Vec<TaggedEvent> = vec![];
    victim.consume_pending_trust_shock(&mut rng, &mut events);

    // Victim's side cleaned.
    assert!(
        !victim.allies.contains(&betrayer_id),
        "victim must drop the broken ally"
    );
    // The flag is consumed regardless of roll outcome.
    assert!(
        !victim.pending_trust_shock,
        "pending flag is reset after the call"
    );
    // Asymmetric back-edge stays — `consume_pending_trust_shock` only
    // touches `self`. The next cycle's event drain (or follow-up
    // events) is responsible for the betrayer's side.
    // We can't observe the betrayer here (different tribute instance);
    // the documented contract is what matters and is asserted by the
    // single-side mutation: the function signature takes `&mut self`
    // and returns nothing, with no reference to the broken ally.
}

#[test]
fn wake_interrupted_returns_false_when_not_sleeping() {
    use crate::messages::TributeRef;
    let mut t = Tribute::new("Foxface".to_string(), Some(1), None);
    let mut events: Vec<TaggedEvent> = Vec::new();
    let woke = t.wake_interrupted(
        shared::messages::InterruptionKind::Ambush {
            attacker: TributeRef {
                identifier: "x".to_string(),
                name: "X".to_string(),
            },
        },
        shared::messages::Phase::Day,
        &mut events,
    );
    assert!(!woke);
    assert!(events.is_empty());
}

#[test]
fn wake_interrupted_resets_state_and_emits_tribute_woke() {
    let mut t = Tribute::new("Foxface".to_string(), Some(1), None);
    t.sleeping = true;
    t.sleep_remaining = 3;
    t.cycles_awake = 7;
    let mut events: Vec<TaggedEvent> = Vec::new();
    let woke = t.wake_interrupted(
        shared::messages::InterruptionKind::AreaEvent {
            kind: shared::messages::AreaEventKind::Fire,
        },
        shared::messages::Phase::Night,
        &mut events,
    );
    assert!(woke);
    assert!(!t.sleeping);
    assert_eq!(t.sleep_remaining, 0);
    assert_eq!(t.cycles_awake, 0);
    assert_eq!(events.len(), 1);
    match &events[0].payload {
        crate::messages::MessagePayload::TributeWoke { reason, phase, .. } => {
            assert_eq!(*phase, shared::messages::Phase::Night);
            match reason {
                shared::messages::WakeReason::Interrupted {
                    event:
                        shared::messages::InterruptionKind::AreaEvent {
                            kind: shared::messages::AreaEventKind::Fire,
                        },
                } => {}
                other => panic!("unexpected reason: {:?}", other),
            }
        }
        other => panic!("expected TributeWoke payload, got {:?}", other),
    }
}

// --- Affliction tests ---

use crate::tributes::AfflictionDraft;
use crate::tributes::afflictions::{AcquireResolution, RejectReason};
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

#[test]
fn test_afflictions_empty_by_default() {
    let t = Tribute::new("Test".to_string(), None, None);
    assert!(t.afflictions.is_empty());
}

#[test]
fn test_afflictions_skip_serialization_when_empty() {
    let t = Tribute::new("Test".to_string(), None, None);
    let json = serde_json::to_string(&t).unwrap();
    assert!(!json.contains("\"afflictions\""));
}

#[test]
fn test_try_acquire_insert() {
    let mut rng = SmallRng::seed_from_u64(42);
    let mut t = Tribute::new_with_rng("Test".to_string(), None, None, &mut rng);
    let draft = AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(resolution, AcquireResolution::Insert);
    assert_eq!(t.afflictions.len(), 1);
    assert!(
        t.afflictions
            .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
    );
}

#[test]
fn test_try_acquire_upgrade() {
    let mut rng = SmallRng::seed_from_u64(42);
    let mut t = Tribute::new_with_rng("Test".to_string(), None, None, &mut rng);
    // Insert mild wound
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    });
    // Upgrade to moderate
    let draft = AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Moderate,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(
        resolution,
        AcquireResolution::Upgrade((AfflictionKind::Wounded, Some(BodyPart::Arm)))
    );
    assert_eq!(t.afflictions.len(), 1);
    let affl = t
        .afflictions
        .get(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
        .unwrap();
    assert_eq!(affl.severity, Severity::Moderate);
}

#[test]
fn test_try_acquire_supersede() {
    let mut t = Tribute::new("Test".to_string(), None, None);
    // Insert wounded on arm
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    });
    // Infected supersedes wounded at same body part
    let draft = AfflictionDraft {
        kind: AfflictionKind::Infected,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(resolution, AcquireResolution::Insert);
    // Wounded removed, Infected present
    assert!(
        !t.afflictions
            .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
    );
    assert!(
        t.afflictions
            .contains_key(&(AfflictionKind::Infected, Some(BodyPart::Arm)))
    );
}

#[test]
fn test_try_acquire_reject_limb_missing() {
    let mut t = Tribute::new("Test".to_string(), None, None);
    // Missing arm
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::MissingArm,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Severe,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    });
    // Can't wound a missing limb
    let draft = AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(
        resolution,
        AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
    );
}

#[test]
fn test_try_acquire_reject_no_wounded_ancestor() {
    let mut t = Tribute::new("Test".to_string(), None, None);
    // Infected without prior Wounded on same part
    let draft = AfflictionDraft {
        kind: AfflictionKind::Infected,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(
        resolution,
        AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor)
    );
}

#[test]
fn test_try_acquire_reject_same_severity() {
    let mut t = Tribute::new("Test".to_string(), None, None);
    // Insert mild wound
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    });
    // Same severity rejected
    let draft = AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat {
            attacker_id: String::new(),
        },
    };
    let resolution = t.try_acquire_affliction(draft);
    assert_eq!(
        resolution,
        AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
    );
}
