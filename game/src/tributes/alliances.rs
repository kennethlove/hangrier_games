//! Tribute alliance formation, breaks, and event queue. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §6–§7.
//!
//! Pure functions only. Phase 2 of the tribute-alliances feature. No
//! `Tribute` mutation lives here; later phases wire these helpers into
//! the simulation loop.

use uuid::Uuid;

use crate::tributes::traits::{REFUSERS, Trait};

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

/// Refuser gate per spec §6.1. Two tributes pass the gate if either both
/// have at least one positive-affinity trait, or neither has any refuser
/// trait. Empty trait sets pass (no refusers).
pub fn passes_gate(self_traits: &[Trait], target_traits: &[Trait]) -> bool {
    let has_positive = |ts: &[Trait]| ts.iter().any(|x| x.alliance_affinity() >= 1.0);
    let has_refuser = |ts: &[Trait]| ts.iter().any(|x| REFUSERS.contains(x));
    (has_positive(self_traits) && has_positive(target_traits))
        || (!has_refuser(self_traits) && !has_refuser(target_traits))
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

    #[test]
    fn paranoid_vs_paranoid_blocked() {
        assert!(!passes_gate(&[Trait::Paranoid], &[Trait::Paranoid]));
    }

    #[test]
    fn lonewolf_vs_friendly_blocked() {
        // LoneWolf affinity 0.6 (no positive) and is a refuser; Friendly is
        // 1.5. (positive AND positive) is false; (no_refuser AND no_refuser)
        // is false because LoneWolf is a refuser. Gate blocks.
        assert!(!passes_gate(&[Trait::LoneWolf], &[Trait::Friendly]));
    }

    #[test]
    fn snake_in_grass_passes_gate() {
        // [Friendly, Paranoid] paired with [Loyal]: both sides have a
        // positive-affinity trait, so the first clause holds.
        assert!(passes_gate(
            &[Trait::Friendly, Trait::Paranoid],
            &[Trait::Loyal],
        ));
    }

    #[test]
    fn empty_traits_pass_gate() {
        assert!(passes_gate(&[], &[]));
    }

    #[test]
    fn neutral_pair_passes_gate() {
        // Tough has affinity 1.0 and is not a refuser; both clauses hold.
        assert!(passes_gate(&[Trait::Tough], &[Trait::Tough]));
    }
}
