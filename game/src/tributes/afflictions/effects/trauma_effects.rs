//! Trauma-specific effect functions: flashback chance, sleep recovery,
//! and avoidance hard veto.

use shared::afflictions::Severity;

/// Chance of a flashback occurring when the trauma stimulus is present.
/// Values from spec §7.2: Mild=5%, Moderate=10%, Severe=20%.
pub fn flashback_chance(severity: Severity) -> f64 {
    match severity {
        Severity::Mild => 0.05,
        Severity::Moderate => 0.10,
        Severity::Severe => 0.20,
    }
}

/// Sleep recovery multiplier from trauma.
/// Spec §6: Moderate=50% recovery, Severe=25% recovery.
/// Returns 1.0 for Mild (no penalty).
pub fn sleep_recovery_multiplier(severity: Severity) -> f64 {
    match severity {
        Severity::Mild => 1.0,
        Severity::Moderate => 0.5,
        Severity::Severe => 0.25,
    }
}

/// Whether trauma severity produces a hard avoidance veto.
/// Spec §7.2: Only Severe produces a hard veto.
pub fn avoidance_hard_veto(severity: Severity) -> bool {
    matches!(severity, Severity::Severe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flashback_chance_table() {
        assert!((flashback_chance(Severity::Mild) - 0.05).abs() < f64::EPSILON);
        assert!((flashback_chance(Severity::Moderate) - 0.10).abs() < f64::EPSILON);
        assert!((flashback_chance(Severity::Severe) - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn sleep_recovery_multiplier_table() {
        assert!((sleep_recovery_multiplier(Severity::Mild) - 1.0).abs() < f64::EPSILON);
        assert!((sleep_recovery_multiplier(Severity::Moderate) - 0.5).abs() < f64::EPSILON);
        assert!((sleep_recovery_multiplier(Severity::Severe) - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn avoidance_veto_only_at_severe() {
        assert!(!avoidance_hard_veto(Severity::Mild));
        assert!(!avoidance_hard_veto(Severity::Moderate));
        assert!(avoidance_hard_veto(Severity::Severe));
    }
}
