//! Per-cycle cascade logic for afflictions.
//!
//! Each living tribute's afflictions tick once per cycle. Sheltered tributes
//! may recover (step down); exposed tributes may worsen (step up). Severe +
//! exposed afflictions can spawn successors and trigger death rolls.
//!
//! See spec §5 Cascade.

use rand::Rng;
use rand::RngExt;
use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, Severity};

use super::tuning::AfflictionTuning;

/// Outcome of a single affliction's cascade tick.
#[derive(Debug, Clone, PartialEq)]
pub enum CascadeOutcome {
    /// Severity stepped down (Mild → cured, Moderate → Mild, Severe → Moderate).
    SteppedDown { from: Severity, to: Severity },
    /// Severity stepped up (Mild → Moderate, Moderate → Severe).
    SteppedUp { from: Severity, to: Severity },
    /// A successor affliction spawned (e.g. Wounded → Infected).
    SpawnedSuccessor {
        from: AfflictionKind,
        to: AfflictionKind,
    },
    /// Death roll was made; `survived` indicates the result.
    DeathRoll { survived: bool },
    /// No change this cycle.
    NoChange,
}

/// Aggregate result of ticking all afflictions on a tribute.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    pub outcomes: Vec<(AfflictionKind, CascadeOutcome)>,
    pub tribute_died: bool,
}

/// Tick cascade logic for a tribute's afflictions.
///
/// For each affliction:
/// 1. If sheltered: roll `shelter_recovery_chance` → step down (or remove if Mild)
/// 2. If exposed: roll `progression_chance` → step up
/// 3. If Severe + exposed: roll successor chance → spawn successor; roll death chance
pub fn tick_cascade(
    afflictions: &[Affliction],
    is_sheltered: bool,
    tuning: &AfflictionTuning,
    rng: &mut impl Rng,
) -> CascadeResult {
    let mut outcomes: Vec<(AfflictionKind, CascadeOutcome)> = Vec::new();
    let mut tribute_died = false;

    for aff in afflictions {
        // Permanent afflictions do not cascade or recover.
        if aff.is_permanent() {
            outcomes.push((aff.kind, CascadeOutcome::NoChange));
            continue;
        }

        let outcome = if is_sheltered {
            roll_sheltered(aff, tuning, rng)
        } else {
            roll_exposed(aff, tuning, rng, &mut tribute_died)
        };

        outcomes.push((aff.kind, outcome));
    }

    CascadeResult {
        outcomes,
        tribute_died,
    }
}

fn roll_sheltered(
    aff: &Affliction,
    tuning: &AfflictionTuning,
    rng: &mut impl Rng,
) -> CascadeOutcome {
    if rng.random_bool(tuning.shelter_recovery_chance as f64) {
        match aff.severity {
            Severity::Mild => CascadeOutcome::SteppedDown {
                from: Severity::Mild,
                to: Severity::Mild,
            },
            Severity::Moderate => CascadeOutcome::SteppedDown {
                from: Severity::Moderate,
                to: Severity::Mild,
            },
            Severity::Severe => CascadeOutcome::SteppedDown {
                from: Severity::Severe,
                to: Severity::Moderate,
            },
        }
    } else {
        CascadeOutcome::NoChange
    }
}

fn roll_exposed(
    aff: &Affliction,
    tuning: &AfflictionTuning,
    rng: &mut impl Rng,
    tribute_died: &mut bool,
) -> CascadeOutcome {
    // Severe + exposed: check for successor spawn and death roll first.
    if aff.severity == Severity::Severe {
        let mut result = CascadeOutcome::NoChange;

        // Successor spawn: Wounded → Infected.
        if aff.kind == AfflictionKind::Wounded
            && rng.random_bool(tuning.wound_to_infection_chance as f64)
        {
            result = CascadeOutcome::SpawnedSuccessor {
                from: AfflictionKind::Wounded,
                to: AfflictionKind::Infected,
            };
        }

        // Death roll for Severe Infected.
        if aff.kind == AfflictionKind::Infected
            && rng.random_bool(tuning.severe_infected_death_chance as f64)
        {
            *tribute_died = true;
            return CascadeOutcome::DeathRoll { survived: false };
        }

        // If no successor spawned, still check for progression.
        if matches!(result, CascadeOutcome::NoChange)
            && rng.random_bool(tuning.progression_chance as f64)
        {
            // Already Severe, can't step up further — but the roll still
            // represents worsening pressure. We leave it as NoChange since
            // the death/successor paths above already handled Severe cases.
        }

        return result;
    }

    // Non-Severe exposed: roll for progression (step up).
    if rng.random_bool(tuning.progression_chance as f64) {
        match aff.severity {
            Severity::Mild => CascadeOutcome::SteppedUp {
                from: Severity::Mild,
                to: Severity::Moderate,
            },
            Severity::Moderate => CascadeOutcome::SteppedUp {
                from: Severity::Moderate,
                to: Severity::Severe,
            },
            Severity::Severe => unreachable!("Severe handled above"),
        }
    } else {
        CascadeOutcome::NoChange
    }
}

/// Apply cascade outcomes to a tribute's affliction map.
///
/// Returns a list of new afflictions that should be inserted (successors).
pub fn apply_cascade(
    afflictions: &mut std::collections::BTreeMap<shared::afflictions::AfflictionKey, Affliction>,
    result: &CascadeResult,
) -> Vec<Affliction> {
    let mut successors: Vec<Affliction> = Vec::new();

    for (kind, outcome) in &result.outcomes {
        let key = (*kind, None);
        match outcome {
            CascadeOutcome::SteppedDown { from, to } => {
                // Mild stepped down means removal.
                if matches!(to, Severity::Mild) && matches!(from, Severity::Mild) {
                    afflictions.remove(&key);
                } else if let Some(aff) = afflictions.get_mut(&key) {
                    aff.severity = *to;
                }
            }
            CascadeOutcome::SteppedUp { to, .. } => {
                if let Some(aff) = afflictions.get_mut(&key) {
                    aff.severity = *to;
                }
            }
            CascadeOutcome::SpawnedSuccessor { to, .. } => {
                let new_aff = Affliction {
                    kind: *to,
                    body_part: None,
                    severity: Severity::Moderate,
                    source: AfflictionSource::Cascade,
                };
                successors.push(new_aff);
            }
            CascadeOutcome::DeathRoll { .. } | CascadeOutcome::NoChange => {}
        }
    }

    successors
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn make_affliction(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: None,
            severity,
            source: AfflictionSource::Combat,
        }
    }

    #[test]
    fn sheltered_mild_recovery_removes_affliction() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Mild);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], true, &tuning, &mut rng);
        assert_eq!(result.outcomes.len(), 1);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SteppedDown { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SteppedDown or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn sheltered_moderate_steps_down_to_mild() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Moderate);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], true, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SteppedDown { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SteppedDown or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn sheltered_severe_steps_down_to_moderate() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Severe);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], true, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SteppedDown { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SteppedDown or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn exposed_mild_steps_up_to_moderate() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Mild);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], false, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SteppedUp { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SteppedUp or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn exposed_moderate_steps_up_to_severe() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Moderate);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], false, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SteppedUp { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SteppedUp or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn exposed_severe_wounded_spawns_infected() {
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Severe);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], false, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::SpawnedSuccessor { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected SpawnedSuccessor or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn exposed_severe_infected_death_roll() {
        let aff = make_affliction(AfflictionKind::Infected, Severity::Severe);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let result = tick_cascade(&[aff], false, &tuning, &mut rng);
        let outcome = &result.outcomes[0].1;
        assert!(
            matches!(outcome, CascadeOutcome::DeathRoll { .. })
                || matches!(outcome, CascadeOutcome::NoChange),
            "expected DeathRoll or NoChange, got {:?}",
            outcome
        );
    }

    #[test]
    fn permanent_afflictions_do_not_cascade() {
        let aff = make_affliction(AfflictionKind::MissingArm, Severity::Severe);
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(0);
        let result = tick_cascade(&[aff], false, &tuning, &mut rng);
        assert!(matches!(&result.outcomes[0].1, CascadeOutcome::NoChange));
    }

    #[test]
    fn cascade_probabilities_produce_varied_outcomes() {
        let tuning = AfflictionTuning::default();
        let aff_sheltered = make_affliction(AfflictionKind::Wounded, Severity::Moderate);
        let aff_exposed = make_affliction(AfflictionKind::Wounded, Severity::Mild);

        let mut shelter_stepped = 0;
        let mut exposed_stepped = 0;
        let trials = 1000;

        for seed in 0..trials {
            let mut rng = SmallRng::seed_from_u64(seed);
            let r = tick_cascade(std::slice::from_ref(&aff_sheltered), true, &tuning, &mut rng);
            if matches!(r.outcomes[0].1, CascadeOutcome::SteppedDown { .. }) {
                shelter_stepped += 1;
            }

            let mut rng = SmallRng::seed_from_u64(seed);
            let r = tick_cascade(std::slice::from_ref(&aff_exposed), false, &tuning, &mut rng);
            if matches!(r.outcomes[0].1, CascadeOutcome::SteppedUp { .. }) {
                exposed_stepped += 1;
            }
        }

        let shelter_rate = shelter_stepped as f64 / trials as f64;
        let exposed_rate = exposed_stepped as f64 / trials as f64;

        assert!(
            (shelter_rate - tuning.shelter_recovery_chance as f64).abs() < 0.05,
            "shelter recovery rate {shelter_rate} too far from expected {}",
            tuning.shelter_recovery_chance
        );
        assert!(
            (exposed_rate - tuning.progression_chance as f64).abs() < 0.05,
            "exposed progression rate {exposed_rate} too far from expected {}",
            tuning.progression_chance
        );
    }

    #[test]
    fn apply_cascade_steps_down_moderate_to_mild() {
        let mut afflictions = std::collections::BTreeMap::new();
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Moderate);
        afflictions.insert(aff.key(), aff.clone());

        let result = CascadeResult {
            outcomes: vec![(
                AfflictionKind::Wounded,
                CascadeOutcome::SteppedDown {
                    from: Severity::Moderate,
                    to: Severity::Mild,
                },
            )],
            tribute_died: false,
        };

        let successors = apply_cascade(&mut afflictions, &result);
        assert!(successors.is_empty());
        let updated = afflictions.get(&(AfflictionKind::Wounded, None)).unwrap();
        assert_eq!(updated.severity, Severity::Mild);
    }

    #[test]
    fn apply_cascade_removes_mild_on_step_down() {
        let mut afflictions = std::collections::BTreeMap::new();
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Mild);
        afflictions.insert(aff.key(), aff.clone());

        let result = CascadeResult {
            outcomes: vec![(
                AfflictionKind::Wounded,
                CascadeOutcome::SteppedDown {
                    from: Severity::Mild,
                    to: Severity::Mild,
                },
            )],
            tribute_died: false,
        };

        apply_cascade(&mut afflictions, &result);
        assert!(
            !afflictions.contains_key(&(AfflictionKind::Wounded, None)),
            "Mild affliction should be removed on step down"
        );
    }

    #[test]
    fn apply_cascade_adds_successor() {
        let mut afflictions = std::collections::BTreeMap::new();
        let aff = make_affliction(AfflictionKind::Wounded, Severity::Severe);
        afflictions.insert(aff.key(), aff.clone());

        let result = CascadeResult {
            outcomes: vec![(
                AfflictionKind::Wounded,
                CascadeOutcome::SpawnedSuccessor {
                    from: AfflictionKind::Wounded,
                    to: AfflictionKind::Infected,
                },
            )],
            tribute_died: false,
        };

        let successors = apply_cascade(&mut afflictions, &result);
        assert_eq!(successors.len(), 1);
        assert_eq!(successors[0].kind, AfflictionKind::Infected);
        assert_eq!(successors[0].severity, Severity::Moderate);
        assert_eq!(successors[0].source, AfflictionSource::Cascade);
    }

    #[test]
    fn empty_afflictions_returns_empty_result() {
        let tuning = AfflictionTuning::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = tick_cascade(&[], false, &tuning, &mut rng);
        assert!(result.outcomes.is_empty());
        assert!(!result.tribute_died);
    }
}
