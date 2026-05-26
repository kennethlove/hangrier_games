pub mod actions;
pub mod afflictions;
pub mod alliances;
pub mod brains;
pub mod combat;
pub mod combat_beat;
pub mod combat_tuning;
pub mod events;
pub mod inventory;
pub mod lifecycle;
pub mod movement;
pub mod stamina_band;
pub mod statuses;
pub mod survival;
pub mod traits;

// Re-export key items from sub-modules
pub use combat::inflict_table::{
    HitSeverity, WeaponKind, lookup_break_mid_swing_inflict, lookup_inflicts,
};
pub use combat::{attack_contest, update_stats};
pub use movement::TravelResult;

pub mod helpers;
use helpers::*;
pub use helpers::{AfflictionDraft, calculate_stamina_cost};

use std::collections::{BTreeMap, BTreeSet};

use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::messages::{AreaRef, ItemRef, MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::afflictions::{AcquireResolution, can_acquire};
use crate::tributes::events::TributeEvent;
use actions::{Action, AttackOutcome};
use brains::Brain;
use fake::Fake;
use fake::faker::name::raw::*;
use fake::locales::*;
use rand::RngExt;
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, PhobiaTrigger, Severity, TraumaMetadata,
    TraumaSource,
};
use statuses::TributeStatus;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ActionSuggestion {
    pub action: Action,
    pub probability: Option<f64>,
}

#[derive(Debug)]
pub struct EnvironmentContext<'a> {
    pub is_day: bool,
    /// Full four-phase value for the current cycle. Used by sleep scoring
    /// (Brain::should_sleep) and emitters (TributeSlept/TributeWoke).
    pub phase: shared::messages::Phase,
    pub area_details: &'a mut AreaDetails,
    pub closed_areas: &'a [Area],
    pub available_destinations: Vec<crate::areas::DestinationInfo>,
    /// All known areas (read-only snapshot). Used by multi-hop
    /// pathfinding so the planner can reason about non-neighbor goals.
    pub all_areas: &'a [AreaDetails],
    /// Per-area count of living tributes. Fed into `Brain::choose_destination`
    /// as a crowd penalty so movement scoring naturally disperses crowded
    /// tributes without a call-site escape hatch.
    pub enemy_density: &'a std::collections::HashMap<Area, u32>,
    /// Current game day (1-indexed). Used to gate day-1-only behavior such
    /// as suppressing sponsor gifts in the opening cycle.
    pub current_day: u32,
    /// Combat & stamina tuning knobs threaded through `Action::Attack` so
    /// `Tribute::attacks` and `attack_contest` can read constants from a
    /// single owned source instead of file-level `const`s.
    pub combat_tuning: &'a crate::tributes::combat_tuning::CombatTuning,
}

#[derive(Clone, Debug)]
pub struct EncounterContext {
    pub nearby_tributes_count: u32,
    pub potential_targets: Vec<Tribute>,
    pub total_living_tributes: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tribute {
    /// Identifier
    pub identifier: String,
    /// Stable typed UUID. Mirrors `identifier` for callers that want a
    /// non-stringly-typed key (alliance graph, betrayal events).
    ///
    /// Serialized as `tribute_id` to avoid collision with SurrealDB's
    /// reserved `id` column on the `tribute` table — the SDK rejects any
    /// payload that carries a non-RecordId `id` field when a record id is
    /// also specified explicitly via `db.create(("tribute", ...))`.
    #[serde(
        default = "Uuid::new_v4",
        rename = "tribute_id",
        serialize_with = "serialize_uuid_as_string",
        deserialize_with = "deserialize_uuid_lenient"
    )]
    pub id: Uuid,
    /// Where are they?
    pub area: Area,
    /// What is their current status?
    pub status: TributeStatus,
    /// This is their thinker. Persisted across saves so runtime state
    /// (psychotic break, preferred-action overrides, derived thresholds)
    /// survives load. `#[serde(default)]` lets pre-fix rows that omit the
    /// `brain` column hydrate via `Brain::default()`.
    #[serde(default)]
    pub brain: Brain,
    /// How they present themselves to the real world
    pub avatar: Option<String>,
    /// Who created them in the real world
    #[serde(rename = "player_name")]
    pub human_player_name: Option<String>,
    /// What they like to go by
    pub name: String,
    /// Where they're from
    pub district: u32,
    /// Stats like fights won
    pub statistics: Statistics,
    /// Attributes like health
    pub attributes: Attributes,
    /// Items the tribute owns
    #[serde(default)]
    pub items: Vec<Item>,
    /// Events that have happened to the tribute
    #[serde(default)]
    pub events: Vec<TributeEvent>,
    #[serde(default)]
    pub editable: bool,
    /// Terrain types this tribute is familiar with
    #[serde(default)]
    pub terrain_affinity: Vec<crate::terrain::BaseTerrain>,
    /// Current stamina for actions
    pub stamina: u32,
    /// Maximum stamina capacity
    pub max_stamina: u32,
    /// Personality/behavior trait set. Replaces `BrainPersonality`.
    /// A tribute with zero traits behaves as the old `Balanced` baseline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<traits::Trait>,
    /// Pair-wise alliance graph. Symmetric: when A allies with B, both
    /// `allies` lists gain the other. Capped at `MAX_ALLIES`.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_uuids_as_strings",
        deserialize_with = "deserialize_uuids_lenient"
    )]
    pub allies: Vec<Uuid>,
    /// Turn counter for the Treacherous betrayal cadence. Reset on betrayal.
    #[serde(default)]
    pub turns_since_last_betrayal: u8,
    /// Set to `true` by the cycle drain when this tribute is the victim of a
    /// betrayal. Consumed at the top of `process_turn_phase` on the victim's
    /// next turn to drive the trust-shock cascade (spec §7.3c1).
    #[serde(default)]
    pub pending_trust_shock: bool,
    /// Per-tribute alliance event buffer, populated during `process_turn_phase`
    /// (e.g. on Treacherous betrayal) and drained by the game cycle into
    /// `Game.alliance_events` between turns. Transient; never persisted.
    #[serde(default, skip)]
    pub alliance_events: Vec<alliances::AllianceEvent>,
    /// Set by combat sites when this tribute is killed by another tribute
    /// (or by themselves on a fumble). Read and cleared by the game cycle
    /// when emitting `AllianceEvent::DeathRecorded` so allies receive the
    /// correct killer attribution. `None` for environmental/status deaths.
    /// Transient; never persisted.
    #[serde(default, skip)]
    pub recently_killed_by: Option<Uuid>,
    /// Survival debt counter; 0 = Sated. See `survival::hunger_band`.
    #[serde(default)]
    pub hunger: u8,
    /// Survival debt counter; 0 = Sated. See `survival::thirst_band`.
    #[serde(default)]
    pub thirst: u8,
    /// Phase index until which the tribute is considered sheltered.
    /// `None` = exposed.
    #[serde(default)]
    pub sheltered_until: Option<u32>,
    /// Escalating step counter for HP drain while Starving.
    #[serde(default)]
    pub starvation_drain_step: u8,
    /// Escalating step counter for HP drain while Dehydrated.
    #[serde(default)]
    pub dehydration_drain_step: u8,
    /// Phases since the tribute last completed a full sleep. Increments by 1
    /// each phase the tribute is *not* sleeping. Resets on
    /// `WakeReason::Rested`. See spec
    /// `2026-05-03-four-phase-day-design.md` §6.4.
    #[serde(default)]
    pub cycles_awake: u32,
    /// True while mid-`Action::Sleep`. Affects ambush vulnerability once the
    /// sleep mechanic lands; this PR keeps it `false` for every brain-driven
    /// tribute (substrate only).
    #[serde(default)]
    pub sleeping: bool,
    /// Phases left in the current `Action::Sleep` countdown. `0` when awake.
    /// Decremented by the engine; on hitting zero the tribute wakes with
    /// `WakeReason::Rested`.
    #[serde(default)]
    pub sleep_remaining: u8,
    /// Active afflictions keyed by (kind, body_part). Empty by default;
    /// serde skips serialization when empty to keep payloads lean.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub afflictions: BTreeMap<AfflictionKey, Affliction>,
}

impl Default for Tribute {
    fn default() -> Self {
        Tribute::new("Default Tribute".to_string(), None, None)
    }
}

impl Tribute {
    /// Creates a new Tribute with full health, sanity, and movement.
    pub fn new(name: String, district: Option<u32>, avatar: Option<String>) -> Self {
        let district = district.unwrap_or(0);
        let attributes = Attributes::new();
        let statistics = Statistics::default();

        let id_uuid: Uuid = Uuid::new_v4();
        let id: String = id_uuid.to_string();

        // Assign terrain affinity, traits, and personality based on district
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let terrain_affinity = if (1..=12).contains(&district) {
            crate::districts::assign_terrain_affinity(district as u8, &mut rng)
        } else {
            vec![]
        };
        let traits = traits::generate_traits(district as u8, &mut rng);
        let brain = Brain::from_traits(&traits, &mut rng);

        let tribute = Self {
            identifier: id,
            id: id_uuid,
            area: Area::Cornucopia,
            name: name.clone(),
            district,
            brain,
            status: TributeStatus::default(),
            avatar,
            human_player_name: None,
            attributes,
            statistics,
            items: vec![],
            events: vec![],
            editable: true,
            terrain_affinity,
            stamina: 100,
            max_stamina: 100,
            traits,
            allies: Vec::new(),
            turns_since_last_betrayal: 0,
            pending_trust_shock: false,
            alliance_events: Vec::new(),
            recently_killed_by: None,
            hunger: 0,
            thirst: 0,
            sheltered_until: None,
            starvation_drain_step: 0,
            dehydration_drain_step: 0,
            cycles_awake: 0,
            sleeping: false,
            sleep_remaining: 0,
            afflictions: BTreeMap::new(),
        };

        // ~5% chance to spawn with an innate fixation.

        tribute
    }

    pub fn random() -> Self {
        let name = Name(EN).fake();
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let district = rng.random_range(1..=12);
        Tribute::new(name, Some(district), None)
    }

    pub fn avatar(&self) -> String {
        if self.avatar.is_none() {
            return "https://fallback.pics/api/v1/400x400".to_string();
        }
        format!("assets/{}", self.avatar.clone().unwrap())
    }

    /// Send a tribute through a game cycle.
    /// This is the main function that runs the tribute's actions.
    /// 1. Ignore dead tributes.
    /// 2. Process status effects including area events.
    /// 3. Check for gifts from sponsors.
    /// 4. Check for nighttime effects.
    /// 5. Check for suggested actions.
    /// 6. Get the tribute's action from the brain.
    /// 7. Perform the action.
    /// 8. Log the action.
    pub fn process_turn_phase(
        &mut self,
        action_suggestion: Option<ActionSuggestion>,
        environment_details: &mut EnvironmentContext<'_>,
        encounter_context: EncounterContext,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        // Tribute is already dead, do nothing. (No event emitted — noise drop.)
        if !self.is_alive() {
            return;
        }

        // Advance per-tribute alliance timers (spec §7.4). Ticks
        // turns_since_last_betrayal so Treacherous tributes can betray on
        // cadence. Skipped for dead tributes via `is_alive` guard above.
        self.tick_alliance_timers();

        // Consume any pending trust-shock from a betrayal recorded last turn
        // (spec §7.3c1). Rolls per ally; broken allies are removed locally.
        self.consume_pending_trust_shock(rng, events);

        let area_details = &mut environment_details.area_details;

        // Update the tribute based on the period's events.
        self.process_status(area_details, rng, events);

        // Tribute died to the period's events.
        if self.status == TributeStatus::RecentlyDead || self.attributes.health == 0 {
            let line = GameOutput::TributeDead(self.name.as_str()).to_string();
            events.push(TaggedEvent::new(
                line,
                MessagePayload::TributeKilled {
                    victim: TributeRef {
                        identifier: self.identifier.clone(),
                        name: self.name.clone(),
                    },
                    killer: None,
                    cause: "untracked".into(),
                },
            ));
            return;
        }

        // Treacherous active betrayal (spec §7.4(b)). When the timer has
        // elapsed and the tribute carries the Treacherous trait, attempt to
        // betray a same-area ally. On success, drop the symmetric pair
        // locally, enqueue BetrayalRecorded so the victim's `allies` is
        // cleaned and `pending_trust_shock` flips on the next drain. The
        // timer resets unconditionally so a missed opportunity does not
        // stack (one chance per cadence).
        if self.traits.contains(&traits::Trait::Treacherous)
            && self.turns_since_last_betrayal >= alliances::TREACHEROUS_BETRAYAL_INTERVAL
        {
            let same_area_ally = encounter_context
                .potential_targets
                .iter()
                .find(|t| self.allies.contains(&t.id) && t.is_alive())
                .cloned();
            if let Some(victim) = same_area_ally {
                self.allies.retain(|id| id != &victim.id);
                self.alliance_events
                    .push(alliances::AllianceEvent::BetrayalRecorded {
                        betrayer: self.id,
                        victim: victim.id,
                    });
            }
            self.turns_since_last_betrayal = 0;
        }

        // Nighttime terror
        let is_day = environment_details.is_day;
        if !is_day && self.is_alive() {
            self.misses_home();
        }

        // Check for psychotic breaks or recovery (sanity-based mental state changes)
        self.brain
            .check_psychotic_break(self.attributes.sanity, rng);
        self.brain.check_recovery(self.attributes.sanity);

        // Set a preferred action if one is suggested
        if let Some(suggestion) = action_suggestion {
            self.brain
                .set_preferred_action(suggestion.action, suggestion.probability.unwrap_or(1.0));
        }

        // Get tribute action
        let number_of_nearby_tributes = encounter_context.nearby_tributes_count;
        let action = if let Some(sleep_action) = self.brain.should_sleep(
            self,
            number_of_nearby_tributes,
            environment_details.phase,
            rng,
        ) {
            sleep_action
        } else {
            self.brain.act(
                self,
                number_of_nearby_tributes,
                &environment_details.available_destinations,
                environment_details.all_areas,
                environment_details.closed_areas,
                environment_details.enemy_density,
                environment_details.phase,
                rng,
            )
        };

        let closed_areas = environment_details.closed_areas;

        match action {
            Action::Move(area) => {
                self.act_move(
                    &area,
                    closed_areas,
                    &environment_details.available_destinations,
                    events,
                );
            }
            Action::Rest => {
                self.act_rest(events);
            }
            Action::Hide => {
                self.act_hide(events);
            }
            Action::Attack => {
                self.act_attack(
                    encounter_context.potential_targets,
                    encounter_context.total_living_tributes,
                    events,
                    rng,
                    environment_details.phase,
                    environment_details.combat_tuning,
                );
            }
            Action::TakeItem => {
                self.act_take_item(area_details, events);
            }
            Action::UseItem(maybe_item) => {
                self.act_use_item(&maybe_item, events);
            }
            Action::None => {}
            Action::ProposeAlliance => {
                self.act_propose_alliance(&encounter_context, rng, events);
            }
            Action::SeekShelter
            | Action::Forage
            | Action::DrinkFromTerrain
            | Action::Eat(_)
            | Action::DrinkItem(_) => {}
            Action::Sleep { duration_phases } => {
                self.act_sleep(duration_phases, environment_details.phase, events);
            }
            Action::Frozen => {
                // Tribute is frozen by phobia — skip this cycle.
                // Event emitted by the brain pipeline caller.
            }
            Action::Flashback { .. } | Action::Avoidance => {
                // Flashback and avoidance are handled by the brain layer
                // (trauma_override) which emits the event. No action needed here.
            }
        }
    }

    /// Pick a target tribute from `targets` to attack, given the number of
    /// living tributes in the game.
    ///
    /// Selection rules:
    /// 1. If there are no targets and the tribute is suicidal (very low sanity),
    ///    target self.
    /// 2. Otherwise, filter out current allies — they are off-limits regardless
    ///    of district.
    /// 3. If any non-allies remain, pick one at random.
    /// 4. If only allies are nearby (and we're not the last two alive), pick no
    ///    target. Final confrontation (only two alive) overrides alliance.
    fn pick_target(
        &self,
        mut targets: Vec<Tribute>,
        living_tributes_count: u32,
        events: &mut Vec<TaggedEvent>,
    ) -> Option<Tribute> {
        // If there are no targets, check if the tribute is feeling suicidal.
        if targets.is_empty() {
            return match self.attributes.sanity {
                0..=SANITY_BREAK_LEVEL => {
                    // attempt suicide
                    let line = GameOutput::TributeSuicide(self.name.as_str()).to_string();
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::TributeKilled {
                            victim: TributeRef {
                                identifier: self.identifier.clone(),
                                name: self.name.clone(),
                            },
                            killer: None,
                            cause: "suicide".into(),
                        },
                    ));
                    Some(self.clone())
                }
                _ => None, // Attack no one
            };
        }

        let enemies: Vec<Tribute> = targets
            .iter()
            .filter(|t| !self.allies.contains(&t.id))
            .cloned()
            .collect();

        if enemies.is_empty() {
            // Only allies in range. Final confrontation overrides loyalty.
            if targets.len() == 1 && living_tributes_count == 2 {
                return Some(targets.pop().unwrap());
            }
            return None;
        }

        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Some(enemies.choose(&mut rng).unwrap().clone())
    }

    // --- Per-Action executor helpers (extracted from process_turn_phase) ---

    fn act_move(
        &mut self,
        area: &Option<Area>,
        closed_areas: &[Area],
        available_destinations: &[crate::areas::DestinationInfo],
        events: &mut Vec<TaggedEvent>,
    ) {
        let tribute_ref = TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };

        let travel_result = match area {
            Some(specific_area) => self.travels(closed_areas, Some(*specific_area), events),
            None => self.travels(closed_areas, None, events),
        };

        match travel_result {
            TravelResult::Success(destination) => {
                let dest_info = available_destinations
                    .iter()
                    .find(|d| d.area == destination);

                match dest_info {
                    Some(info) => {
                        if self.stamina >= info.stamina_cost {
                            self.area = destination;
                            self.stamina = self.stamina.saturating_sub(info.stamina_cost);
                        } else {
                            events.pop();
                            self.short_rests();
                            let line = GameOutput::TributeTravelExhausted(
                                self.name.as_str(),
                                &self.area.to_string(),
                            )
                            .to_string();
                            events.push(TaggedEvent::new(
                                line,
                                MessagePayload::TributeRested {
                                    tribute: tribute_ref,
                                    hp_restored: 0,
                                },
                            ));
                        }
                    }
                    None => {
                        self.short_rests();
                    }
                }
            }
            TravelResult::Failure => {
                self.short_rests();
            }
        }
    }

    fn act_rest(&mut self, events: &mut Vec<TaggedEvent>) {
        let tribute_ref = TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };
        let line = GameOutput::TributeRest(self.name.as_str()).to_string();
        events.push(TaggedEvent::new(
            line,
            MessagePayload::TributeRested {
                tribute: tribute_ref,
                hp_restored: 0,
            },
        ));
        self.long_rests();
    }

    fn act_hide(&mut self, events: &mut Vec<TaggedEvent>) {
        let tribute_ref = TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };
        let area_ref = |a: Area| {
            let s = a.to_string();
            AreaRef {
                identifier: s.clone(),
                name: s,
            }
        };

        let _hidden = self.hides();
        let current_area = self.area;
        let line = GameOutput::TributeHide(self.name.as_str()).to_string();
        events.push(TaggedEvent::new(
            line,
            MessagePayload::TributeHidden {
                tribute: tribute_ref,
                area: area_ref(current_area),
            },
        ));
    }

    fn act_attack(
        &mut self,
        potential_targets: Vec<Tribute>,
        total_living_tributes: u32,
        events: &mut Vec<TaggedEvent>,
        rng: &mut impl Rng,
        phase: shared::messages::Phase,
        combat_tuning: &crate::tributes::combat_tuning::CombatTuning,
    ) {
        let target = self.pick_target(potential_targets, total_living_tributes, events);
        if let Some(mut target) = target {
            let outcome = self.attacks(&mut target, rng, events, phase, combat_tuning);
            match outcome {
                AttackOutcome::Kill(_, mut target) => {
                    self.statistics.kills += 1;
                    target.statistics.day_killed = Some(self.statistics.game.parse().unwrap_or(1));
                }
                AttackOutcome::Wound(_, _) | AttackOutcome::Miss(_, _) => {}
            }
        }
    }

    fn act_take_item(&mut self, area_details: &mut AreaDetails, events: &mut Vec<TaggedEvent>) {
        if let Some(item) = self.take_nearby_item(area_details) {
            let tribute_ref = TributeRef {
                identifier: self.identifier.clone(),
                name: self.name.clone(),
            };
            let area_ref = |a: Area| {
                let s = a.to_string();
                AreaRef {
                    identifier: s.clone(),
                    name: s,
                }
            };
            let line = GameOutput::TributeTakeItem(self.name.as_str(), &item.name).to_string();
            let item_ref = ItemRef {
                identifier: item.identifier.clone(),
                name: item.name.clone(),
            };
            let current_area = self.area;
            events.push(TaggedEvent::new(
                line,
                MessagePayload::ItemFound {
                    tribute: tribute_ref,
                    item: item_ref,
                    area: area_ref(current_area),
                },
            ));
        }
    }

    fn act_use_item(&mut self, maybe_item: &Option<Item>, events: &mut Vec<TaggedEvent>) {
        if let Some(item) = maybe_item {
            let tribute_ref = TributeRef {
                identifier: self.identifier.clone(),
                name: self.name.clone(),
            };
            if let Err(error) = self.try_use_consumable(item) {
                let line = GameOutput::TributeCannotUseItem(self.name.as_str(), &error.to_string())
                    .to_string();
                let item_ref = ItemRef {
                    identifier: item.identifier.clone(),
                    name: item.name.clone(),
                };
                events.push(TaggedEvent::new(
                    line,
                    MessagePayload::ItemUsed {
                        tribute: tribute_ref,
                        item: item_ref,
                    },
                ));
            } else {
                let line = GameOutput::TributeUseItem(self.name.as_str(), item).to_string();
                let item_ref = ItemRef {
                    identifier: item.identifier.clone(),
                    name: item.name.clone(),
                };
                events.push(TaggedEvent::new(
                    line,
                    MessagePayload::ItemUsed {
                        tribute: tribute_ref,
                        item: item_ref,
                    },
                ));
            }
        }
    }

    fn act_propose_alliance(
        &mut self,
        encounter_context: &EncounterContext,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        use crate::tributes::alliances::{
            MAX_ALLIES, deciding_factor, passes_gate, try_form_alliance,
        };

        let tribute_ref = TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };

        let candidates: Vec<&Tribute> = encounter_context
            .potential_targets
            .iter()
            .filter(|t| t.is_alive())
            .filter(|t| !self.allies.contains(&t.id))
            .filter(|t| t.allies.len() < MAX_ALLIES)
            .filter(|t| passes_gate(&self.traits, &t.traits))
            .collect();

        // Phobia veto (qqqx PR3 spec §12):
        // Hard veto: cannot propose alliance if you have Phobia(Tribute).
        if self
            .afflictions
            .values()
            .any(|aff| matches!(aff.kind, AfflictionKind::Phobia(PhobiaTrigger::Tribute)))
        {
            return;
        }

        // Soft penalty: Phobia(TraitGroup) reduces formation chance.
        let phobia_penalty: f64 = self
            .afflictions
            .values()
            .filter_map(|aff| {
                if matches!(aff.kind, AfflictionKind::Phobia(PhobiaTrigger::TraitGroup)) {
                    Some(aff.severity.ordinal() as f64 * 0.15)
                } else {
                    None
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Trauma penalty (5477 PR3 spec §12):
        // Soft penalty: each trauma reduces formation chance by -0.15 per severity tier.
        let trauma_penalty: f64 = self
            .afflictions
            .values()
            .filter(|aff| matches!(aff.kind, AfflictionKind::Trauma))
            .map(|aff| aff.severity.ordinal() as f64 * 0.15)
            .sum::<f64>()
            .min(1.0); // Cap at 1.0 so chance can't go below 0

        // Hard veto: Severe trauma with Betrayal source hard-blocks alliance.
        let has_betrayal_veto = self.afflictions.values().any(|aff| {
            if !matches!(aff.kind, AfflictionKind::Trauma) {
                return false;
            }
            if aff.severity < Severity::Severe {
                return false;
            }
            matches!(
                aff.trauma_metadata,
                Some(ref m) if matches!(m.source, TraumaSource::Betrayal { .. })
            )
        });
        if has_betrayal_veto {
            return;
        }

        if candidates.is_empty() {
            return;
        }
        let target = {
            use rand::seq::IndexedRandom;
            candidates.choose(rng).cloned().unwrap()
        };
        let target_ref = TributeRef {
            identifier: target.identifier.clone(),
            name: target.name.clone(),
        };

        let proposed_line = format!("🤝 {} proposes an alliance to {}.", self.name, target.name);
        events.push(TaggedEvent::new(
            proposed_line,
            MessagePayload::AllianceProposed {
                proposer: tribute_ref,
                target: target_ref.clone(),
            },
        ));

        // Sympathetic bond (5477 PR3 spec §12):
        // If target has observed a flashback, +0.10 bonus offsetting trauma penalty.
        let trauma_observer_bonus: f64 = if self.afflictions.values().any(|aff| {
            if !matches!(aff.kind, AfflictionKind::Trauma) {
                return false;
            }
            aff.trauma_metadata
                .as_ref()
                .is_some_and(|m| m.observed_by.contains(&target.identifier))
        }) {
            0.10
        } else {
            0.0
        };

        let same_district = self.district == target.district;
        let formed = try_form_alliance(
            &self.traits,
            &target.traits,
            same_district,
            self.allies.len(),
            target.allies.len(),
            phobia_penalty,
            trauma_penalty - trauma_observer_bonus,
            rng,
        );
        if formed {
            let factor = deciding_factor(&self.traits, &target.traits, same_district);
            let factor_label = factor
                .as_ref()
                .map(|f| f.label())
                .unwrap_or("mutual circumstance")
                .to_string();
            self.alliance_events
                .push(alliances::AllianceEvent::FormationRecorded {
                    proposer: self.id,
                    target: target.id,
                    factor: factor_label,
                });
        }
    }

    fn act_sleep(
        &mut self,
        duration_phases: u8,
        phase: shared::messages::Phase,
        events: &mut Vec<TaggedEvent>,
    ) {
        let tribute_ref = TributeRef {
            identifier: self.identifier.clone(),
            name: self.name.clone(),
        };
        self.sleeping = true;
        self.sleep_remaining = duration_phases;
        events.push(TaggedEvent::new(
            GameOutput::TributeSleeps(self.name.as_str()).to_string(),
            MessagePayload::TributeSlept {
                tribute: tribute_ref,
                phase,
                restored_stamina: 0,
                restored_hp: 0,
            },
        ));
    }

    /// Wake an interrupted sleeper (PR2c.2, bd-1zju). Resets the sleep
    /// state and `cycles_awake` per spec §6.4 ("rude awakening = no rest")
    /// and pushes a `TributeWoke { Interrupted }` event onto `events`.
    /// Returns `true` if the tribute was actually sleeping (and was woken),
    /// `false` otherwise — letting callers branch on whether the
    /// interruption-trigger path applies further consequences.
    pub fn wake_interrupted(
        &mut self,
        reason: shared::messages::InterruptionKind,
        phase: shared::messages::Phase,
        events: &mut Vec<TaggedEvent>,
    ) -> bool {
        use shared::messages::WakeReason;

        if !self.sleeping {
            return false;
        }
        self.sleeping = false;
        self.sleep_remaining = 0;
        self.cycles_awake = 0;
        events.push(TaggedEvent::new(
            crate::output::GameOutput::TributeWakesInterrupted(self.name.as_str()).to_string(),
            MessagePayload::TributeWoke {
                tribute: TributeRef {
                    identifier: self.identifier.clone(),
                    name: self.name.clone(),
                },
                phase,
                reason: WakeReason::Interrupted { event: reason },
            },
        ));
        true
    }

    /// Drain this tribute's per-turn alliance event buffer. Called by
    /// `Game::run_tribute_cycle` after each tribute's turn so the events
    /// are appended to the game's queue and processed before the next
    /// tribute acts. See spec §7.5.
    pub fn drain_alliance_events(&mut self) -> Vec<alliances::AllianceEvent> {
        std::mem::take(&mut self.alliance_events)
    }

    /// Advance per-tribute alliance bookkeeping for one turn (spec §7.4).
    ///
    /// Ticks `turns_since_last_betrayal`, which gates Treacherous-trait
    /// betrayals to fire at most every
    /// [`alliances::TREACHEROUS_BETRAYAL_INTERVAL`] turns. Saturates at
    /// `u8::MAX` so a long-lived tribute never overflows or wraps back
    /// through the betrayal trigger. Dead tributes are skipped.
    pub fn tick_alliance_timers(&mut self) {
        if !self.is_alive() {
            return;
        }
        self.turns_since_last_betrayal = self.turns_since_last_betrayal.saturating_add(1);
    }

    /// Consume a pending trust-shock flag (set when this tribute was the
    /// victim of a betrayal). For each current ally, roll
    /// [`alliances::trust_shock_roll`]; on success drop that ally from this
    /// tribute's `allies` list and emit a message. The flag is reset
    /// unconditionally so it never carries past the turn it fires.
    ///
    /// Note: this only mutates `self`. The symmetric back-edge on the broken
    /// ally's side is left to the next cycle's processing or to subsequent
    /// alliance events; per Phase 4 of the implementation plan, full
    /// symmetric cleanup is deferred. See spec §7.3c1.
    pub fn consume_pending_trust_shock(
        &mut self,
        rng: &mut impl rand::Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        if !self.pending_trust_shock {
            return;
        }
        let limit = self.brain.thresholds.extreme_low_sanity;
        let sanity = self.attributes.sanity;
        let mut broken: Vec<Uuid> = Vec::new();
        for ally_id in &self.allies {
            if alliances::trust_shock_roll(sanity, limit, rng) {
                broken.push(*ally_id);
            }
        }
        for ally_id in &broken {
            self.allies.retain(|x| x != ally_id);
            let ally_str = ally_id.to_string();
            let line = format!(
                "{} loses faith and breaks ties with ally {}.",
                self.name, ally_id
            );
            events.push(TaggedEvent::new(
                line,
                MessagePayload::TrustShockBreak {
                    tribute: TributeRef {
                        identifier: self.identifier.clone(),
                        name: self.name.clone(),
                    },
                    partner: TributeRef {
                        identifier: ally_str.clone(),
                        name: ally_str,
                    },
                },
            ));
        }
        self.pending_trust_shock = false;
    }

    /// Attempt to acquire a new affliction, resolving against existing afflictions.
    /// Returns the resolution (Insert, Upgrade, Supersede, or Reject).
    pub fn try_acquire_affliction(&mut self, draft: AfflictionDraft) -> AcquireResolution {
        let provisional = Affliction {
            kind: draft.kind.clone(),
            body_part: draft.body_part,
            severity: draft.severity,
            source: draft.source.clone(),
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
        };
        let resolution = can_acquire(&self.afflictions, &provisional);

        match &resolution {
            AcquireResolution::Insert
            | AcquireResolution::Upgrade(_)
            | AcquireResolution::Supersede(_) => {
                // Handle Supersede: remove superseded afflictions
                if let AcquireResolution::Supersede(_) = resolution {
                    // MissingArm/MissingLeg supersedes all afflictions on that body part
                    let part = draft.body_part;
                    let keys_to_remove: Vec<_> = self
                        .afflictions
                        .keys()
                        .filter(|(_, bp)| bp == &part)
                        .cloned()
                        .collect();
                    for key in keys_to_remove {
                        self.afflictions.remove(&key);
                    }
                }

                // Infected supersedes Wounded at same body part (cascade rule).
                // can_acquire returns Insert for this case, but we still need
                // to remove the Wounded ancestor.
                if draft.kind == AfflictionKind::Infected {
                    let wounded_key = (AfflictionKind::Wounded, draft.body_part);
                    self.afflictions.remove(&wounded_key);
                }

                let insert_key = (draft.kind.clone(), draft.body_part);
                let affliction = Affliction {
                    kind: draft.kind,
                    body_part: draft.body_part,
                    severity: draft.severity,
                    source: draft.source,
                    acquired_cycle: 0,
                    last_progressed_cycle: 0,
                    trauma_metadata: None,
                    phobia_metadata: None,
                    fixation_metadata: None,
                };
                let is_fixation = matches!(affliction.kind, AfflictionKind::Fixation(_));
                self.afflictions.insert(insert_key.clone(), affliction);

                // Set default fixation metadata for Fixation kinds inserted
                // through this pathway (used by generic affliction acquisition).
                if is_fixation && let Some(aff) = self.afflictions.get_mut(&insert_key) {
                    aff.fixation_metadata = Some(shared::afflictions::FixationMetadata {
                        origin: shared::afflictions::FixationOrigin::Innate,
                    });
                }
            }
            AcquireResolution::Reject(_) => {}
        }

        resolution
    }

    /// Attempt to acquire or reinforce trauma on this tribute.
    ///
    /// If the tribute already has trauma, it is reinforced to the higher
    /// severity (or stays the same if already at that severity). If not,
    /// new trauma is acquired.
    pub fn try_acquire_trauma(
        &mut self,
        source: shared::afflictions::TraumaSource,
        severity: shared::afflictions::Severity,
    ) -> crate::tributes::afflictions::TraumaAcquisition {
        use crate::tributes::afflictions::TraumaAcquisition;
        use shared::afflictions::{AfflictionKind, AfflictionSource};

        let key = (AfflictionKind::Trauma, None);
        if let Some(existing) = self.afflictions.get_mut(&key) {
            let from = existing.severity;
            let to = std::cmp::max(from, severity);
            let floor_bumped = to > from;
            if floor_bumped {
                existing.severity = to;
                existing.trauma_metadata = Some(TraumaMetadata {
                    source,
                    cycles_since_last_fire: 0,
                    observed_by: BTreeSet::new(),
                    observer_seen_cycle: BTreeMap::new(),
                });
            }
            TraumaAcquisition::Reinforced {
                from_severity: from,
                to_severity: to,
                floor_bumped,
            }
        } else {
            let aff = shared::afflictions::Affliction {
                kind: AfflictionKind::Trauma,
                body_part: None,
                severity,
                source: AfflictionSource::Environmental,
                acquired_cycle: 0,
                last_progressed_cycle: 0,
                trauma_metadata: Some(TraumaMetadata {
                    source: source.clone(),
                    cycles_since_last_fire: 0,
                    observed_by: BTreeSet::new(),
                    observer_seen_cycle: BTreeMap::new(),
                }),
                phobia_metadata: None,
                fixation_metadata: None,
            };
            self.afflictions.insert(key, aff);
            TraumaAcquisition::Acquired { severity, source }
        }
    }

    /// Apply affliction-based hard gates to an action at execution time.
    ///
    /// This is the terrain-aware complement to the pre-decision
    /// `affliction_override` layer. Some gates (MissingLeg → cliff/swamp
    /// terrain) require knowledge of the destination area's terrain, which
    /// is only available after the brain has decided on a Move action.
    ///
    /// Returns `Some(fallback)` if the action is blocked, `None` if allowed.
    ///
    /// Gates (spec §11):
    /// - MissingLeg (Moderate+) + cliff/swamp destination → Rest
    /// - Blind (Moderate+) + ranged attack → Move(None)
    ///   (Action::Attack covers both melee+ranged; gated when distinct
    ///   ranged variant exists.)
    /// - MissingArm (Moderate+) + 2H weapon → fallback
    ///   (Weapon info not in Action yet; enforced at combat resolution.)
    pub fn affliction_action_gate(
        &self,
        action: &Action,
        destination_terrain: Option<crate::terrain::BaseTerrain>,
    ) -> Option<Action> {
        crate::tributes::brains::affliction_override::hard_gates_with_terrain(
            self,
            action,
            destination_terrain,
        )
    }

    /// Compute visible stat modifiers for this tribute, combining
    /// affliction penalties with phobia stat penalties.
    ///
    /// Returns a `StatModifiers` struct with all penalties composed.
    /// Phobia penalties are additive with affliction penalties, capped
    /// at -10 total for phobias (applied to atk and def).
    pub fn visible_modifiers(
        &self,
        config: &crate::config::GameConfig,
        phobia_ctx: Option<&crate::tributes::brains::phobia_override::PhobiaBrainContext<'_>>,
    ) -> afflictions::StatModifiers {
        let mut mods = afflictions::compute_stat_modifiers(
            &self.afflictions.values().cloned().collect::<Vec<_>>(),
        );

        // Compose phobia penalties if enabled and context available.
        if config.phobias_enabled
            && !self.afflictions.is_empty()
            && let Some(ctx) = phobia_ctx
        {
            let phobia_penalty =
                crate::tributes::brains::phobia_override::phobia_stat_penalty(self, ctx);
            mods.atk += phobia_penalty;
            mods.def += phobia_penalty;
        }

        mods
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Statistics {
    /// What day, if any, were they killed?
    pub day_killed: Option<u32>,
    /// Who or what killed them?
    pub killed_by: Option<String>,
    /// How many tributes did they kill?
    pub kills: u32,
    /// How many fights did they win?
    pub wins: u32,
    /// How many fights did they lose?
    pub defeats: u32,
    /// How many fights ended in a draw?
    pub draws: u32,
    /// Which game do these stats relate to?
    pub game: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attributes {
    /// How much damage can they take?
    pub health: u32,
    /// How much suffering can they handle? Are they still sane?
    pub sanity: u32,
    /// How far can they move before they need a rest?
    pub movement: u32,
    /// How hard do they hit?
    pub strength: u32,
    /// How hard of a hit can they take?
    pub defense: u32,
    /// Will they jump into dangerous situations?
    pub bravery: u32,
    /// How well do they avoid traps?
    pub intelligence: u32,
    /// Can they talk their way out of, or into, things?
    pub persuasion: u32,
    /// Are they likely to get gifts or come out slightly ahead?
    pub luck: u32,
    /// How quickly they react in a turn.
    pub agility: u32,
    /// Can other tributes see them?
    pub is_hidden: bool,
}

impl Default for Attributes {
    /// Provides a maxed-out set of Attributes
    fn default() -> Self {
        Self {
            health: 100,
            sanity: 100,
            movement: 100,
            strength: 50,
            defense: 50,
            bravery: 100,
            intelligence: 100,
            persuasion: 100,
            luck: 100,
            agility: 50,
            is_hidden: false,
        }
    }
}

impl Attributes {
    /// Provides a randomized set of Attributes using default config values
    pub fn new() -> Self {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let config = crate::config::GameConfig::default();

        Self {
            health: rng.random_range(50..=config.max_health),
            sanity: rng.random_range(50..=config.max_sanity),
            movement: config.max_movement,
            strength: rng.random_range(1..=config.max_strength),
            defense: rng.random_range(1..=config.max_defense),
            bravery: rng.random_range(1..=config.max_bravery),
            intelligence: rng.random_range(1..=config.max_intelligence),
            persuasion: rng.random_range(1..=config.max_persuasion),
            luck: rng.random_range(1..=config.max_luck),
            agility: rng.random_range(1..=config.max_agility),
            is_hidden: false,
        }
    }
}

#[cfg(test)]
mod tests;
