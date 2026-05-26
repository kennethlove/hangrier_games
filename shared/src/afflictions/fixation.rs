use serde::{Deserialize, Serialize};
use std::fmt;

/// Origin of a fixation affliction. Innate fixations are lifelong dispositions;
/// Acquired fixations develop through interaction with the target.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixationOrigin {
    Innate,
    Acquired { event_ref: String },
}

impl fmt::Display for FixationOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixationOrigin::Innate => write!(f, "innate"),
            FixationOrigin::Acquired { event_ref } => write!(f, "acquired:{event_ref}"),
        }
    }
}

/// Metadata attached to Fixation afflictions. Tracks origin.
/// Only populated for Fixation kinds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixationMetadata {
    pub origin: FixationOrigin,
}

/// Reasons a fixation can be thwarted — the target is no longer relevant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThwartReason {
    TargetLost,
    TargetUnreachable,
}

impl fmt::Display for ThwartReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThwartReason::TargetLost => write!(f, "target_lost"),
            ThwartReason::TargetUnreachable => write!(f, "target_unreachable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixation_origin_roundtrip_innate() {
        let origin = FixationOrigin::Innate;
        let json = serde_json::to_string(&origin).unwrap();
        let restored: FixationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(origin, restored);
    }

    #[test]
    fn fixation_origin_roundtrip_acquired() {
        let origin = FixationOrigin::Acquired {
            event_ref: "pickup:item-1".to_string(),
        };
        let json = serde_json::to_string(&origin).unwrap();
        let restored: FixationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(origin, restored);
    }

    #[test]
    fn fixation_origin_display_innate() {
        assert_eq!(FixationOrigin::Innate.to_string(), "innate");
    }

    #[test]
    fn fixation_origin_display_acquired() {
        let origin = FixationOrigin::Acquired {
            event_ref: "pickup:item-1".to_string(),
        };
        assert_eq!(origin.to_string(), "acquired:pickup:item-1");
    }
}
