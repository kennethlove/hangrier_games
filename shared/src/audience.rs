use serde::{Deserialize, Serialize};

use crate::messages::TributeRef;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudienceEventKind {
    KillMade,
    KillReceived,
    AttackTrapped,
    RescueAlly,
    AllianceFormed,
    BetrayalCommitted,
    AfflictionAcquired,
    SurvivedAreaEvent,
    UnderdogVictory,
    DistrictLoyaltyAct,
    Cowardice,
    TrapSet,
    TrapTriggered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudienceEvent {
    KillMade {
        actor: TributeRef,
        victim: TributeRef,
        magnitude: u32,
        modifier: f32,
    },
    KillReceived {
        victim: TributeRef,
        actor: Option<TributeRef>,
        magnitude: u32,
        modifier: f32,
    },
    AttackTrapped {
        actor: TributeRef,
        victim: TributeRef,
    },
    RescueAlly {
        actor: TributeRef,
        ally: TributeRef,
    },
    AllianceFormed {
        tributes: Vec<TributeRef>,
    },
    BetrayalCommitted {
        actor: TributeRef,
        victim: TributeRef,
    },
    AfflictionAcquired {
        tribute: TributeRef,
        kind: String,
    },
    SurvivedAreaEvent {
        tribute: TributeRef,
    },
    UnderdogVictory {
        actor: TributeRef,
        victim: TributeRef,
    },
    DistrictLoyaltyAct {
        actor: TributeRef,
        district: u8,
    },
    Cowardice {
        tribute: TributeRef,
    },
    /// A tribute sets a trap in an area.
    TrapSet {
        tribute: TributeRef,
    },
    /// A tribute triggers a trap.
    TrapTriggered {
        victim: TributeRef,
    },
}

impl AudienceEvent {
    pub fn kind(&self) -> AudienceEventKind {
        match self {
            Self::KillMade { .. } => AudienceEventKind::KillMade,
            Self::KillReceived { .. } => AudienceEventKind::KillReceived,
            Self::AttackTrapped { .. } => AudienceEventKind::AttackTrapped,
            Self::RescueAlly { .. } => AudienceEventKind::RescueAlly,
            Self::AllianceFormed { .. } => AudienceEventKind::AllianceFormed,
            Self::BetrayalCommitted { .. } => AudienceEventKind::BetrayalCommitted,
            Self::AfflictionAcquired { .. } => AudienceEventKind::AfflictionAcquired,
            Self::SurvivedAreaEvent { .. } => AudienceEventKind::SurvivedAreaEvent,
            Self::UnderdogVictory { .. } => AudienceEventKind::UnderdogVictory,
            Self::DistrictLoyaltyAct { .. } => AudienceEventKind::DistrictLoyaltyAct,
            Self::Cowardice { .. } => AudienceEventKind::Cowardice,
            Self::TrapSet { .. } => AudienceEventKind::TrapSet,
            Self::TrapTriggered { .. } => AudienceEventKind::TrapTriggered,
        }
    }

    /// Base × modifier; floor at 1 to avoid 0-magnitude triggers.
    pub fn magnitude_score(&self) -> u32 {
        let (base, modifier) = match self {
            Self::KillMade {
                magnitude,
                modifier,
                ..
            }
            | Self::KillReceived {
                magnitude,
                modifier,
                ..
            } => (*magnitude, *modifier),
            Self::AttackTrapped { .. } => (6, 1.0),
            Self::RescueAlly { .. } => (5, 1.0),
            Self::AllianceFormed { .. } => (3, 1.0),
            Self::BetrayalCommitted { .. } => (7, 1.0),
            Self::AfflictionAcquired { .. } => (3, 1.0),
            Self::SurvivedAreaEvent { .. } => (4, 1.0),
            Self::UnderdogVictory { .. } => (10, 1.0),
            Self::DistrictLoyaltyAct { .. } => (5, 1.0),
            Self::Cowardice { .. } => (2, 1.0),
            Self::TrapSet { .. } => (3, 1.0),
            Self::TrapTriggered { .. } => (4, 1.0),
        };
        ((base as f32 * modifier).max(1.0)) as u32
    }

    /// Tributes whose affinity-with-sponsor is updated by this event.
    pub fn affected_tributes(&self) -> Vec<&TributeRef> {
        match self {
            Self::KillMade { actor, victim, .. }
            | Self::AttackTrapped { actor, victim }
            | Self::BetrayalCommitted { actor, victim }
            | Self::UnderdogVictory { actor, victim } => vec![actor, victim],
            Self::KillReceived { victim, actor, .. } => match actor {
                Some(a) => vec![victim, a],
                None => vec![victim],
            },
            Self::RescueAlly { actor, ally } => vec![actor, ally],
            Self::AllianceFormed { tributes } => tributes.iter().collect(),
            Self::AfflictionAcquired { tribute, .. }
            | Self::SurvivedAreaEvent { tribute }
            | Self::Cowardice { tribute } => vec![tribute],
            Self::DistrictLoyaltyAct { actor, .. } => vec![actor],
            Self::TrapSet { tribute } => vec![tribute],
            Self::TrapTriggered { victim } => vec![victim],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(name: &str) -> TributeRef {
        TributeRef {
            identifier: name.into(),
            name: name.into(),
        }
    }

    #[test]
    fn kill_made_magnitude_uses_base_times_modifier() {
        let ev = AudienceEvent::KillMade {
            actor: t("a"),
            victim: t("b"),
            magnitude: 5,
            modifier: 2.0,
        };
        assert_eq!(ev.magnitude_score(), 10);
    }

    #[test]
    fn betrayal_kind_roundtrips() {
        let ev = AudienceEvent::BetrayalCommitted {
            actor: t("a"),
            victim: t("b"),
        };
        assert_eq!(ev.kind(), AudienceEventKind::BetrayalCommitted);
    }

    #[test]
    fn alliance_affects_all_members() {
        let ev = AudienceEvent::AllianceFormed {
            tributes: vec![t("a"), t("b"), t("c")],
        };
        assert_eq!(ev.affected_tributes().len(), 3);
    }

    #[test]
    fn magnitude_score_never_zero() {
        let ev = AudienceEvent::KillMade {
            actor: t("a"),
            victim: t("b"),
            magnitude: 0,
            modifier: 0.0,
        };
        assert!(ev.magnitude_score() >= 1);
    }
}
