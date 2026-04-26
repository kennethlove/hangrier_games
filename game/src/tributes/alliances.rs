//! Tribute alliance formation, breaks, and event queue. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §6–§7.
//!
//! Pure functions only. Phase 2 of the tribute-alliances feature. No
//! `Tribute` mutation lives here; later phases wire these helpers into
//! the simulation loop.

use uuid::Uuid;

/// Per-tribute hard cap on direct alliances.
pub const MAX_ALLIES: usize = 5;
/// Base chance per encounter that two tributes form an alliance.
pub const BASE_ALLIANCE_CHANCE: f64 = 0.20;
/// Treacherous betrayal cadence in turns.
pub const TREACHEROUS_BETRAYAL_INTERVAL: u8 = 5;

/// Events emitted by tribute turns and drained by the game cycle. Pure data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllianceEvent {
    BetrayalRecorded {
        betrayer: Uuid,
        victim: Uuid,
    },
    DeathRecorded {
        deceased: Uuid,
        killer: Option<Uuid>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alliance_event_variants_distinct() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let betrayal = AllianceEvent::BetrayalRecorded {
            betrayer: a,
            victim: b,
        };
        let death = AllianceEvent::DeathRecorded {
            deceased: a,
            killer: Some(b),
        };
        assert_ne!(betrayal, death);
        let betrayal2 = AllianceEvent::BetrayalRecorded {
            betrayer: a,
            victim: b,
        };
        assert_eq!(betrayal, betrayal2);
    }

    #[test]
    fn death_event_killer_optional() {
        let id = Uuid::new_v4();
        let unattributed = AllianceEvent::DeathRecorded {
            deceased: id,
            killer: None,
        };
        if let AllianceEvent::DeathRecorded { killer, .. } = unattributed {
            assert!(killer.is_none());
        } else {
            panic!("expected DeathRecorded");
        }
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAX_ALLIES, 5);
        assert!((BASE_ALLIANCE_CHANCE - 0.20).abs() < f64::EPSILON);
        assert_eq!(TREACHEROUS_BETRAYAL_INTERVAL, 5);
    }
}
