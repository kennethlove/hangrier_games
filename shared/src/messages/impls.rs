use super::*;

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        use MessagePayload::*;
        match self {
            TributeKilled { .. } => MessageKind::Death,
            Combat(_) => MessageKind::Combat,
            TributeAttacked { .. } => MessageKind::Combat,
            CombatSwing(_) => MessageKind::CombatSwing,
            AllianceFormed { .. }
            | AllianceProposed { .. }
            | AllianceDissolved { .. }
            | BetrayalTriggered { .. }
            | TrustShockBreak { .. } => MessageKind::Alliance,
            TributeMoved { .. } | TributeHidden { .. } | AreaClosed { .. } | AreaEvent { .. } => {
                MessageKind::Movement
            }
            ItemFound { .. } | ItemUsed { .. } | ItemDropped { .. } => MessageKind::Item,
            SponsorGift { .. } => MessageKind::SponsorGift,
            CycleStart { .. }
            | CycleEnd { .. }
            | PhaseStarted { .. }
            | PhaseEnded { .. }
            | GameEnded { .. } => MessageKind::State,
            TributeWounded { .. }
            | TributeRested { .. }
            | TributeStarved { .. }
            | TributeDehydrated { .. }
            | SanityBreak { .. }
            | HungerBandChanged { .. }
            | ThirstBandChanged { .. }
            | StaminaBandChanged { .. }
            | ShelterSought { .. }
            | Foraged { .. }
            | Drank { .. }
            | Ate { .. }
            | TributeSlept { .. }
            | TributeWoke { .. } => MessageKind::State,
            TraumaAcquired { .. }
            | TraumaReinforced { .. }
            | TraumaEscalated { .. }
            | TraumaFlashback { .. }
            | TraumaAvoidance { .. }
            | TraumaObserved { .. }
            | TraumaForgotten { .. }
            | TraumaHabituated { .. } => MessageKind::Trauma,
            PhobiaAcquired { .. }
            | PhobiaTriggered { .. }
            | PhobiaEscalated { .. }
            | PhobiaHabituated { .. }
            | PhobiaObserved { .. }
            | PhobiaForgotten { .. } => MessageKind::Phobia,
            FixationAcquired { .. }
            | FixationEscalated { .. }
            | FixationFired { .. }
            | FixationConsummated { .. }
            | FixationThwarted { .. }
            | FixationFaded { .. } => MessageKind::Fixation,
            AfflictionAcquired { .. }
            | AfflictionProgressed { .. }
            | AfflictionHealed { .. }
            | AfflictionCascaded { .. }
            | SubstanceUsed { .. }
            | AddictionAcquired { .. }
            | AddictionReinforced { .. }
            | AddictionEscalated { .. }
            | AddictionResisted { .. }
            | AddictionRelapse { .. }
            | AddictionCraving { .. }
            | AddictionObserved { .. }
            | AddictionForgotten { .. }
            | AddictionHabituated { .. } => MessageKind::Affliction,
            TributeTrapped { .. }
            | Struggling { .. }
            | TrappedEscaped { .. }
            | TributeDiedWhileTrapped { .. }
            | RescueAttempted { .. }
            | PartialRescueProgress { .. } => MessageKind::Trapped,
            MessagePayload::TrapSet { .. } => MessageKind::Combat,
            MessagePayload::TrapTriggered { .. } => MessageKind::Combat,
            SleepIncident { .. } => MessageKind::Sleep,
            Generic => MessageKind::State,
        }
    }

    /// True if the payload references the tribute (by identifier). Used by
    /// the per-tribute timeline filter so events involving a given tribute
    /// — as victim, killer, attacker, ally, mover, item handler, etc. —
    /// are kept while everything else is dropped.
    pub fn involves(&self, tribute_identifier: &str) -> bool {
        use MessagePayload::*;
        let id = tribute_identifier;
        let r = |t: &TributeRef| t.identifier == id;
        match self {
            TributeKilled { victim, killer, .. } => r(victim) || killer.as_ref().is_some_and(r),
            TributeWounded {
                victim, attacker, ..
            }
            | TributeAttacked { victim, attacker } => r(victim) || attacker.as_ref().is_some_and(r),
            Combat(engagement) => r(&engagement.attacker) || r(&engagement.target),
            CombatSwing(beat) => r(&beat.attacker) || r(&beat.target),
            AllianceFormed { members } | AllianceDissolved { members, .. } => members.iter().any(r),
            AllianceProposed { proposer, target } => r(proposer) || r(target),
            BetrayalTriggered { betrayer, victim } => r(betrayer) || r(victim),
            TrustShockBreak { tribute, partner } => r(tribute) || r(partner),
            TributeMoved { tribute, .. }
            | TributeHidden { tribute, .. }
            | ItemFound { tribute, .. }
            | ItemUsed { tribute, .. }
            | ItemDropped { tribute, .. }
            | TributeRested { tribute, .. }
            | TributeStarved { tribute, .. }
            | TributeDehydrated { tribute, .. }
            | SanityBreak { tribute }
            | HungerBandChanged { tribute, .. }
            | ThirstBandChanged { tribute, .. }
            | StaminaBandChanged { tribute, .. }
            | ShelterSought { tribute, .. }
            | Foraged { tribute, .. }
            | Drank { tribute, .. }
            | Ate { tribute, .. }
            | TributeSlept { tribute, .. }
            | TributeWoke { tribute, .. }
            | SleepIncident { tribute, .. } => r(tribute),
            AfflictionAcquired { tribute_id, .. }
            | AfflictionProgressed { tribute_id, .. }
            | AfflictionHealed { tribute_id, .. }
            | AfflictionCascaded { tribute_id, .. }
            | TraumaAcquired {
                tribute: tribute_id,
                ..
            }
            | TraumaReinforced {
                tribute: tribute_id,
                ..
            }
            | TraumaEscalated {
                tribute: tribute_id,
                ..
            }
            | TraumaFlashback {
                tribute: tribute_id,
                ..
            }
            | TraumaAvoidance {
                tribute: tribute_id,
                ..
            }
            | TraumaHabituated {
                tribute: tribute_id,
                ..
            }
            | PhobiaAcquired {
                tribute: tribute_id,
                ..
            }
            | PhobiaTriggered {
                tribute: tribute_id,
                ..
            }
            | PhobiaEscalated {
                tribute: tribute_id,
                ..
            }
            | PhobiaHabituated {
                tribute: tribute_id,
                ..
            }
            | FixationAcquired { tribute_id, .. }
            | FixationEscalated { tribute_id, .. }
            | FixationFired { tribute_id, .. }
            | FixationConsummated { tribute_id, .. }
            | FixationThwarted { tribute_id, .. }
            | FixationFaded { tribute_id, .. } => tribute_id == id,
            PhobiaObserved {
                observer, subject, ..
            } => observer == id || subject == id,
            PhobiaForgotten {
                observer, subject, ..
            } => observer == id || subject == id,
            TraumaObserved {
                observer, subject, ..
            } => observer == id || subject == id,
            TraumaForgotten {
                observer, subject, ..
            } => observer == id || subject == id,
            SubstanceUsed { tribute, .. }
            | AddictionAcquired { tribute, .. }
            | AddictionReinforced { tribute, .. }
            | AddictionEscalated { tribute, .. }
            | AddictionResisted { tribute, .. }
            | AddictionRelapse { tribute, .. }
            | AddictionCraving { tribute, .. } => tribute == id,
            AddictionObserved {
                observer, subject, ..
            } => observer == id || subject == id,
            AddictionForgotten {
                observer, subject, ..
            } => observer == id || subject == id,
            AddictionHabituated { tribute, .. } => tribute == id,
            TributeTrapped { tribute, .. }
            | Struggling { tribute, .. }
            | TrappedEscaped { tribute, .. }
            | TributeDiedWhileTrapped { tribute, .. } => tribute == id,
            RescueAttempted {
                rescuer, target, ..
            } => rescuer == id || target == id,
            PartialRescueProgress {
                rescuer, target, ..
            } => rescuer == id || target == id,
            MessagePayload::TrapSet { tribute, .. } => r(tribute),
            MessagePayload::TrapTriggered { victim, .. } => r(victim),
            SponsorGift { recipient, .. } => r(recipient),
            Generic | AreaClosed { .. } | AreaEvent { .. } => false,
            CycleStart { .. } | CycleEnd { .. } | PhaseStarted { .. } | PhaseEnded { .. } => false,
            GameEnded { winner } => winner.as_ref().is_some_and(r),
        }
    }
}
