use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Cause string used in `MessagePayload::TributeKilled` for starvation deaths.
pub const CAUSE_STARVATION: &str = "starvation";
/// Cause string used in `MessagePayload::TributeKilled` for dehydration deaths.
pub const CAUSE_DEHYDRATION: &str = "dehydration";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum MessageSource {
    #[serde(rename = "Game")]
    Game(String), // Game identifier
    #[serde(rename = "Area")]
    Area(String), // Area name
    #[serde(rename = "Tribute")]
    Tribute(String), // Tribute identifier
}

/// One of the four narrative beats within a game-day. Ordinal order
/// (`Dawn = 0, Day = 1, Dusk = 2, Night = 3`) drives all chronological
/// sorting of `GameMessage`s; the `Day` and `Night` variants keep their
/// pre-existing serialized forms (`"day"` / `"night"`) so persisted games
/// from the two-phase era continue to deserialize.
///
/// See `docs/superpowers/specs/2026-05-03-four-phase-day-design.md` (§§2-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl Phase {
    /// Numeric ordinal used to sort messages within a game-day. Stable
    /// across the wire format because `summarize_periods` relies on it.
    pub const fn ord(self) -> u8 {
        match self {
            Phase::Dawn => 0,
            Phase::Day => 1,
            Phase::Dusk => 2,
            Phase::Night => 3,
        }
    }

    /// Next phase in the canonical `Dawn → Day → Dusk → Night → Dawn`
    /// cycle. Day-boundary handling (incrementing `current_day` after
    /// `Night`) lives in the engine driver, not here.
    pub const fn next(self) -> Phase {
        match self {
            Phase::Dawn => Phase::Day,
            Phase::Day => Phase::Dusk,
            Phase::Dusk => Phase::Night,
            Phase::Night => Phase::Dawn,
        }
    }

    /// All four phases in canonical order. Useful for iterating a full day.
    pub const fn all() -> [Phase; 4] {
        [Phase::Dawn, Phase::Day, Phase::Dusk, Phase::Night]
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Dawn => write!(f, "dawn"),
            Phase::Day => write!(f, "day"),
            Phase::Dusk => write!(f, "dusk"),
            Phase::Night => write!(f, "night"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePhaseError;

impl std::fmt::Display for ParsePhaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "phase must be 'dawn', 'day', 'dusk', or 'night'")
    }
}

impl std::error::Error for ParsePhaseError {}

impl FromStr for Phase {
    type Err = ParsePhaseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dawn" => Ok(Phase::Dawn),
            "day" => Ok(Phase::Day),
            "dusk" => Ok(Phase::Dusk),
            "night" => Ok(Phase::Night),
            _ => Err(ParsePhaseError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TributeRef {
    pub identifier: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AreaRef {
    pub identifier: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemRef {
    pub identifier: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaEventKind {
    Hazard,
    Storm,
    Mutts,
    Earthquake,
    Flood,
    Fire,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEngagement {
    pub attacker: TributeRef,
    pub target: TributeRef,
    pub outcome: CombatOutcome,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatOutcome {
    Killed,
    Wounded,
    TargetFled,
    AttackerFled,
    Stalemate,
}

/// Source of a `Drank` event: either a terrain water source or a Water item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DrinkSource {
    Terrain { area: AreaRef },
    Item { item: ItemRef },
}

/// Coarse-grained category for a `GameMessage`. Derived from `MessagePayload`
/// via `MessagePayload::kind()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    Death,
    Combat,
    /// One swing of physical combat (see `CombatBeat`).
    CombatSwing,
    Alliance,
    Movement,
    Item,
    State,
    /// Sponsor gift delivery.
    SponsorGift,
    /// Trauma acquisition or reinforcement.
    Trauma,
    /// Phobia acquired, triggered, observed, escalated, habituated, or forgotten.
    Phobia,
    /// Affliction acquired, progressed, healed, or cascaded.
    /// Fixation acquired, escalated, fired, consummated, thwarted, or faded.
    Fixation,
    /// Trapped affliction events: tribute trapped, struggling, escaped, or died.
    Trapped,
    Affliction,
}

/// Visible fatigue band derived from a tribute's stamina/max_stamina ratio.
/// Lives in `shared/` because it is wire-visible via
/// `MessagePayload::StaminaBandChanged`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StaminaBand {
    Fresh,
    Winded,
    Exhausted,
}

/// Visible hunger band derived from a tribute's hunger counter. Lives in
/// `shared/` because it is wire-visible via
/// `MessagePayload::HungerBandChanged`. The mapping (counter → band) lives
/// in `game::tributes::survival::hunger_band`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HungerBand {
    Sated,
    Peckish,
    Hungry,
    Starving,
}

/// Visible thirst band derived from a tribute's thirst counter. Lives in
/// `shared/` because it is wire-visible via
/// `MessagePayload::ThirstBandChanged`. The mapping (counter → band) lives
/// in `game::tributes::survival::thirst_band`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThirstBand {
    Sated,
    Thirsty,
    Parched,
    Dehydrated,
}

impl StaminaBand {
    pub fn as_str(self) -> &'static str {
        match self {
            StaminaBand::Fresh => "Fresh",
            StaminaBand::Winded => "Winded",
            StaminaBand::Exhausted => "Exhausted",
        }
    }
}

impl std::fmt::Display for StaminaBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl HungerBand {
    pub fn as_str(self) -> &'static str {
        match self {
            HungerBand::Sated => "Sated",
            HungerBand::Peckish => "Peckish",
            HungerBand::Hungry => "Hungry",
            HungerBand::Starving => "Starving",
        }
    }
}

impl std::fmt::Display for HungerBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ThirstBand {
    pub fn as_str(self) -> &'static str {
        match self {
            ThirstBand::Sated => "Sated",
            ThirstBand::Thirsty => "Thirsty",
            ThirstBand::Parched => "Parched",
            ThirstBand::Dehydrated => "Dehydrated",
        }
    }
}

impl std::fmt::Display for ThirstBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a sleeping tribute woke. Pairs with `MessagePayload::TributeWoke`.
/// See spec `2026-05-03-four-phase-day-design.md` §6.4 / §8.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "reason")]
pub enum WakeReason {
    /// The tribute slept the planned duration without interruption.
    Rested,
    /// Sleep was cut short. `event` describes the interrupting cause.
    Interrupted { event: InterruptionKind },
}

/// Concrete cause of a sleep interruption (`WakeReason::Interrupted`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "interruption")]
pub enum InterruptionKind {
    /// Another tribute attacked the sleeper.
    Ambush { attacker: TributeRef },
    /// An area event (storm, mutts, etc.) hit the sleeper's area.
    AreaEvent { kind: AreaEventKind },
    /// An ally summoned the tribute (alliance event cascade).
    AllianceSummons { ally: TributeRef },
    /// A sleep incident (theft, relocation, animal, etc.) woke the tribute.
    Incident { kind: SleepIncidentKind },
}

/// Category of sleep incident that can occur while a tribute is unconscious.
/// Carried in `InterruptionKind::Incident` for typed wire format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "incident", rename_all = "snake_case")]
pub enum SleepIncidentKind {
    /// Annoying but harmless (squirrel on chest, weird dream). No mechanical effect.
    Annoying,
    /// A random item was stolen from the sleeper.
    Theft,
    /// The sleeper was relocated to a different area while unconscious.
    Relocation,
    /// An animal (named) disturbed the sleep.
    AnimalEncounter { animal: String },
    /// Hallucination or bad dream — sanity damage.
    Hallucination,
    /// An ally abandoned the sleeper during the night.
    AllyAbandonment,
    /// Comedic limb issue (leg fell asleep, etc.) — temporary affliction.
    LimbInjury,
}

/// Effect category for a `PhobiaTriggered` event. Mirrors the game-layer
/// `PhobiaEffect` enum so the wire format is self-contained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhobiaEffect {
    Penalty,
    Flee,
    Freeze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    /// Generic narrative event — prose-only, no structured payload consumers.
    Generic,
    TributeKilled {
        victim: TributeRef,
        killer: Option<TributeRef>,
        cause: String,
    },
    TributeWounded {
        victim: TributeRef,
        attacker: Option<TributeRef>,
        hp_lost: u32,
    },
    TributeAttacked {
        victim: TributeRef,
        attacker: Option<TributeRef>,
    },

    Combat(CombatEngagement),
    /// One physical-combat swing in fully typed form (see `CombatBeat`).
    /// Emitted alongside the existing `Combat`/`TributeKilled`/`TributeWounded`
    /// payloads so consumers can render structured swing data without parsing
    /// `detail_lines` strings.
    CombatSwing(crate::combat_beat::CombatBeat),

    AllianceFormed {
        members: Vec<TributeRef>,
    },
    AllianceProposed {
        proposer: TributeRef,
        target: TributeRef,
    },
    AllianceDissolved {
        members: Vec<TributeRef>,
        reason: String,
    },
    BetrayalTriggered {
        betrayer: TributeRef,
        victim: TributeRef,
    },
    TrustShockBreak {
        tribute: TributeRef,
        partner: TributeRef,
    },

    TributeMoved {
        tribute: TributeRef,
        from: AreaRef,
        to: AreaRef,
    },
    TributeHidden {
        tribute: TributeRef,
        area: AreaRef,
    },
    AreaClosed {
        area: AreaRef,
    },
    AreaEvent {
        area: AreaRef,
        kind: AreaEventKind,
        description: String,
    },

    ItemFound {
        tribute: TributeRef,
        item: ItemRef,
        area: AreaRef,
    },
    ItemUsed {
        tribute: TributeRef,
        item: ItemRef,
    },
    ItemDropped {
        tribute: TributeRef,
        item: ItemRef,
        area: AreaRef,
    },
    SponsorGift {
        recipient: TributeRef,
        item: ItemRef,
        donor: String,
    },

    TributeRested {
        tribute: TributeRef,
        hp_restored: u32,
    },
    TributeStarved {
        tribute: TributeRef,
        hp_lost: u32,
    },
    TributeDehydrated {
        tribute: TributeRef,
        hp_lost: u32,
    },
    SanityBreak {
        tribute: TributeRef,
    },

    // Survival events (shelter + hunger/thirst spec).
    HungerBandChanged {
        tribute: TributeRef,
        from: HungerBand,
        to: HungerBand,
    },
    ThirstBandChanged {
        tribute: TributeRef,
        from: ThirstBand,
        to: ThirstBand,
    },
    StaminaBandChanged {
        tribute: TributeRef,
        from: StaminaBand,
        to: StaminaBand,
    },
    ShelterSought {
        tribute: TributeRef,
        area: AreaRef,
        success: bool,
        roll: u8,
    },
    Foraged {
        tribute: TributeRef,
        area: AreaRef,
        success: bool,
        debt_recovered: u8,
    },
    Drank {
        tribute: TributeRef,
        source: DrinkSource,
        debt_recovered: u8,
    },
    Ate {
        tribute: TributeRef,
        item: ItemRef,
        debt_recovered: u8,
    },

    // Lifecycle / cycle-boundary announcements (formerly an AreaEvent fallback
    // synthesised by Game::log() for MessageSource::Game; see
    // hangrier_games-xamw).
    /// Emitted at the very start of a day or night phase.
    CycleStart {
        day: u32,
        phase: Phase,
    },
    /// Emitted at the very end of a day or night phase.
    CycleEnd {
        day: u32,
        phase: Phase,
    },
    /// Emitted at the start of each phase (Dawn/Day/Dusk/Night).
    /// Replaces the legacy `GameDayStart`/`GameNightStart` events.
    PhaseStarted {
        day: u32,
        phase: Phase,
        weather_summary: Option<String>,
    },
    /// Emitted at the end of each phase.
    /// Replaces the legacy `GameDayEnd`/`GameNightEnd` events.
    PhaseEnded {
        day: u32,
        phase: Phase,
    },
    /// Emitted when a tribute begins sleeping (resolution of `Action::Sleep`).
    /// `restored_*` fields are zero on the first phase of a multi-phase sleep
    /// and accumulate per phase as the engine ticks the sleeper. See spec
    /// `2026-05-03-four-phase-day-design.md` §6.4 / §8.
    TributeSlept {
        tribute: TributeRef,
        phase: Phase,
        restored_stamina: u32,
        restored_hp: u32,
    },
    /// Emitted when a sleeping tribute wakes — either because the planned
    /// sleep duration elapsed (`WakeReason::Rested`) or because something
    /// interrupted them (`WakeReason::Interrupted`).
    TributeWoke {
        tribute: TributeRef,
        phase: Phase,
        reason: WakeReason,
    },
    /// Emitted when the game ends. `winner` is `Some` for the lone-survivor
    /// case and `None` for "no survivors".
    GameEnded {
        winner: Option<TributeRef>,
    },

    // Affliction events (health conditions PR2).
    AfflictionAcquired {
        tribute_id: String,
        affliction: String,
        severity: String,
    },
    AfflictionProgressed {
        tribute_id: String,
        affliction: String,
        from_severity: String,
        to_severity: String,
    },
    AfflictionHealed {
        tribute_id: String,
        affliction: String,
    },
    AfflictionCascaded {
        tribute_id: String,
        from_affliction: String,
        to_affliction: String,
    },

    // Trauma events (trauma producer pipeline PR2).
    TraumaAcquired {
        tribute: String,
        severity: String,
        source: String,
    },
    TraumaReinforced {
        tribute: String,
        from_severity: String,
        to_severity: String,
        floor_bumped: bool,
    },

    // Phobia events (phobia brain layer PR2).
    PhobiaAcquired {
        tribute: String,
        trigger: String,
        severity: String,
        origin: String,
    },
    PhobiaTriggered {
        tribute: String,
        trigger: String,
        severity: String,
        effect: PhobiaEffect,
    },
    /// Phobia escalation (Traumatic origin only).
    PhobiaEscalated {
        tribute: String,
        trigger: String,
        from_severity: String,
        to_severity: String,
    },
    /// Phobia habituation (Traumatic origin, severity decayed or cured).
    PhobiaHabituated {
        tribute: String,
        trigger: String,
        from_severity: String,
        to_severity: Option<String>,
    },
    /// A tribute observed someone else's phobia firing.
    PhobiaObserved {
        observer: String,
        subject: String,
        trigger: String,
    },
    /// A tribute forgot someone else's phobia (observer decay).
    PhobiaForgotten {
        observer: String,
        subject: String,
        trigger: String,
    },
    // Trauma escalation/effects (trauma PR3 brain layer).
    /// Trauma severity escalated (producer reinforcement roll).
    TraumaEscalated {
        tribute: String,
        from_severity: String,
        to_severity: String,
    },
    /// Tribute experienced a trauma flashback.
    TraumaFlashback {
        tribute: String,
        severity: String,
        source: String,
    },
    /// Tribute avoided an action due to trauma avoidance.
    TraumaAvoidance {
        tribute: String,
        source: String,
        prevented_action: String,
    },
    /// A tribute observed someone else's trauma firing.
    TraumaObserved {
        observer: String,
        subject: String,
        source: String,
    },
    /// A tribute forgot someone else's trauma (observer decay).
    TraumaForgotten {
        observer: String,
        subject: String,
        source: String,
    },
    /// Trauma severity decayed or cured (habituation).
    TraumaHabituated {
        tribute: String,
        from_severity: String,
        to_severity: Option<String>,
    },

    // Fixation events (fixation brain layer PR2).
    /// A tribute acquired a fixation on a target.
    FixationAcquired {
        tribute_id: String,
        target: String,
        severity: String,
        origin: String,
    },
    /// A fixation's severity escalated.
    FixationEscalated {
        tribute_id: String,
        target: String,
        old_severity: String,
        new_severity: String,
    },
    /// A fixation fired — the brain is overriding toward the target.
    FixationFired {
        tribute_id: String,
        target: String,
        severity: String,
        action: String,
    },
    /// A fixation was consummated (target reached/acquired).
    FixationConsummated {
        tribute_id: String,
        target: String,
    },
    /// A fixation was thwarted (target lost/unreachable).
    FixationThwarted {
        tribute_id: String,
        target: String,
        reason: String,
    },
    /// A fixation faded (severity decayed to nothing).
    FixationFaded {
        tribute_id: String,
        target: String,
    },
    // Addiction events (addiction PR2).
    /// Tribute used a substance (addictive consumable).
    SubstanceUsed {
        tribute: String,
        item: String,
        substance: String,
    },
    /// Tribute acquired a new addiction.
    AddictionAcquired {
        tribute: String,
        substance: String,
        severity: String,
        use_count: u32,
    },
    /// Existing addiction reinforced (used while already addicted).
    AddictionReinforced {
        tribute: String,
        substance: String,
        severity: String,
    },
    /// Addiction severity escalated (12% sensitization roll).
    AddictionEscalated {
        tribute: String,
        substance: String,
        from_severity: String,
        to_severity: String,
    },
    /// Addiction acquisition prevented (at cap, or roll failed).
    AddictionResisted {
        tribute: String,
        substance: String,
        reason: String,
    },
    /// Relapse — cured tribute auto-reacquired on first use.
    AddictionRelapse {
        tribute: String,
        substance: String,
        prior_uses: u32,
    },
    /// Tribute is craving a substance (visible to observers).
    AddictionCraving {
        tribute: String,
        substance: String,
        severity: String,
    },
    /// A tribute observed someone's addiction behavior.
    AddictionObserved {
        observer: String,
        subject: String,
        substance: String,
    },
    /// A tribute forgot someone's addiction (observer decay).
    AddictionForgotten {
        observer: String,
        subject: String,
        substance: String,
    },
    /// Addiction severity decayed or cured (habituation).
    AddictionHabituated {
        tribute: String,
        substance: String,
        from_severity: String,
        to_severity: Option<String>,
    },
    /// Tribute became trapped by a hazard (drowning, buried, etc.).
    TributeTrapped {
        tribute: String,
        kind: crate::afflictions::TrapKind,
        severity: crate::afflictions::Severity,
    },
    /// Tribute is still trapped — ongoing struggle with cumulative effect.
    Struggling {
        tribute: String,
        kind: crate::afflictions::TrapKind,
        severity: crate::afflictions::Severity,
        cycles_trapped: u8,
    },
    /// Tribute escaped the trap (may have been rescued).
    TrappedEscaped {
        tribute: String,
        kind: crate::afflictions::TrapKind,
        cycles_trapped: u8,
        rescued_by: Vec<String>,
    },
    /// Tribute died while trapped — could not escape in time.
    TributeDiedWhileTrapped {
        tribute: String,
        kind: crate::afflictions::TrapKind,
    },
    /// A tribute set a trap.
    TrapSet {
        tribute: TributeRef,
        trap_kind: String,
    },
    /// A tribute triggered a trap.
    TrapTriggered {
        victim: TributeRef,
        trap_kind: String,
    },
    /// Rescuer attempted to free a trapped tribute this cycle.
    /// Emitted every cycle a tribute performs Action::Rescue.
    RescueAttempted {
        rescuer: String,
        target: String,
        kind: crate::afflictions::TrapKind,
        severity: crate::afflictions::Severity,
        bonus: f32,
    },
    /// Partial rescue progress accumulated — rescue bonus increased.
    /// Emitted when a rescue attempt contributes toward the escape threshold,
    /// but the target is not yet freed.
    PartialRescueProgress {
        rescuer: String,
        target: String,
        kind: crate::afflictions::TrapKind,
        severity: crate::afflictions::Severity,
        bonus: f32,
        progress: u8,
        /// How many rescue cycles are needed before the rescue bonus applies.
        /// Only meaningful at Severe; always `PARTIAL_RESCUE_THRESHOLD` (2).
        threshold: u8,
    },
}

pub mod impls;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub phase: Phase,
    pub tick: u32,
    pub emit_index: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub payload: MessagePayload,
}

impl PartialEq for GameMessage {
    /// Identity equality via `identifier`. `MessagePayload` is not `PartialEq`
    /// (would require deriving across the entire payload graph); identity
    /// equality is sufficient for cache dedup since each persisted message
    /// has a unique identifier.
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl GameMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: MessageSource,
        game_day: u32,
        phase: Phase,
        tick: u32,
        emit_index: u32,
        subject: String,
        content: String,
        payload: MessagePayload,
    ) -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            source,
            game_day,
            phase,
            tick,
            emit_index,
            subject,
            timestamp: Utc::now(),
            content,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodSummary {
    pub day: u32,
    pub phase: Phase,
    pub deaths: u32,
    pub event_count: u32,
    pub is_current: bool,
}

/// Newtype wrapper around the period list returned by `summarize_periods`.
/// Exists so the API surface stays a stable named type as the timeline payload
/// grows (e.g. recap, totals) without breaking clients.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineSummary {
    pub periods: Vec<PeriodSummary>,
}

/// Aggregate messages into one summary per (day, phase). Includes empty periods
/// up to and including `current` so the hub shows the live period even when
/// nothing has been emitted there yet. Periods past `current` are not emitted.
///
/// Day 1 is special-cased per the four-phase day spec (§3): tributes rise on
/// the pedestals at `Day` so `Dawn1` is never emitted by the engine and is
/// excluded from the back-fill. Subsequent days walk all four phases.
pub fn summarize_periods(messages: &[GameMessage], current: (u32, Phase)) -> Vec<PeriodSummary> {
    use std::collections::BTreeMap;

    let (current_day, current_phase) = current;
    let mut bucket: BTreeMap<(u32, u8), (u32, u32)> = BTreeMap::new();

    for m in messages {
        let key = (m.game_day, m.phase.ord());
        let entry = bucket.entry(key).or_insert((0, 0));
        if !matches!(
            m.payload,
            MessagePayload::PhaseStarted { .. } | MessagePayload::PhaseEnded { .. }
        ) {
            entry.1 += 1;
        }
        if matches!(m.payload, MessagePayload::TributeKilled { .. })
            || matches!(
                &m.payload,
                MessagePayload::Combat(engagement)
                    if engagement.outcome == CombatOutcome::Killed
            )
        {
            entry.0 += 1;
        }
    }

    // Always seed the current period so the hub shows it even before any
    // events are emitted (e.g. day 0 of a NotStarted game). Then back-fill
    // every prior (day, phase) pair starting at day 1 so the summary list
    // is dense up to the live period without gaps for empty cycles.
    //
    // Day 0 only ever has a `Day` phase (NotStarted seed). Day 1 skips
    // `Dawn`. Day 2+ runs all four phases.
    bucket
        .entry((current_day, current_phase.ord()))
        .or_insert((0, 0));
    for d in 1..=current_day {
        let max_ord = if d < current_day {
            Phase::Night.ord()
        } else {
            current_phase.ord()
        };
        for phase in Phase::all() {
            // Skip Dawn1 — never emitted by the engine.
            if d == 1 && phase == Phase::Dawn {
                continue;
            }
            if phase.ord() <= max_ord {
                bucket.entry((d, phase.ord())).or_insert((0, 0));
            }
        }
    }

    bucket
        .into_iter()
        .map(|((day, p), (deaths, count))| {
            let phase = match p {
                0 => Phase::Dawn,
                1 => Phase::Day,
                2 => Phase::Dusk,
                _ => Phase::Night,
            };
            PeriodSummary {
                day,
                phase,
                deaths,
                event_count: count,
                is_current: day == current_day && phase == current_phase,
            }
        })
        .collect()
}

#[cfg(test)]
mod survival_event_tests;
#[cfg(test)]
mod tests;
