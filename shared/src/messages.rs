use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

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

/// Day or night phase within a game tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Day,
    Night,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Day => write!(f, "day"),
            Phase::Night => write!(f, "night"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePhaseError;

impl std::fmt::Display for ParsePhaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "phase must be 'day' or 'night'")
    }
}

impl std::error::Error for ParsePhaseError {}

impl FromStr for Phase {
    type Err = ParsePhaseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "day" => Ok(Phase::Day),
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

/// Coarse-grained category for a `GameMessage`. Derived from `MessagePayload`
/// via `MessagePayload::kind()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    Death,
    Combat,
    Alliance,
    Movement,
    Item,
    State,
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
}

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        use MessagePayload::*;
        match self {
            TributeKilled { .. } => MessageKind::Death,
            Combat(_) => MessageKind::Combat,
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
            TributeWounded { .. }
            | TributeRested { .. }
            | TributeStarved { .. }
            | TributeDehydrated { .. }
            | SanityBreak { .. } => MessageKind::State,
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
pub fn summarize_periods(messages: &[GameMessage], current: (u32, Phase)) -> Vec<PeriodSummary> {
    use std::collections::BTreeMap;

    let (current_day, current_phase) = current;
    let mut bucket: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    let phase_ord = |p: Phase| match p {
        Phase::Day => 0,
        Phase::Night => 1,
    };

    for m in messages {
        let key = (m.game_day, phase_ord(m.phase));
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

    for d in 1..=current_day {
        bucket.entry((d, 0)).or_insert((0, 0));
        if d < current_day || matches!(current_phase, Phase::Night) {
            bucket.entry((d, 1)).or_insert((0, 0));
        }
    }

    bucket
        .into_iter()
        .map(|((day, p), (deaths, count))| {
            let phase = if p == 0 { Phase::Day } else { Phase::Night };
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
        assert_eq!(Phase::Day.to_string(), "day");
        assert_eq!(Phase::Night.to_string(), "night");
        assert_eq!("day".parse::<Phase>().unwrap(), Phase::Day);
        assert_eq!("night".parse::<Phase>().unwrap(), Phase::Night);
        assert!("noon".parse::<Phase>().is_err());
    }

    #[test]
    fn phase_serde_lowercase() {
        let s = serde_json::to_string(&Phase::Day).unwrap();
        assert_eq!(s, "\"day\"");
        let p: Phase = serde_json::from_str("\"night\"").unwrap();
        assert_eq!(p, Phase::Night);
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
        assert!(result.is_empty(), "no periods reached when current_day=0");
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
        assert_eq!(result.len(), 3);
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
                phase: Phase::Night,
                deaths: 0,
                event_count: 1,
                is_current: false
            }
        );
        assert_eq!(
            result[2],
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
        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0],
            PeriodSummary {
                day: 1,
                phase: Phase::Day,
                deaths: 0,
                event_count: 0,
                is_current: false
            }
        );
        assert_eq!(
            result[1],
            PeriodSummary {
                day: 1,
                phase: Phase::Night,
                deaths: 0,
                event_count: 0,
                is_current: false
            }
        );
        assert_eq!(
            result[2],
            PeriodSummary {
                day: 2,
                phase: Phase::Day,
                deaths: 0,
                event_count: 0,
                is_current: true
            }
        );
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
