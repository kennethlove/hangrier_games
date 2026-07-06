use super::*;
use crate::ids::TributeId;

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        MessageKind::from(self)
    }

    pub fn tribute_refs(&self) -> Vec<&TributeRef> {
        use MessagePayload::*;
        let mut refs = Vec::new();
        match self {
            TributeKilled { victim, killer, .. } => {
                refs.push(victim);
                if let Some(k) = killer {
                    refs.push(k);
                }
            }
            TributeWounded {
                victim, attacker, ..
            }
            | TributeAttacked { victim, attacker } => {
                refs.push(victim);
                if let Some(a) = attacker {
                    refs.push(a);
                }
            }
            Combat(e) => {
                refs.push(&e.attacker);
                refs.push(&e.target);
            }
            CombatSwing(b) => {
                refs.push(&b.attacker);
                refs.push(&b.target);
            }
            AllianceFormed { members } | AllianceDissolved { members, .. } => refs.extend(members),
            AllianceProposed { proposer, target } => {
                refs.push(proposer);
                refs.push(target);
            }
            BetrayalTriggered { betrayer, victim } => {
                refs.push(betrayer);
                refs.push(victim);
            }
            TrustShockBreak { tribute, partner } => {
                refs.push(tribute);
                refs.push(partner);
            }
            TributeMoved { tribute, .. }
            | TributeHidden { tribute, .. }
            | ItemFound { tribute, .. }
            | ItemUsed { tribute, .. }
            | ItemDropped { tribute, .. }
            | TributeRested { tribute, .. }
            | TributeStarved { tribute, .. }
            | TributeDehydrated { tribute, .. }
            | SanityBreak { tribute }
            | TributeBledOut { tribute }
            | WoundInfected { tribute, .. }
            | WoundHealed { tribute, .. }
            | HungerBandChanged { tribute, .. }
            | ThirstBandChanged { tribute, .. }
            | StaminaBandChanged { tribute, .. }
            | ShelterSought { tribute, .. }
            | Foraged { tribute, .. }
            | Drank { tribute, .. }
            | Ate { tribute, .. }
            | TributeSlept { tribute, .. }
            | TributeWoke { tribute, .. }
            | SleepIncident { tribute, .. }
            | TrapSet { tribute, .. } => refs.push(tribute),
            TrapTriggered { victim, .. } => refs.push(victim),
            SponsorGift { recipient, .. } => refs.push(recipient),
            GameEnded { winner } => {
                if let Some(w) = winner {
                    refs.push(w);
                }
            }
            AfflictionAcquired { .. }
            | AfflictionProgressed { .. }
            | AfflictionHealed { .. }
            | AfflictionCascaded { .. }
            | TraumaAcquired { .. }
            | TraumaReinforced { .. }
            | TraumaEscalated { .. }
            | TraumaFlashback { .. }
            | TraumaAvoidance { .. }
            | TraumaHabituated { .. }
            | PhobiaAcquired { .. }
            | PhobiaTriggered { .. }
            | PhobiaEscalated { .. }
            | PhobiaHabituated { .. }
            | FixationAcquired { .. }
            | FixationEscalated { .. }
            | FixationFired { .. }
            | FixationConsummated { .. }
            | FixationThwarted { .. }
            | FixationFaded { .. }
            | SubstanceUsed { .. }
            | AddictionAcquired { .. }
            | AddictionReinforced { .. }
            | AddictionEscalated { .. }
            | AddictionResisted { .. }
            | AddictionRelapse { .. }
            | AddictionCraving { .. }
            | AddictionHabituated { .. }
            | TributeTrapped { .. }
            | Struggling { .. }
            | TrappedEscaped { .. }
            | TributeDiedWhileTrapped { .. }
            | RescueAttempted { .. }
            | PartialRescueProgress { .. }
            | PhobiaObserved { .. }
            | PhobiaForgotten { .. }
            | TraumaObserved { .. }
            | TraumaForgotten { .. }
            | AddictionObserved { .. }
            | AddictionForgotten { .. }
            | WoundInflicted { .. }
            | WoundBled { .. }
            | WoundTreated { .. }
            | WoundAmputated { .. }
            | ConditionAcquired { .. }
            | ConditionResolved { .. }
            | TributeDesperate { .. }
            | Generic
            | AreaClosed { .. }
            | AreaEvent { .. }
            | CycleStart { .. }
            | CycleEnd { .. }
            | PhaseStarted { .. }
            | PhaseEnded { .. } => {}
        }
        refs
    }

    pub fn involves(&self, tribute_identifier: &str) -> bool {
        let id = tribute_identifier.parse::<TributeId>().unwrap();
        if self.tribute_refs().iter().any(|t| t.identifier == id) {
            return true;
        }
        use MessagePayload::*;
        match self {
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
            | FixationFaded { tribute_id, .. }
            | SubstanceUsed {
                tribute: tribute_id,
                ..
            }
            | AddictionAcquired {
                tribute: tribute_id,
                ..
            }
            | AddictionReinforced {
                tribute: tribute_id,
                ..
            }
            | AddictionEscalated {
                tribute: tribute_id,
                ..
            }
            | AddictionResisted {
                tribute: tribute_id,
                ..
            }
            | AddictionRelapse {
                tribute: tribute_id,
                ..
            }
            | AddictionCraving {
                tribute: tribute_id,
                ..
            }
            | AddictionHabituated {
                tribute: tribute_id,
                ..
            }
            | TributeTrapped {
                tribute: tribute_id,
                ..
            }
            | Struggling {
                tribute: tribute_id,
                ..
            }
            | TrappedEscaped {
                tribute: tribute_id,
                ..
            }
            | TributeDiedWhileTrapped {
                tribute: tribute_id,
                ..
            } => tribute_id.parse::<TributeId>().ok() == Some(id),
            PhobiaObserved {
                observer, subject, ..
            }
            | PhobiaForgotten {
                observer, subject, ..
            }
            | TraumaObserved {
                observer, subject, ..
            }
            | TraumaForgotten {
                observer, subject, ..
            }
            | AddictionObserved {
                observer, subject, ..
            }
            | AddictionForgotten {
                observer, subject, ..
            } => {
                observer.parse::<TributeId>().ok() == Some(id.clone())
                    || subject.parse::<TributeId>().ok() == Some(id)
            }
            RescueAttempted {
                rescuer, target, ..
            }
            | PartialRescueProgress {
                rescuer, target, ..
            } => {
                rescuer.parse::<TributeId>().ok() == Some(id.clone())
                    || target.parse::<TributeId>().ok() == Some(id)
            }
            _ => false,
        }
    }
}
