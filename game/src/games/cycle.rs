use super::*;
use crate::areas::{Area, AreaDetails};
use crate::items::{Item, OwnsItems};
use crate::messages::{AreaRef, ItemRef, MessagePayload, TributeRef};
use crate::tributes::events::TributeEvent;
use crate::tributes::incidents::{SleepIncident, SleepShelter, apply_sleep_incident};
use crate::tributes::statuses::TributeStatus;
use crate::tributes::{
    ActionSuggestion, EncounterContext, EnvironmentContext, Tribute, calculate_stamina_cost,
};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use shared::messages::SleepIncidentKind;
use std::collections::HashMap;

impl Game {
    /// Pre-computed, immutable view of game state used by `execute_cycle`.
    ///
    /// `build_cycle_context` materialises this from `&self` so the executor
    /// half of the cycle (which holds `&mut self`) can read pre-snapshotted
    /// data without re-borrowing. This split also gives gamemaker overrides
    /// (and future cycle-modifier hooks) a typed seam between the
    /// "what the world looks like" phase and the "what each tribute does"
    /// phase.
    pub(super) fn build_cycle_context(
        &self,
        phase: crate::messages::Phase,
        closed_areas: Vec<Area>,
        living_tributes: Vec<Tribute>,
        living_tributes_count: usize,
    ) -> CycleContext {
        use crate::messages::Phase;
        let day = phase == Phase::Day;
        let action_suggestion = match (self.day, day) {
            (Some(1), true) => Some(ActionSuggestion {
                action: Action::Move(None),
                probability: Some(0.5),
            }),
            (Some(3), true) => Some(ActionSuggestion {
                action: Action::Move(Some(Area::Cornucopia)),
                probability: Some(0.75),
            }),
            (_, _) => None,
        };

        let mut area_details_map = HashMap::with_capacity(self.areas.len());
        for (i, area_detail) in self.areas.iter().enumerate() {
            if let Some(area) = &area_detail.area {
                area_details_map.insert(*area, i);
            }
        }

        let mut tributes_by_area: HashMap<Area, Vec<Tribute>> = HashMap::new();
        for tribute in living_tributes {
            tributes_by_area
                .entry(tribute.area)
                .or_default()
                .push(tribute);
        }

        // Per-area living-tribute density. Threaded into `EnvironmentContext`
        // so `Brain::choose_destination` can apply a per-enemy crowd penalty
        // and disperse crowded areas without a call-site escape hatch
        // (hangrier_games-4wnj).
        let enemy_density: HashMap<Area, u32> = tributes_by_area
            .iter()
            .map(|(area, tributes)| (*area, tributes.len() as u32))
            .collect();

        CycleContext {
            is_day: day,
            phase,
            current_day: self.day.unwrap_or(1),
            action_suggestion,
            area_details_map,
            tributes_by_area,
            enemy_density,
            combat_tuning_snapshot: self.combat_tuning.clone(),
            all_areas_snapshot: self.areas.clone(),
            closed_areas,
            living_tributes_count,
        }
    }

    /// Iterate over `self.tributes`, applying survival ticks, brain decisions,
    /// and combat using the pre-built `CycleContext`. After the iteration
    /// ends the collected per-tribute events are drained into `self.messages`
    /// via `flush_tribute_events`, and any alliance events emitted during the
    /// cycle are processed.
    pub(super) fn execute_cycle(
        &mut self,
        ctx: CycleContext,
        rng: &mut SmallRng,
    ) -> Result<(), GameError> {
        let CycleContext {
            is_day: day,
            phase,
            current_day,
            action_suggestion,
            area_details_map,
            tributes_by_area,
            enemy_density,
            combat_tuning_snapshot,
            all_areas_snapshot,
            closed_areas,
            living_tributes_count,
        } = ctx;

        let mut collected_events: Vec<CollectedEvent> = Vec::new();
        let mut drained_alliance_events: Vec<crate::tributes::alliances::AllianceEvent> =
            Vec::new();

        // Two-phase resolution (tm6a): collect indices of tributes that
        // survive survival/sleep ticks first, then execute actions in a
        // second pass with liveness checks so tributes killed by earlier
        // actions in the same phase cannot act.
        let mut tributes_to_act: Vec<usize> = Vec::new();

        for (idx, tribute) in self.tributes.iter_mut().enumerate() {
            if !tribute.is_alive() {
                // Newly-dead tributes (status=RecentlyDead going into this
                // cycle) trigger a DeathRecorded event so allies process the
                // ally-death cascade. Killer attribution is read from the
                // tribute's transient `recently_killed_by` field, which combat
                // sites set when the death was caused by another tribute.
                // Environmental/status deaths leave it `None`. Promote to Dead
                // after enqueueing so the same tribute does not re-emit on
                // subsequent cycles.
                if tribute.status == TributeStatus::RecentlyDead {
                    let killer = tribute.recently_killed_by.take();
                    drained_alliance_events.push(
                        crate::tributes::alliances::AllianceEvent::DeathRecorded {
                            deceased: tribute.id,
                            killer,
                        },
                    );
                }
                tribute.status = TributeStatus::Dead;
                continue;
            }

            if !rng.random_bool(tribute.attributes.luck as f64 / 100.0) {
                tribute.events.push(TributeEvent::random());
            }

            // Survival tick (spec §6, §7). Each living tribute, once per
            // phase: tick hunger/thirst, apply escalating drain, emit any
            // band-change events, and route 0-HP starvation/dehydration
            // deaths through TributeKilled with the appropriate cause.
            // Loot drop is handled centrally by clean_up_recent_deaths
            // after the cycle ends.
            {
                use crate::areas::weather::current_weather;
                use crate::messages::{MessagePayload, TributeRef};
                use crate::tributes::survival::{
                    apply_dehydration_drain, apply_starvation_drain, hunger_band, thirst_band,
                    tick_survival,
                };
                use shared::messages::{CAUSE_DEHYDRATION, CAUSE_STARVATION};

                let weather = current_weather();
                let phase_index: u32 = self.day.unwrap_or(1) * 2 + u32::from(!day);
                let sheltered = tribute
                    .sheltered_until
                    .is_some_and(|until| until > phase_index);

                // Affliction cascade tick (spec §5). Runs once per phase per
                // living tribute. Sheltered tributes may recover; exposed may
                // worsen. Severe + exposed can spawn successors or kill.
                {
                    use crate::tributes::afflictions::tuning::AfflictionTuning;
                    use crate::tributes::afflictions::{
                        CascadeOutcome, apply_cascade, tick_cascade,
                    };
                    use shared::afflictions::Severity;

                    let affliction_list: Vec<_> = tribute.afflictions.values().cloned().collect();
                    if !affliction_list.is_empty() {
                        let tuning = AfflictionTuning::default();
                        let cascade_result =
                            tick_cascade(&affliction_list, sheltered, &tuning, rng);
                        let successors = apply_cascade(&mut tribute.afflictions, &cascade_result);

                        for succ in &successors {
                            tribute.afflictions.insert(succ.key(), succ.clone());
                        }

                        let tref = TributeRef {
                            identifier: tribute.identifier.clone(),
                            name: tribute.name.clone(),
                        };

                        for (kind, outcome) in &cascade_result.outcomes {
                            match outcome {
                                CascadeOutcome::SteppedDown { from, to } => {
                                    if matches!(to, Severity::Mild)
                                        && matches!(from, Severity::Mild)
                                    {
                                        let line = format!("{}'s {} healed.", tribute.name, kind);
                                        collected_events.push((
                                            tribute.identifier.clone(),
                                            tribute.name.clone(),
                                            line,
                                            Some(MessagePayload::AfflictionHealed {
                                                tribute_id: tref.identifier.clone(),
                                                affliction: kind.to_string(),
                                            }),
                                            None,
                                        ));
                                    } else {
                                        let line = format!(
                                            "{}'s {} improved: {} → {}.",
                                            tribute.name, kind, from, to
                                        );
                                        collected_events.push((
                                            tribute.identifier.clone(),
                                            tribute.name.clone(),
                                            line,
                                            Some(MessagePayload::AfflictionProgressed {
                                                tribute_id: tref.identifier.clone(),
                                                affliction: kind.to_string(),
                                                from_severity: from.to_string(),
                                                to_severity: to.to_string(),
                                            }),
                                            None,
                                        ));
                                    }
                                }
                                CascadeOutcome::SteppedUp { from, to } => {
                                    let line = format!(
                                        "{}'s {} worsened: {} → {}.",
                                        tribute.name, kind, from, to
                                    );
                                    collected_events.push((
                                        tribute.identifier.clone(),
                                        tribute.name.clone(),
                                        line,
                                        Some(MessagePayload::AfflictionProgressed {
                                            tribute_id: tref.identifier.clone(),
                                            affliction: kind.to_string(),
                                            from_severity: from.to_string(),
                                            to_severity: to.to_string(),
                                        }),
                                        None,
                                    ));
                                }
                                CascadeOutcome::SpawnedSuccessor { from, to } => {
                                    let line = format!(
                                        "{}'s {} cascaded into {}.",
                                        tribute.name, from, to
                                    );
                                    collected_events.push((
                                        tribute.identifier.clone(),
                                        tribute.name.clone(),
                                        line,
                                        Some(MessagePayload::AfflictionCascaded {
                                            tribute_id: tref.identifier.clone(),
                                            from_affliction: from.to_string(),
                                            to_affliction: to.to_string(),
                                        }),
                                        None,
                                    ));
                                }
                                CascadeOutcome::DeathRoll { survived: false } => {
                                    let line = format!("{} succumbs to {}.", tribute.name, kind);
                                    collected_events.push((
                                        tribute.identifier.clone(),
                                        tribute.name.clone(),
                                        line,
                                        Some(MessagePayload::TributeKilled {
                                            victim: tref.clone(),
                                            killer: None,
                                            cause: kind.to_string(),
                                        }),
                                        None,
                                    ));
                                    tribute.status = TributeStatus::RecentlyDead;
                                }
                                _ => {}
                            }
                        }

                        if cascade_result.tribute_died {
                            continue;
                        }
                    }
                }

                let prior_hunger = hunger_band(tribute.hunger);
                let prior_thirst = thirst_band(tribute.thirst);

                // Sleep substrate (bd-s0je): once per phase, every living
                // tribute that did NOT spend the phase asleep ages by one
                // cycle. The brain doesn't yet score `Action::Sleep`, so this
                // simply tracks accumulated wakefulness for downstream PRs.
                if !tribute.sleeping {
                    tribute.cycles_awake = tribute.cycles_awake.saturating_add(1);
                }

                tick_survival(tribute, &weather, sheltered);
                let hp_lost_starv = apply_starvation_drain(tribute);
                let hp_lost_dehy = apply_dehydration_drain(tribute);

                let new_hunger = hunger_band(tribute.hunger);
                let new_thirst = thirst_band(tribute.thirst);
                let tref = TributeRef {
                    identifier: tribute.identifier.clone(),
                    name: tribute.name.clone(),
                };

                if new_hunger != prior_hunger {
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        String::new(),
                        Some(MessagePayload::HungerBandChanged {
                            tribute: tref.clone(),
                            from: prior_hunger,
                            to: new_hunger,
                        }),
                        None,
                    ));
                }
                if new_thirst != prior_thirst {
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        String::new(),
                        Some(MessagePayload::ThirstBandChanged {
                            tribute: tref.clone(),
                            from: prior_thirst,
                            to: new_thirst,
                        }),
                        None,
                    ));
                }

                // Stamina recovery + band-cross detection. Runs once per
                // phase per living tribute. For v1 we use Action::None (idle
                // recovery 5/phase); proper Rest/sheltered scaling lands when
                // the action chosen by `process_turn_phase` is plumbed back
                // here. `sheltered` reuses the value computed above for the
                // hunger/thirst tick.
                if tribute.attributes.health > 0 {
                    use crate::tributes::stamina_band::stamina_band;

                    let prior_band = stamina_band(
                        tribute.stamina,
                        tribute.max_stamina,
                        &combat_tuning_snapshot,
                    );
                    tribute.recover_stamina(
                        &crate::tributes::actions::Action::None,
                        sheltered,
                        new_hunger,
                        new_thirst,
                        &combat_tuning_snapshot,
                    );
                    let new_band = stamina_band(
                        tribute.stamina,
                        tribute.max_stamina,
                        &combat_tuning_snapshot,
                    );
                    if new_band != prior_band {
                        let line = format!(
                            "{} stamina: {:?} -> {:?}",
                            tribute.name, prior_band, new_band
                        );
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            line,
                            Some(MessagePayload::StaminaBandChanged {
                                tribute: tref.clone(),
                                from: prior_band,
                                to: new_band,
                            }),
                            None,
                        ));
                    }
                }

                // Death routing for survival-induced 0 HP. Dehydration takes
                // precedence over starvation when both landed in the same
                // tick.
                if tribute.attributes.health == 0 && (hp_lost_starv > 0 || hp_lost_dehy > 0) {
                    let cause = if hp_lost_dehy > 0 {
                        CAUSE_DEHYDRATION
                    } else {
                        CAUSE_STARVATION
                    };
                    let line = format!("{} succumbs to {}.", tribute.name, cause);
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        line,
                        Some(MessagePayload::TributeKilled {
                            victim: tref,
                            killer: None,
                            cause: cause.to_string(),
                        }),
                        None,
                    ));
                    tribute.status = TributeStatus::RecentlyDead;
                    continue;
                }
            }

            // Sleep tick (PR2c.1, bd-9sjj). Sleeping tributes skip the
            // brain pipeline entirely: regen stamina (always) and HP
            // (gated on absence of Wounded / Infected / Sick per spec
            // §6.4), then decrement `sleep_remaining`. When the
            // countdown drains to zero, flip `sleeping = false`, reset
            // `cycles_awake`, and emit `TributeWoke { Rested }`.
            // Interruption handling lives in PR2c.2 (bd-1zju); this PR
            // ships the natural-wake path only.
            if tribute.sleeping {
                use crate::messages::{MessagePayload, TributeRef};
                use shared::messages::WakeReason;

                // Spec §6.4 PR2c.2 (bd-1zju). Before regenerating, check
                // whether an area event in the sleeper's current area is
                // active. If so, wake the tribute with the appropriate
                // `InterruptionKind::AreaEvent` and skip regen this phase
                // — they didn't actually rest, they were jolted awake.
                let area_event_kind = area_details_map
                    .get(&tribute.area)
                    .and_then(|&idx| self.areas.get(idx))
                    .and_then(|a| a.events.first())
                    .map(area_event_to_kind);
                if let Some(kind) = area_event_kind {
                    let mut wake_events: Vec<crate::messages::TaggedEvent> = Vec::new();
                    let woke = tribute.wake_interrupted(
                        shared::messages::InterruptionKind::AreaEvent { kind },
                        phase,
                        &mut wake_events,
                    );
                    if woke {
                        for ev in wake_events.drain(..) {
                            collected_events.push((
                                tribute.identifier.clone(),
                                tribute.name.clone(),
                                ev.content,
                                Some(ev.payload),
                                None,
                            ));
                        }
                        tribute.sleep_shelter = None;
                        continue;
                    }
                }

                // ── Sleep incident roll (we6l) ──
                // While unconscious, the tribute is vulnerable. Roll for a
                // random sleep incident each sleeping phase. Wake-causing
                // incidents (theft, relocation, animal, ally abandonment,
                // limb injury) interrupt sleep immediately. Flavor-only
                // incidents (annoying) are remembered for the natural wake.
                let biome = area_details_map
                    .get(&tribute.area)
                    .and_then(|&idx| self.areas.get(idx))
                    .map(|a| a.terrain.base)
                    .unwrap_or(crate::terrain::types::BaseTerrain::Clearing);
                let phase_index: u32 = self.day.unwrap_or(1) * 4 + phase.ord() as u32;
                let is_sheltered = tribute
                    .sheltered_until
                    .is_some_and(|until| until > phase_index);
                if let Some(incident) = SleepIncident::roll(
                    rng,
                    phase,
                    biome,
                    is_sheltered,
                    tribute
                        .sleep_shelter
                        .as_ref()
                        .unwrap_or(&SleepShelter::None),
                    current_day,
                ) {
                    let description = apply_sleep_incident(tribute, &incident, rng);
                    let incident_kind: shared::messages::SleepIncidentKind = (&incident).into();

                    if incident.wakes_tribute() {
                        // Wake-causing incident: emit flavor event, then
                        // wake the tribute with the incident as the reason.
                        let flavor_line = crate::output::GameOutput::TributeSleepFlavor(
                            tribute.name.as_str(),
                            &description,
                        )
                        .to_string();
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            flavor_line,
                            Some(MessagePayload::SleepIncident {
                                tribute: TributeRef {
                                    identifier: tribute.identifier.clone(),
                                    name: tribute.name.clone(),
                                },
                                kind: incident_kind.clone(),
                                description: description.clone(),
                            }),
                            None,
                        ));

                        let incident_msg = crate::output::GameOutput::TributeWakesFromIncident(
                            tribute.name.as_str(),
                            &description,
                        )
                        .to_string();
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            incident_msg,
                            Some(MessagePayload::TributeWoke {
                                tribute: TributeRef {
                                    identifier: tribute.identifier.clone(),
                                    name: tribute.name.clone(),
                                },
                                phase,
                                reason: shared::messages::WakeReason::Interrupted {
                                    event: shared::messages::InterruptionKind::Incident {
                                        kind: incident_kind,
                                    },
                                },
                            }),
                            None,
                        ));
                        tribute.sleeping = false;
                        tribute.sleep_remaining = 0;
                        tribute.cycles_awake = 0;
                        tribute.pending_sleep_incident = None;
                        tribute.sleep_shelter = None;
                        continue;
                    } else {
                        // Flavor-only incident: emit flavor, remember it,
                        // then continue with regen as normal.
                        let flavor_line = crate::output::GameOutput::TributeSleepFlavor(
                            tribute.name.as_str(),
                            &description,
                        )
                        .to_string();
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            flavor_line,
                            Some(MessagePayload::SleepIncident {
                                tribute: TributeRef {
                                    identifier: tribute.identifier.clone(),
                                    name: tribute.name.clone(),
                                },
                                kind: incident_kind.clone(),
                                description: description.clone(),
                            }),
                            None,
                        ));
                        tribute.pending_sleep_incident = Some(incident_kind);
                    }
                }

                let blocked = tribute.afflictions.keys().any(|(kind, _)| {
                    matches!(
                        kind,
                        shared::afflictions::AfflictionKind::Wounded
                            | shared::afflictions::AfflictionKind::Infected
                            | shared::afflictions::AfflictionKind::Sick
                    )
                });
                let prior_stamina = tribute.stamina;
                let prior_hp = tribute.attributes.health;
                tribute.stamina = tribute
                    .stamina
                    .saturating_add(SLEEP_STAMINA_PER_PHASE)
                    .min(tribute.max_stamina);
                if !blocked {
                    tribute.attributes.health = tribute
                        .attributes
                        .health
                        .saturating_add(SLEEP_HP_PER_PHASE)
                        .min(SLEEP_HP_CAP);
                }
                let restored_stamina = tribute.stamina.saturating_sub(prior_stamina);
                let restored_hp = tribute.attributes.health.saturating_sub(prior_hp);

                // The TributeSlept emission for the *entry* phase happens
                // in process_turn_phase. Each subsequent phase the sleeper
                // is silent except for the final TributeWoke; we do not
                // re-emit per-phase regen events to avoid log spam. The
                // restored_* totals are consumed by the consolidated
                // TributeWoke payload.
                tribute.sleep_remaining = tribute.sleep_remaining.saturating_sub(1);
                if tribute.sleep_remaining == 0 {
                    tribute.sleeping = false;
                    tribute.cycles_awake = 0;
                    tribute.sleep_shelter = None;
                    let tref = TributeRef {
                        identifier: tribute.identifier.clone(),
                        name: tribute.name.clone(),
                    };
                    let incident_suffix = tribute
                        .pending_sleep_incident
                        .take()
                        .map(|kind| match kind {
                            SleepIncidentKind::Hallucination => {
                                " — still shaken by strange dreams".to_string()
                            }
                            _ => " — though their sleep was restless".to_string(),
                        })
                        .unwrap_or_default();
                    let line = format!(
                        "{} {}",
                        crate::output::GameOutput::TributeWakesRested(tribute.name.as_str()),
                        incident_suffix,
                    );
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        line,
                        Some(MessagePayload::TributeWoke {
                            tribute: tref,
                            phase,
                            reason: WakeReason::Rested,
                        }),
                        None,
                    ));
                }
                let _ = (restored_stamina, restored_hp);
                continue;
            }

            // Tribute survived survival + sleep ticks — queue for action phase.
            // Two-phase resolution (tm6a): actions execute in a second pass
            // with liveness checks so tributes killed by earlier actions
            // cannot act.
            tributes_to_act.push(idx);
        }

        // ── Phobia scan ────────────────────────────────────────────
        // Run after survival/sleep ticks, before action execution.
        // Detects firing phobias, handles reinforcement/decay,
        // tracks observer state, emits escalation/habituation/observation
        // messages so severity changes take effect before brain decisions.
        if self.config.phobias_enabled && !tributes_to_act.is_empty() {
            use crate::tributes::afflictions::phobia::scan_tribute;
            use crate::tributes::afflictions::phobia::triggers::PhobiaContext;

            // Monotonically increasing cycle number (Day 1 Day = 1, etc.).
            let phobia_cycle = (current_day.saturating_sub(1)) * 4 + phase.ord() as u32;

            for &idx in &tributes_to_act {
                let area = self.tributes[idx].area;
                let Some(area_details) = all_areas_snapshot.iter().find(|ad| ad.area == Some(area))
                else {
                    continue;
                };

                // Use the pre-built tribute snapshot for observer tracking.
                let other_tributes: &[Tribute] = tributes_by_area
                    .get(&area)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let phobia_ctx = PhobiaContext {
                    area: area_details,
                    is_night: !day,
                    other_tributes_in_area: other_tributes,
                    cycle_messages: &[],
                    cycle: phobia_cycle,
                };

                let scan_result =
                    scan_tribute(&mut self.tributes[idx], &phobia_ctx, phobia_cycle, rng);

                for msg in scan_result.messages {
                    let line = phobia_message_line(&msg, &self.tributes[idx].name);
                    collected_events.push((
                        self.tributes[idx].identifier.clone(),
                        self.tributes[idx].name.clone(),
                        line,
                        Some(msg),
                        None,
                    ));
                }
            }
        }

        // ── Trauma cycle processing ──────────────────────────────────
        // Run after phobia scan, before action execution.
        // Handles flashback rolls, observer tracking, and decay.
        if self.config.trauma_enabled && !tributes_to_act.is_empty() {
            let trauma_cycle = (current_day.saturating_sub(1)) * 4 + phase.ord() as u32;

            for &idx in &tributes_to_act {
                let area = self.tributes[idx].area;
                let other_tributes: &[Tribute] = tributes_by_area
                    .get(&area)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let t_result = crate::tributes::afflictions::trauma::process_traumas(
                    &mut self.tributes[idx],
                    other_tributes,
                    trauma_cycle,
                    rng,
                );

                for msg in t_result.messages {
                    let line = format_trauma_message(&msg, &self.tributes[idx].name);
                    collected_events.push((
                        self.tributes[idx].identifier.clone(),
                        self.tributes[idx].name.clone(),
                        line,
                        Some(msg),
                        None,
                    ));
                }
            }
        }

        // ── Addiction cycle processing ────────────────────────────────
        // Run after trauma processing, before action execution.
        // Handles High/Withdrawal tick, decay, observer tracking.
        if self.config.addiction_enabled && !tributes_to_act.is_empty() {
            let addiction_cycle = (current_day.saturating_sub(1)) * 4 + phase.ord() as u32;

            for &idx in &tributes_to_act {
                let area = self.tributes[idx].area;
                let other_tributes: &[Tribute] = tributes_by_area
                    .get(&area)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let msgs = crate::tributes::afflictions::addiction::process_addictions(
                    &mut self.tributes[idx],
                    other_tributes,
                    addiction_cycle,
                    rng,
                );

                for msg in msgs {
                    let line = format_addiction_message(&msg, &self.tributes[idx].name);
                    collected_events.push((
                        self.tributes[idx].identifier.clone(),
                        self.tributes[idx].name.clone(),
                        line,
                        Some(msg),
                        None,
                    ));
                }
            }
        }

        // ── Hangover tick-down ──────────────────────────────────────
        for &idx in &tributes_to_act {
            let t = &mut self.tributes[idx];
            if t.hangover_cycles_remaining > 0 {
                t.hangover_cycles_remaining -= 1;
                if t.hangover_cycles_remaining == 0 {
                    let line = format!("{}'s hangover fades — rough night, clear head", t.name);
                    collected_events.push((t.identifier.clone(), t.name.clone(), line, None, None));
                }
            }
        }

        // Sort by initiative so faster tributes act first (tm6a).
        tributes_to_act.sort_by_cached_key(|&idx| {
            let agility = self.tributes[idx].attributes.agility;
            std::cmp::Reverse(initiative_score(agility, rng))
        });

        // --- Phase 2: Execute actions with liveness checks ---
        let mut pending_thefts: Vec<(usize, Uuid)> = Vec::new();
        for idx in tributes_to_act {
            // Build sleeping_nearby BEFORE the mutable tribute borrow so
            // the self.tributes.iter() doesn't conflict (ls5a).
            // Wasted work on dead tributes (continue'd below) but harmless.
            let tribute_area = self.tributes[idx].area;
            let sleeping_nearby: Vec<(Uuid, String)> = self
                .tributes
                .iter()
                .filter(|t| {
                    t.is_alive() && t.sleeping && t.area == tribute_area && !t.items.is_empty()
                })
                .map(|t| (t.id, t.name.clone()))
                .collect();

            let tribute = &mut self.tributes[idx];

            // Liveness gate: tribute may have been killed by an earlier
            // action in this same phase (tm6a).
            if !tribute.is_alive() {
                if tribute.status == TributeStatus::RecentlyDead {
                    let killer = tribute.recently_killed_by.take();
                    drained_alliance_events.push(
                        crate::tributes::alliances::AllianceEvent::DeathRecorded {
                            deceased: tribute.id,
                            killer,
                        },
                    );
                }
                tribute.status = TributeStatus::Dead;
                continue;
            }

            let area_index = match area_details_map.get(&tribute.area) {
                Some(&old_area_idx) => old_area_idx,
                None => continue,
            };

            // Build available destinations BEFORE taking mutable borrow of area_details
            let available_destinations = tribute
                .area
                .neighbors()
                .into_iter()
                .filter_map(|neighbor_area| {
                    // Find the AreaDetails for this neighbor
                    self.areas
                        .iter()
                        .find(|ad| ad.area == Some(neighbor_area))
                        .map(|ad| {
                            // Calculate stamina cost to move to this area
                            let move_action = Action::Move(Some(neighbor_area));
                            let stamina_cost =
                                calculate_stamina_cost(&move_action, &ad.terrain, tribute);

                            crate::areas::DestinationInfo {
                                area: neighbor_area,
                                terrain: ad.terrain.clone(),
                                active_events: ad.events.clone(),
                                stamina_cost,
                            }
                        })
                })
                .collect();

            let area_details = &mut self.areas[area_index];

            let mut environment_details = EnvironmentContext {
                is_day: day,
                phase,
                area_details,
                closed_areas: &closed_areas,
                available_destinations,
                all_areas: &all_areas_snapshot,
                enemy_density: &enemy_density,
                current_day,
                combat_tuning: &combat_tuning_snapshot,
                sleeping_nearby,
            };

            // Get nearby tributes using the pre-computed map
            let ev = Vec::new();
            let nearby_tributes = {
                match tributes_by_area.get(&tribute.area) {
                    Some(tributes) => tributes,
                    None => &ev,
                }
            };
            let nearby_tributes_count = nearby_tributes.len() as u32;

            let targets: Vec<Tribute> = nearby_tributes
                .iter()
                .filter(|t| t.is_visible() && t.identifier != tribute.identifier)
                .cloned()
                .collect();

            let encounter_context = EncounterContext {
                nearby_tributes_count,
                potential_targets: targets,
                total_living_tributes: living_tributes_count as u32,
            };

            // ── Brain rescue priority override ──
            // Before executing, check if this tribute should rescue a
            // co-located trapped tribute instead of their chosen action.
            let mut override_suggestion = action_suggestion.clone();
            if tribute.is_alive()
                && let Some(target_id) = crate::tributes::rescue::evaluate_rescue_opportunity(
                    tribute,
                    environment_details.area_details,
                    nearby_tributes,
                    rng,
                )
            {
                override_suggestion = Some(ActionSuggestion {
                    action: Action::Rescue { target: target_id },
                    probability: Some(1.0),
                });
            }
            let mut tribute_events: Vec<crate::messages::TaggedEvent> = Vec::new();
            tribute.process_turn_phase(
                override_suggestion,
                &mut environment_details,
                encounter_context,
                rng,
                &mut tribute_events,
            );
            for tagged in tribute_events {
                collected_events.push((
                    tribute.identifier.clone(),
                    tribute.name.clone(),
                    tagged.content,
                    Some(tagged.payload),
                    None,
                ));
            }
            drained_alliance_events.append(&mut tribute.drain_alliance_events());

            // Collect pending theft from sleeping tribute (ls5a).
            // The pending_theft_target was set by act_take_item during
            // process_turn_phase. Actual item transfer happens after the
            // tribute borrow is released (below) to avoid conflicting
            // &mut self.tributes borrows.
            if let Some(sleeper_uuid) = tribute.pending_theft_target.take() {
                pending_thefts.push((idx, sleeper_uuid));
            }
        }

        // ── Process pending sleep theft (ls5a) ──
        // Iterate thefts collected during Phase 2. The tribute borrow is
        // released by now so we can split_at_mut on self.tributes freely.
        for (thief_idx, sleeper_uuid) in &pending_thefts {
            let sleeper_idx = match self.tributes.iter().position(|t| t.id == *sleeper_uuid) {
                Some(idx) => idx,
                None => continue, // sleeper died or vanished
            };
            if sleeper_idx == *thief_idx {
                continue; // shouldn't happen
            }

            // split_at_mut for simultaneous mutable access to thief + sleeper
            let (thief, sleeper) = if *thief_idx < sleeper_idx {
                let (left, right) = self.tributes.split_at_mut(sleeper_idx);
                (&mut left[*thief_idx], &mut right[0])
            } else {
                let (left, right) = self.tributes.split_at_mut(*thief_idx);
                (&mut right[0], &mut left[sleeper_idx])
            };

            if sleeper.items.is_empty() {
                continue;
            }
            let item_idx = rng.random_range(0..sleeper.items.len());
            let stolen = sleeper.items.remove(item_idx);
            let item_name = stolen.name.clone();
            let thief_name = thief.name.clone();
            let sleeper_name = sleeper.name.clone();
            thief.add_item(stolen.clone());

            let line = format!(
                "{} steals {} from sleeping {}!",
                thief_name, item_name, sleeper_name
            );
            collected_events.push((
                thief.identifier.clone(),
                thief.name.clone(),
                line,
                Some(MessagePayload::ItemFound {
                    tribute: TributeRef {
                        identifier: thief.identifier.clone(),
                        name: thief.name.clone(),
                    },
                    item: ItemRef {
                        identifier: stolen.identifier.clone(),
                        name: item_name,
                    },
                    area: AreaRef {
                        identifier: thief.area.to_string(),
                        name: thief.area.to_string(),
                    },
                }),
                None,
            ));
        }

        // ── Fixation processing ──
        // Run after Phase 2 actions so drained_alliance_events contains
        // DeathRecorded events (with killer attributions) from this cycle.
        if self.config.fixations_enabled {
            let fixation_indices: Vec<usize> = self
                .tributes
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    t.is_alive()
                        && crate::tributes::afflictions::fixation::count_fixations(&t.afflictions)
                            > 0
                })
                .map(|(i, _)| i)
                .collect();

            if !fixation_indices.is_empty() {
                use crate::tributes::afflictions::fixation::{
                    FixationContext, process_tribute_fixations,
                };
                use std::collections::HashMap;
                use uuid::Uuid;

                // Build identifier → UUID lookup.
                let id_to_uuid: HashMap<String, Uuid> = self
                    .tributes
                    .iter()
                    .map(|t| (t.identifier.clone(), t.id))
                    .collect();

                // Build dead-tribute → killer lookup from drained alliance events.
                let mut dead_tribute_killers: HashMap<Uuid, Option<Uuid>> = HashMap::new();
                for event in &drained_alliance_events {
                    if let crate::tributes::alliances::AllianceEvent::DeathRecorded {
                        deceased,
                        killer,
                    } = event
                    {
                        dead_tribute_killers.insert(*deceased, *killer);
                    }
                }

                // Closed areas (lowercased for matching).
                let closed_areas: std::collections::BTreeSet<String> = self
                    .areas
                    .iter()
                    .filter(|a| !a.is_open())
                    .filter_map(|a| a.area.map(|area| area.to_string().to_lowercase()))
                    .collect();

                // All item IDs still present in the game.
                let all_item_ids: std::collections::BTreeSet<String> = self
                    .tributes
                    .iter()
                    .flat_map(|t| t.items.iter().map(|i| i.identifier.clone()))
                    .chain(
                        self.areas
                            .iter()
                            .flat_map(|a| a.items.iter().map(|i| i.identifier.clone())),
                    )
                    .collect();

                // Tribute identifier → area name mapping for same-area contact checks.
                let tribute_areas: HashMap<String, String> = self
                    .tributes
                    .iter()
                    .map(|t| (t.identifier.clone(), t.area.to_string()))
                    .collect();

                let fix_ctx = FixationContext {
                    cycle: (current_day.saturating_sub(1)) * 4 + phase.ord() as u32,
                    dead_tribute_killers: &dead_tribute_killers,
                    id_to_uuid: &id_to_uuid,
                    tribute_areas: &tribute_areas,
                    closed_areas: &closed_areas,
                    all_item_ids: &all_item_ids,
                };

                for idx in &fixation_indices {
                    let msgs = process_tribute_fixations(&mut self.tributes[*idx], &fix_ctx);
                    for msg in msgs {
                        let line = fixation_message_line(&msg, &self.tributes[*idx].name);
                        collected_events.push((
                            self.tributes[*idx].identifier.clone(),
                            self.tributes[*idx].name.clone(),
                            line,
                            Some(msg),
                            None,
                        ));
                    }
                }
            }
        }

        self.flush_tribute_events(collected_events);

        // Promote drained alliance events into the game queue and process them
        // so betrayal/death cascades take effect before the next cycle.
        if !drained_alliance_events.is_empty() {
            self.alliance_events.append(&mut drained_alliance_events);
            self.process_alliance_events(rng);
        }
        Ok(())
    }

    /// Drain collected per-tribute events into `self.messages`.
    ///
    /// Each contiguous run of events sharing the same `identifier` is one
    /// tribute action and gets a single fresh tick from `self.tick_counter`.
    /// Per-event `emit_index` (advanced inside `push_message`) preserves
    /// intra-tribute ordering. Sites carrying a typed `MessagePayload` push
    /// that payload directly; legacy stringly sites synthesise a fallback.
    ///
    /// Message coalescing (spec §11.5): repeated `MovedTo` events for the
    /// same area within a phase collapse into a single emission — only the
    /// first is kept, duplicates are silently dropped.
    pub(super) fn flush_tribute_events(&mut self, collected_events: Vec<CollectedEvent>) {
        use crate::messages::MessagePayload;

        let mut last_identifier: Option<String> = None;
        let mut current_tick: u32 = self.tick_counter.boundary();
        // Track last MovedTo area per tribute for coalescing.
        let mut last_move_area: HashMap<String, String> = HashMap::new();

        for (identifier, _name, content, payload, _event) in collected_events {
            // Coalesce: skip TributeMoved if same destination as last move for this tribute.
            if let Some(MessagePayload::TributeMoved { to: dest_area, .. }) = &payload
                && let Some(last_area) = last_move_area.get(&identifier)
                && last_area == &dest_area.name
            {
                continue;
            }

            // Record the destination area for future coalescing.
            if let Some(MessagePayload::TributeMoved { to: dest_area, .. }) = &payload {
                last_move_area.insert(identifier.clone(), dest_area.name.clone());
            }

            if last_identifier.as_ref() != Some(&identifier) {
                current_tick = self.tick_counter.next();
                last_identifier = Some(identifier.clone());
            }
            let source = crate::messages::MessageSource::Tribute(identifier.clone());
            let payload = payload.unwrap_or_else(|| Self::fallback_payload(&source));
            self.push_message(source, identifier, content, payload, current_tick);
        }
    }

    /// Runs the tributes' logic for the current cycle.
    ///
    /// Thin wrapper that builds the immutable `CycleContext` from `&self`
    /// then runs `execute_cycle` with `&mut self`. The split makes the
    /// "snapshot" and "mutate" halves separately testable and gives
    /// gamemaker overrides a typed seam to inject suggestions.
    pub(super) fn run_tribute_cycle(
        &mut self,
        phase: crate::messages::Phase,
        rng: &mut SmallRng,
        closed_areas: Vec<Area>,
        living_tributes: Vec<Tribute>,
        living_tributes_count: usize,
    ) -> Result<(), GameError> {
        // Lazy-spawn sponsors for in-progress games created before sponsorship landed.
        if self.sponsors.is_empty() {
            self.spawn_sponsors(rng);
        }

        let ctx =
            self.build_cycle_context(phase, closed_areas, living_tributes, living_tributes_count);

        // Snapshot message count so we can isolate this cycle's payloads for
        // the sponsorship translator.
        let pre_cycle_msg_len = self.messages.len();

        self.execute_cycle(ctx, rng)?;

        // Sponsorship PR1: translate cycle messages → AudienceEvents and update affinities.
        // PR2: resolve gifts and deliver them.
        let cycle_payloads: Vec<shared::messages::MessagePayload> = self
            .messages
            .iter()
            .skip(pre_cycle_msg_len)
            .map(|m| m.payload.clone())
            .collect();
        {
            let ctx = crate::sponsors::SponsorContext::new(self);
            let mut all_events = Vec::new();
            for p in &cycle_payloads {
                all_events.extend(crate::sponsors::translate(p, &ctx));
            }
            crate::sponsors::update_affinities(self, &all_events);

            // Gift resolution: sponsors with high affinity and remaining budget
            // may deliver items to affected tributes.
            let gifts = crate::sponsors::resolve_gifts(self, &all_events, rng);
            for gift in gifts {
                let recipient_id = match &gift.payload {
                    shared::messages::MessagePayload::SponsorGift { recipient, .. } => {
                        recipient.identifier.clone()
                    }
                    _ => unreachable!(),
                };
                let line =
                    crate::output::GameOutput::SponsorGift(&recipient_id, &gift.item).to_string();
                let source = crate::messages::MessageSource::Game(self.identifier.clone());
                let subject = format!("sponsor_gift:{recipient_id}");
                let tick = self.tick_counter.next();
                let payload = gift.payload;
                if let Some(tribute) = self
                    .tributes
                    .iter_mut()
                    .find(|t| t.identifier == recipient_id)
                {
                    tribute.add_item(gift.item);
                }
                self.push_message(source, subject, line, payload, tick);
            }
        }

        Ok(())
    }

    /// Runs a cycle of the game, either day or night.
    /// 1. Announce area events.
    /// 2. Open an area if there are no open areas.
    /// 3. Trigger any events for this cycle if we're past the first three days.
    /// 4. Trigger Feast Day events.
    /// 5. Close more areas by spawning more events if the tributes are getting low.
    /// 6. Run the tribute cycle.
    /// 7. Update the tributes in the game.
    pub(super) fn do_a_cycle(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        // Announce area events
        self.announce_area_events()?;

        // If there are no open areas, we need to open one.
        self.ensure_open_area();

        // Trigger any events for this cycle
        self.trigger_cycle_events(phase, &mut rng)?;

        // If the tribute count is low, constrain them by closing areas.
        self.constrain_areas(&mut rng)?;

        self.tributes.shuffle(&mut rng);
        let closed_areas: Vec<Area> = self
            .closed_areas()
            .iter()
            .filter_map(|ad| ad.area)
            .clone()
            .collect();
        let living_tributes = self.living_tributes();
        let living_tributes_count: usize = living_tributes.len();

        self.run_tribute_cycle(
            phase,
            &mut rng,
            closed_areas,
            living_tributes,
            living_tributes_count,
        )?;
        Ok(())
    }

    /// Any tributes who have died in the current cycle will be moved to the "dead" list,
    /// and their items will be added to the area they died in.
    pub(super) fn clean_up_recent_deaths(&mut self) {
        let tribute_count = self.tributes.len();

        for i in 0..tribute_count {
            // Using a for loop to avoid mutable borrow issues
            if self.tributes[i].is_alive() {
                continue;
            }
            let tribute_items: Vec<Item> = self.tributes[i].items.clone();

            if self.tributes[i].status == TributeStatus::RecentlyDead {
                self.tributes[i].statistics.day_killed = self.day;
                let tribute_area = self.tributes[i].area;

                if let Some(area) = self.get_area_details_mut(tribute_area) {
                    for item in tribute_items {
                        area.add_item(item.clone());
                    }
                }
            }

            self.tributes[i].dies();
        }
    }

    /// Get a mutable reference to the area details for a given area.
    pub(super) fn get_area_details_mut(&mut self, area: Area) -> Option<&mut AreaDetails> {
        self.areas.iter_mut().find(|ad| ad.area == Some(area))
    }
}
