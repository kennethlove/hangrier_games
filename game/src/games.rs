use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::items::OwnsItems;
use crate::tributes::actions::Action;
use crate::tributes::events::TributeEvent;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::{
    ActionSuggestion, EncounterContext, EnvironmentContext, Tribute, calculate_stamina_cost,
};
use rand::RngExt;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use shared::GameStatus;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use uuid::Uuid;

/// Errors that can occur during game operations.
#[derive(Debug, Clone, PartialEq)]
pub enum GameError {
    /// Error related to message operations
    MessageError(String),
    /// Area not found in game
    AreaNotFound(String),
    /// Tribute not found in game
    TributeNotFound(String),
}

impl Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::MessageError(msg) => write!(f, "Message error: {}", msg),
            GameError::AreaNotFound(msg) => write!(f, "Area not found: {}", msg),
            GameError::TributeNotFound(msg) => write!(f, "Tribute not found: {}", msg),
        }
    }
}

impl std::error::Error for GameError {}

impl From<String> for GameError {
    fn from(error: String) -> Self {
        GameError::MessageError(error)
    }
}

const LOW_TRIBUTE_THRESHOLD: u32 = 8;
const FEAST_WEAPON_COUNT: u32 = 2;
const FEAST_SHIELD_COUNT: u32 = 2;
const FEAST_CONSUMABLE_COUNT: u32 = 4;
const DAY_EVENT_FREQUENCY: f64 = 1.0 / 4.0;
const NIGHT_EVENT_FREQUENCY: f64 = 1.0 / 8.0;

/// Per-period tick counter. Resets to 0 at every phase boundary.
/// Phase-boundary side-effect messages get tick=0.
/// First action in a phase gets tick=1.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TickCounter {
    current: u32,
}

impl TickCounter {
    pub fn reset(&mut self) {
        self.current = 0;
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        self.current += 1;
        self.current
    }
    pub fn boundary(&self) -> u32 {
        0
    }
}

/// Represents the current state of the game.
///
/// `PartialEq` is implemented manually (identity-only via `identifier`) because
/// the transient `messages` buffer holds `GameMessage`s carrying
/// `MessagePayload`, which is not `PartialEq` (and adding it would require
/// deriving across the entire payload graph). The web crate's
/// `dioxus-query` cache enums require `PartialEq` on their variants; identity
/// equality is sufficient for cache dedup since a game is uniquely keyed by
/// its identifier.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Game {
    pub identifier: String,
    pub name: String,
    pub status: GameStatus,
    pub day: Option<u32>,
    #[serde(default)]
    pub areas: Vec<AreaDetails>,
    #[serde(default)]
    pub tributes: Vec<Tribute>,
    pub private: bool,
    #[serde(default)]
    pub config: crate::config::GameConfig,
    /// Transient buffer of events emitted during the current cycle.
    /// Drained and persisted by the API layer after each `run_day_night_cycle`.
    /// Skipped during serialization since events live in their own table.
    #[serde(default, skip_serializing)]
    pub messages: Vec<crate::messages::GameMessage>,
    /// Transient queue of alliance lifecycle events drained between tribute
    /// turns inside `run_day_night_cycle`. Lives only for the duration of a
    /// single cycle; never persisted. See spec §7.5.
    #[serde(default, skip)]
    pub alliance_events: Vec<crate::tributes::alliances::AllianceEvent>,
    /// Per-period tick counter; transient, never persisted.
    #[serde(skip, default)]
    pub tick_counter: TickCounter,
    /// Current phase (Day/Night) for the in-flight cycle. Transient, used
    /// by helper logging methods to stamp `GameMessage.phase`. Defaults to
    /// `Day` outside of an active cycle.
    #[serde(skip, default = "default_phase")]
    pub current_phase: crate::messages::Phase,
    /// Per-period emit index for the in-flight cycle. Transient. Reset at
    /// every phase boundary alongside `tick_counter`.
    #[serde(skip, default)]
    pub emit_index: u32,

    /// Tunable combat & stamina knobs. See spec
    /// `2026-05-03-stamina-combat-resource-design.md`.
    #[serde(default)]
    pub combat_tuning: crate::tributes::combat_tuning::CombatTuning,
}

fn default_phase() -> crate::messages::Phase {
    crate::messages::Phase::Day
}

impl PartialEq for Game {
    /// Identity equality: two `Game`s are considered equal iff they share an
    /// `identifier`. See struct docs for rationale.
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Default for Game {
    /// Creates a new game with a random name.
    fn default() -> Game {
        let wp_gen = crate::witty_phrase_generator::WPGen::new();
        let mut name = String::new();
        if let Some(words) = wp_gen.with_words(3) {
            name = words.join("-").to_string();
        };

        Game {
            identifier: Uuid::new_v4().to_string(),
            name,
            status: Default::default(),
            day: None,
            areas: vec![],
            tributes: vec![],
            private: true,
            config: Default::default(),
            messages: vec![],
            alliance_events: vec![],
            tick_counter: TickCounter::default(),
            current_phase: crate::messages::Phase::Day,
            emit_index: 0,
            combat_tuning: crate::tributes::combat_tuning::CombatTuning::default(),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// One per-tribute event collected during `execute_cycle` and drained
/// into `Game::messages` by `flush_tribute_events`.
///
/// Tuple shape: `(actor_identifier, actor_name, content, payload, optional GameEvent)`.
/// Sites carrying a typed `MessagePayload` (Some) push that payload directly;
/// legacy stringly sites carry `None` and synthesise a fallback in the drain.
type CollectedEvent = (
    String,
    String,
    String,
    Option<crate::messages::MessagePayload>,
    Option<crate::events::GameEvent>,
);

/// Pre-computed, immutable view of cycle inputs produced by
/// `Game::build_cycle_context` and consumed by `Game::execute_cycle`.
///
/// Splitting these out gives gamemaker overrides (and future cycle
/// modifiers) a typed seam between the "snapshot the world" phase and the
/// "iterate `&mut self.tributes`" phase that previously lived together
/// in `run_tribute_cycle`.
struct CycleContext {
    /// True for the day half of the cycle, false for night.
    is_day: bool,
    /// Current game day (1-indexed). Mirrors `Game::day.unwrap_or(1)` and
    /// is forwarded into `EnvironmentContext::current_day`.
    current_day: u32,
    /// Optional global action suggestion (e.g. day-1 spread, day-3
    /// Cornucopia push).
    action_suggestion: Option<ActionSuggestion>,
    /// `Area -> index into self.areas` for O(1) area lookup during the
    /// `&mut self.tributes` iteration.
    area_details_map: HashMap<Area, usize>,
    /// Owned per-area tribute snapshots used to build `EncounterContext`
    /// without re-borrowing `self.tributes` during the executor loop.
    tributes_by_area: HashMap<Area, Vec<Tribute>>,
    /// Per-area living-tribute density (4wnj). Read by
    /// `Brain::choose_destination` as a crowd penalty.
    enemy_density: HashMap<Area, u32>,
    /// Cached combat tuning so the executor never has to re-borrow `self`.
    combat_tuning_snapshot: crate::tributes::combat_tuning::CombatTuning,
    /// Read-only snapshot of every area for multi-hop pathfinding.
    all_areas_snapshot: Vec<AreaDetails>,
    /// Areas closed for this cycle, propagated into `EnvironmentContext`.
    closed_areas: Vec<Area>,
    /// Total living tribute count (used by `EncounterContext`).
    living_tributes_count: usize,
}

impl Game {
    /// Create a new game with a given name.
    pub fn new(name: &str) -> Self {
        Game {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Runs at the end of the game.
    pub fn end(&mut self) {
        self.status = GameStatus::Finished
    }

    /// Runs at the start of the game.
    pub fn start(&mut self) -> Result<(), GameError> {
        self.status = GameStatus::InProgress;
        Ok(())
    }

    /// Returns the tributes that are alive.
    pub fn living_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.is_alive())
            .cloned()
            .collect()
    }

    /// Returns the count of living tributes without cloning.
    pub fn living_tributes_count(&self) -> usize {
        self.tributes.iter().filter(|t| t.is_alive()).count()
    }

    /// Returns the tributes that are recently dead, i.e., died in the current round.
    #[cfg_attr(not(test), allow(dead_code))]
    fn recently_dead_tributes(&self) -> Vec<Tribute> {
        self.tributes
            .iter()
            .filter(|t| t.status == TributeStatus::RecentlyDead)
            .cloned()
            .collect()
    }

    /// Returns the tribute that is the winner of the game if there is one.
    pub fn winner(&self) -> Option<Tribute> {
        let living: Vec<&Tribute> = self.tributes.iter().filter(|t| t.is_alive()).collect();
        match living.len() {
            1 => Some(living[0].clone()),
            _ => None,
        }
    }

    /// Returns a random area from the game.
    fn random_area(&mut self) -> Option<&mut AreaDetails> {
        self.areas.choose_mut(&mut rand::rng())
    }

    /// Returns a random open area from the game.
    fn random_open_area(&self) -> Option<AreaDetails> {
        self.open_areas().choose(&mut rand::rng()).cloned()
    }

    /// Returns a vec of open areas.
    fn open_areas(&self) -> Vec<AreaDetails> {
        self.areas.iter().filter(|a| a.is_open()).cloned().collect()
    }

    /// Returns a vec of closed areas.
    #[allow(dead_code)]
    fn closed_areas(&self) -> Vec<AreaDetails> {
        self.areas
            .iter()
            .filter(|a| !a.is_open())
            .cloned()
            .collect()
    }

    /// Construct a fallback `MessagePayload` when a caller doesn't supply
    /// a typed payload. Picks an existing variant suited to the message
    /// source so the schema-required `payload` field is always present.
    /// This is a transitional helper used by the legacy log helpers
    /// pending full migration of every emission site to typed payloads.
    fn fallback_payload(
        source: &crate::messages::MessageSource,
    ) -> crate::messages::MessagePayload {
        use crate::messages::{AreaEventKind, AreaRef, MessagePayload, MessageSource, TributeRef};
        match source {
            MessageSource::Tribute(id) => MessagePayload::SanityBreak {
                tribute: TributeRef {
                    identifier: id.clone(),
                    name: String::new(),
                },
            },
            MessageSource::Area(name) => MessagePayload::AreaEvent {
                area: AreaRef {
                    identifier: name.clone(),
                    name: name.clone(),
                },
                kind: AreaEventKind::Other,
                description: String::new(),
            },
            MessageSource::Game(_) => MessagePayload::GameEnded { winner: None },
        }
    }

    /// Build and push a `GameMessage` with the supplied typed payload.
    /// Stamps `(game_day, phase, tick, emit_index)` from the game's
    /// transient cycle state. The `tick` argument is supplied by the
    /// caller because some sites (cycle announcements, area events)
    /// emit at the phase boundary (`tick = 0`) while per-tribute
    /// action emissions advance the tick counter.
    fn push_message(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        content: String,
        payload: crate::messages::MessagePayload,
        tick: u32,
    ) {
        let game_day = self.day.unwrap_or(0);
        // Always prefix subject with the game identifier so that the
        // API's per-game log queries (`WHERE string::starts_with(subject,
        // $game_id)`) match every emitted message regardless of source
        // type (Game / Area / Tribute). Without this prefix, area and
        // tribute messages would be invisible to the day-page and
        // timeline-summary endpoints.
        let scoped_subject = if subject.starts_with(&format!("{}:", self.identifier)) {
            subject
        } else {
            format!("{}:{}", self.identifier, subject)
        };
        let msg = crate::messages::GameMessage::new(
            source,
            game_day,
            self.current_phase,
            tick,
            self.emit_index,
            scoped_subject,
            content,
            payload,
        );
        self.messages.push(msg);
        self.emit_index = self.emit_index.saturating_add(1);
    }

    /// Push a message into the cycle's transient event buffer.
    /// The API layer drains and persists this buffer after each cycle.
    ///
    /// This legacy helper synthesises a fallback payload (see
    /// `fallback_payload`) suited to the source. New emission sites
    /// should construct a typed `MessagePayload` and call
    /// [`Self::push_message`] directly.
    pub fn log(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        content: String,
    ) {
        let payload = Self::fallback_payload(&source);
        let tick = self.tick_counter.boundary();
        self.push_message(source, subject, content, payload, tick);
    }

    /// Log a structured game output by rendering its `Display` impl into a `GameMessage`.
    pub fn log_output<D: std::fmt::Display>(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        output: D,
    ) {
        self.log(source, subject, output.to_string());
    }

    /// Legacy helper: log a string output and tag with a typed `MessageKind`.
    /// `kind` is now derived from `MessagePayload::kind()` so the explicit
    /// `kind` argument is ignored — the variant of the synthesised
    /// fallback payload determines the kind. New sites should construct
    /// a typed `MessagePayload` and call [`Self::push_message`] directly.
    pub fn log_output_kind<D: std::fmt::Display>(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        output: D,
        _kind: crate::messages::MessageKind,
    ) {
        self.log(source, subject, output.to_string());
    }

    /// Log a structured [`crate::events::GameEvent`] by rendering its
    /// `Display` impl into the `GameMessage.content`. The typed
    /// `MessagePayload` defaults to a source-appropriate fallback;
    /// callers needing a specific payload variant should instead build
    /// it themselves and call [`Self::push_message`].
    pub fn log_event(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        event: crate::events::GameEvent,
    ) {
        self.log(source, subject, event.to_string());
    }

    /// Log a structured `GameEvent` and tag with `MessageKind`.
    /// As with [`Self::log_output_kind`], the `kind` argument is now
    /// derived from the payload and is accepted for backwards
    /// compatibility only.
    pub fn log_event_kind(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        event: crate::events::GameEvent,
        _kind: crate::messages::MessageKind,
    ) {
        self.log(source, subject, event.to_string());
    }

    fn check_for_winner(&mut self) -> Result<(), GameError> {
        if let Some(winner) = self.winner() {
            let game_id = self.identifier.clone();
            let payload = crate::messages::MessagePayload::GameEnded {
                winner: Some(crate::messages::TributeRef {
                    identifier: winner.identifier.clone(),
                    name: winner.name.clone(),
                }),
            };
            let tick = self.tick_counter.boundary();
            self.push_message(
                crate::messages::MessageSource::Game(game_id.clone()),
                format!("game:{}", game_id),
                format!("{} has won the game!", winner.name),
                payload,
                tick,
            );
            self.end();
        } else if self.living_tributes_count() == 0 {
            let game_id = self.identifier.clone();
            let payload = crate::messages::MessagePayload::GameEnded { winner: None };
            let tick = self.tick_counter.boundary();
            self.push_message(
                crate::messages::MessageSource::Game(game_id.clone()),
                format!("game:{}", game_id),
                "The game has ended with no survivors.".to_string(),
                payload,
                tick,
            );
            self.end();
        }
        Ok(())
    }

    /// Prepares the game state for a new cycle.
    /// Clears old messages and area events.
    /// Increments day count by 1 if this is the first phase of a new day.
    /// (Day 1 starts at `Phase::Day`; Day 2+ starts at `Phase::Dawn`.)
    fn prepare_cycle(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
        if self.is_new_day_boundary(phase) {
            self.day = Some(self.day.unwrap_or(0) + 1);
        }

        // Clear all events from the previous cycle
        for area in self.areas.iter_mut() {
            area.events.clear();
        }
        Ok(())
    }

    /// True when this phase begins a new game-day. Day 1 begins at
    /// `Phase::Day` (no Dawn1 per spec §3); Day 2+ begins at `Phase::Dawn`.
    fn is_new_day_boundary(&self, phase: crate::messages::Phase) -> bool {
        matches!(
            (self.day, phase),
            (None | Some(0), crate::messages::Phase::Day) | (Some(_), crate::messages::Phase::Dawn)
        )
    }

    /// Announces the start of the cycle.
    fn announce_cycle_start(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
        let current_day = self.day.unwrap_or(1);
        let game_id = self.identifier.clone();
        let subject = format!("game:{}", game_id);

        let content = match phase {
            crate::messages::Phase::Dawn => {
                format!("Dawn {} breaks pale over the arena.", current_day)
            }
            crate::messages::Phase::Day => match current_day {
                1 => format!("Day {}: The games have begun!", current_day),
                3 => format!(
                    "Day {}: Sponsors take note of the remaining tributes.",
                    current_day
                ),
                _ => format!("Day {} dawns over the arena.", current_day),
            },
            crate::messages::Phase::Dusk => {
                format!("Dusk {} settles in long shadows.", current_day)
            }
            crate::messages::Phase::Night => {
                format!("Night {} falls. The arena grows dark.", current_day)
            }
        };

        let payload = crate::messages::MessagePayload::CycleStart {
            day: current_day,
            phase,
        };
        let tick = self.tick_counter.boundary();
        self.push_message(
            crate::messages::MessageSource::Game(game_id),
            subject,
            content,
            payload,
            tick,
        );

        Ok(())
    }

    /// Announces the end of a cycle
    fn announce_cycle_end(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
        let game_id = self.identifier.clone();
        let current_day = self.day.unwrap_or(1);

        // Death announcements are now emitted at the kill site as typed
        // `MessagePayload::TributeKilled` (combat: `Combat(Killed)`; env /
        // status: `TributeKilled`). Re-announcing "X has fallen" here
        // duplicated those events with a `SanityBreak` fallback payload
        // which mis-classified them in the timeline.

        let payload = crate::messages::MessagePayload::CycleEnd {
            day: current_day,
            phase,
        };
        let tick = self.tick_counter.boundary();
        self.push_message(
            crate::messages::MessageSource::Game(game_id.clone()),
            format!("game:{}", game_id),
            format!("End of {} {}.", phase, current_day),
            payload,
            tick,
        );
        Ok(())
    }

    /// Process survival checks for all tributes in an area when an event occurs
    pub fn process_event_for_area(
        &mut self,
        area: &Area,
        event: &AreaEvent,
        rng: &mut impl Rng,
    ) -> Result<(), GameError> {
        // Get area terrain and events
        let (terrain, area_events) = {
            let area_idx = self
                .areas
                .iter()
                .position(|a| a.area.as_ref() == Some(area));

            match area_idx {
                Some(idx) => (self.areas[idx].terrain.base, self.areas[idx].events.clone()),
                None => return Ok(()), // Area not found
            }
        };

        // Find all alive tributes in this area
        let tribute_indices: Vec<usize> = self
            .tributes
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_alive() && t.area == area)
            .map(|(idx, _)| idx)
            .collect();

        if tribute_indices.is_empty() {
            return Ok(()); // No tributes in area, nothing to do
        }

        let most_severe_event = if area_events.len() > 1 {
            // Find event with highest severity
            area_events
                .iter()
                .max_by_key(|e| e.severity_in_terrain(&terrain))
                .cloned()
                .unwrap_or_else(|| event.clone())
        } else {
            event.clone()
        };

        // Announce the event itself in the area channel so the broader narrative
        // captures *what happened* even when no tributes are present to react.
        let area_name = area.to_string();
        let area_subject = format!("area:{}", area_name);
        self.log_event(
            crate::messages::MessageSource::Area(area_name.clone()),
            area_subject.clone(),
            crate::events::GameEvent::AreaEvent {
                area_event: most_severe_event.to_string(),
                area_name: area_name.clone(),
            },
        );

        // Process each tribute's survival check
        for tribute_idx in tribute_indices {
            // Outcome messages we'll log after the tribute borrow is released.
            // Each entry carries an optional typed payload so death lines
            // become `MessagePayload::TributeKilled` (and render as
            // DeathCard / count toward timeline death tallies) instead of
            // falling through to the legacy `SanityBreak` fallback.
            type PendingMsg = (
                crate::messages::MessageSource,
                String,
                String,
                Option<crate::messages::MessagePayload>,
            );
            let mut pending_messages: Vec<PendingMsg> = Vec::new();

            {
                let tribute = &mut self.tributes[tribute_idx];

                // Check modifiers
                let has_affinity = tribute.terrain_affinity.contains(&terrain);

                // Check for protective items (shields only for physical events)
                let is_physical_event = matches!(
                    most_severe_event,
                    AreaEvent::Avalanche | AreaEvent::Rockslide | AreaEvent::Earthquake
                );
                let has_item_bonus =
                    is_physical_event && tribute.items.iter().any(|item| item.is_defensive());

                let is_desperate = tribute.attributes.health < 30;
                let current_health = tribute.attributes.health;

                // Run survival check with config parameters
                let result = most_severe_event.survival_check(
                    &terrain,
                    has_affinity,
                    has_item_bonus,
                    is_desperate,
                    current_health,
                    self.config.instant_death_enabled,
                    self.config.catastrophic_severity_multiplier,
                    rng,
                );

                let source = crate::messages::MessageSource::Tribute(tribute.identifier.clone());
                let subject = format!("tribute:{}", tribute.identifier);
                let roll_detail = format!(
                    "[{:?} severity, rolled {}{}]",
                    result.severity,
                    result.roll,
                    if result.modifier == 0 {
                        String::new()
                    } else if result.modifier > 0 {
                        format!(" +{}", result.modifier)
                    } else {
                        format!(" {}", result.modifier)
                    }
                );

                // Apply results
                if !result.survived {
                    tribute.attributes.health = 0;
                    let cause = most_severe_event.to_string();
                    tribute.statistics.killed_by = Some(cause.clone());
                    // Mark as RecentlyDead so the end-of-cycle announcement
                    // and the alliance-cascade pipeline both pick this death
                    // up. Without this, env-killed tributes were silently
                    // promoted to Dead at the next cycle and never triggered
                    // "has fallen" or DeathRecorded.
                    tribute.status = crate::tributes::statuses::TributeStatus::RecentlyDead;

                    let content = if result.instant_death {
                        format!(
                            "{} is instantly killed by the catastrophic {}! {}",
                            tribute.name, most_severe_event, roll_detail
                        )
                    } else {
                        format!(
                            "{} dies from the {} {}",
                            tribute.name, most_severe_event, roll_detail
                        )
                    };
                    let payload = crate::messages::MessagePayload::TributeKilled {
                        victim: crate::messages::TributeRef {
                            identifier: tribute.identifier.clone(),
                            name: tribute.name.clone(),
                        },
                        killer: None,
                        cause,
                    };
                    pending_messages.push((source, subject, content, Some(payload)));
                } else {
                    // Always announce the survival itself so the narrative captures
                    // who weathered the event, even when no rewards land.
                    pending_messages.push((
                        source.clone(),
                        subject.clone(),
                        format!(
                            "{} survives the {} {}",
                            tribute.name, most_severe_event, roll_detail
                        ),
                        None,
                    ));

                    // Survivor - apply rewards if any
                    if result.stamina_restored > 0 {
                        tribute.stamina = tribute.stamina.saturating_add(result.stamina_restored);
                        pending_messages.push((
                            source.clone(),
                            subject.clone(),
                            format!(
                                "{} recovers {} stamina from the {}",
                                tribute.name, result.stamina_restored, most_severe_event
                            ),
                            None,
                        ));
                    }

                    if result.sanity_restored > 0 {
                        tribute.attributes.sanity = tribute
                            .attributes
                            .sanity
                            .saturating_add(result.sanity_restored);
                        pending_messages.push((
                            source.clone(),
                            subject.clone(),
                            format!(
                                "{} recovers {} sanity from the {}",
                                tribute.name, result.sanity_restored, most_severe_event
                            ),
                            None,
                        ));
                    }

                    if result.reward_item.is_some() {
                        let item = Item::new_random_consumable();
                        let item_name = item.name.clone();
                        tribute.items.push(item);
                        pending_messages.push((
                            source,
                            subject,
                            format!(
                                "{} finds a {} after surviving the {}",
                                tribute.name, item_name, most_severe_event
                            ),
                            None,
                        ));
                    }
                }
            }

            // Each env-event message is its own action group, so advance
            // the per-phase tick before pushing so they sort after the
            // area announcement and any prior tribute actions.
            for (source, subject, content, payload) in pending_messages {
                let tick = self.tick_counter.next();
                let payload = payload.unwrap_or_else(|| Self::fallback_payload(&source));
                self.push_message(source, subject, content, payload, tick);
            }
        }
        Ok(())
    }

    /// Run one phase of the game. Substrate for the four-phase day model
    /// (spec `2026-05-03-four-phase-day-design.md`). Replaces the legacy
    /// `run_day_night_cycle(bool)` boundary; callers driving an entire day
    /// should use `run_full_day` instead.
    pub fn run_phase(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
        // Phase boundary: reset transient cycle state. `tick` and `emit_index`
        // are per-phase and must restart at every flip so causal
        // ordering inside a phase is contiguous from zero.
        self.current_phase = phase;
        self.tick_counter.reset();
        self.emit_index = 0;

        // Check if the game is over, and if so, end it.
        self.check_for_winner()?;

        self.prepare_cycle(phase)?;
        self.announce_cycle_start(phase)?;
        self.do_a_cycle(phase)?;
        self.announce_cycle_end(phase)?;

        // Clean up any deaths
        self.clean_up_recent_deaths();
        Ok(())
    }

    /// Run every phase of the next game-day in canonical order. Day 1 has
    /// no Dawn (per spec §3); Day 2+ runs all four phases.
    pub fn run_full_day(&mut self) -> Result<(), GameError> {
        use crate::messages::Phase;
        let next_day = self.day.unwrap_or(0) + 1;
        let phases: &[Phase] = if next_day <= 1 {
            &[Phase::Day, Phase::Dusk, Phase::Night]
        } else {
            &[Phase::Dawn, Phase::Day, Phase::Dusk, Phase::Night]
        };
        for &p in phases {
            self.run_phase(p)?;
        }
        Ok(())
    }

    /// Announce events in closed areas.
    ///
    /// Emits one `MessageSource::Area` line per active event (using
    /// `GameOutput::AreaEvent`) plus a closing `GameOutput::AreaClose`
    /// summary so consumers know the area is currently uninhabitable.
    fn announce_area_events(&mut self) -> Result<(), GameError> {
        // Snapshot to avoid borrow conflicts with self.log_output below.
        let snapshots: Vec<(String, Vec<String>)> = self
            .areas
            .iter()
            .filter(|a| !a.is_open())
            .filter_map(|a| {
                a.area.map(|area| {
                    (
                        area.to_string(),
                        a.events.iter().map(|e| e.to_string()).collect(),
                    )
                })
            })
            .collect();

        for (area_name, event_names) in snapshots {
            let subject = format!("area:{}", area_name);
            for event_name in &event_names {
                self.log_event(
                    crate::messages::MessageSource::Area(area_name.clone()),
                    subject.clone(),
                    crate::events::GameEvent::AreaEvent {
                        area_event: event_name.clone(),
                        area_name: area_name.clone(),
                    },
                );
            }
            self.log_event(
                crate::messages::MessageSource::Area(area_name.clone()),
                subject,
                crate::events::GameEvent::AreaClose {
                    area_name: area_name.clone(),
                },
            );
        }
        Ok(())
    }

    /// Ensures at least one area is open. If not, opens a random area by clearing its events.
    fn ensure_open_area(&mut self) {
        if self.random_open_area().is_none()
            && let Some(area) = self.random_area()
        {
            area.events.clear();
        }
    }

    /// Triggers events for the current cycle.
    fn trigger_cycle_events(
        &mut self,
        phase: crate::messages::Phase,
        rng: &mut SmallRng,
    ) -> Result<(), GameError> {
        use crate::messages::Phase;
        let frequency = match phase {
            Phase::Day => DAY_EVENT_FREQUENCY,
            Phase::Night => NIGHT_EVENT_FREQUENCY,
            // Substrate-only: Dawn/Dusk are silent in PR1. PR2 redistributes.
            Phase::Dawn | Phase::Dusk => return Ok(()),
        };
        let day = phase == Phase::Day;

        // Collect events to trigger (avoid borrow conflicts)
        let mut events_to_process: Vec<(Area, AreaEvent)> = Vec::new();

        // If it's nighttime, trigger an event
        // If it is daytime and not day #1 or day #3, trigger an event
        if !day || ![1, 3].contains(&self.day.unwrap_or(1)) {
            for area_details in self.areas.iter_mut() {
                if rng.random_bool(frequency) {
                    // Generate terrain-appropriate event
                    let area_event = AreaEvent::random_for_terrain(&area_details.terrain.base, rng);
                    let area = area_details.area.unwrap();

                    // Add event to area
                    area_details.events.push(area_event.clone());

                    // Announce event
                    let _event_name = area_event.to_string();
                    let _area_name = area.to_string();

                    // Collect for processing
                    events_to_process.push((area, area_event));
                }
            }
        }

        // Process survival checks for all triggered events
        for (area, event) in events_to_process {
            self.process_event_for_area(&area, &event, rng)?;
        }

        // Day 3 is Feast Day, refill the Cornucopia with a random assortment of items
        if day
            && self.day == Some(3)
            && let Some(area_details) = self
                .areas
                .iter_mut()
                .find(|ad| ad.area == Some(Area::Cornucopia))
        {
            for _ in 0..rng.random_range(1..=FEAST_WEAPON_COUNT) {
                area_details.add_item(Item::new_random_weapon());
            }
            for _ in 0..rng.random_range(1..=FEAST_SHIELD_COUNT) {
                area_details.add_item(Item::new_random_shield());
            }
            for _ in 0..rng.random_range(1..=FEAST_CONSUMABLE_COUNT) {
                area_details.add_item(Item::new_random_consumable());
            }
        }
        Ok(())
    }

    /// If the tribute count is low, constrain them by closing areas.
    /// We achieve this by spawning events in open areas.
    fn constrain_areas(&mut self, rng: &mut SmallRng) -> Result<(), GameError> {
        let tribute_count = self.living_tributes_count() as u32;
        let odds = tribute_count as f64 / 24.0;
        let mut area_events: HashMap<String, (AreaDetails, Vec<AreaEvent>)> = HashMap::new();

        if (1..LOW_TRIBUTE_THRESHOLD).contains(&tribute_count) {
            // If there is an open area, close it.
            if let Some(area_details) = self.random_open_area() {
                let event = AreaEvent::random(rng);
                let area_name = area_details.area.unwrap().to_string();
                area_events.insert(area_name, (area_details.clone(), vec![event.clone()]));
            }

            if rng.random_bool(odds) {
                // Assuming there's still an open area.
                if let Some(area_details) = self.random_open_area() {
                    let event = AreaEvent::random(rng);
                    let area_name = area_details.area.unwrap().to_string();
                    if area_events.contains_key(&area_name) {
                        let mut events = area_events[&area_name].1.clone();
                        events.push(event.clone());
                        area_events.insert(area_name, (area_details.clone(), events));
                    } else {
                        area_events.insert(area_name, (area_details.clone(), vec![event.clone()]));
                    }
                }
            }

            // Add events to each area and announce them
            for (area_name, (mut area_details, events)) in area_events.drain() {
                for event in events {
                    area_details.events.push(event.clone());
                    let _event_name = event.to_string();
                    // let area_name = area_details.area.clone().unwrap().to_string();
                }

                // Update the corresponding area with the new events
                for area in self.areas.iter_mut() {
                    let key = area.area.unwrap().to_string();
                    if key == area_name {
                        area.events = area_details.events;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Pre-computed, immutable view of game state used by `execute_cycle`.
    ///
    /// `build_cycle_context` materialises this from `&self` so the executor
    /// half of the cycle (which holds `&mut self`) can read pre-snapshotted
    /// data without re-borrowing. This split also gives gamemaker overrides
    /// (and future cycle-modifier hooks) a typed seam between the
    /// "what the world looks like" phase and the "what each tribute does"
    /// phase.
    fn build_cycle_context(
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
    fn execute_cycle(&mut self, ctx: CycleContext, rng: &mut SmallRng) -> Result<(), GameError> {
        let CycleContext {
            is_day: day,
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

        for tribute in self.tributes.iter_mut() {
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
                    let line = format!(
                        "{} hunger: {:?} -> {:?}",
                        tribute.name, prior_hunger, new_hunger
                    );
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        line,
                        Some(MessagePayload::HungerBandChanged {
                            tribute: tref.clone(),
                            from: prior_hunger,
                            to: new_hunger,
                        }),
                        None,
                    ));
                }
                if new_thirst != prior_thirst {
                    let line = format!(
                        "{} thirst: {:?} -> {:?}",
                        tribute.name, prior_thirst, new_thirst
                    );
                    collected_events.push((
                        tribute.identifier.clone(),
                        tribute.name.clone(),
                        line,
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

            let area_index = match area_details_map.get(&tribute.area) {
                Some(&idx) => idx,
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
                area_details,
                closed_areas: &closed_areas,
                available_destinations,
                all_areas: &all_areas_snapshot,
                enemy_density: &enemy_density,
                current_day,
                combat_tuning: &combat_tuning_snapshot,
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

            let mut tribute_events: Vec<crate::messages::TaggedEvent> = Vec::new();
            tribute.process_turn_phase(
                action_suggestion.clone(),
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
    fn flush_tribute_events(&mut self, collected_events: Vec<CollectedEvent>) {
        let mut last_identifier: Option<String> = None;
        let mut current_tick: u32 = self.tick_counter.boundary();
        for (identifier, _name, content, payload, _event) in collected_events {
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
    fn run_tribute_cycle(
        &mut self,
        phase: crate::messages::Phase,
        rng: &mut SmallRng,
        closed_areas: Vec<Area>,
        living_tributes: Vec<Tribute>,
        living_tributes_count: usize,
    ) -> Result<(), GameError> {
        let ctx =
            self.build_cycle_context(phase, closed_areas, living_tributes, living_tributes_count);
        self.execute_cycle(ctx, rng)
    }

    /// Runs a cycle of the game, either day or night.
    /// 1. Announce area events.
    /// 2. Open an area if there are no open areas.
    /// 3. Trigger any events for this cycle if we're past the first three days.
    /// 4. Trigger Feast Day events.
    /// 5. Close more areas by spawning more events if the tributes are getting low.
    /// 6. Run the tribute cycle.
    /// 7. Update the tributes in the game.
    fn do_a_cycle(&mut self, phase: crate::messages::Phase) -> Result<(), GameError> {
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
    fn clean_up_recent_deaths(&mut self) {
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
    fn get_area_details_mut(&mut self, area: Area) -> Option<&mut AreaDetails> {
        self.areas.iter_mut().find(|ad| ad.area == Some(area))
    }

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
                            crate::messages::MessageSource::Tribute(v_id),
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
                                    identifier: deceased.to_string(),
                                    name: String::new(),
                                },
                            )
                        });

                    for ally_id in allies_of_deceased {
                        if let Some(ally) = self.tributes.iter_mut().find(|t| t.id == ally_id) {
                            // `extreme_low_sanity` is the §7.3a low-limit
                            // mapping (see PersonalityThresholds doc).
                            let limit = ally.brain.thresholds.extreme_low_sanity;
                            let sanity = ally.attributes.sanity;
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
                                    crate::messages::MessageSource::Tribute(aid),
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
                            crate::messages::MessageSource::Tribute(p_id),
                            p_name,
                            event.to_string(),
                            payload,
                            tick,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_game_with_tributes(tributes: Vec<Tribute>) -> Game {
        Game {
            identifier: "test-game".to_string(),
            name: "Test Game".to_string(),
            status: GameStatus::InProgress,
            day: Some(1),
            areas: vec![],
            tributes,
            private: true,
            config: Default::default(),
            messages: vec![],
            alliance_events: vec![],
            tick_counter: TickCounter::default(),
            current_phase: crate::messages::Phase::Day,
            emit_index: 0,
            combat_tuning: crate::tributes::combat_tuning::CombatTuning::default(),
        }
    }

    fn create_tribute(name: &str, is_alive: bool) -> Tribute {
        let mut tribute = Tribute::new(name.to_string(), None, None);
        if is_alive {
            tribute.attributes.health = 100;
            tribute.status = TributeStatus::Healthy;
        } else {
            tribute.attributes.health = 0;
            tribute.status = TributeStatus::Dead;
        }
        tribute
    }

    #[test]
    fn test_game_new() {
        let game = Game::new("Test Game");
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.status, GameStatus::NotStarted);
        assert_eq!(game.day, None);
        assert_eq!(game.tributes.len(), 0);
    }

    #[test]
    fn game_has_empty_alliance_event_queue_on_new() {
        let g = Game::default();
        assert!(g.alliance_events.is_empty());
    }

    #[test]
    fn test_game_start() {
        let mut game = Game::new("Test Game");
        game.start().expect("Failed to start game");
        assert_eq!(game.status, GameStatus::InProgress);
        assert_eq!(game.day, None);
    }

    #[test]
    fn test_game_end() {
        let mut game = Game::new("Test Game");
        game.start().expect("Failed to start game");
        game.end();
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_living_and_recently_dead_tributes() {
        let mut game = Game::new("Test Game");
        let t1 = Tribute::default();
        let t2 = Tribute::default();
        game.tributes.push(t1);
        game.tributes.push(t2);
        assert_eq!(game.living_tributes().len(), 2);
        assert_eq!(game.recently_dead_tributes().len(), 0);
        game.tributes[0].status = TributeStatus::RecentlyDead;
        assert_eq!(game.living_tributes().len(), 1);
        assert_eq!(game.recently_dead_tributes().len(), 1);
    }

    #[test]
    fn test_game_winner() {
        let mut game = Game::new("Test Game");
        let t1 = Tribute::default();
        let t2 = Tribute::default();
        game.tributes.push(t1);
        game.tributes.push(t2.clone());
        game.start().expect("Failed to start game");
        assert_eq!(game.winner(), None);
        game.tributes[0].status = TributeStatus::Dead;
        assert_eq!(game.winner().unwrap().name, t2.name);
    }

    #[test]
    fn test_random_open_area() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
        game.areas.push(area1);
        game.areas.push(area2.clone());
        assert!(game.random_area().is_some());
        let mut rng = rand::rng();
        let event = AreaEvent::random(&mut rng);
        game.areas[0].events.push(event.clone());
        assert_eq!(game.random_open_area().unwrap(), area2);
    }

    #[test]
    fn test_clean_up_recent_deaths() {
        let mut game = Game::new("Test Game");

        let mut tribute = Tribute::default();
        tribute.set_status(TributeStatus::RecentlyDead);
        game.tributes.push(tribute.clone());

        assert_eq!(game.recently_dead_tributes().len(), 1);
        assert_eq!(game.recently_dead_tributes()[0], tribute);

        game.clean_up_recent_deaths();
        assert_eq!(game.tributes[0].status, TributeStatus::Dead);
    }

    #[test]
    fn test_check_game_state_winner_exists() {
        let winner_tribute = create_tribute("Winner", true);
        let loser_tribute = create_tribute("Loser", false);
        let mut game =
            create_test_game_with_tributes(vec![winner_tribute.clone(), loser_tribute.clone()]);

        // Game should have only one living tribute and they should be the winner
        assert_eq!(game.living_tributes().len(), 1);
        assert_eq!(game.winner(), Some(winner_tribute.clone()));

        let _ = game.check_for_winner();

        // Game should be finished
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_check_game_state_no_survivors() {
        let loser_tribute = create_tribute("Loser", false);
        let loser2_tribute = create_tribute("Loser 2", false);
        let mut game =
            create_test_game_with_tributes(vec![loser_tribute.clone(), loser2_tribute.clone()]);

        // Game should have only no living tributes and no winner
        assert!(game.living_tributes().is_empty());
        assert!(game.winner().is_none());

        let _ = game.check_for_winner();

        // Game should be finished
        assert_eq!(game.status, GameStatus::Finished);
    }

    #[test]
    fn test_check_game_state_continues() {
        let living_tribute1 = create_tribute("Living1", true);
        let living_tribute2 = create_tribute("Living2", true);
        let mut game =
            create_test_game_with_tributes(vec![living_tribute1.clone(), living_tribute2.clone()]);
        let starting_state = game.status.clone();

        // Game should have only one living tribute and they should be the winner
        assert_eq!(game.living_tributes().len(), 2);
        assert!(game.winner().is_none());

        let _ = game.check_for_winner();

        // Game should be finished
        assert_eq!(game.status, starting_state);
    }

    #[test]
    fn test_prepare_cycle() {
        use crate::messages::Phase;
        let mut game = Game::new("Test Game");
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
        let mut rng = rand::rng();
        let event = AreaEvent::random(&mut rng);
        game.day = Some(1);
        game.areas.push(area);
        game.areas[0].events.push(event.clone());
        // Dawn on Day 2+ advances the day and clears events.
        let _ = game.prepare_cycle(Phase::Dawn);
        assert_eq!(game.day, Some(2));
        assert_eq!(game.areas[0].events.len(), 0);

        game.areas[0].events.push(event.clone());
        // Night never advances the day.
        let _ = game.prepare_cycle(Phase::Night);
        assert_eq!(game.day, Some(2));
        assert_eq!(game.areas[0].events.len(), 0);
    }

    #[test]
    fn test_announce_cycle_start() {
        // Clear any messages from other tests running in parallel

        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);
        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        game.day = Some(1);
        let _ = game.announce_cycle_start(crate::messages::Phase::Day);
        // Day 1 has a single announcement.
        assert_eq!(game.messages.len(), 1);
    }

    #[test]
    fn test_announce_cycle_end() {
        // Clear any messages from other tests running in parallel

        let tribute1 = create_tribute("Tribute1", true);
        let mut tribute2 = create_tribute("Tribute2", false);
        tribute2.set_status(TributeStatus::RecentlyDead);
        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        game.day = Some(1);
        let _ = game.announce_cycle_end(crate::messages::Phase::Day);
        // Death announcements moved to the kill site as typed
        // `MessagePayload::TributeKilled`. Only the cycle-end summary
        // remains here.
        assert_eq!(game.messages.len(), 1);
    }

    #[test]
    fn test_announce_area_events() {
        let mut game = Game::new("Test Game");
        let mut area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        let mut rng = rand::rng();
        area.events.push(AreaEvent::random(&mut rng));
        area.events.push(AreaEvent::random(&mut rng));
        game.areas.push(area);

        assert!(!game.areas[0].is_open());
        let _ = game.announce_area_events();

        // 2 AreaEvent lines + 1 AreaClose summary
        assert_eq!(game.messages.len(), 3);
        // All emitted under the Area channel for the affected area.
        let area_name = Area::Cornucopia.to_string();
        for msg in &game.messages {
            assert_eq!(
                msg.source,
                crate::messages::MessageSource::Area(area_name.clone())
            );
            assert_eq!(
                msg.subject,
                format!("{}:area:{}", game.identifier, area_name)
            );
        }
    }

    /// Regression for hangrier_games-i7rq: every emitted message's
    /// subject must start with the game identifier so the API's
    /// per-game log queries (`WHERE string::starts_with(subject,
    /// $game_id)`) match. Without this, day pages and timeline summary
    /// were always empty.
    #[test]
    fn message_subjects_are_prefixed_with_game_id() {
        let mut game = Game::new("Subject Prefix Test");
        game.log(
            crate::messages::MessageSource::Game(game.identifier.clone()),
            format!("game:{}", game.identifier),
            "hello".to_string(),
        );
        game.log(
            crate::messages::MessageSource::Area("Cornucopia".to_string()),
            "area:Cornucopia".to_string(),
            "boom".to_string(),
        );
        game.log(
            crate::messages::MessageSource::Tribute("trib-id".to_string()),
            "tribute:trib-id".to_string(),
            "ouch".to_string(),
        );
        let prefix = format!("{}:", game.identifier);
        for msg in &game.messages {
            assert!(
                msg.subject.starts_with(&prefix),
                "subject {:?} missing game-id prefix {:?}",
                msg.subject,
                prefix
            );
        }
        // Idempotent: calling log twice should not double-prefix.
        let count_before = game.messages.len();
        let already_prefixed = format!("{}:area:Other", game.identifier);
        game.log(
            crate::messages::MessageSource::Area("Other".to_string()),
            already_prefixed.clone(),
            "ok".to_string(),
        );
        assert_eq!(
            game.messages[count_before].subject, already_prefixed,
            "subject already prefixed should not be double-prefixed"
        );
    }

    #[test]
    fn test_ensure_open_area() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
        game.areas.push(area1);
        game.areas.push(area2);

        assert!(game.random_open_area().is_some());

        // Close the areas
        let mut rng = rand::rng();
        game.areas[0].events.push(AreaEvent::random(&mut rng));
        game.areas[1].events.push(AreaEvent::random(&mut rng));

        assert!(game.random_open_area().is_none());

        game.ensure_open_area();
        assert!(game.random_open_area().is_some());
    }

    #[test]
    fn test_trigger_cycle_events() {}

    #[test]
    fn test_constrain_areas() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
        game.areas.push(area1);
        game.areas.push(area2);

        // Add tributes to the game
        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);
        game.tributes.push(tribute1.clone());
        game.tributes.push(tribute2.clone());

        // Constrain areas
        // Use a fixed seed so the area-selection branch is deterministic.
        let mut rng = SmallRng::seed_from_u64(0);
        let _ = game.constrain_areas(&mut rng);

        // Check if at least one area is closed
        assert!(game.random_open_area().is_some());
        assert_eq!(game.open_areas().len(), 1);
        assert_eq!(game.closed_areas().len(), 1);
    }

    #[test]
    fn test_run_tribute_cycle() {
        // Add tributes
        let tribute1 = create_tribute("Tribute1", true);
        let tribute2 = create_tribute("Tribute2", true);

        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        game.areas.push(area);
        let closed_areas = game
            .areas
            .iter()
            .filter(|ad| ad.area.is_some() & !ad.is_open())
            .map(|ad| ad.area.unwrap())
            .collect::<Vec<Area>>();

        // Run the tribute cycle
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![tribute1.clone(), tribute2.clone()],
            2,
        );

        // Check if the tributes are updated correctly
        let new_tribute1 = game.tributes[0].clone();
        let new_tribute2 = game.tributes[1].clone();
        assert_ne!(tribute1, new_tribute1);
        assert_ne!(tribute2, new_tribute2);
    }

    #[test]
    fn test_open_and_closed_areas() {
        let mut game = Game::new("Test Game");
        let area1 = AreaDetails::new(Some("Lake".to_string()), Area::Sector1);
        let area2 = AreaDetails::new(Some("Forest".to_string()), Area::Sector4);
        game.areas.push(area1);
        game.areas.push(area2);

        assert_eq!(game.open_areas().len(), 2);
        assert!(game.closed_areas().is_empty());

        // Close one area
        let mut rng = rand::rng();
        game.areas[0].events.push(AreaEvent::random(&mut rng));

        assert_eq!(game.open_areas().len(), 1);
        assert_eq!(game.closed_areas().len(), 1);
    }

    // ---- Phase 4: alliance event drain -----------------------------------

    #[test]
    fn process_alliance_events_betrayal_removes_pair_on_victim_side() {
        // Victim still lists betrayer in allies (betrayer's own list was
        // already cleaned by the betrayal trigger that enqueued the event).
        let mut betrayer = Tribute::new("Betrayer".to_string(), Some(1), None);
        let mut victim = Tribute::new("Victim".to_string(), Some(2), None);
        victim.allies.push(betrayer.id);
        // Sanity force victim to a state where the drain path runs cleanly.
        victim.attributes.health = 100;
        betrayer.attributes.health = 100;
        let bid = betrayer.id;
        let vid = victim.id;

        let mut game = create_test_game_with_tributes(vec![betrayer, victim]);
        game.alliance_events.push(
            crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
                betrayer: bid,
                victim: vid,
            },
        );

        let mut rng = SmallRng::seed_from_u64(53);
        game.process_alliance_events(&mut rng);

        let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
        assert!(!v.allies.contains(&bid));
        assert!(v.pending_trust_shock);
        assert!(game.alliance_events.is_empty());
    }

    #[test]
    fn process_alliance_events_betrayer_not_marked_for_trust_shock() {
        let betrayer = Tribute::new("Betrayer".to_string(), Some(1), None);
        let victim = Tribute::new("Victim".to_string(), Some(2), None);
        let bid = betrayer.id;
        let vid = victim.id;

        let mut game = create_test_game_with_tributes(vec![betrayer, victim]);
        game.alliance_events.push(
            crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
                betrayer: bid,
                victim: vid,
            },
        );

        let mut rng = SmallRng::seed_from_u64(53);
        game.process_alliance_events(&mut rng);

        let b = game.tributes.iter().find(|t| t.id == bid).unwrap();
        assert!(!b.pending_trust_shock, "betrayer must not roll trust-shock");
    }

    #[test]
    fn process_alliance_events_death_removes_deceased_from_all_ally_lists() {
        // Three tributes; the deceased was in two allies' lists.
        let deceased = Tribute::new("Deceased".to_string(), Some(1), None);
        let mut a = Tribute::new("A".to_string(), Some(2), None);
        let mut b = Tribute::new("B".to_string(), Some(3), None);
        a.allies.push(deceased.id);
        b.allies.push(deceased.id);
        // Force ally sanity well above any threshold so the cascade roll
        // never fires; we want to verify the unconditional cleanup path.
        a.attributes.sanity = 100;
        b.attributes.sanity = 100;

        let did = deceased.id;
        let mut game = create_test_game_with_tributes(vec![deceased, a, b]);
        game.alliance_events
            .push(crate::tributes::alliances::AllianceEvent::DeathRecorded {
                deceased: did,
                killer: None,
            });

        let mut rng = SmallRng::seed_from_u64(89);
        game.process_alliance_events(&mut rng);

        for t in game.tributes.iter().filter(|t| t.id != did) {
            assert!(
                !t.allies.contains(&did),
                "tribute {} still lists deceased",
                t.name
            );
        }
        assert!(game.alliance_events.is_empty());
    }

    #[test]
    fn run_tribute_cycle_drains_tribute_alliance_events_into_game_queue() {
        // Pre-load tribute1's alliance_events buffer; after run_tribute_cycle
        // it must be drained into the game queue and processed (BetrayalRecorded
        // cleans the victim's allies and flags pending_trust_shock).
        let mut tribute1 = create_tribute("Tribute1", true);
        let mut tribute2 = create_tribute("Tribute2", true);
        // Make tribute2 list tribute1 as ally; betrayal has tribute1 as betrayer,
        // tribute2 as victim. Plumbing only — we just need the side effects we
        // can observe on the victim.
        tribute2.allies.push(tribute1.id);
        let bid = tribute1.id;
        let vid = tribute2.id;
        tribute1.alliance_events.push(
            crate::tributes::alliances::AllianceEvent::BetrayalRecorded {
                betrayer: bid,
                victim: vid,
            },
        );

        let mut game = create_test_game_with_tributes(vec![tribute1.clone(), tribute2.clone()]);
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        game.areas.push(area);
        let closed_areas = game
            .areas
            .iter()
            .filter(|ad| ad.area.is_some() & !ad.is_open())
            .map(|ad| ad.area.unwrap())
            .collect::<Vec<Area>>();

        let mut rng = SmallRng::seed_from_u64(211);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![tribute1.clone(), tribute2.clone()],
            2,
        );

        // After cycle: queue empty (drained + processed), each tribute's local
        // buffer empty, victim's allies cleaned, victim flagged for trust shock.
        assert!(game.alliance_events.is_empty(), "game queue must drain");
        for t in &game.tributes {
            assert!(
                t.alliance_events.is_empty(),
                "tribute {} buffer must drain",
                t.name
            );
        }
        let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
        assert!(!v.allies.contains(&bid), "victim allies cleaned");
        assert!(v.pending_trust_shock, "victim flagged for trust shock");
    }

    #[test]
    fn run_tribute_cycle_forms_alliance_between_compatible_same_area_tributes() {
        // Two Friendly tributes from the same district sharing an area
        // should be able to form an alliance during a cycle. With both
        // sides starting at 0 allies, district bonus, and Friendly affinity
        // 1.5 each, roll_chance ≈ 0.675; with a fixed seed and many trials
        // we deterministically observe at least one cycle that forms.
        use crate::tributes::traits::Trait;
        let mut t1 = create_tribute("Cinna", true);
        let mut t2 = create_tribute("Portia", true);
        // Force compatibility: same district + Friendly traits.
        t1.district = 1;
        t2.district = 1;
        t1.traits = vec![Trait::Friendly];
        t2.traits = vec![Trait::Friendly];
        // Place both in Cornucopia (default `Tribute::new` already does this,
        // but be explicit for the test's intent).
        t1.area = Area::Cornucopia;
        t2.area = Area::Cornucopia;

        let id1 = t1.id;
        let id2 = t2.id;

        let mut game = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        game.areas.push(area);
        let closed_areas = game
            .areas
            .iter()
            .filter(|ad| ad.area.is_some() & !ad.is_open())
            .map(|ad| ad.area.unwrap())
            .collect::<Vec<Area>>();

        // Loop a few seeded cycles until at least one forms; if production
        // wiring is correct this should hit within a handful of trials.
        // Alliance formation is now a deliberate `Action::ProposeAlliance`
        // gated by Brain::wants_to_propose_alliance (5%-15% per turn for
        // eligible tributes). Sweep many seeds so we deterministically hit at
        // least one cycle where a Friendly same-district pair proposes and
        // succeeds.
        let mut formed = false;
        for seed in 0u64..400 {
            let mut g = game.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = g.run_tribute_cycle(
                crate::messages::Phase::Day,
                &mut rng,
                closed_areas.clone(),
                vec![t1.clone(), t2.clone()],
                2,
            );
            let a1 = g.tributes.iter().find(|t| t.id == id1).unwrap();
            let a2 = g.tributes.iter().find(|t| t.id == id2).unwrap();
            if a1.allies.contains(&id2) && a2.allies.contains(&id1) {
                formed = true;
                break;
            }
        }
        assert!(
            formed,
            "Friendly same-district pair must form an alliance within a few cycles"
        );
    }

    #[test]
    fn run_tribute_cycle_treacherous_tribute_betrays_same_area_ally_when_timer_elapses() {
        // Treacherous tribute with an ally in the same area, timer at the
        // betrayal interval, must enqueue BetrayalRecorded during the cycle.
        // After process_alliance_events, the victim must have:
        //   - pending_trust_shock set
        //   - betrayer removed from allies
        // and the betrayer must have:
        //   - victim removed from allies
        //   - timer reset to 0
        use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
        use crate::tributes::traits::Trait;

        let mut betrayer = create_tribute("Cato", true);
        let mut victim = create_tribute("Glimmer", true);
        betrayer.traits = vec![Trait::Treacherous];
        // Strip any auto-generated traits from victim that might accidentally
        // form an alliance back during the cycle (we want a pre-existing ally).
        victim.traits = vec![Trait::Tough];
        // Pre-existing alliance set up manually.
        betrayer.allies.push(victim.id);
        victim.allies.push(betrayer.id);
        // Timer at threshold so betrayal fires this turn.
        betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
        // Same area.
        betrayer.area = Area::Cornucopia;
        victim.area = Area::Cornucopia;
        // Different districts so we don't accidentally re-form alliances
        // during the formation pass.
        betrayer.district = 1;
        victim.district = 2;

        let bid = betrayer.id;
        let vid = victim.id;

        let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
        let area = AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia);
        game.areas.push(area);
        let closed_areas = game
            .areas
            .iter()
            .filter(|ad| ad.area.is_some() & !ad.is_open())
            .map(|ad| ad.area.unwrap())
            .collect::<Vec<Area>>();

        let mut rng = SmallRng::seed_from_u64(313);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![betrayer.clone(), victim.clone()],
            2,
        );

        let v = game.tributes.iter().find(|t| t.id == vid).unwrap();
        let b = game.tributes.iter().find(|t| t.id == bid).unwrap();
        assert!(!v.allies.contains(&bid), "victim allies cleaned by event");
        assert!(v.pending_trust_shock, "victim flagged for trust shock");
        assert!(!b.allies.contains(&vid), "betrayer dropped victim locally");
        // tick_alliance_timers ran and incremented to TREACHEROUS_BETRAYAL_INTERVAL+1
        // before betrayal fired? No: betrayal logic must reset to 0. After reset,
        // the rest of process_turn_phase doesn't tick again, so we expect 0.
        assert_eq!(
            b.turns_since_last_betrayal, 0,
            "betrayal resets the cooldown timer"
        );
    }

    #[test]
    fn run_tribute_cycle_treacherous_no_betrayal_without_same_area_ally_resets_timer() {
        // Treacherous tribute alone in its area: no betrayal possible, but
        // the timer should still reset (one missed opportunity per cycle).
        use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
        use crate::tributes::traits::Trait;

        let mut loner = create_tribute("Foxface", true);
        let mut other = create_tribute("Marvel", true);
        loner.traits = vec![Trait::Treacherous];
        other.traits = vec![Trait::Tough];
        loner.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
        // Different areas so other is not a same-area ally.
        loner.area = Area::Sector1;
        other.area = Area::Sector4;
        loner.district = 5;
        other.district = 6;
        let lid = loner.id;

        let mut game = create_test_game_with_tributes(vec![loner.clone(), other.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Hill".to_string()), Area::Sector1));
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Sector4));
        let closed_areas = game
            .areas
            .iter()
            .filter(|ad| ad.area.is_some() & !ad.is_open())
            .map(|ad| ad.area.unwrap())
            .collect::<Vec<Area>>();

        let mut rng = SmallRng::seed_from_u64(419);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![loner.clone(), other.clone()],
            2,
        );

        let l = game.tributes.iter().find(|t| t.id == lid).unwrap();
        assert_eq!(
            l.turns_since_last_betrayal, 0,
            "missed opportunity also resets the timer per spec §7.4(b)"
        );
    }

    #[test]
    fn run_tribute_cycle_enqueues_death_recorded_for_recently_dead_ally() {
        // A tribute who died last cycle (status=RecentlyDead) must trigger
        // a DeathRecorded event so allies process the ally-death cascade.
        // After the cycle, the deceased's allies should have:
        //   - the deceased removed from their `allies` lists (via cascade);
        //   - process_alliance_events drained the queue.
        use crate::tributes::traits::Trait;

        let mut deceased = create_tribute("Rue", true);
        let mut survivor = create_tribute("Katniss", true);
        // Make survivor highly likely to break on cascade: low sanity, high
        // threshold makes deficit_ratio close to 1.0 → near-certain break.
        survivor.attributes.sanity = 0;
        survivor.brain.thresholds.extreme_low_sanity = 50;
        survivor.traits = vec![Trait::Tough];
        deceased.traits = vec![Trait::Tough];
        // Pre-existing alliance (survivor lists deceased as ally).
        survivor.allies.push(deceased.id);
        deceased.allies.push(survivor.id);
        // Mark deceased as RecentlyDead going into the cycle.
        deceased.attributes.health = 0;
        deceased.status = TributeStatus::RecentlyDead;
        // Same area so deceased is "in the cycle" but the early skip applies.
        deceased.area = Area::Cornucopia;
        survivor.area = Area::Cornucopia;
        deceased.district = 11;
        survivor.district = 12;

        let did = deceased.id;
        let sid = survivor.id;

        let mut game = create_test_game_with_tributes(vec![deceased.clone(), survivor.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        // Living tributes snapshot: deceased is RecentlyDead so excluded.
        let living = game.living_tributes();
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(547);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            living,
            1,
        );

        // Cycle drained the queue (no leftovers).
        assert!(
            game.alliance_events.is_empty(),
            "queue must drain after cycle"
        );
        // Deceased promoted to Dead.
        let d = game.tributes.iter().find(|t| t.id == did).unwrap();
        assert_eq!(d.status, TributeStatus::Dead);
        // Survivor's ally list cleaned of deceased (cascade fired with high
        // probability given sanity=0 vs threshold=50; even on the rare miss
        // the alliance edge is still broken because process_alliance_events
        // does symmetric removal of dead from all surviving allies' lists).
        let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
        assert!(
            !s.allies.contains(&did),
            "survivor must not retain a dead ally edge"
        );
    }

    #[test]
    fn run_tribute_cycle_three_way_preserves_existing_alliance() {
        // Three-way scenario: A and B are pre-allied; C is a LoneWolf in
        // the same area (refuser → cannot form an alliance with either).
        // After a cycle, A and B's bond must remain intact and C must
        // remain unallied. This pins that:
        //   1. The formation pass does not silently rebreak existing
        //      same-area alliances.
        //   2. The presence of a third unalliable tribute does not
        //      perturb the pair's bond.
        use crate::tributes::traits::Trait;

        let mut a = create_tribute("Katniss", true);
        let mut b = create_tribute("Peeta", true);
        let mut c = create_tribute("Cato", true);
        a.traits = vec![Trait::Friendly];
        b.traits = vec![Trait::Loyal];
        c.traits = vec![Trait::LoneWolf];
        // Pre-existing symmetric alliance between A and B.
        a.allies.push(b.id);
        b.allies.push(a.id);
        a.area = Area::Cornucopia;
        b.area = Area::Cornucopia;
        c.area = Area::Cornucopia;
        a.district = 1;
        b.district = 2;
        c.district = 3;

        let aid = a.id;
        let bid = b.id;
        let cid = c.id;

        let mut game = create_test_game_with_tributes(vec![a.clone(), b.clone(), c.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let living = game.living_tributes();
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(547);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            living,
            3,
        );

        let a2 = game.tributes.iter().find(|t| t.id == aid).unwrap();
        let b2 = game.tributes.iter().find(|t| t.id == bid).unwrap();
        let c2 = game.tributes.iter().find(|t| t.id == cid).unwrap();
        assert!(a2.allies.contains(&bid), "A still allied with B");
        assert!(b2.allies.contains(&aid), "B still allied with A");
        assert!(!a2.allies.contains(&cid), "A did not bond with LoneWolf C");
        assert!(!b2.allies.contains(&cid), "B did not bond with LoneWolf C");
        assert!(c2.allies.is_empty(), "LoneWolf C remains unallied");
    }

    #[test]
    fn run_tribute_cycle_consumes_recently_killed_by_for_combat_death() {
        // A tribute who died at a combat site has `recently_killed_by` set
        // by the combat code. The cycle must read it, emit DeathRecorded
        // with that killer, and clear the field so it does not leak into
        // subsequent cycles.
        let mut deceased = create_tribute("Rue", true);
        let mut killer = create_tribute("Cato", true);
        let mut survivor = create_tribute("Katniss", true);

        let did = deceased.id;
        let kid = killer.id;
        let sid = survivor.id;

        // Pre-existing alliance so DeathRecorded has a cascade target.
        survivor.allies.push(deceased.id);
        deceased.allies.push(survivor.id);

        // Simulate combat outcome going into the cycle.
        deceased.attributes.health = 0;
        deceased.status = TributeStatus::RecentlyDead;
        deceased.recently_killed_by = Some(kid);

        deceased.area = Area::Cornucopia;
        killer.area = Area::Cornucopia;
        survivor.area = Area::Cornucopia;
        deceased.district = 11;
        killer.district = 2;
        survivor.district = 12;

        let mut game = create_test_game_with_tributes(vec![
            deceased.clone(),
            killer.clone(),
            survivor.clone(),
        ]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let living = game.living_tributes();
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(547);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            living,
            1,
        );

        // Cycle drained the queue.
        assert!(
            game.alliance_events.is_empty(),
            "queue must drain after cycle"
        );
        // Deceased promoted to Dead and field cleared.
        let d = game.tributes.iter().find(|t| t.id == did).unwrap();
        assert_eq!(d.status, TributeStatus::Dead);
        assert!(
            d.recently_killed_by.is_none(),
            "cycle must take() the killer field after emitting DeathRecorded"
        );
        // Cascade fired (deceased removed from survivor's allies).
        let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
        assert!(
            !s.allies.contains(&did),
            "survivor must not retain a dead ally edge"
        );
    }

    #[test]
    fn run_tribute_cycle_environmental_death_emits_killer_none() {
        // A tribute who died from environmental/status damage has no
        // `recently_killed_by` set. The cycle must still emit DeathRecorded
        // but with killer: None. We assert the field stays None across the
        // cycle and the cascade still fires (downstream behavior unchanged).
        let mut deceased = create_tribute("Rue", true);
        let mut survivor = create_tribute("Katniss", true);

        let did = deceased.id;
        let sid = survivor.id;

        survivor.allies.push(deceased.id);
        deceased.allies.push(survivor.id);

        // Environmental death: health=0, RecentlyDead, killer field None.
        deceased.attributes.health = 0;
        deceased.status = TributeStatus::RecentlyDead;
        assert!(deceased.recently_killed_by.is_none());

        deceased.area = Area::Cornucopia;
        survivor.area = Area::Cornucopia;
        deceased.district = 11;
        survivor.district = 12;

        let mut game = create_test_game_with_tributes(vec![deceased.clone(), survivor.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let living = game.living_tributes();
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(547);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            living,
            1,
        );

        let d = game.tributes.iter().find(|t| t.id == did).unwrap();
        assert_eq!(d.status, TributeStatus::Dead);
        assert!(
            d.recently_killed_by.is_none(),
            "environmental death keeps killer field None"
        );
        let s = game.tributes.iter().find(|t| t.id == sid).unwrap();
        assert!(!s.allies.contains(&did));
    }

    #[test]
    fn alliance_formation_emits_message_with_alliance_formed_kind() {
        // Friendly + same district guarantees a high formation chance; loop
        // a few seeds until at least one cycle forms an alliance and assert
        // the resulting message carries kind = AllianceFormed and the exact
        // display string from `GameOutput::AllianceFormed`.

        use crate::tributes::traits::Trait;

        let mut t1 = create_tribute("Cinna", true);
        let mut t2 = create_tribute("Portia", true);
        t1.district = 1;
        t2.district = 1;
        t1.traits = vec![Trait::Friendly];
        t2.traits = vec![Trait::Friendly];
        t1.area = Area::Cornucopia;
        t2.area = Area::Cornucopia;

        let base = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
        let mut game_with_area = base.clone();
        game_with_area
            .areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let closed_areas: Vec<Area> = vec![];

        // Alliance formation is now a deliberate Action::ProposeAlliance
        // (5%-15% per turn for eligible tributes); sweep many seeds so we
        // deterministically observe at least one AllianceFormed message.
        let mut hit: Option<crate::messages::GameMessage> = None;
        for seed in 0u64..400 {
            let mut g = game_with_area.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = g.run_tribute_cycle(
                crate::messages::Phase::Day,
                &mut rng,
                closed_areas.clone(),
                vec![t1.clone(), t2.clone()],
                2,
            );
            if let Some(m) = g.messages.iter().find(|m| {
                matches!(
                    m.payload,
                    crate::messages::MessagePayload::AllianceFormed { .. }
                )
            }) {
                hit = Some(m.clone());
                break;
            }
        }
        let m = hit.expect("at least one cycle must emit AllianceFormed");
        assert!(
            matches!(
                m.payload,
                crate::messages::MessagePayload::AllianceFormed { .. }
            ),
            "expected AllianceFormed payload, got {:?}",
            m.payload
        );
        assert!(
            m.content.contains("form an alliance"),
            "content should match GameOutput::AllianceFormed display, got: {}",
            m.content
        );
    }

    #[test]
    fn betrayal_emits_message_with_betrayal_triggered_kind() {
        use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
        use crate::tributes::traits::Trait;

        let mut betrayer = create_tribute("Cato", true);
        let mut victim = create_tribute("Glimmer", true);
        betrayer.traits = vec![Trait::Treacherous];
        victim.traits = vec![Trait::Tough];
        betrayer.allies.push(victim.id);
        victim.allies.push(betrayer.id);
        betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
        betrayer.area = Area::Cornucopia;
        victim.area = Area::Cornucopia;
        betrayer.district = 1;
        victim.district = 2;

        let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(313);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![betrayer.clone(), victim.clone()],
            2,
        );

        let m = game
            .messages
            .iter()
            .find(|m| {
                matches!(
                    m.payload,
                    crate::messages::MessagePayload::BetrayalTriggered { .. }
                )
            })
            .expect("betrayal cycle must emit BetrayalTriggered message");
        assert_eq!(
            m.content,
            "Cato betrays Glimmer — true to their treacherous nature."
        );
    }

    /// mqi.2 parity: at the alliance-formation emission site, the structured
    /// `GameEvent::AllianceFormed` constructed inside `run_tribute_cycle`
    /// renders to the exact same string that ends up as `GameMessage.content`.
    /// Catches future drift between the typed event and the legacy renderer
    /// at the actual call site (not just at the type level — that is covered
    /// by the parity table in `events::tests`).
    #[test]
    fn alliance_formed_message_content_matches_game_event_display() {
        use crate::tributes::traits::Trait;

        let mut t1 = create_tribute("Cinna", true);
        let mut t2 = create_tribute("Portia", true);
        t1.district = 1;
        t2.district = 1;
        t1.traits = vec![Trait::Friendly];
        t2.traits = vec![Trait::Friendly];
        t1.area = Area::Cornucopia;
        t2.area = Area::Cornucopia;

        let base = create_test_game_with_tributes(vec![t1.clone(), t2.clone()]);
        let mut game_with_area = base.clone();
        game_with_area
            .areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let closed_areas: Vec<Area> = vec![];

        // Alliance formation is now a deliberate Action::ProposeAlliance
        // (5%-15% per turn for eligible tributes); sweep many seeds so we
        // deterministically observe at least one AllianceFormed message.
        let mut hit: Option<crate::messages::GameMessage> = None;
        for seed in 0u64..400 {
            let mut g = game_with_area.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            let _ = g.run_tribute_cycle(
                crate::messages::Phase::Day,
                &mut rng,
                closed_areas.clone(),
                vec![t1.clone(), t2.clone()],
                2,
            );
            if let Some(m) = g.messages.iter().find(|m| {
                matches!(
                    m.payload,
                    crate::messages::MessagePayload::AllianceFormed { .. }
                )
            }) {
                hit = Some(m.clone());
                break;
            }
        }
        let m = hit.expect("at least one cycle must emit AllianceFormed");

        // Reconstruct the structured event with the same inputs the engine
        // used. The factor label depends on trait-overlap math; rather than
        // recompute it here (and re-couple the test to that algorithm) we
        // parse it back out of the rendered message, which is exactly what
        // mqi.4+ consumers will rely on. The point of this test is parity
        // between `GameEvent::AllianceFormed::Display` and
        // `GameMessage.content` at the call site, not validation of the
        // factor-selection logic.
        let factor = m
            .content
            .rsplit_once('(')
            .and_then(|(_, rest)| rest.rsplit_once(')').map(|(f, _)| f.to_string()))
            .expect("rendered alliance message must contain a parenthesised factor");
        let candidates = [
            crate::events::GameEvent::AllianceFormed {
                tribute_a_id: t1.id,
                tribute_a_name: t1.name.clone(),
                tribute_b_id: t2.id,
                tribute_b_name: t2.name.clone(),
                factor: factor.clone(),
            },
            crate::events::GameEvent::AllianceFormed {
                tribute_a_id: t2.id,
                tribute_a_name: t2.name.clone(),
                tribute_b_id: t1.id,
                tribute_b_name: t1.name.clone(),
                factor,
            },
        ];
        assert!(
            candidates.iter().any(|ev| ev.to_string() == m.content),
            "GameMessage.content {:?} must match GameEvent::AllianceFormed Display \
             for one of {:?}",
            m.content,
            candidates.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        );
    }

    /// mqi.2 parity: at the betrayal emission site, the structured
    /// `GameEvent::BetrayalTriggered` constructed inside
    /// `process_alliance_events` renders identically to `GameMessage.content`.
    #[test]
    fn betrayal_triggered_message_content_matches_game_event_display() {
        use crate::events::GameEvent;

        use crate::tributes::alliances::TREACHEROUS_BETRAYAL_INTERVAL;
        use crate::tributes::traits::Trait;

        let mut betrayer = create_tribute("Cato", true);
        let mut victim = create_tribute("Glimmer", true);
        betrayer.traits = vec![Trait::Treacherous];
        victim.traits = vec![Trait::Tough];
        betrayer.allies.push(victim.id);
        victim.allies.push(betrayer.id);
        betrayer.turns_since_last_betrayal = TREACHEROUS_BETRAYAL_INTERVAL;
        betrayer.area = Area::Cornucopia;
        victim.area = Area::Cornucopia;
        betrayer.district = 1;
        victim.district = 2;

        let betrayer_id = betrayer.id;
        let victim_id = victim.id;
        let betrayer_name = betrayer.name.clone();
        let victim_name = victim.name.clone();

        let mut game = create_test_game_with_tributes(vec![betrayer.clone(), victim.clone()]);
        game.areas
            .push(AreaDetails::new(Some("Lake".to_string()), Area::Cornucopia));
        let closed_areas: Vec<Area> = vec![];

        let mut rng = SmallRng::seed_from_u64(313);
        let _ = game.run_tribute_cycle(
            crate::messages::Phase::Day,
            &mut rng,
            closed_areas,
            vec![betrayer.clone(), victim.clone()],
            2,
        );

        let m = game
            .messages
            .iter()
            .find(|m| {
                matches!(
                    m.payload,
                    crate::messages::MessagePayload::BetrayalTriggered { .. }
                )
            })
            .expect("betrayal cycle must emit BetrayalTriggered message");

        let event = GameEvent::BetrayalTriggered {
            betrayer_id,
            betrayer_name,
            victim_id,
            victim_name,
        };
        assert_eq!(
            m.content,
            event.to_string(),
            "GameMessage.content must equal GameEvent::BetrayalTriggered Display"
        );
    }

    #[test]
    #[ignore = "scenario gap: needs game with populated areas + deterministic combat outcome; \
                see plan 2026-04-26-game-timeline-pr1-backend.md task 12. Asserts no \
                TributeMoved for tribute B at a later (tick, emit_index) than B's \
                TributeKilled within the same (game_day, phase)."]
    fn dead_tribute_has_no_movement_event_after_death_in_same_period() {
        use crate::messages::MessagePayload;

        let tribute_a = create_tribute("A", true);
        let tribute_b = create_tribute("B", true);
        let mut game = create_test_game_with_tributes(vec![tribute_a, tribute_b]);
        let _ = game.run_phase(crate::messages::Phase::Day);

        let b_killed = game.messages.iter().find(|m| {
            matches!(&m.payload,
                MessagePayload::TributeKilled { victim, .. } if victim.name == "B")
        });
        let b_killed = b_killed.expect("B should have died");

        let later_b_move = game.messages.iter().find(|m| {
            m.game_day == b_killed.game_day
                && m.phase == b_killed.phase
                && (m.tick, m.emit_index) > (b_killed.tick, b_killed.emit_index)
                && matches!(&m.payload,
                    MessagePayload::TributeMoved { tribute, .. } if tribute.name == "B")
        });

        assert!(
            later_b_move.is_none(),
            "no TributeMoved for B should appear after B's TributeKilled in the same period"
        );
    }

    // ---- Survival tick wiring (spec §6, §7) ------------------------------

    #[test]
    fn survival_tick_increments_hunger_and_thirst_per_phase() {
        let mut a = Tribute::new("A".to_string(), Some(1), None);
        let mut b = Tribute::new("B".to_string(), Some(2), None);
        // Mid-range attributes so the survival tick lands the +1/+1 base
        // path (not the low-strength every-other-phase or high-strength
        // double-tick branches). Stamina at half its max keeps thirst on
        // the +1 path too.
        for t in [&mut a, &mut b] {
            t.attributes.strength = 30;
            t.stamina = t.max_stamina / 2;
        }
        let mut game = create_test_game_with_tributes(vec![a, b]);
        game.day = Some(1);
        // Run a single day cycle; survival tick fires once per living tribute.
        let _ = game.run_phase(crate::messages::Phase::Day);
        for t in &game.tributes {
            assert_eq!(t.hunger, 1, "{} hunger should be 1 after one tick", t.name);
            assert_eq!(t.thirst, 1, "{} thirst should be 1 after one tick", t.name);
        }
    }

    #[test]
    fn survival_tick_routes_dehydration_death_through_tribute_killed() {
        use crate::messages::MessagePayload;
        use shared::messages::CAUSE_DEHYDRATION;
        let mut a = Tribute::new("Doomed".to_string(), Some(1), None);
        // Already at the dehydrated band with 1 HP and a high
        // dehydration drain step so the next tick definitely lands fatal
        // damage (≥ 1 HP at extreme band).
        a.thirst = 4;
        a.dehydration_drain_step = 5;
        a.attributes.health = 1;
        let mut game = create_test_game_with_tributes(vec![a]);
        game.day = Some(1);
        let _ = game.run_phase(crate::messages::Phase::Day);
        let killed = game.messages.iter().any(|m| {
            matches!(&m.payload,
                MessagePayload::TributeKilled { cause, .. } if cause == CAUSE_DEHYDRATION)
        });
        assert!(killed, "expected a TributeKilled with cause=dehydration");
    }
}
