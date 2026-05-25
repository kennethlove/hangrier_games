use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Severity tier for tier-scaled afflictions. Permanent kinds are always
/// `Severe` in practice; tier ordering is total (Mild < Moderate < Severe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
}

impl Severity {
    /// Returns the ordinal value for this severity: 0 for Mild, 1 for Moderate, 2 for Severe.
    pub fn ordinal(&self) -> u8 {
        match self {
            Severity::Mild => 0,
            Severity::Moderate => 1,
            Severity::Severe => 2,
        }
    }

    /// Steps up one severity tier. `Severe` caps at `Severe`.
    pub fn next_tier(&self) -> Self {
        match self {
            Severity::Mild => Severity::Moderate,
            Severity::Moderate => Severity::Severe,
            Severity::Severe => Severity::Severe,
        }
    }

    /// Steps down one severity tier. `Mild` returns `None` (cured).
    pub fn prev_tier(&self) -> Option<Self> {
        match self {
            Severity::Severe => Some(Severity::Moderate),
            Severity::Moderate => Some(Severity::Mild),
            Severity::Mild => None,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Mild => write!(f, "mild"),
            Severity::Moderate => write!(f, "moderate"),
            Severity::Severe => write!(f, "severe"),
        }
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mild" => Ok(Severity::Mild),
            "moderate" => Ok(Severity::Moderate),
            "severe" => Ok(Severity::Severe),
            other => Err(format!("unknown Severity: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering_is_correct() {
        assert_eq!(Severity::Mild.ordinal(), 0);
        assert_eq!(Severity::Moderate.ordinal(), 1);
        assert_eq!(Severity::Severe.ordinal(), 2);
        assert!(Severity::Mild < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Severe);
    }

    #[test]
    fn severity_next_tier_steps_up_correctly() {
        assert_eq!(Severity::Mild.next_tier(), Severity::Moderate);
        assert_eq!(Severity::Moderate.next_tier(), Severity::Severe);
        assert_eq!(Severity::Severe.next_tier(), Severity::Severe);
    }

    #[test]
    fn severity_prev_tier_steps_down_correctly() {
        assert_eq!(Severity::Severe.prev_tier(), Some(Severity::Moderate));
        assert_eq!(Severity::Moderate.prev_tier(), Some(Severity::Mild));
        assert_eq!(Severity::Mild.prev_tier(), None);
    }
}
