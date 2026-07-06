use super::*;

impl Game {
    /// Drain the alliance event queue accumulated during the current cycle.
    /// Called between tribute turns inside `run_tribute_cycle` so cascades
    /// resolve before the next tribute acts. Per spec §7.5:
    /// - `BetrayalRecorded`: remove the symmetric pair on the victim's side
    ///   (betrayer's side was already cleaned at trigger time) and flag the
    ///   victim for a trust-shock roll on their next turn. The betrayer is
    ///   never flagged.
    /// - `DeathRecorded`: roll a sanity-break per direct ally of the deceased
    ///   (consistent with §7.3a thresholds) and emit a break message on
    ///   success. After the cascade, unconditionally scrub the deceased's
    ///   id from every surviving tribute's `allies` list.
    pub fn process_alliance_events(&mut self, rng: &mut impl Rng) {
        use crate::tributes::alliances::{AllianceEvent, sanity_break_roll};

        // Collect drained events into a local Vec so we can release the
        // borrow on `self.alliance_events` before mutating `self.tributes`.
        let drained: Vec<AllianceEvent> = self.alliance_events.drain(..).collect();

        for ev in drained {
            match ev {
                AllianceEvent::BetrayalRecorded { betrayer, victim } => {
                    // Snapshot names before the mutable borrow on `victim` so
                    // we can emit the message after victim mutation completes.
                    let betrayer_info = self
                        .tributes
                        .iter()
                        .find(|t| t.id == betrayer)
                        .map(|t| (t.identifier.clone(), t.name.clone()));
                    let victim_info = self
                        .tributes
                        .iter()
                        .find(|t| t.id == victim)
                        .map(|t| (t.identifier.clone(), t.name.clone()));
                    if let Some(v) = self.tributes.iter_mut().find(|t| t.id == victim) {
                        v.allies.retain(|x| *x != betrayer);
                        v.pending_trust_shock = true;
                    }
                    // Spec §7.5: betrayer is never enqueued for trust-shock.
                    if let (Some((b_id, b_name)), Some((v_id, v_name))) =
                        (betrayer_info, victim_info)
                    {
                        let event = crate::events::GameEvent::BetrayalTriggered {
                            betrayer_id: betrayer,
                            betrayer_name: b_name.clone(),
                            victim_id: victim,
                            victim_name: v_name.clone(),
                        };
                        let payload = crate::messages::MessagePayload::BetrayalTriggered {
                            betrayer: crate::messages::TributeRef {
                                identifier: b_id,
                                name: b_name,
                            },
                            victim: crate::messages::TributeRef {
                                identifier: v_id.clone(),
                                name: v_name.clone(),
                            },
                        };
                        let tick = self.tick_counter.next();
                        self.push_message(
                            crate::messages::MessageSource::Tribute(v_id.to_string()),
                            v_name,
                            event.to_string(),
                            payload,
                            tick,
                        );
                    }
                }
                AllianceEvent::DeathRecorded {
                    deceased,
                    killer: _,
                } => {
                    // Snapshot the deceased's allies and identifying refs
                    // before mutation so we can roll the cascade per direct
                    // ally and emit typed `TrustShockBreak` payloads.
                    let (allies_of_deceased, deceased_ref): (Vec<Uuid>, _) = self
                        .tributes
                        .iter()
                        .find(|t| t.id == deceased)
                        .map(|d| {
                            (
                                d.allies.clone(),
                                crate::messages::TributeRef {
                                    identifier: d.identifier.clone(),
                                    name: d.name.clone(),
                                },
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                Vec::new(),
                                crate::messages::TributeRef {
                                    identifier: deceased.to_string().into(),
                                    name: String::new(),
                                },
                            )
                        });

                    for ally_id in allies_of_deceased {
                        if let Some(ally) = self.tributes.iter_mut().find(|t| t.id == ally_id) {
                            // `extreme_low_sanity` is the §7.3a low-limit
                            // mapping (see PersonalityThresholds doc).
                            let limit = ally.brain.thresholds.extreme_low_sanity;
                            let sanity = ally.attributes.sanity();
                            if sanity_break_roll(sanity, limit, rng) {
                                ally.allies.retain(|x| *x != deceased);
                                let aid = ally.identifier.clone();
                                let aname = ally.name.clone();
                                let ally_uuid = ally.id;
                                let event = crate::events::GameEvent::TrustShockBreak {
                                    tribute_id: ally_uuid,
                                    tribute_name: aname.clone(),
                                };
                                let payload = crate::messages::MessagePayload::TrustShockBreak {
                                    tribute: crate::messages::TributeRef {
                                        identifier: aid.clone(),
                                        name: aname.clone(),
                                    },
                                    partner: deceased_ref.clone(),
                                };
                                let tick = self.tick_counter.next();
                                self.push_message(
                                    crate::messages::MessageSource::Tribute(aid.to_string()),
                                    aname,
                                    event.to_string(),
                                    payload,
                                    tick,
                                );
                            }
                        }
                    }

                    // Unconditional cleanup: ensure the deceased's id is
                    // removed from every surviving tribute's allies list,
                    // even if their cascade roll failed.
                    for t in self.tributes.iter_mut() {
                        t.allies.retain(|x| *x != deceased);
                    }
                }
                AllianceEvent::FormationRecorded {
                    proposer,
                    target,
                    factor,
                } => {
                    let proposer_info = self
                        .tributes
                        .iter()
                        .find(|t| t.id == proposer)
                        .map(|t| (t.identifier.clone(), t.name.clone()));
                    let target_info = self
                        .tributes
                        .iter()
                        .find(|t| t.id == target)
                        .map(|t| (t.identifier.clone(), t.name.clone()));
                    let mut idx_p: Option<usize> = None;
                    let mut idx_t: Option<usize> = None;
                    for (i, t) in self.tributes.iter().enumerate() {
                        if t.id == proposer {
                            idx_p = Some(i);
                        }
                        if t.id == target {
                            idx_t = Some(i);
                        }
                    }
                    let (Some(ip), Some(it)) = (idx_p, idx_t) else {
                        continue;
                    };
                    if self.tributes[ip].allies.len() >= crate::tributes::alliances::MAX_ALLIES
                        || self.tributes[it].allies.len() >= crate::tributes::alliances::MAX_ALLIES
                    {
                        continue;
                    }
                    if !self.tributes[ip].allies.contains(&target) {
                        self.tributes[ip].allies.push(target);
                    }
                    if !self.tributes[it].allies.contains(&proposer) {
                        self.tributes[it].allies.push(proposer);
                    }
                    if let (Some((p_id, p_name)), Some((t_id, t_name))) =
                        (proposer_info, target_info)
                    {
                        let event = crate::events::GameEvent::AllianceFormed {
                            tribute_a_id: proposer,
                            tribute_a_name: p_name.clone(),
                            tribute_b_id: target,
                            tribute_b_name: t_name.clone(),
                            factor: factor.clone(),
                        };
                        let payload = crate::messages::MessagePayload::AllianceFormed {
                            members: vec![
                                crate::messages::TributeRef {
                                    identifier: p_id.clone(),
                                    name: p_name.clone(),
                                },
                                crate::messages::TributeRef {
                                    identifier: t_id,
                                    name: t_name,
                                },
                            ],
                        };
                        let tick = self.tick_counter.next();
                        self.push_message(
                            crate::messages::MessageSource::Tribute(p_id.to_string()),
                            p_name,
                            event.to_string(),
                            payload,
                            tick,
                        );
                    }
                }
                AllianceEvent::AllianceSummons { summoner, target } => {
                    // Spec §6.4 PR2c.2 (bd-1zju). When the target is asleep,
                    // an ally's summons interrupts the rest. Currently no
                    // production code emits this event; the handler is in
                    // place so future PRs (or test scaffolding) can wake
                    // sleeping allies through the standard alliance pipeline.
                    let summoner_ref = self.tributes.iter().find(|t| t.id == summoner).map(|t| {
                        crate::messages::TributeRef {
                            identifier: t.identifier.clone(),
                            name: t.name.clone(),
                        }
                    });
                    let Some(s_ref) = summoner_ref else { continue };
                    let phase = self.current_phase;
                    let mut wake_events: Vec<crate::messages::TaggedEvent> = Vec::new();
                    let woke_info =
                        if let Some(t) = self.tributes.iter_mut().find(|t| t.id == target) {
                            if t.wake_interrupted(
                                shared::messages::InterruptionKind::AllianceSummons { ally: s_ref },
                                phase,
                                &mut wake_events,
                            ) {
                                Some((t.identifier.clone(), t.name.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                    if let Some((t_id, t_name)) = woke_info {
                        for ev in wake_events.drain(..) {
                            let tick = self.tick_counter.next();
                            self.push_message(
                                crate::messages::MessageSource::Tribute(t_id.to_string()),
                                t_name.clone(),
                                ev.content,
                                ev.payload,
                                tick,
                            );
                        }
                    }
                }
            }
        }
    }
}
