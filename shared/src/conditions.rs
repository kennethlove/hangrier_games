use serde::{Deserialize, Serialize};
use std::fmt;

/// Mental conditions that affect a tribute's sanity and behavior.
/// Conditions are recalculated each period based on wounds, stress, and state.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MentalCondition {
    /// Pain from active wounds. Severity scales with wound count and severity.
    /// Reduces effective sanity and may cause panic at high levels.
    Pain { severity: ConditionSeverity },
    /// Horrified from witnessing violence (kills, severe wounds).
    /// Temporary condition that fades over time.
    Horrified {
        severity: ConditionSeverity,
        remaining_periods: u32,
    },
    /// Panicking from extreme stress or very low sanity.
    /// Causes unpredictable behavior.
    Panicking { severity: ConditionSeverity },
    /// Despondent from prolonged suffering or isolation.
    /// Reduces willingness to fight or cooperate.
    Despondent { severity: ConditionSeverity },
}

/// Severity tiers for mental conditions.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConditionSeverity {
    Mild,
    Moderate,
    Severe,
    Critical,
}

impl ConditionSeverity {
    /// Sanity penalty per period for this severity.
    pub fn sanity_drain(&self) -> u32 {
        match self {
            ConditionSeverity::Mild => 1,
            ConditionSeverity::Moderate => 3,
            ConditionSeverity::Severe => 5,
            ConditionSeverity::Critical => 10,
        }
    }
}

impl fmt::Display for ConditionSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionSeverity::Mild => write!(f, "mild"),
            ConditionSeverity::Moderate => write!(f, "moderate"),
            ConditionSeverity::Severe => write!(f, "severe"),
            ConditionSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl fmt::Display for MentalCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MentalCondition::Pain { severity } => write!(f, "pain ({severity})"),
            MentalCondition::Horrified {
                severity,
                remaining_periods,
            } => {
                write!(
                    f,
                    "horrified ({severity}, {remaining_periods} periods left)"
                )
            }
            MentalCondition::Panicking { severity } => write!(f, "panicking ({severity})"),
            MentalCondition::Despondent { severity } => write!(f, "despondent ({severity})"),
        }
    }
}

impl MentalCondition {
    /// Returns the severity of this condition.
    pub fn severity(&self) -> ConditionSeverity {
        match self {
            MentalCondition::Pain { severity }
            | MentalCondition::Horrified { severity, .. }
            | MentalCondition::Panicking { severity }
            | MentalCondition::Despondent { severity } => *severity,
        }
    }

    /// Whether this condition has expired (e.g. Horrified with 0 remaining periods).
    pub fn is_expired(&self) -> bool {
        match self {
            MentalCondition::Horrified {
                remaining_periods, ..
            } => *remaining_periods == 0,
            _ => false,
        }
    }

    /// Tick this condition forward one period. Returns whether it expired.
    pub fn tick(&mut self) -> bool {
        match self {
            MentalCondition::Horrified {
                remaining_periods, ..
            } => {
                if *remaining_periods > 0 {
                    *remaining_periods -= 1;
                }
                *remaining_periods == 0
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pain_display() {
        let p = MentalCondition::Pain {
            severity: ConditionSeverity::Moderate,
        };
        assert_eq!(p.to_string(), "pain (moderate)");
    }

    #[test]
    fn horrified_ticks_down() {
        let mut h = MentalCondition::Horrified {
            severity: ConditionSeverity::Severe,
            remaining_periods: 2,
        };
        assert!(!h.tick());
        assert!(!h.is_expired());
        assert!(h.tick());
        assert!(h.is_expired());
    }

    #[test]
    fn pain_never_expires() {
        let mut p = MentalCondition::Pain {
            severity: ConditionSeverity::Critical,
        };
        assert!(!p.tick());
        assert!(!p.is_expired());
    }

    #[test]
    fn severity_sanity_drain() {
        assert_eq!(ConditionSeverity::Mild.sanity_drain(), 1);
        assert_eq!(ConditionSeverity::Moderate.sanity_drain(), 3);
        assert_eq!(ConditionSeverity::Severe.sanity_drain(), 5);
        assert_eq!(ConditionSeverity::Critical.sanity_drain(), 10);
    }
}
