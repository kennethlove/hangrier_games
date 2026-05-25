use crate::areas::events::AreaEvent;
use crate::areas::{Area, AreaDetails};
use crate::items::Item;
use crate::items::OwnsItems;
use crate::tributes::actions::Action;
use crate::tributes::statuses::TributeStatus;
use crate::tributes::{ActionSuggestion, Tribute};
use rand::Rng;
use rand::RngExt;
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use shared::GameStatus;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use uuid::Uuid;

/// Stamina restored per phase to a sleeping tribute (PR2c.1, bd-9sjj).
/// Placeholder pending observability tuning per spec
/// `2026-05-03-four-phase-day-design.md` §6.4.
const SLEEP_STAMINA_PER_PHASE: u32 = 25;
/// HP restored per phase to a sleeping tribute, gated on absence of
/// Wounded / Infected / Sick.
const SLEEP_HP_PER_PHASE: u32 = 5;
/// Soft cap for sleep-driven HP regen so it never exceeds the natural
/// 100-point ceiling. (Tributes' `attributes.health` is `u32` without an
/// explicit per-tribute max field.)
const SLEEP_HP_CAP: u32 = 100;

/// Project a game-side `AreaEvent` onto the cross-cutting
/// `shared::messages::AreaEventKind` taxonomy used by sleep-interruption
/// payloads (PR2c.2, bd-1zju). The mapping collapses several seasonal /
/// terrain variants into the same broad "kind" so downstream consumers
/// (UI badges, announcer prompts) can treat them uniformly.
fn area_event_to_kind(ev: &AreaEvent) -> shared::messages::AreaEventKind {
    use shared::messages::AreaEventKind as K;
    match ev {
        AreaEvent::Wildfire | AreaEvent::Sandstorm => K::Fire,
        AreaEvent::Flood | AreaEvent::Drought => K::Flood,
        AreaEvent::Earthquake
        | AreaEvent::Avalanche
        | AreaEvent::Landslide
        | AreaEvent::Rockslide => K::Earthquake,
        AreaEvent::Blizzard | AreaEvent::Heatwave => K::Storm,
    }
}

/// Generate a human-readable line for trauma-related messages.
fn format_trauma_message(payload: &crate::messages::MessagePayload, tribute_name: &str) -> String {
    use crate::messages::MessagePayload;
    match payload {
        MessagePayload::TraumaFlashback { source, .. } => {
            format!("{tribute_name} is haunted by {source}.")
        }
        MessagePayload::TraumaObserved {
            observer, subject, ..
        } => {
            format!("{observer} witnesses {subject}'s distress.")
        }
        MessagePayload::TraumaHabituated {
            from_severity,
            to_severity: Some(to),
            ..
        } => {
            format!("{tribute_name}'s trauma response weakens from {from_severity} to {to}.")
        }
        MessagePayload::TraumaHabituated {
            from_severity,
            to_severity: None,
            ..
        } => {
            format!("{tribute_name} begins to heal from {from_severity} trauma.")
        }
        _ => String::new(),
    }
}

/// Generate a human-readable line for phobia-related messages.
fn phobia_message_line(payload: &crate::messages::MessagePayload, tribute_name: &str) -> String {
    use crate::messages::MessagePayload;
    match payload {
        MessagePayload::PhobiaEscalated {
            trigger,
            from_severity,
            to_severity,
            ..
        } => {
            format!(
                "{tribute_name}'s fear of {trigger} intensifies from {from_severity} to {to_severity}."
            )
        }
        MessagePayload::PhobiaHabituated {
            trigger,
            from_severity,
            to_severity,
            ..
        } => {
            if let Some(to) = to_severity {
                format!("{tribute_name}'s fear of {trigger} fades from {from_severity} to {to}.")
            } else {
                format!("{tribute_name} has overcome their fear of {trigger}.")
            }
        }
        MessagePayload::PhobiaObserved {
            observer,
            subject,
            trigger,
            ..
        } => {
            format!("{observer} sees {subject}'s fear of {trigger}.")
        }
        MessagePayload::PhobiaForgotten {
            observer,
            subject,
            trigger,
            ..
        } => {
            format!("{observer} no longer remembers {subject}'s fear of {trigger}.")
        }
        _ => String::new(),
    }
}

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
/// deriving across the entire payload graph). Cache consumers requiring
/// `PartialEq` can rely on identity equality, which is sufficient for cache
/// dedup since a game is uniquely keyed by its identifier.
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

    /// NPC sponsors that observe events and build per-tribute affinity.
    /// Lazily spawned on first cycle for backward-compat with pre-sponsorship games.
    #[serde(default)]
    pub sponsors: Vec<shared::sponsors::Sponsor>,
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
            sponsors: vec![],
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
    /// Full four-phase value (Dawn / Day / Dusk / Night) for this cycle.
    /// Threaded through to `EnvironmentContext` and the sleep tick handler
    /// so brain scoring and `TributeSlept` / `TributeWoke` payloads see the
    /// real phase rather than reconstructing it from `is_day`.
    phase: crate::messages::Phase,
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

/// Calculate initiative score for a tribute: agility + random fuzz (0-20).
/// Higher score = acts earlier in the phase (tm6a).
fn initiative_score(agility: u32, rng: &mut impl Rng) -> u32 {
    agility + rng.random_range(0..=20)
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
    pub(crate) fn push_message(
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
            crate::messages::MessageSource::Game(game_id.clone()),
            subject.clone(),
            content.clone(),
            payload,
            tick,
        );

        // PhaseStarted: four-phase day substrate (spec §4 step 4).
        // Reuse the same message — no duplicate content string.
        let phase_payload = crate::messages::MessagePayload::PhaseStarted {
            day: current_day,
            phase,
            weather_summary: None,
        };
        self.push_message(
            crate::messages::MessageSource::Game(game_id),
            subject,
            String::new(),
            phase_payload,
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

        // PhaseEnded: four-phase day substrate (spec §4 step 4).
        // Reuse the same message — no duplicate content string.
        let phase_payload = crate::messages::MessagePayload::PhaseEnded {
            day: current_day,
            phase,
        };
        self.push_message(
            crate::messages::MessageSource::Game(game_id.clone()),
            format!("game:{}", game_id),
            String::new(),
            phase_payload,
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
        self.run_trauma_producers(phase);
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

    /// Run the trauma producer pipeline for the given phase.
    /// Scans the current phase's messages and acquires/reinforces trauma
    /// afflictions on living tributes who witnessed or survived traumatic events.
    fn run_trauma_producers(&mut self, _phase: crate::messages::Phase) {
        crate::tributes::afflictions::trauma_producers::run_trauma_producers(self);
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
                                crate::messages::MessageSource::Tribute(t_id.clone()),
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

    /// Spawn one sponsor per archetype using the shared catalog.
    /// Loyalist gets a randomly-assigned district (1..=12). Budget is rolled
    /// inside the archetype's budget band. Idempotent: no-op if `self.sponsors`
    /// is already populated.
    pub fn spawn_sponsors(&mut self, rng: &mut impl Rng) {
        use shared::sponsors::{ARCHETYPES, ArchetypeId, Sponsor};
        use std::collections::HashMap;

        if !self.sponsors.is_empty() {
            return;
        }

        for (idx, archetype) in ARCHETYPES.iter().enumerate() {
            let (lo, hi) = archetype.budget_band;
            let budget = rng.random_range(lo..=hi);
            let bound_district = if archetype.id == ArchetypeId::Loyalist {
                Some(rng.random_range(1u8..=12))
            } else {
                None
            };

            self.sponsors.push(Sponsor {
                id: idx as u32,
                archetype: archetype.id,
                budget_remaining: budget,
                bound_district,
                affinity: HashMap::new(),
            });
        }
    }

    /// Test helper: returns `(canonical_name, tribute_identifier, affinity)` triples.
    pub fn sponsor_affinity_snapshot(&self) -> Vec<(&'static str, String, i32)> {
        let mut out = Vec::new();
        for s in &self.sponsors {
            let mut entries: Vec<_> = s.affinity.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (tribute, value) in entries {
                out.push((s.canonical_name(), tribute.clone(), *value));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;

mod cycle;
