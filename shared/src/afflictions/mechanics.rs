use super::severity::Severity;
use rand::RngExt;

/// Outcome of a traumatic reinforcement roll.
/// `escalated` is true only when severity actually increased.
pub struct ReinforcementOutcome {
    pub escalated: bool,
    pub new_severity: Severity,
}

/// Outcome of a decay tick.
/// `new_severity` is `None` when the affliction is cured (dropped off the bottom).
pub struct DecayOutcome {
    pub decayed: bool,
    pub new_severity: Option<Severity>,
}

/// Apply traumatic reinforcement to an affliction severity.
///
/// Rolls against `escalation_chance` (0.0–1.0). On success the severity
/// steps up one tier; `Severe` is the cap and never rolls.
///
/// # Arguments
/// * `current_severity` — the affliction's current severity tier.
/// * `escalation_chance` — probability of stepping up (e.g. 0.12 for 12%).
/// * `rng` — any type implementing `rand::Rng`.
pub fn apply_traumatic_reinforcement(
    current_severity: Severity,
    escalation_chance: f64,
    rng: &mut impl rand::Rng,
) -> ReinforcementOutcome {
    if current_severity == Severity::Severe {
        return ReinforcementOutcome {
            escalated: false,
            new_severity: Severity::Severe,
        };
    }
    if rng.random_bool(escalation_chance) {
        ReinforcementOutcome {
            escalated: true,
            new_severity: current_severity.next_tier(),
        }
    } else {
        ReinforcementOutcome {
            escalated: false,
            new_severity: current_severity,
        }
    }
}

/// Tick decay for a tier-scaled affliction.
///
/// If `cycles_since_last` has not reached `decay_threshold` the affliction
/// holds. Once the threshold is met the severity steps down one tier;
/// `Mild` decays to `None` (cured).
///
/// # Arguments
/// * `current_severity` — the affliction's current severity tier.
/// * `cycles_since_last` — cycles elapsed since the affliction last fired.
/// * `decay_threshold` — cycles required before decay triggers (5 for
///   phobia/fixation, 10 for trauma).
pub fn tick_decay(
    current_severity: Severity,
    cycles_since_last: u32,
    decay_threshold: u32,
) -> DecayOutcome {
    if cycles_since_last < decay_threshold {
        return DecayOutcome {
            decayed: false,
            new_severity: Some(current_severity),
        };
    }
    DecayOutcome {
        decayed: true,
        new_severity: current_severity.prev_tier(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic RNG that always yields the same `u64`.
    /// Implements `TryRng` → blanket impl provides `Rng`.
    struct FixedRng(u64);
    impl rand::TryRng for FixedRng {
        type Error = std::convert::Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok(self.0 as u32)
        }
        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok(self.0)
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
            for chunk in dest.chunks_mut(8) {
                let bytes = self.0.to_le_bytes();
                chunk.copy_from_slice(&bytes[..chunk.len()]);
            }
            Ok(())
        }
    }

    #[test]
    fn reinforcement_mild_to_moderate_on_success() {
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Moderate);
    }

    #[test]
    fn reinforcement_mild_stays_mild_on_failure() {
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 0.5, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Mild);
    }

    #[test]
    fn reinforcement_moderate_to_severe_on_success() {
        let mut rng = FixedRng(0);
        let outcome = apply_traumatic_reinforcement(Severity::Moderate, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    #[test]
    fn reinforcement_severe_stays_severe_capped() {
        let mut rng = FixedRng(0);
        let outcome = apply_traumatic_reinforcement(Severity::Severe, 1.0, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    #[test]
    fn reinforcement_chance_zero_never_esculates() {
        let mut rng = FixedRng(0);
        let outcome = apply_traumatic_reinforcement(Severity::Mild, 0.0, &mut rng);
        assert!(!outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Mild);
    }

    #[test]
    fn reinforcement_chance_one_always_esculates() {
        let mut rng = FixedRng(u64::MAX);
        let outcome = apply_traumatic_reinforcement(Severity::Moderate, 1.0, &mut rng);
        assert!(outcome.escalated);
        assert_eq!(outcome.new_severity, Severity::Severe);
    }

    // ── Decay ──────────────────────────────────────────────────────────

    #[test]
    fn decay_below_threshold_no_decay() {
        let outcome = tick_decay(Severity::Severe, 3, 5);
        assert!(!outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Severe));
    }

    #[test]
    fn decay_at_threshold_severe_to_moderate() {
        let outcome = tick_decay(Severity::Severe, 5, 5);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Moderate));
    }

    #[test]
    fn decay_at_threshold_moderate_to_mild() {
        let outcome = tick_decay(Severity::Moderate, 5, 5);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Mild));
    }

    #[test]
    fn decay_at_threshold_mild_to_cured() {
        let outcome = tick_decay(Severity::Mild, 5, 5);
        assert!(outcome.decayed);
        assert!(outcome.new_severity.is_none());
    }

    #[test]
    fn decay_above_threshold_same_as_at_threshold() {
        let outcome_severe = tick_decay(Severity::Severe, 100, 5);
        assert!(outcome_severe.decayed);
        assert_eq!(outcome_severe.new_severity, Some(Severity::Moderate));

        let outcome_mild = tick_decay(Severity::Mild, 100, 5);
        assert!(outcome_mild.decayed);
        assert!(outcome_mild.new_severity.is_none());
    }

    #[test]
    fn decay_trauma_threshold_10() {
        let outcome = tick_decay(Severity::Severe, 9, 10);
        assert!(!outcome.decayed);

        let outcome = tick_decay(Severity::Severe, 10, 10);
        assert!(outcome.decayed);
        assert_eq!(outcome.new_severity, Some(Severity::Moderate));
    }
}
