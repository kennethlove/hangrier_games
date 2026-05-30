//! Rolling per-tribute digest.
//!
//! Tracks a rolling summary of each tribute's status, location, allies, and
//! notable events — updated every phase from the phase's `GameMessage` list.
//! Capped at 8 notable events per tribute (oldest pruned).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shared::combat_beat::SwingOutcome;
use shared::messages::{GameMessage, MessagePayload, TributeRef};

use crate::types::TributeDigest;

/// Maximum number of notable events retained per tribute.
///
/// At 4 phases per day, 30 events covers ~7.5 game-days — enough for the
/// announcers to reference a tribute's full arc in a typical game. Older
/// events are pruned oldest-first when the cap is exceeded.
/// Games longer than this will still have the most recent ~week visible,
/// which keeps the LLM prompt size bounded while preserving narrative
/// continuity.
const MAX_NOTABLE_EVENTS: usize = 30;

// ---------------------------------------------------------------------------
// TributeHistories
// ---------------------------------------------------------------------------

/// Rolling digest collection, keyed by tribute identifier.
///
/// ```rust,ignore
/// let mut histories = TributeHistories::new(initial_digests);
/// histories.update(&phase_events);
/// let digests = histories.digests();  // sorted by name
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TributeHistories {
    inner: HashMap<String, TributeDigest>,
}

impl TributeHistories {
    /// Create a new history tracker from an initial tribute roster.
    ///
    /// Each `TributeDigest` in the roster should have `name`, `district`,
    /// and the default `status: "alive"`, `injury_level: "unharmed"`,
    /// and an empty `location`.
    pub fn new(tributes: Vec<TributeDigest>) -> Self {
        let inner = tributes
            .into_iter()
            .map(|t| (t.identifier.clone(), t))
            .collect();
        Self { inner }
    }

    /// Process one phase's messages and update every referenced tribute.
    ///
    /// For each `GameMessage`, the payload is inspected and the involved
    /// tribute(s) get their digest updated:
    ///
    /// * **Deaths** — mark as deceased, append to victim + killer histories.
    /// * **Combat** — append to attacker + target histories.
    /// * **Alliance** — update allies lists, append to member histories.
    /// * **Movement** — update location, append to tribute history.
    /// * **Items, state, survival** — prose-only append to tribute history.
    pub fn update(&mut self, events: &[GameMessage]) {
        for msg in events {
            match &msg.payload {
                // ------- Lifecycle -------
                MessagePayload::TributeKilled {
                    victim,
                    killer,
                    cause,
                } => {
                    self.set_status(&victim.identifier, "deceased");
                    self.set_injury_level(&victim.identifier, "deceased");
                    self.push_event(
                        &victim.identifier,
                        &format!("Killed by {}", killer_name(killer)),
                    );
                    if let Some(k) = killer {
                        self.push_event(
                            &k.identifier,
                            &format!("Killed {} (cause: {cause})", victim.name),
                        );
                        self.push_highlight(
                            &k.identifier,
                            &format!("Killed {} ({})", victim.name, cause),
                        );
                        // Bump killer's spree streak; victim's resets on death.
                        self.bump_streak(&k.identifier);
                    }
                    self.reset_streak(&victim.identifier);
                }

                MessagePayload::TributeWounded {
                    victim,
                    attacker,
                    hp_lost,
                } => {
                    let sev = crate::severity::describe_damage(*hp_lost);
                    self.set_injury_level(&victim.identifier, sev);
                    self.push_event(
                        &victim.identifier,
                        &format!("Wounded ({sev} hit) by {}", attacker_name(attacker)),
                    );
                    // Victim got hurt — their streak breaks. Attacker won the
                    // exchange so their streak holds steady (no increment, no
                    // reset). Only a kill increments; only getting killed or
                    // wounded yourself resets.
                    self.reset_streak(&victim.identifier);
                    if let Some(a) = attacker {
                        self.push_event(
                            &a.identifier,
                            &format!("Landed a {sev} hit on {}", victim.name),
                        );
                    }
                }

                MessagePayload::TributeAttacked { victim, attacker } => {
                    self.push_event(
                        &victim.identifier,
                        &format!("Attacked by {}", attacker_name(attacker)),
                    );
                    // Victim got jumped — streak breaks. Attacker initiated
                    // successfully so their streak holds.
                    self.reset_streak(&victim.identifier);
                    if let Some(a) = attacker {
                        self.push_event(
                            &a.identifier,
                            &format!("Attacked {}", victim.name),
                        );
                    }
                }

                // ------- Combat -------
                MessagePayload::Combat(engagement) => {
                    let outcome_label = match &engagement.outcome {
                        shared::messages::CombatOutcome::Killed => "killed",
                        shared::messages::CombatOutcome::Wounded => {
                            // Target lost — streak breaks. Attacker won the
                            // exchange (even without a kill) so streak holds.
                            self.reset_streak(&engagement.target.identifier);
                            "wounded"
                        }
                        shared::messages::CombatOutcome::TargetFled => {
                            // Attacker won by forcing a retreat — streak holds.
                            self.reset_streak(&engagement.target.identifier);
                            "target fled"
                        }
                        shared::messages::CombatOutcome::AttackerFled => {
                            // Attacker lost — streak breaks.
                            self.reset_streak(&engagement.attacker.identifier);
                            "attacker fled"
                        }
                        shared::messages::CombatOutcome::Stalemate => {
                            // Draw — neither side clearly lost.
                            "stalemate"
                        }
                    };
                    self.push_event(
                        &engagement.attacker.identifier,
                        &format!(
                            "Combat vs {} — outcome: {outcome_label}",
                            engagement.target.name
                        ),
                    );
                    self.push_event(
                        &engagement.target.identifier,
                        &format!(
                            "Combat vs {} — outcome: {outcome_label}",
                            engagement.attacker.name
                        ),
                    );
                }

                MessagePayload::CombatSwing(beat) => {
                    let outcome_label = match &beat.outcome {
                        SwingOutcome::Wound { .. } => "hit",
                        SwingOutcome::Miss => "missed",
                        SwingOutcome::CriticalHitWound { .. } => "critical hit",
                        SwingOutcome::BlockWound { .. } => "blocked",
                        SwingOutcome::SelfAttackWound { .. } => "self-wound",
                        SwingOutcome::Suicide { .. } => "suicide",
                        SwingOutcome::FumbleSurvive { .. } => "fumble (survived)",
                        SwingOutcome::FumbleDeath { .. } => "fumble (died)",
                        SwingOutcome::AttackerDied { .. } => "attacker died",
                        SwingOutcome::Kill { .. } => "kill",
                    };
                    self.push_event(
                        &beat.attacker.identifier,
                        &format!("Swung at {} — {outcome_label}", beat.target.name),
                    );
                    self.push_event(
                        &beat.target.identifier,
                        &format!("Target of {}'s swing — {outcome_label}", beat.attacker.name),
                    );
                }

                // ------- Alliance -------
                MessagePayload::AllianceFormed { members } => {
                    for m in members {
                        let allies: Vec<String> = members
                            .iter()
                            .filter(|o| o.identifier != m.identifier)
                            .map(|o| o.name.clone())
                            .collect();
                        if let Some(digest) = self.inner.get_mut(&m.identifier) {
                            for ally in &allies {
                                if !digest.allies.contains(ally) {
                                    digest.allies.push(ally.clone());
                                }
                            }
                        }
                        self.push_event(
                            &m.identifier,
                            &format!("Formed alliance with {}", join_names(&allies)),
                        );
                        self.push_highlight(
                            &m.identifier,
                            &format!("Allied with {}", join_names(&allies)),
                        );
                    }
                }

                MessagePayload::AllianceProposed { proposer, target } => {
                    self.push_event(
                        &proposer.identifier,
                        &format!("Proposed alliance to {}", target.name),
                    );
                    self.push_event(
                        &target.identifier,
                        &format!("Received alliance proposal from {}", proposer.name),
                    );
                }

                MessagePayload::AllianceDissolved { members, reason } => {
                    for m in members {
                        if let Some(digest) = self.inner.get_mut(&m.identifier) {
                            digest.allies.clear();
                        }
                        self.push_event(&m.identifier, &format!("Alliance dissolved — {reason}"));
                    }
                }

                MessagePayload::BetrayalTriggered { betrayer, victim } => {
                    self.push_event(
                        &betrayer.identifier,
                        &format!("Betrayed {}", victim.name),
                    );
                    self.push_highlight(
                        &betrayer.identifier,
                        &format!("Betrayed {}", victim.name),
                    );
                    self.push_event(
                        &victim.identifier,
                        &format!("Betrayed by {}", betrayer.name),
                    );
                    self.push_highlight(
                        &victim.identifier,
                        &format!("Betrayed by {}", betrayer.name),
                    );
                }

                MessagePayload::TrustShockBreak { tribute, partner } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Trust shattered with {}", partner.name),
                    );
                }

                // ------- Movement -------
                MessagePayload::TributeMoved { tribute, to, .. } => {
                    self.set_location(&tribute.identifier, &to.name);
                    self.push_event(
                        &tribute.identifier,
                        &format!("Moved to {}", to.name),
                    );
                }

                MessagePayload::TributeHidden { tribute, area } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Hiding in {}", area.name),
                    );
                }

                // ------- Items -------
                MessagePayload::ItemFound { tribute, item, area } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Found {} in {}", item.name, area.name),
                    );
                }

                MessagePayload::ItemUsed { tribute, item } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Used {}", item.name),
                    );
                }

                MessagePayload::ItemDropped { tribute, item, area } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Dropped {} in {}", item.name, area.name),
                    );
                }

                MessagePayload::SponsorGift {
                    recipient, item, ..
                } => {
                    self.push_event(
                        &recipient.identifier,
                        &format!("Received {} from sponsor", item.name),
                    );
                }

                // ------- State / survival -------
                MessagePayload::TributeRested { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Rested");
                }

                MessagePayload::TributeStarved { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Suffered starvation damage");
                }

                MessagePayload::TributeDehydrated { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Suffered dehydration damage");
                }

                MessagePayload::SanityBreak { tribute } => {
                    self.push_event(&tribute.identifier, "Suffered a sanity break");
                }

                MessagePayload::HungerBandChanged { tribute, to, .. } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Hunger level: {to:?}"),
                    );
                }

                MessagePayload::ThirstBandChanged { tribute, to, .. } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Thirst level: {to:?}"),
                    );
                }

                MessagePayload::StaminaBandChanged { tribute, to, .. } => {
                    self.push_event(
                        &tribute.identifier,
                        &format!("Stamina level: {to:?}"),
                    );
                }

                MessagePayload::ShelterSought { tribute, success, .. } => {
                    if *success {
                        self.push_event(&tribute.identifier, "Found shelter");
                    } else {
                        self.push_event(&tribute.identifier, "Failed to find shelter");
                    }
                }

                MessagePayload::Foraged { tribute, success, .. } => {
                    if *success {
                        self.push_event(&tribute.identifier, "Successfully foraged");
                    } else {
                        self.push_event(&tribute.identifier, "Found nothing foraging");
                    }
                }

                MessagePayload::Drank { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Drank water");
                }

                MessagePayload::Ate { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Ate food");
                }

                // ------- Sleep -------
                MessagePayload::TributeSlept { tribute, .. } => {
                    self.push_event(&tribute.identifier, "Went to sleep");
                }

                MessagePayload::TributeWoke { tribute, reason, .. } => {
                    let label = match reason {
                        shared::messages::WakeReason::Rested => "after resting",
                        shared::messages::WakeReason::Interrupted { .. } => "interrupted",
                    };
                    self.push_event(
                        &tribute.identifier,
                        &format!("Woke up ({label})"),
                    );
                }

                // ------- Afflictions (string-keyed) -------
                MessagePayload::AfflictionAcquired {
                    tribute_id, affliction, ..
                } => {
                    self.push_event(tribute_id, &format!("Acquired affliction: {affliction}"));
                }

                MessagePayload::AfflictionProgressed {
                    tribute_id,
                    affliction,
                    to_severity,
                    ..
                } => {
                    self.push_event(
                        tribute_id,
                        &format!("Affliction {affliction} worsened to {to_severity}"),
                    );
                }

                MessagePayload::AfflictionHealed {
                    tribute_id, affliction, ..
                } => {
                    self.push_event(tribute_id, &format!("Affliction healed: {affliction}"));
                }

                MessagePayload::AfflictionCascaded {
                    tribute_id,
                    from_affliction,
                    to_affliction,
                    ..
                } => {
                    self.push_event(
                        tribute_id,
                        &format!("{from_affliction} cascaded into {to_affliction}"),
                    );
                }

                // ------- Trauma (string-keyed) -------
                MessagePayload::TraumaAcquired {
                    tribute, severity, ..
                } => {
                    self.push_event(tribute, &format!("Acquired trauma ({severity})"));
                }

                MessagePayload::TraumaReinforced {
                    tribute,
                    to_severity,
                    ..
                } => {
                    self.push_event(
                        tribute,
                        &format!("Trauma reinforced to {to_severity}"),
                    );
                }

                MessagePayload::TraumaFlashback {
                    tribute, severity, ..
                } => {
                    self.push_event(tribute, &format!("Trauma flashback ({severity})"));
                }

                MessagePayload::TraumaAvoidance {
                    tribute, source, ..
                } => {
                    self.push_event(
                        tribute,
                        &format!("Avoided action due to {source} trauma"),
                    );
                }

                // ------- Phobia (string-keyed) -------
                MessagePayload::PhobiaAcquired {
                    tribute, trigger, ..
                } => {
                    self.push_event(tribute, &format!("Developed phobia of {trigger}"));
                }

                MessagePayload::PhobiaTriggered {
                    tribute, trigger, ..
                } => {
                    self.push_event(tribute, &format!("Phobia triggered by {trigger}"));
                }

                // ------- Fixation (string-keyed) -------
                MessagePayload::FixationAcquired {
                    tribute_id, target, ..
                } => {
                    self.push_event(tribute_id, &format!("Fixated on {target}"));
                }

                MessagePayload::FixationFired {
                    tribute_id, target, ..
                } => {
                    self.push_event(tribute_id, &format!("Fixation on {target} driving actions"));
                }

                MessagePayload::FixationConsummated {
                    tribute_id, target, ..
                } => {
                    self.push_event(tribute_id, &format!("Fixation on {target} consummated"));
                }

                // ------- Addiction (string-keyed) -------
                MessagePayload::AddictionAcquired {
                    tribute, substance, ..
                } => {
                    self.push_event(tribute, &format!("Acquired addiction: {substance}"));
                }

                MessagePayload::SubstanceUsed {
                    tribute, substance, ..
                } => {
                    self.push_event(tribute, &format!("Used substance: {substance}"));
                }

                MessagePayload::AddictionCraving {
                    tribute, substance, ..
                } => {
                    self.push_event(tribute, &format!("Craving {substance}"));
                }

                // ------- Cycle / phase: skip (no tribute-specific content) -------
                MessagePayload::CycleStart { .. }
                | MessagePayload::CycleEnd { .. }
                | MessagePayload::PhaseStarted { .. }
                | MessagePayload::PhaseEnded { .. }
                | MessagePayload::GameEnded { .. } => {}

                // ------- Observation events (don't add to subject's history) -------
                MessagePayload::PhobiaObserved { .. }
                | MessagePayload::PhobiaForgotten { .. }
                | MessagePayload::TraumaObserved { .. }
                | MessagePayload::TraumaForgotten { .. }
                | MessagePayload::AddictionObserved { .. }
                | MessagePayload::AddictionForgotten { .. }
                | MessagePayload::AddictionHabituated { .. }
                | MessagePayload::AddictionRelapse { .. }
                | MessagePayload::AddictionResisted { .. }
                | MessagePayload::AddictionReinforced { .. }
                | MessagePayload::AddictionEscalated { .. }
                | MessagePayload::TraumaEscalated { .. }
                | MessagePayload::TraumaHabituated { .. }
                | MessagePayload::PhobiaEscalated { .. }
                | MessagePayload::PhobiaHabituated { .. }
                | MessagePayload::FixationEscalated { .. }
                | MessagePayload::FixationFaded { .. }
                | MessagePayload::FixationThwarted { .. }
                | MessagePayload::AreaEvent { .. }
                | MessagePayload::AreaClosed { .. } => {}

            }
        }
    }

    /// Return all digests, sorted alphabetically by tribute name.
    pub fn digests(&self) -> Vec<TributeDigest> {
        let mut digests: Vec<TributeDigest> = self.inner.values().cloned().collect();
        digests.sort_by(|a, b| a.name.cmp(&b.name));
        digests
    }

    /// Return the digest for a specific tribute, if present.
    pub fn get(&self, name: &str) -> Option<&TributeDigest> {
        self.inner.get(name)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn set_status(&mut self, id: &str, status: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            digest.status = status.to_string();
        }
    }

    fn set_injury_level(&mut self, id: &str, level: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            digest.injury_level = level.to_string();
        }
    }

    fn set_location(&mut self, id: &str, location: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            digest.location = location.to_string();
        }
    }

    /// Append a notable event for a tribute. Capped at `MAX_NOTABLE_EVENTS`;
    /// the oldest entry is pruned when the cap is exceeded.
    fn push_event(&mut self, id: &str, event: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            digest.notable_events.insert(0, event.to_string());
            digest.notable_events.truncate(MAX_NOTABLE_EVENTS);
        }
    }

    /// Append a permanent highlight for a tribute. Capped at `MAX_HIGHLIGHTS`
    /// (20); oldest prunes first. Highlights persist across phases alongside
    /// the rolling `notable_events`.
    fn push_highlight(&mut self, id: &str, event: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            digest.highlights.push(event.to_string());
            if digest.highlights.len() > crate::types::MAX_HIGHLIGHTS {
                digest.highlights.remove(0);
            }
        }
    }

    /// Increment a tribute's kill streak (they scored a kill this phase).
    /// Fires a milestone event when crossing a spree tier boundary
    /// (2 → "heating up", 4 → "on fire", 6 → "dominating", 8 → "unstoppable").
    fn bump_streak(&mut self, id: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            let old = digest.kill_streak;
            let new = digest.kill_streak.saturating_add(1);
            digest.kill_streak = new;

            let old_label = crate::types::spree_label(old);
            let new_label = crate::types::spree_label(new);
            if !new_label.is_empty() && new_label != old_label {
                let name = digest.name.clone();
                let milestone = format!(
                    "{name} is {new_label} — {} kills in a row!",
                    new
                );
                // Push directly (can't call self.push_event recursively
                // in the same scope).
                digest.notable_events.insert(0, milestone);
                digest.notable_events.truncate(MAX_NOTABLE_EVENTS);
            }
        }
    }

    /// Reset a tribute's kill streak to 0 (they died or lost a fight).
    /// If they had an active spree (streak >= 2), logs a spree-break event.
    fn reset_streak(&mut self, id: &str) {
        if let Some(digest) = self.inner.get_mut(id) {
            let had_spree = digest.kill_streak >= 2;
            let old_label = crate::types::spree_label(digest.kill_streak);
            digest.kill_streak = 0;
            if had_spree {
                let name = digest.name.clone();
                let label = if old_label.is_empty() {
                    "spree"
                } else {
                    old_label
                };
                let note = format!("{name}'s {label} spree has been broken!");
                digest.notable_events.insert(0, note);
                digest.notable_events.truncate(MAX_NOTABLE_EVENTS);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn killer_name(killer: &Option<TributeRef>) -> String {
    killer
        .as_ref()
        .map(|k| k.name.clone())
        .unwrap_or_else(|| "unknown causes".to_string())
}

fn attacker_name(attacker: &Option<TributeRef>) -> String {
    attacker
        .as_ref()
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "an unknown assailant".to_string())
}

fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [single] => single.clone(),
        [a, b] => format!("{a} and {b}"),
        many => {
            let (head, tail) = many.split_at(many.len() - 1);
            format!("{}, and {}", head.join(", "), tail[0])
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::Phase;

    fn make_tribute(name: &str, district: u8) -> TributeDigest {
        TributeDigest {
            identifier: format!("id-{name}"),
            name: name.into(),
            district,
            status: "alive".into(),
            injury_level: "unharmed".into(),
            location: "Cornucopia".into(),
            allies: vec![],
            kill_streak: 0,
            highlights: vec![],
            notable_events: vec![],
        }
    }

    fn tr(name: &str) -> TributeRef {
        TributeRef {
            identifier: format!("id-{name}"),
            name: name.into(),
        }
    }

    fn make_msg(payload: MessagePayload) -> GameMessage {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        GameMessage {
            identifier: format!("msg-{n}"),
            source: shared::messages::MessageSource::Game("game-1".into()),
            game_day: 1,
            phase: Phase::Day,
            tick: 0,
            emit_index: n,
            subject: String::new(),
            timestamp: chrono::DateTime::from_timestamp_nanos(0),
            content: "test".into(),
            payload,
        }
    }

    #[test]
    fn new_collection_contains_all_tributes() {
        let roster = vec![
            make_tribute("Katniss", 12),
            make_tribute("Peeta", 12),
            make_tribute("Cato", 2),
        ];
        let h = TributeHistories::new(roster);
        assert_eq!(h.digests().len(), 3);
        assert_eq!(h.digests()[0].name, "Cato"); // sorted
    }

    #[test]
    fn killed_marks_as_deceased() {
        let roster = vec![make_tribute("Katniss", 12)];
        let mut h = TributeHistories::new(roster);
        h.update(&[make_msg(MessagePayload::TributeKilled {
            victim: tr("Katniss"),
            killer: None,
            cause: "starvation".into(),
        })]);
        let d = h.get("id-Katniss").unwrap();
        assert_eq!(d.status, "deceased");
        assert_eq!(d.injury_level, "deceased");
        assert_eq!(d.notable_events[0], "Killed by unknown causes");
    }

    #[test]
    fn wounded_tracks_injury_level() {
        let roster = vec![make_tribute("Cato", 2)];
        let mut h = TributeHistories::new(roster);
        h.update(&[make_msg(MessagePayload::TributeWounded {
            victim: tr("Cato"),
            attacker: Some(tr("Katniss")),
            hp_lost: 12,
        })]);
        let d = h.get("id-Cato").unwrap();
        assert_eq!(d.injury_level, "devastating");
        assert!(d.notable_events[0].contains("devastating"));
    }

    #[test]
    fn movement_updates_location() {
        let roster = vec![make_tribute("Peeta", 12)];
        let mut h = TributeHistories::new(roster);
        h.update(&[make_msg(MessagePayload::TributeMoved {
            tribute: tr("Peeta"),
            from: shared::messages::AreaRef {
                identifier: "area-1".into(),
                name: "Cornucopia".into(),
            },
            to: shared::messages::AreaRef {
                identifier: "area-2".into(),
                name: "Forest".into(),
            },
        })]);
        let d = h.get("id-Peeta").unwrap();
        assert_eq!(d.location, "Forest");
    }

    #[test]
    fn alliance_adds_allies() {
        let roster = vec![
            make_tribute("Katniss", 12),
            make_tribute("Rue", 11),
            make_tribute("Peeta", 12),
        ];
        let mut h = TributeHistories::new(roster);
        h.update(&[make_msg(MessagePayload::AllianceFormed {
            members: vec![tr("Katniss"), tr("Rue")],
        })]);
        let k = h.get("id-Katniss").unwrap();
        assert!(k.allies.contains(&"Rue".to_string()));
        let r = h.get("id-Rue").unwrap();
        assert!(r.allies.contains(&"Katniss".to_string()));
        let p = h.get("id-Peeta").unwrap();
        assert!(p.allies.is_empty());
    }

    #[test]
    fn notable_events_capped_at_max() {
        let roster = vec![make_tribute("Test", 1)];
        let mut h = TributeHistories::new(roster);
        // Push more events than the cap (MAX_NOTABLE_EVENTS = 30).
        let events: Vec<GameMessage> = (0..35)
            .map(|_| {
                make_msg(MessagePayload::TributeStarved {
                    tribute: tr("Test"),
                    hp_lost: 1,
                })
            })
            .collect();
        h.update(&events);
        let d = h.get("id-Test").unwrap();
        assert_eq!(d.notable_events.len(), 30);
    }

    #[test]
    fn digests_sorted_by_name() {
        let roster = vec![
            make_tribute("Zoe", 1),
            make_tribute("Alice", 2),
            make_tribute("Bob", 3),
        ];
        let h = TributeHistories::new(roster);
        let digests = h.digests();
        let names: Vec<&str> = digests.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["Alice", "Bob", "Zoe"]);
    }

    #[test]
    fn update_empty_events_does_nothing() {
        let roster = vec![make_tribute("Katniss", 12)];
        let mut h = TributeHistories::new(roster);
        h.update(&[]);
        let d = h.get("id-Katniss").unwrap();
        assert_eq!(d.notable_events.len(), 0);
        assert_eq!(d.status, "alive");
    }

    #[test]
    fn unknown_tribute_does_not_panic() {
        let roster = vec![make_tribute("Katniss", 12)];
        let mut h = TributeHistories::new(roster);
        h.update(&[make_msg(MessagePayload::TributeStarved {
            tribute: tr("Nonexistent"),
            hp_lost: 5,
        })]);
        // Should not panic, just skip
        assert_eq!(h.get("id-Katniss").unwrap().notable_events.len(), 0);
    }
}
