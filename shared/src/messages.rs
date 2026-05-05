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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
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
    /// Emitted when the game ends. `winner` is `Some` for the lone-survivor
    /// case and `None` for "no survivors".
    GameEnded {
        winner: Option<TributeRef>,
    },
}

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        use MessagePayload::*;
        match self {
            TributeKilled { .. } => MessageKind::Death,
            Combat(_) => MessageKind::Combat,
            CombatSwing(_) => MessageKind::CombatSwing,
            AllianceFormed { .. }
            | AllianceProposed { .. }
            | AllianceDissolved { .. }
            | BetrayalTriggered { .. }
            | TrustShockBreak { .. } => MessageKind::Alliance,
            TributeMoved { .. } | TributeHidden { .. } | AreaClosed { .. } | AreaEvent { .. } => {
                MessageKind::Movement
            }
            ItemFound { .. } | ItemUsed { .. } | ItemDropped { .. } | SponsorGift { .. } => {
                MessageKind::Item
            }
            CycleStart { .. } | CycleEnd { .. } | GameEnded { .. } => MessageKind::State,
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
            | Ate { .. } => MessageKind::State,
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
            } => r(victim) || attacker.as_ref().is_some_and(r),
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
            | Ate { tribute, .. } => r(tribute),
            SponsorGift { recipient, .. } => r(recipient),
            AreaClosed { .. } | AreaEvent { .. } => false,
            CycleStart { .. } | CycleEnd { .. } => false,
            GameEnded { winner } => winner.as_ref().is_some_and(r),
        }
    }
}

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
        entry.1 += 1;
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
mod tests {
    use super::*;

    fn t(name: &str) -> TributeRef {
        TributeRef {
            identifier: format!("id-{name}"),
            name: name.into(),
        }
    }

    #[test]
    fn phase_display_roundtrip() {
        for p in Phase::all() {
            let s = p.to_string();
            assert_eq!(s.parse::<Phase>().unwrap(), p);
        }
        assert_eq!(Phase::Dawn.to_string(), "dawn");
        assert_eq!(Phase::Day.to_string(), "day");
        assert_eq!(Phase::Dusk.to_string(), "dusk");
        assert_eq!(Phase::Night.to_string(), "night");
        assert!("noon".parse::<Phase>().is_err());
    }

    #[test]
    fn phase_serde_lowercase() {
        assert_eq!(serde_json::to_string(&Phase::Dawn).unwrap(), "\"dawn\"");
        assert_eq!(serde_json::to_string(&Phase::Day).unwrap(), "\"day\"");
        assert_eq!(serde_json::to_string(&Phase::Dusk).unwrap(), "\"dusk\"");
        assert_eq!(serde_json::to_string(&Phase::Night).unwrap(), "\"night\"");
        let p: Phase = serde_json::from_str("\"night\"").unwrap();
        assert_eq!(p, Phase::Night);
    }

    #[test]
    fn phase_ord_and_next_canonical_cycle() {
        assert_eq!(Phase::Dawn.ord(), 0);
        assert_eq!(Phase::Day.ord(), 1);
        assert_eq!(Phase::Dusk.ord(), 2);
        assert_eq!(Phase::Night.ord(), 3);
        // Canonical cycle wraps Night -> Dawn so the engine can advance the
        // game-day at the boundary without special-casing the wire format.
        assert_eq!(Phase::Dawn.next(), Phase::Day);
        assert_eq!(Phase::Day.next(), Phase::Dusk);
        assert_eq!(Phase::Dusk.next(), Phase::Night);
        assert_eq!(Phase::Night.next(), Phase::Dawn);
        assert_eq!(
            Phase::all(),
            [Phase::Dawn, Phase::Day, Phase::Dusk, Phase::Night]
        );
    }

    #[test]
    fn message_kind_serde_roundtrip() {
        for kind in [
            MessageKind::Death,
            MessageKind::Combat,
            MessageKind::Alliance,
            MessageKind::Movement,
            MessageKind::Item,
            MessageKind::State,
            MessageKind::CombatSwing,
        ] {
            let s = serde_json::to_string(&kind).unwrap();
            let back: MessageKind = serde_json::from_str(&s).unwrap();
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn kind_lifecycle_variants_map_correctly() {
        let p = MessagePayload::TributeKilled {
            victim: t("v"),
            killer: None,
            cause: "fall".into(),
        };
        assert_eq!(p.kind(), MessageKind::Death);

        let p = MessagePayload::TributeWounded {
            victim: t("v"),
            attacker: None,
            hp_lost: 5,
        };
        assert_eq!(p.kind(), MessageKind::State);
    }

    #[test]
    fn kind_combat_maps_to_combat() {
        let p = MessagePayload::Combat(CombatEngagement {
            attacker: t("a"),
            target: t("b"),
            outcome: CombatOutcome::Killed,
            detail_lines: vec![],
        });
        assert_eq!(p.kind(), MessageKind::Combat);
    }

    #[test]
    fn kind_alliance_variants_map_correctly() {
        for p in [
            MessagePayload::AllianceFormed {
                members: vec![t("a"), t("b")],
            },
            MessagePayload::AllianceProposed {
                proposer: t("a"),
                target: t("b"),
            },
            MessagePayload::AllianceDissolved {
                members: vec![t("a")],
                reason: "x".into(),
            },
            MessagePayload::BetrayalTriggered {
                betrayer: t("a"),
                victim: t("b"),
            },
            MessagePayload::TrustShockBreak {
                tribute: t("a"),
                partner: t("b"),
            },
        ] {
            assert_eq!(p.kind(), MessageKind::Alliance);
        }
    }

    #[test]
    fn kind_movement_variants_map_correctly() {
        let area = AreaRef {
            identifier: "a1".into(),
            name: "A".into(),
        };
        for p in [
            MessagePayload::TributeMoved {
                tribute: t("a"),
                from: area.clone(),
                to: area.clone(),
            },
            MessagePayload::TributeHidden {
                tribute: t("a"),
                area: area.clone(),
            },
            MessagePayload::AreaClosed { area: area.clone() },
            MessagePayload::AreaEvent {
                area: area.clone(),
                kind: AreaEventKind::Storm,
                description: "x".into(),
            },
        ] {
            assert_eq!(p.kind(), MessageKind::Movement);
        }
    }

    #[test]
    fn kind_item_variants_map_correctly() {
        let area = AreaRef {
            identifier: "a1".into(),
            name: "A".into(),
        };
        let item = ItemRef {
            identifier: "i1".into(),
            name: "I".into(),
        };
        for p in [
            MessagePayload::ItemFound {
                tribute: t("a"),
                item: item.clone(),
                area: area.clone(),
            },
            MessagePayload::ItemUsed {
                tribute: t("a"),
                item: item.clone(),
            },
            MessagePayload::ItemDropped {
                tribute: t("a"),
                item: item.clone(),
                area: area.clone(),
            },
            MessagePayload::SponsorGift {
                recipient: t("a"),
                item: item.clone(),
                donor: "Capitol".into(),
            },
        ] {
            assert_eq!(p.kind(), MessageKind::Item);
        }
    }

    #[test]
    fn kind_state_variants_map_correctly() {
        for p in [
            MessagePayload::TributeRested {
                tribute: t("a"),
                hp_restored: 3,
            },
            MessagePayload::TributeStarved {
                tribute: t("a"),
                hp_lost: 1,
            },
            MessagePayload::TributeDehydrated {
                tribute: t("a"),
                hp_lost: 2,
            },
            MessagePayload::SanityBreak { tribute: t("a") },
        ] {
            assert_eq!(p.kind(), MessageKind::State);
        }
    }

    #[test]
    fn unknown_payload_tag_hard_errors() {
        let raw = serde_json::json!({ "type": "DefinitelyNotAVariant" });
        let result: Result<MessagePayload, _> = serde_json::from_value(raw);
        assert!(result.is_err());
    }

    #[test]
    fn game_message_new_populates_required_fields() {
        let msg = GameMessage::new(
            MessageSource::Game("g".into()),
            2,
            Phase::Night,
            3,
            0,
            "subj".into(),
            "content".into(),
            MessagePayload::SanityBreak { tribute: t("a") },
        );
        assert_eq!(msg.game_day, 2);
        assert_eq!(msg.phase, Phase::Night);
        assert_eq!(msg.tick, 3);
        assert_eq!(msg.emit_index, 0);
        assert_eq!(msg.payload.kind(), MessageKind::State);
    }

    fn make_msg(day: u32, phase: Phase, payload: MessagePayload) -> GameMessage {
        GameMessage::new(
            MessageSource::Game("g".into()),
            day,
            phase,
            1,
            0,
            "subject".into(),
            "content".into(),
            payload,
        )
    }

    #[test]
    fn summarize_empty_input_with_current_day_zero() {
        let result = summarize_periods(&[], (0, Phase::Day));
        assert_eq!(
            result.len(),
            1,
            "current period (day 0, Day) should always be seeded"
        );
        assert_eq!(result[0].day, 0);
        assert_eq!(result[0].phase, Phase::Day);
        assert!(result[0].is_current);
        assert_eq!(result[0].event_count, 0);
        assert_eq!(result[0].deaths, 0);
    }

    #[test]
    fn summarize_groups_by_day_and_phase() {
        let tref = TributeRef {
            identifier: "t".into(),
            name: "T".into(),
        };
        let killed = MessagePayload::TributeKilled {
            victim: tref.clone(),
            killer: None,
            cause: "x".into(),
        };
        let moved = MessagePayload::TributeHidden {
            tribute: tref.clone(),
            area: AreaRef {
                identifier: "a".into(),
                name: "A".into(),
            },
        };

        let msgs = vec![
            make_msg(1, Phase::Day, killed.clone()),
            make_msg(1, Phase::Day, moved.clone()),
            make_msg(1, Phase::Night, moved.clone()),
            make_msg(2, Phase::Day, killed.clone()),
        ];
        let result = summarize_periods(&msgs, (2, Phase::Day));
        // Day 1: Day/Dusk/Night (Dawn1 skipped per spec §3) + Day 2: Dawn/Day.
        assert_eq!(result.len(), 5);
        assert_eq!(
            result[0],
            PeriodSummary {
                day: 1,
                phase: Phase::Day,
                deaths: 1,
                event_count: 2,
                is_current: false
            }
        );
        assert_eq!(
            result[1],
            PeriodSummary {
                day: 1,
                phase: Phase::Dusk,
                deaths: 0,
                event_count: 0,
                is_current: false
            }
        );
        assert_eq!(
            result[2],
            PeriodSummary {
                day: 1,
                phase: Phase::Night,
                deaths: 0,
                event_count: 1,
                is_current: false
            }
        );
        assert_eq!(
            result[3],
            PeriodSummary {
                day: 2,
                phase: Phase::Dawn,
                deaths: 0,
                event_count: 0,
                is_current: false
            }
        );
        assert_eq!(
            result[4],
            PeriodSummary {
                day: 2,
                phase: Phase::Day,
                deaths: 1,
                event_count: 1,
                is_current: true
            }
        );
    }

    #[test]
    fn summarize_includes_empty_reached_periods() {
        let result = summarize_periods(&[], (2, Phase::Day));
        // Day 1: Day/Dusk/Night (Dawn1 skipped) + Day 2: Dawn/Day.
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].day, 1);
        assert_eq!(result[0].phase, Phase::Day);
        assert_eq!(result[1].phase, Phase::Dusk);
        assert_eq!(result[2].phase, Phase::Night);
        assert_eq!(result[3].day, 2);
        assert_eq!(result[3].phase, Phase::Dawn);
        assert_eq!(result[4].phase, Phase::Day);
        assert!(result[4].is_current);
    }

    #[test]
    fn summarize_counts_combat_kills_as_deaths() {
        let combat_kill = MessagePayload::Combat(CombatEngagement {
            attacker: t("a"),
            target: t("b"),
            outcome: CombatOutcome::Killed,
            detail_lines: vec![],
        });
        let combat_wound = MessagePayload::Combat(CombatEngagement {
            attacker: t("a"),
            target: t("b"),
            outcome: CombatOutcome::Wounded,
            detail_lines: vec![],
        });
        let msgs = vec![
            make_msg(1, Phase::Day, combat_kill.clone()),
            make_msg(1, Phase::Day, combat_wound),
            make_msg(1, Phase::Day, combat_kill),
        ];
        let result = summarize_periods(&msgs, (1, Phase::Day));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].deaths, 2);
        assert_eq!(result[0].event_count, 3);
    }

    #[test]
    fn summarize_is_current_flag_set_correctly() {
        let tref = TributeRef {
            identifier: "t".into(),
            name: "T".into(),
        };
        let p = MessagePayload::TributeRested {
            tribute: tref,
            hp_restored: 1,
        };
        let msgs = vec![make_msg(2, Phase::Night, p.clone())];
        let result = summarize_periods(&msgs, (2, Phase::Night));
        let current: Vec<_> = result.iter().filter(|s| s.is_current).collect();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].day, 2);
        assert_eq!(current[0].phase, Phase::Night);
    }
}

#[cfg(test)]
mod survival_event_tests {
    use super::*;

    fn tref() -> TributeRef {
        TributeRef {
            identifier: "t1".into(),
            name: "Cato".into(),
        }
    }
    fn aref() -> AreaRef {
        AreaRef {
            identifier: "a1".into(),
            name: "Forest".into(),
        }
    }
    fn iref() -> ItemRef {
        ItemRef {
            identifier: "i1".into(),
            name: "Berries".into(),
        }
    }

    #[test]
    fn shelter_sought_round_trip() {
        let p = MessagePayload::ShelterSought {
            tribute: tref(),
            area: aref(),
            success: true,
            roll: 2,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: MessagePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
        assert_eq!(p.kind(), MessageKind::State);
    }

    #[test]
    fn band_change_payloads_round_trip() {
        let p = MessagePayload::HungerBandChanged {
            tribute: tref(),
            from: HungerBand::Sated,
            to: HungerBand::Hungry,
        };
        let back: MessagePayload =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
        let p = MessagePayload::ThirstBandChanged {
            tribute: tref(),
            from: ThirstBand::Sated,
            to: ThirstBand::Parched,
        };
        let back: MessagePayload =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
    }

    #[test]
    fn stamina_band_change_round_trips_and_routes_to_state() {
        let p = MessagePayload::StaminaBandChanged {
            tribute: tref(),
            from: StaminaBand::Fresh,
            to: StaminaBand::Winded,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: MessagePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
        assert_eq!(p.kind(), MessageKind::State);
        assert!(p.involves(&tref().identifier));
    }

    #[test]
    fn stamina_band_enum_round_trips() {
        for band in [
            StaminaBand::Fresh,
            StaminaBand::Winded,
            StaminaBand::Exhausted,
        ] {
            let s = serde_json::to_string(&band).unwrap();
            let back: StaminaBand = serde_json::from_str(&s).unwrap();
            assert_eq!(band, back);
        }
    }

    #[test]
    fn foraged_drank_ate_round_trip_and_kind() {
        let foraged = MessagePayload::Foraged {
            tribute: tref(),
            area: aref(),
            success: true,
            debt_recovered: 3,
        };
        let drank = MessagePayload::Drank {
            tribute: tref(),
            source: DrinkSource::Terrain { area: aref() },
            debt_recovered: 2,
        };
        let drank_item = MessagePayload::Drank {
            tribute: tref(),
            source: DrinkSource::Item { item: iref() },
            debt_recovered: 1,
        };
        let ate = MessagePayload::Ate {
            tribute: tref(),
            item: iref(),
            debt_recovered: 4,
        };
        for p in [foraged, drank, drank_item, ate] {
            let back: MessagePayload =
                serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
            assert_eq!(format!("{:?}", p), format!("{:?}", back));
            assert_eq!(p.kind(), MessageKind::State);
        }
    }

    #[test]
    fn cause_constants_exist() {
        assert_eq!(CAUSE_STARVATION, "starvation");
        assert_eq!(CAUSE_DEHYDRATION, "dehydration");
    }

    #[test]
    fn survival_payloads_involve_tribute() {
        let p = MessagePayload::Ate {
            tribute: tref(),
            item: iref(),
            debt_recovered: 1,
        };
        assert!(p.involves("t1"));
        assert!(!p.involves("other"));
    }
}
