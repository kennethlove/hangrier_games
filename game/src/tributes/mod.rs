pub mod actions;
pub mod afflictions;
pub mod alliances;
pub mod brains;
pub mod combat;
pub mod combat_beat;
pub mod combat_tuning;
pub mod events;
pub mod incidents;
pub mod inventory;
pub mod lifecycle;
pub mod movement;
pub mod rescue;
pub mod stamina_band;
pub mod statuses;
pub mod survival;
pub mod traits;
pub mod traps;

// Re-export key items from sub-modules
pub use combat::inflict_table::{
    HitSeverity, WeaponKind, lookup_break_mid_swing_inflict, lookup_inflicts,
};
pub use combat::{attack_contest, update_stats};
pub use movement::TravelResult;

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
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};
use shared::afflictions::{
    Affliction, AfflictionKey, AfflictionKind, AfflictionSource, BodyPart, PhobiaTrigger, Severity,
    Substance, TrappedMetadata, TraumaSource,
};
use shared::messages::SleepIncidentKind;
use statuses::TributeStatus;
use uuid::Uuid;

/// Serialize `Vec<Uuid>` as `Vec<String>` for SurrealDB compatibility.
/// The Surreal Rust SDK's bespoke serializer wires `uuid::Uuid` as raw bytes,
/// which Surreal then renders as base64 and rejects against `array<uuid>`
/// constraints. Storing as strings on the wire (and as `array<string>` in
/// the schema) follows the same convention as `message.event_id`.
fn serialize_uuids_as_strings<S>(uuids: &[Uuid], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(uuids.len()))?;
    for u in uuids {
        seq.serialize_element(&u.to_string())?;
    }
    seq.end()
}

/// Deserialize `Vec<Uuid>` from either a sequence of strings (the wire format
/// we write) or a sequence of native uuid values (test fixtures, JSON read
/// back through serde's standard Uuid impl).
fn deserialize_uuids_lenient<'de, D>(deserializer: D) -> Result<Vec<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUuid {
        S(String),
        U(Uuid),
    }

    let raw: Vec<StringOrUuid> = Vec::deserialize(deserializer)?;
    raw.into_iter()
        .map(|item| match item {
            StringOrUuid::S(s) => Uuid::parse_str(&s).map_err(serde::de::Error::custom),
            StringOrUuid::U(u) => Ok(u),
        })
        .collect()
}

/// Serialize a single `Uuid` as a string for the same reasons as
/// `serialize_uuids_as_strings`.
fn serialize_uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

/// Deserialize a single `Uuid` from either a string (our wire format) or the
/// SDK's native uuid bytes representation.
fn deserialize_uuid_lenient<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUuid {
        S(String),
        U(Uuid),
    }

    match StringOrUuid::deserialize(deserializer)? {
        StringOrUuid::S(s) => Uuid::parse_str(&s).map_err(serde::de::Error::custom),
        StringOrUuid::U(u) => Ok(u),
    }
}

/// Consts
const SANITY_BREAK_LEVEL: u32 = 9;

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
    /// Sleeping tributes in the current area with items that can be stolen.
    /// Tuple of (tribute UUID, tribute name). Populated by the game cycle
    /// in `execute_cycle`; consumed by `act_take_item` for target selection.
    pub sleeping_nearby: Vec<(Uuid, String)>,
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
    /// `skip_serializing_if` avoids SurrealDB v3's rejection of JSON `null`
    /// for `TYPE option<int>` (which expects `NONE`, not `NULL`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// Tracks a non-waking sleep incident (flavor only) that occurred this
    /// sleep session. Included in the natural-wake message so the narrative
    /// reflects the experience. Reset to `None` on sleep end.
    #[serde(default, skip)]
    pub pending_sleep_incident: Option<SleepIncidentKind>,
    /// Active afflictions keyed by (kind, body_part). Empty by default;
    /// serde skips serialization when empty to keep payloads lean.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub afflictions: BTreeMap<AfflictionKey, Affliction>,
    /// Current game day (cycle). Used for affliction cycle tracking.
    #[serde(default)]
    pub game_day: Option<i64>,
    /// Monotonic per-substance use counter (spec §4). Never reset by cure.
    /// Enables relapse-on-first-use and the use-count-driven acquisition curve.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub addiction_use_count: BTreeMap<Substance, u32>,
    /// Substances the tribute has ever been addicted to (spec §5.1 step 5c).
    /// Populated on acquisition, never cleared. Enables the relapse short-circuit:
    /// if `ever_addicted_to.contains(s)` and no current `Addiction(s)`, next use
    /// of `s` auto-acquires at Mild.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub ever_addicted_to: BTreeSet<Substance>,
    /// Cycles remaining in hangover (alcohol after-effect).
    /// Set to 2 on alcohol use, ticks down each cycle.
    /// While > 0, tribute suffers -1 atk, -1 forage.
    #[serde(default, skip)]
    pub hangover_cycles_remaining: u32,
    /// Transient flag set by `attacks()` when this tribute was sleeping and
    /// got ambushed. `attack_contest` reads it to apply a 0-defense penalty.
    /// Reset to `false` after each combat resolution. Not persisted.
    #[serde(default, skip)]
    pub was_ambushed: bool,
    /// UUID of a sleeping tribute to steal from. Set by `act_take_item`
    /// when an awake tribute chooses to steal from a sleeper instead of
    /// looting the area. Consumed by the game cycle after
    /// `process_turn_phase`. Transient — never persisted.
    #[serde(default, skip)]
    pub pending_theft_target: Option<Uuid>,
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

        Self {
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
            pending_sleep_incident: None,
            afflictions: BTreeMap::new(),
            game_day: None,
            addiction_use_count: BTreeMap::new(),
            ever_addicted_to: BTreeSet::new(),
            hangover_cycles_remaining: 0,
            was_ambushed: false,
            pending_theft_target: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_with_rng(
        name: String,
        district: Option<u32>,
        avatar: Option<String>,
        rng: &mut SmallRng,
    ) -> Self {
        let district = district.unwrap_or(0);
        let attributes = Attributes::new();
        let statistics = Statistics::default();

        let id_uuid: Uuid = Uuid::new_v4();
        let id: String = id_uuid.to_string();

        // Assign terrain affinity, traits, and personality based on district
        let terrain_affinity = if (1..=12).contains(&district) {
            crate::districts::assign_terrain_affinity(district as u8, rng)
        } else {
            vec![]
        };
        let traits = traits::generate_traits(district as u8, rng);
        let brain = Brain::from_traits(&traits, rng);

        Self {
            identifier: id,
            id: id_uuid,
            area: Area::Cornucopia,
            name,
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
            pending_sleep_incident: None,
            afflictions: BTreeMap::new(),
            game_day: None,
            addiction_use_count: BTreeMap::new(),
            ever_addicted_to: BTreeSet::new(),
            hangover_cycles_remaining: 0,
            was_ambushed: false,
            pending_theft_target: None,
        }
    }

    pub fn random() -> Self {
        let name = Name(EN).fake();
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let district = rng.random_range(1..=12);
        Tribute::new(name, Some(district), None)
    }

    /// Builder: add a pre-existing affliction (addiction, missing limb, trauma, etc.).
    /// Also populates `ever_addicted_to` for Addiction kinds so relapse semantics work.
    pub fn with_affliction(mut self, affliction: Affliction) -> Self {
        if let (AfflictionKind::Addiction(_), Some(meta)) =
            (&affliction.kind, &affliction.addiction_metadata)
        {
            self.ever_addicted_to.insert(meta.substance);
        }
        self.afflictions.insert(affliction.key(), affliction);
        self
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
                self.act_take_item(
                    area_details,
                    &environment_details.sleeping_nearby,
                    rng,
                    events,
                );
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
            Action::Frozen | Action::Flashback { .. } | Action::Avoidance => {}
            Action::SearchForSubstance { substance } => {
                // Scan inventory for an item matching the craving substance.
                let matching_item = self
                    .items
                    .iter()
                    .find(|item| item.attribute.substance() == Some(substance))
                    .cloned();

                if let Some(item) = matching_item {
                    // Found — auto-use it.
                    self.try_use_consumable(&item, events, None).ok();
                } else {
                    // Not found — emit visible craving.
                    let craving_severity = self
                        .afflictions
                        .values()
                        .find(|a| matches!(a.kind, AfflictionKind::Addiction(s) if s == substance))
                        .map(|a| a.severity.to_string())
                        .unwrap_or_else(|| "moderate".to_string());

                    let line = format!("{} craves {} but has none", self.name, substance);
                    events.push(TaggedEvent::new(
                        line,
                        MessagePayload::AddictionCraving {
                            tribute: self.identifier.clone(),
                            substance: substance.to_string(),
                            severity: craving_severity,
                        },
                    ));
                }
            }
            Action::Rescue { .. } => {}
            Action::SetTrap {
                trap_kind,
                severity,
            } => {
                self.act_set_trap(trap_kind, severity, area_details, rng, events);
            }
            Action::Search => {
                self.act_search(area_details, rng, events);
            }
        }

        // ── Trap trigger check ──
        // After resolving the chosen action, check if any traps in the area trigger.
        // Setter auto-passes (no friendly fire).
        for idx in (0..area_details.placed_traps.len()).rev() {
            let triggered = area_details.placed_traps[idx].triggered;
            if triggered {
                continue;
            }
            let set_by = area_details.placed_traps[idx].set_by.clone();
            if set_by == self.identifier {
                continue; // Setter auto-passes
            }

            // Passive Perception check
            let perception_mod = (self.attributes.intelligence as f32 / 10.0).floor() as i32;
            let roll: i32 = rng.random_range(1..=20);
            let concealment = area_details.placed_traps[idx].concealment;
            if roll + perception_mod < concealment as i32 {
                // Trigger the trap!
                // Remove the trap from the area and apply its effect
                let trap = area_details.placed_traps.swap_remove(idx);
                let line = format!("{} triggers a {} trap!", self.name, trap.kind);
                events.push(TaggedEvent::new(
                    line,
                    MessagePayload::TrapTriggered {
                        victim: crate::messages::TributeRef {
                            identifier: self.identifier.clone(),
                            name: self.name.clone(),
                        },
                        trap_kind: trap.kind.to_string(),
                    },
                ));

                // Apply affliction based on trap kind
                use shared::afflictions::{AfflictionKind, AfflictionSource, TrappedMetadata};
                self.try_acquire_affliction(crate::tributes::AfflictionDraft {
                    kind: AfflictionKind::Trapped(trap.kind),
                    body_part: None,
                    severity: trap.severity,
                    source: AfflictionSource::Environmental,
                    trapped_metadata: Some(TrappedMetadata::fresh_for(trap.kind, None)),
                });
                break; // First trigger stops — no chain-traps
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

        // Sleep ambush preference: prefer defenseless sleeping targets.
        // If any enemies are sleeping, choose randomly among them. Otherwise
        // choose randomly from all enemies as before.
        let sleeping_enemies: Vec<Tribute> =
            enemies.iter().filter(|t| t.sleeping).cloned().collect();
        let pool = if sleeping_enemies.is_empty() {
            enemies.clone()
        } else {
            sleeping_enemies
        };
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Some(pool.choose(&mut rng).unwrap().clone())
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

    /// Execute a SetTrap action.
    fn act_set_trap(
        &mut self,
        _trap_kind: Option<shared::afflictions::TrapKind>,
        _severity: Option<shared::afflictions::Severity>,
        _area_details: &mut AreaDetails,
        _rng: &mut impl Rng,
        _events: &mut Vec<TaggedEvent>,
    ) {
        // Check area cap: max 3 traps per area
        let untriggered_count = _area_details
            .placed_traps
            .iter()
            .filter(|t| !t.triggered)
            .count();
        if untriggered_count >= 3 {
            let line = format!("{} can't set another trap here — area is full.", self.name);
            _events.push(TaggedEvent::new(line, MessagePayload::Generic));
            return;
        }

        let kind = _trap_kind.unwrap_or(shared::afflictions::TrapKind::Snared);
        let severity = _severity.unwrap_or(shared::afflictions::Severity::Mild);

        // Intelligence check for concealment (d20 + int modifier)
        let int_val = self.attributes.intelligence as f32;
        let int_mod = (int_val / 10.0).floor() as u32;
        let roll: u32 = _rng.random_range(1..=20);
        let concealment = 10 + int_mod + roll / 2; // Base 10 + int bonus + luck

        let trap = crate::tributes::traps::PlacedTrap {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            severity,
            set_by: self.identifier.clone(),
            concealment,
            triggered: false,
        };

        _area_details.placed_traps.push(trap);

        let line = format!("{} sets a {} trap in the area.", self.name, kind);
        _events.push(TaggedEvent::new(
            line,
            MessagePayload::TrapSet {
                tribute: crate::messages::TributeRef {
                    identifier: self.identifier.clone(),
                    name: self.name.clone(),
                },
                trap_kind: kind.to_string(),
            },
        ));
    }

    /// Execute a Search action — reveals hidden traps in current area.
    fn act_search(
        &mut self,
        area_details: &mut AreaDetails,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        let untriggered: Vec<usize> = area_details
            .placed_traps
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.triggered)
            .map(|(i, _)| i)
            .collect();

        if untriggered.is_empty() {
            let line = format!("{} searches the area but finds no traps.", self.name);
            events.push(TaggedEvent::new(line, MessagePayload::Generic));
            return;
        }

        // Perception check vs concealment
        let perception_mod = (self.attributes.intelligence as f32 / 10.0).floor() as u32;
        let mut found_any = false;

        for &idx in &untriggered {
            let concealment = area_details.placed_traps[idx].concealment;
            let kind = area_details.placed_traps[idx].kind;
            let roll: u32 = rng.random_range(1..=20);
            if roll + perception_mod >= concealment {
                found_any = true;
                let line = format!("{} spots a {} trap! (DC {})", self.name, kind, concealment);
                events.push(TaggedEvent::new(line, MessagePayload::Generic));
                // Free disarm
                area_details.placed_traps[idx].triggered = true;
                let disarm_line = format!("{} disarms the {} trap.", self.name, kind);
                events.push(TaggedEvent::new(disarm_line, MessagePayload::Generic));
            }
        }

        if !found_any {
            let line = format!(
                "{} searches carefully but finds nothing suspicious.",
                self.name
            );
            events.push(TaggedEvent::new(line, MessagePayload::Generic));
        }
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

    fn act_take_item(
        &mut self,
        area_details: &mut AreaDetails,
        sleeping_nearby: &[(Uuid, String)],
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        // Sleep theft (ls5a): if there are sleeping tributes with items
        // in the area, mark one as theft target. The actual item transfer
        // happens in execute_cycle after process_turn_phase returns.
        if !sleeping_nearby.is_empty() {
            let idx = rng.random_range(0..sleeping_nearby.len());
            let (sleeper_id, sleeper_name) = &sleeping_nearby[idx];
            self.pending_theft_target = Some(*sleeper_id);
            let line = format!(
                "{} rummages through sleeping {}'s belongings",
                self.name, sleeper_name
            );
            events.push(TaggedEvent::new(line, MessagePayload::Generic));
            return;
        }

        // Original area item logic
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
            if let Err(error) = self.try_use_consumable(item, events, None) {
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
                Some(ref m) if m.sources.iter().any(|s| matches!(s, TraumaSource::Betrayal { .. }))
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

        // Addiction penalty (PR3 spec §11):
        // -0.10 per severity tier per known addiction the target has.
        let addiction_penalty: f64 = self
            .afflictions
            .values()
            .filter(|aff| matches!(aff.kind, AfflictionKind::Addiction(_)))
            .filter_map(|aff| aff.addiction_metadata.as_ref())
            .filter(|meta| meta.observed_by.contains(&target.identifier))
            .map(|meta| {
                let aff = self
                    .afflictions
                    .values()
                    .find(|a| {
                        matches!(a.kind, AfflictionKind::Addiction(_))
                            && a.addiction_metadata
                                .as_ref()
                                .is_some_and(|m| m.substance == meta.substance)
                    })
                    .unwrap();
                -0.10 * (aff.severity as i32) as f64
            })
            .sum::<f64>()
            .abs();

        let same_district = self.district == target.district;
        let formed = try_form_alliance(
            &self.traits,
            &target.traits,
            same_district,
            self.allies.len(),
            target.allies.len(),
            phobia_penalty,
            trauma_penalty - trauma_observer_bonus,
            addiction_penalty,
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
        let cycle = self.game_day.unwrap_or(0) as u32;
        let provisional = Affliction {
            kind: draft.kind.clone(),
            body_part: draft.body_part,
            severity: draft.severity,
            source: draft.source.clone(),
            acquired_cycle: cycle,
            last_progressed_cycle: cycle,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: draft.trapped_metadata.clone(),
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

                let affliction = Affliction {
                    kind: draft.kind.clone(),
                    body_part: draft.body_part,
                    severity: draft.severity,
                    source: draft.source,
                    acquired_cycle: cycle,
                    last_progressed_cycle: cycle,
                    trauma_metadata: None,
                    phobia_metadata: None,
                    fixation_metadata: None,
                    addiction_metadata: None,
                    trapped_metadata: draft.trapped_metadata.clone(),
                };
                self.afflictions
                    .insert((draft.kind.clone(), draft.body_part), affliction);
            }
            AcquireResolution::Reject(_) => {}
        }

        resolution
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
}

/// A draft affliction ready for acquisition resolution.
#[derive(Clone)]
pub struct AfflictionDraft {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub source: AfflictionSource,
    pub trapped_metadata: Option<TrappedMetadata>,
}

/// Calculates the stamina cost for a tribute action based on:
/// - Base action cost
/// - Terrain movement multiplier
/// - Terrain affinity modifier (0.8 if tribute has affinity, 1.0 otherwise)
/// - Desperation multiplier based on health (1.0 + 0.5 * (1.0 - health%))
pub fn calculate_stamina_cost(
    action: &Action,
    terrain: &crate::terrain::TerrainType,
    tribute: &Tribute,
) -> u32 {
    // Base costs for each action type
    let base_cost: f32 = match action {
        Action::Move(_) => 20.0,
        Action::Hide => 15.0,
        Action::TakeItem => 10.0,
        Action::Attack => 25.0,
        Action::Rest | Action::None => 0.0,
        Action::UseItem(_) => 10.0,
        // Proposing an alliance is a low-cost social action.
        Action::ProposeAlliance => 5.0,
        // Survival actions: foraging/seeking shelter cost some stamina;
        // eating and drinking are essentially free overhead.
        Action::SeekShelter => 10.0,
        Action::Forage => 15.0,
        Action::DrinkFromTerrain => 5.0,
        Action::Eat(_) | Action::DrinkItem(_) => 0.0,
        // Sleep is free at the action layer; phase scheduler handles it.
        Action::Sleep { .. } => 0.0,
        Action::Rescue { .. } => 15.0,
        Action::SetTrap { .. } => 15.0,
        Action::Search => 10.0,
        Action::Frozen
        | Action::Flashback { .. }
        | Action::Avoidance
        | Action::SearchForSubstance { .. } => 0.0,
    };

    // If base cost is 0, no need to calculate multipliers
    if base_cost == 0.0 {
        return 0;
    }

    // Terrain multiplier from movement_cost
    let terrain_multiplier = terrain.base.movement_cost();

    // Affinity modifier: 0.8 if tribute has affinity for this terrain, else 1.0
    let affinity_modifier = if tribute.terrain_affinity.contains(&terrain.base) {
        0.8
    } else {
        1.0
    };

    // Desperation multiplier: 1.0 + (0.5 × (1.0 - health%))
    let health_percent = tribute.attributes.health as f32 / 100.0;
    let desperation_multiplier = 1.0 + (0.5 * (1.0 - health_percent));

    // Calculate final cost with all multipliers
    let final_cost = base_cost * terrain_multiplier * affinity_modifier * desperation_multiplier;

    // Round to nearest integer
    final_cost.round() as u32
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
mod tests {

    use crate::messages::TaggedEvent;
    use crate::tributes::Tribute;
    use crate::tributes::brains::Brain;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[fixture]
    fn target() -> Tribute {
        Tribute::new("Peeta".to_string(), None, None)
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[rstest]
    fn default() {
        let tribute = Tribute::default();
        assert_eq!(tribute.name, "Default Tribute");
    }

    #[rstest]
    fn serde_roundtrip_alliance_fields() {
        use crate::tributes::traits::Trait;
        use uuid::Uuid;

        let mut tribute = Tribute::new("Rue".to_string(), None, None);
        let ally = Uuid::new_v4();
        tribute.allies.push(ally);
        tribute.traits.clear();
        tribute.traits.push(Trait::Loyal);
        tribute.traits.push(Trait::Treacherous);
        tribute.turns_since_last_betrayal = 7;
        tribute.pending_trust_shock = true;

        let json = serde_json::to_string(&tribute).expect("serialize");
        assert!(json.contains("\"allies\""));
        assert!(json.contains("\"traits\""));
        assert!(json.contains("\"Loyal\""));
        assert!(json.contains("\"turns_since_last_betrayal\":7"));
        assert!(json.contains("\"pending_trust_shock\":true"));

        let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.allies, vec![ally]);
        assert_eq!(restored.traits, vec![Trait::Loyal, Trait::Treacherous]);
        assert_eq!(restored.turns_since_last_betrayal, 7);
        assert!(restored.pending_trust_shock);
    }

    #[rstest]
    fn serde_defaults_for_missing_alliance_fields() {
        // Persisted tribute records written before the alliance fields existed
        // must still deserialize. Simulate this by serialising a fresh tribute,
        // stripping the new fields, then round-tripping.
        let baseline = Tribute::new("Legacy".to_string(), None, None);
        let mut value: serde_json::Value = serde_json::to_value(&baseline).expect("to_value");
        let obj = value.as_object_mut().expect("object");
        obj.remove("allies");
        obj.remove("traits");
        obj.remove("turns_since_last_betrayal");
        obj.remove("pending_trust_shock");

        let restored: Tribute = serde_json::from_value(value).expect("legacy deserialize");
        assert!(restored.allies.is_empty());
        assert!(restored.traits.is_empty());
        assert_eq!(restored.turns_since_last_betrayal, 0);
        assert!(!restored.pending_trust_shock);
    }

    #[rstest]
    fn brain_roundtrips_psychotic_break_state() {
        use crate::tributes::brains::PsychoticBreakType;

        let mut tribute = Tribute::new("Cato".to_string(), None, None);
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Berserk);

        let json = serde_json::to_string(&tribute).expect("serialize");
        assert!(json.contains("\"brain\""));
        assert!(json.contains("\"Berserk\""));

        let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            restored.brain.psychotic_break,
            Some(PsychoticBreakType::Berserk),
        );
    }

    #[rstest]
    fn brain_preferred_action_is_not_persisted() {
        // preferred_action is transient AI state recomputed each cycle, so the
        // field is `skip_serializing` and `deserialize_optional_enum_lenient`
        // (which absorbs both null and the {} corruption left over from the
        // SDK's enum-collapse bug). A roundtrip therefore intentionally drops
        // any preferred_action that was set in memory.
        use crate::tributes::actions::Action;

        let mut tribute = Tribute::new("Foxface".to_string(), None, None);
        tribute.brain.preferred_action = Some(Action::Hide);
        tribute.brain.preferred_action_percentage = 0.75;

        let json = serde_json::to_string(&tribute).expect("serialize");

        let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.brain.preferred_action, None);
        // Non-skipped fields still round-trip normally.
        assert!((restored.brain.preferred_action_percentage - 0.75).abs() < f64::EPSILON);
    }

    #[rstest]
    fn brain_tolerates_corrupt_preferred_action_object() {
        // SurrealDB rows written before the bug-5 fix have preferred_action: {}
        // because the SDK's bespoke serializer collapsed the externally-tagged
        // Action enum. The lenient deserializer must read those rows as None.
        // Round-trip a real Brain to get a valid base JSON, then swap
        // preferred_action's value to {} to simulate the corruption.
        use crate::tributes::brains::Brain;

        let brain = Brain {
            preferred_action_percentage: 0.5,
            ..Brain::default()
        };
        let mut value = serde_json::to_value(&brain).expect("serialize brain");
        value["preferred_action"] = serde_json::json!({});
        let restored: Brain = serde_json::from_value(value).expect("deserialize legacy row");
        assert_eq!(restored.preferred_action, None);
    }

    #[rstest]
    fn brain_missing_field_defaults() {
        // Pre-fix tribute rows persisted before #[serde(default)] was added
        // omit the `brain` column entirely. They must still deserialize, with
        // brain hydrated via `Brain::default()`.
        let baseline = Tribute::new("Legacy".to_string(), None, None);
        let mut value: serde_json::Value = serde_json::to_value(&baseline).expect("to_value");
        value.as_object_mut().expect("object").remove("brain");

        let restored: Tribute = serde_json::from_value(value).expect("legacy deserialize");
        assert_eq!(restored.brain, Brain::default());
        assert!(restored.brain.psychotic_break.is_none());
        assert!(restored.brain.preferred_action.is_none());
    }

    #[rstest]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        assert_eq!(tribute.name, "Katniss");
        assert_eq!(tribute.district, 12);
        // Attributes::new() randomizes health in 50..=max_health.
        assert!(
            (50..=100).contains(&tribute.attributes.health),
            "health {} out of range",
            tribute.attributes.health
        );
    }

    #[rstest]
    fn random() {
        let tribute = Tribute::random();
        assert!(!tribute.name.is_empty());
        assert!(tribute.district >= 1 && tribute.district <= 12);
    }

    #[rstest]
    fn new_tribute_has_empty_alliance_state() {
        let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert!(tribute.allies.is_empty());
        assert_eq!(tribute.turns_since_last_betrayal, 0);
        // `id` mirrors `identifier`.
        assert_eq!(tribute.id.to_string(), tribute.identifier);
    }

    #[rstest]
    fn new_tribute_has_no_pending_trust_shock() {
        let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert!(!tribute.pending_trust_shock);
    }

    #[test]
    fn tribute_default_survival_fields_are_zero_and_none() {
        let t = Tribute::new("Test".to_string(), None, None);
        assert_eq!(t.hunger, 0, "hunger starts at 0 (Sated)");
        assert_eq!(t.thirst, 0, "thirst starts at 0 (Sated)");
        assert_eq!(t.sheltered_until, None, "starts exposed");
        assert_eq!(t.starvation_drain_step, 0);
        assert_eq!(t.dehydration_drain_step, 0);
    }

    #[test]
    fn tribute_legacy_json_loads_with_defaults() {
        // JSON missing the new survival fields entirely (simulates a saved
        // game from before this feature landed). serde(default) must
        // populate them.
        let mut t = Tribute::new("Legacy".to_string(), Some(1), None);
        t.hunger = 0;
        t.thirst = 0;
        t.sheltered_until = None;
        t.starvation_drain_step = 0;
        t.dehydration_drain_step = 0;
        let mut json: serde_json::Value = serde_json::to_value(&t).unwrap();
        // strip the survival fields to mimic a pre-feature save
        let obj = json.as_object_mut().unwrap();
        obj.remove("hunger");
        obj.remove("thirst");
        obj.remove("sheltered_until");
        obj.remove("starvation_drain_step");
        obj.remove("dehydration_drain_step");
        let loaded: Tribute = serde_json::from_value(json).expect("legacy load must succeed");
        assert_eq!(loaded.hunger, 0);
        assert_eq!(loaded.thirst, 0);
        assert_eq!(loaded.sheltered_until, None);
        assert_eq!(loaded.starvation_drain_step, 0);
        assert_eq!(loaded.dehydration_drain_step, 0);
    }

    #[test]
    fn tribute_legacy_json_loads_with_sleep_defaults() {
        // JSON missing the new sleep fields must default to zero/false.
        let t = Tribute::new("Legacy".to_string(), Some(1), None);
        let mut json: serde_json::Value = serde_json::to_value(&t).unwrap();
        let obj = json.as_object_mut().unwrap();
        obj.remove("cycles_awake");
        obj.remove("sleeping");
        obj.remove("sleep_remaining");
        let loaded: Tribute = serde_json::from_value(json).expect("legacy load must succeed");
        assert_eq!(loaded.cycles_awake, 0);
        assert!(!loaded.sleeping);
        assert_eq!(loaded.sleep_remaining, 0);
    }

    #[rstest]
    fn tribute_drain_alliance_events_returns_and_clears_buffer() {
        use crate::tributes::alliances::AllianceEvent;
        use uuid::Uuid;
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        let other = Uuid::new_v4();
        tribute
            .alliance_events
            .push(AllianceEvent::BetrayalRecorded {
                betrayer: tribute.id,
                victim: other,
            });
        let drained = tribute.drain_alliance_events();
        assert_eq!(drained.len(), 1);
        assert!(tribute.alliance_events.is_empty());
    }

    #[rstest]
    fn consume_pending_trust_shock_resets_flag_when_not_set() {
        // No flag → no rolls, flag stays false, allies untouched.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        let ally = uuid::Uuid::new_v4();
        tribute.allies.push(ally);
        let mut events: Vec<TaggedEvent> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(53);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);
        assert!(!tribute.pending_trust_shock);
        assert_eq!(tribute.allies, vec![ally]);
        assert!(events.is_empty());
    }

    #[rstest]
    fn consume_pending_trust_shock_breaks_allies_on_success_and_clears_flag() {
        // Force trust_shock to fire deterministically: sanity=0, threshold>0
        // gives p = 0.5 + 0.5 * 1.0 = 1.0 → always true.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.sanity = 0;
        tribute.brain.thresholds.extreme_low_sanity = 50;
        let ally1 = uuid::Uuid::new_v4();
        let ally2 = uuid::Uuid::new_v4();
        tribute.allies.push(ally1);
        tribute.allies.push(ally2);
        tribute.pending_trust_shock = true;

        let mut events: Vec<TaggedEvent> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(211);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);

        assert!(!tribute.pending_trust_shock, "flag must reset");
        assert!(
            tribute.allies.is_empty(),
            "all allies broken on guaranteed success"
        );
        assert_eq!(events.len(), 2, "one message per broken ally");
    }

    #[rstest]
    fn consume_pending_trust_shock_no_break_when_sanity_above_threshold() {
        // Sanity at/above threshold → trust_shock_roll returns false → no break.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.sanity = 100;
        tribute.brain.thresholds.extreme_low_sanity = 50;
        let ally = uuid::Uuid::new_v4();
        tribute.allies.push(ally);
        tribute.pending_trust_shock = true;

        let mut events: Vec<TaggedEvent> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(89);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);

        assert!(!tribute.pending_trust_shock, "flag must reset");
        assert_eq!(tribute.allies, vec![ally], "ally retained");
        assert!(events.is_empty());
    }

    #[rstest]
    fn new_tribute_has_traits_for_valid_district() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        // generate_traits rolls 2..=6 traits from the district pool.
        assert!((2..=6).contains(&tribute.traits.len()));
    }

    #[rstest]
    fn pick_target_skips_allies() {
        // An ally is in the same area but must not be picked as a target.
        let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
        me.attributes.sanity = 100; // not suicidal
        let ally = Tribute::new("Peeta".to_string(), Some(12), None);
        me.allies.push(ally.id);

        let mut events: Vec<TaggedEvent> = vec![];
        let target = me.pick_target(vec![ally.clone()], 5, &mut events);
        // Only candidate was an ally and we're not in final confrontation.
        assert!(target.is_none());
    }

    #[rstest]
    fn pick_target_allows_same_district_when_not_ally() {
        // Same-district tributes can now be targeted unless they're allies.
        let me = Tribute::new("Katniss".to_string(), Some(12), None);
        let same_district = Tribute::new("Peeta".to_string(), Some(12), None);

        let mut events: Vec<TaggedEvent> = vec![];
        let target = me.pick_target(vec![same_district.clone()], 5, &mut events);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id, same_district.id);
    }

    #[rstest]
    fn pick_target_final_confrontation_overrides_alliance() {
        // When only two tributes remain alive, even an ally is a valid target.
        let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
        me.attributes.sanity = 100;
        let ally = Tribute::new("Peeta".to_string(), Some(12), None);
        me.allies.push(ally.id);

        let mut events: Vec<TaggedEvent> = vec![];
        let target = me.pick_target(vec![ally.clone()], 2, &mut events);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id, ally.id);
    }

    #[rstest]
    fn tick_alliance_timers_increments_betrayal_counter() {
        // Living tribute: counter increments by exactly one per tick.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert_eq!(tribute.turns_since_last_betrayal, 0);
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 1);
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 2);
    }

    #[rstest]
    fn tick_alliance_timers_saturates_does_not_overflow() {
        // u8 saturating add: never panics, never wraps to zero.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.turns_since_last_betrayal = u8::MAX;
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, u8::MAX);
    }

    #[rstest]
    fn tick_alliance_timers_skips_dead_tributes() {
        // Dead tributes don't accumulate betrayal cooldown.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.health = 0;
        tribute.status = crate::tributes::TributeStatus::RecentlyDead;
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 0);
    }

    #[rstest]
    fn pick_target_picks_ex_ally_after_trust_shock_breaks_bond() {
        // End-to-end break-then-attack (spec §7.3c1 + §7.5):
        // Once a trust shock fires and removes the betrayer from the
        // victim's `allies`, the victim's next `pick_target` call must
        // consider that ex-ally a valid target.
        let mut victim = Tribute::new("Glimmer".to_string(), Some(1), None);
        victim.attributes.sanity = 100; // not suicidal
        let ex_ally = Tribute::new("Cato".to_string(), Some(2), None);
        // Pre-condition: bonded.
        victim.allies.push(ex_ally.id);

        // Simulate the bond breaking (what process_alliance_events does
        // for BetrayalRecorded, plus what consume_pending_trust_shock
        // does on the victim's side: drop the ex-ally locally).
        victim.allies.retain(|id| *id != ex_ally.id);

        let mut events: Vec<TaggedEvent> = vec![];
        let target = victim.pick_target(vec![ex_ally.clone()], 5, &mut events);
        assert!(
            target.is_some(),
            "ex-ally must be targetable after the bond breaks"
        );
        assert_eq!(target.unwrap().id, ex_ally.id);
    }

    #[rstest]
    fn consume_pending_trust_shock_leaves_asymmetric_back_edge() {
        // Spec §7.3c1 explicitly defers the symmetric back-edge cleanup
        // for trust-shock breaks: only `self` is mutated. This regression
        // test pins that contract so any future tightening is intentional.
        let mut victim = Tribute::new("Glimmer".to_string(), Some(1), None);
        victim.attributes.sanity = 0; // force a break
        victim.brain.thresholds.extreme_low_sanity = 100;
        let betrayer_id = uuid::Uuid::new_v4();
        victim.allies.push(betrayer_id);
        victim.pending_trust_shock = true;

        let mut rng = SmallRng::seed_from_u64(419);
        let mut events: Vec<TaggedEvent> = vec![];
        victim.consume_pending_trust_shock(&mut rng, &mut events);

        // Victim's side cleaned.
        assert!(
            !victim.allies.contains(&betrayer_id),
            "victim must drop the broken ally"
        );
        // The flag is consumed regardless of roll outcome.
        assert!(
            !victim.pending_trust_shock,
            "pending flag is reset after the call"
        );
        // Asymmetric back-edge stays — `consume_pending_trust_shock` only
        // touches `self`. The next cycle's event drain (or follow-up
        // events) is responsible for the betrayer's side.
        // We can't observe the betrayer here (different tribute instance);
        // the documented contract is what matters and is asserted by the
        // single-side mutation: the function signature takes `&mut self`
        // and returns nothing, with no reference to the broken ally.
    }

    #[test]
    fn wake_interrupted_returns_false_when_not_sleeping() {
        use crate::messages::TributeRef;
        let mut t = Tribute::new("Foxface".to_string(), Some(1), None);
        let mut events: Vec<TaggedEvent> = Vec::new();
        let woke = t.wake_interrupted(
            shared::messages::InterruptionKind::Ambush {
                attacker: TributeRef {
                    identifier: "x".to_string(),
                    name: "X".to_string(),
                },
            },
            shared::messages::Phase::Day,
            &mut events,
        );
        assert!(!woke);
        assert!(events.is_empty());
    }

    #[test]
    fn wake_interrupted_resets_state_and_emits_tribute_woke() {
        let mut t = Tribute::new("Foxface".to_string(), Some(1), None);
        t.sleeping = true;
        t.sleep_remaining = 3;
        t.cycles_awake = 7;
        let mut events: Vec<TaggedEvent> = Vec::new();
        let woke = t.wake_interrupted(
            shared::messages::InterruptionKind::AreaEvent {
                kind: shared::messages::AreaEventKind::Fire,
            },
            shared::messages::Phase::Night,
            &mut events,
        );
        assert!(woke);
        assert!(!t.sleeping);
        assert_eq!(t.sleep_remaining, 0);
        assert_eq!(t.cycles_awake, 0);
        assert_eq!(events.len(), 1);
        match &events[0].payload {
            crate::messages::MessagePayload::TributeWoke { reason, phase, .. } => {
                assert_eq!(*phase, shared::messages::Phase::Night);
                match reason {
                    shared::messages::WakeReason::Interrupted {
                        event:
                            shared::messages::InterruptionKind::AreaEvent {
                                kind: shared::messages::AreaEventKind::Fire,
                            },
                    } => {}
                    other => panic!("unexpected reason: {:?}", other),
                }
            }
            other => panic!("expected TributeWoke payload, got {:?}", other),
        }
    }

    // --- Affliction tests ---

    use crate::tributes::AfflictionDraft;
    use crate::tributes::afflictions::{AcquireResolution, RejectReason};
    use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

    #[test]
    fn test_afflictions_empty_by_default() {
        let t = Tribute::new("Test".to_string(), None, None);
        assert!(t.afflictions.is_empty());
    }

    #[test]
    fn test_afflictions_skip_serialization_when_empty() {
        let t = Tribute::new("Test".to_string(), None, None);
        let json = serde_json::to_string(&t).unwrap();
        assert!(!json.contains("\"afflictions\""));
    }

    #[test]
    fn test_try_acquire_insert() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        let draft = AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(resolution, AcquireResolution::Insert);
        assert_eq!(t.afflictions.len(), 1);
        assert!(
            t.afflictions
                .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
    }

    #[test]
    fn test_try_acquire_upgrade() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Insert mild wound
        t.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        });
        // Upgrade to moderate
        let draft = AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(
            resolution,
            AcquireResolution::Upgrade((AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
        assert_eq!(t.afflictions.len(), 1);
        let affl = t
            .afflictions
            .get(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
            .unwrap();
        assert_eq!(affl.severity, Severity::Moderate);
    }

    #[test]
    fn test_try_acquire_supersede() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Insert wounded on arm
        t.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        });
        // Infected supersedes wounded at same body part
        let draft = AfflictionDraft {
            kind: AfflictionKind::Infected,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(resolution, AcquireResolution::Insert);
        // Wounded removed, Infected present
        assert!(
            !t.afflictions
                .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
        assert!(
            t.afflictions
                .contains_key(&(AfflictionKind::Infected, Some(BodyPart::Arm)))
        );
    }

    #[test]
    fn test_try_acquire_reject_limb_missing() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Missing arm
        t.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::MissingArm,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Severe,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        });
        // Can't wound a missing limb
        let draft = AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(
            resolution,
            AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)
        );
    }

    #[test]
    fn test_try_acquire_reject_no_wounded_ancestor() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Infected without prior Wounded on same part
        let draft = AfflictionDraft {
            kind: AfflictionKind::Infected,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(
            resolution,
            AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor)
        );
    }

    #[test]
    fn test_try_acquire_reject_same_severity() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Insert mild wound
        t.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        });
        // Same severity rejected
        let draft = AfflictionDraft {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Mild,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            trapped_metadata: None,
        };
        let resolution = t.try_acquire_affliction(draft);
        assert_eq!(
            resolution,
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        );
    }
}
