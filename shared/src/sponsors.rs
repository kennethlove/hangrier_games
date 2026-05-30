use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::audience::AudienceEventKind;

pub const MIN_AFFINITY: i32 = -100;
pub const MAX_AFFINITY: i32 = 100;
pub const AFFINITY_FLOOR: i32 = 25;
pub const TRIGGER_FLOOR: u32 = 8;

/// Affinity penalty applied when attacking a trapped (defenseless) target.
pub const SPONSOR_PENALTY_ATTACK_TRAPPED: i32 = -15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchetypeId {
    Aesthete,
    Gambler,
    Loyalist,
    Sadist,
    Compassionate,
    Strategist,
}

/// Tags used by the archetype gift-preference table.
/// Resolved against `game::items::Item` discriminants in the gift-resolver (PR2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKindTag {
    Food,
    Water,
    Bandage,
    Antidote,
    Map,
    Signal,
    WeaponBasic,
    WeaponRare,
    Shield,
}

/// Item cost table used by the gift resolver.
pub const ITEM_COSTS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food, 5),
    (ItemKindTag::Water, 5),
    (ItemKindTag::Bandage, 10),
    (ItemKindTag::Antidote, 18),
    (ItemKindTag::Map, 12),
    (ItemKindTag::Signal, 20),
    (ItemKindTag::WeaponBasic, 25),
    (ItemKindTag::WeaponRare, 45),
    (ItemKindTag::Shield, 30),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sponsor {
    pub id: u32,
    pub archetype: ArchetypeId,
    pub budget_remaining: u32,
    /// Some(d) for Loyalist, None for others.
    pub bound_district: Option<u8>,
    /// keyed by `TributeRef.identifier`
    pub affinity: HashMap<String, i32>,
}

impl Sponsor {
    pub fn canonical_name(&self) -> &'static str {
        archetype(self.archetype).canonical_name
    }
}

pub struct Archetype {
    pub id: ArchetypeId,
    pub canonical_name: &'static str,
    /// Inclusive (min, max) for per-game budget roll.
    pub budget_band: (u32, u32),
    pub event_weights: &'static [(AudienceEventKind, i32)],
    pub gift_preferences: &'static [(ItemKindTag, u32)],
}

pub const ARCHETYPE_PRIORITY_ORDER: &[ArchetypeId] = &[
    ArchetypeId::Aesthete,
    ArchetypeId::Strategist,
    ArchetypeId::Compassionate,
    ArchetypeId::Gambler,
    ArchetypeId::Sadist,
    ArchetypeId::Loyalist,
];

pub fn priority_rank(id: ArchetypeId) -> usize {
    ARCHETYPE_PRIORITY_ORDER
        .iter()
        .position(|a| *a == id)
        .unwrap_or(usize::MAX)
}

// ---------- Per-archetype constants ----------

const AESTHETE_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::KillMade, 8),
    (AudienceEventKind::AttackTrapped, -6),
    (AudienceEventKind::BetrayalCommitted, -3),
    (AudienceEventKind::Cowardice, -5),
];
const AESTHETE_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::WeaponRare, 6),
    (ItemKindTag::WeaponBasic, 3),
    (ItemKindTag::Shield, 2),
];

const GAMBLER_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::UnderdogVictory, 12),
    (AudienceEventKind::SurvivedAreaEvent, 4),
    (AudienceEventKind::Cowardice, -2),
];
const GAMBLER_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::WeaponBasic, 4),
    (ItemKindTag::Bandage, 3),
    (ItemKindTag::Antidote, 2),
];

const LOYALIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::DistrictLoyaltyAct, 10),
    (AudienceEventKind::KillMade, 3),
    (AudienceEventKind::KillReceived, -8),
];
const LOYALIST_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food, 4),
    (ItemKindTag::Water, 4),
    (ItemKindTag::Bandage, 3),
    (ItemKindTag::WeaponBasic, 2),
];

const SADIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::AttackTrapped, 8),
    (AudienceEventKind::BetrayalCommitted, 9),
    (AudienceEventKind::AllianceFormed, -3),
    (AudienceEventKind::RescueAlly, -4),
];
const SADIST_PREFS: &[(ItemKindTag, u32)] =
    &[(ItemKindTag::WeaponRare, 5), (ItemKindTag::WeaponBasic, 4)];

const COMPASSIONATE_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::RescueAlly, 9),
    (AudienceEventKind::AllianceFormed, 5),
    (AudienceEventKind::SurvivedAreaEvent, 3),
    (AudienceEventKind::AttackTrapped, -7),
    (AudienceEventKind::BetrayalCommitted, -8),
];
const COMPASSIONATE_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food, 5),
    (ItemKindTag::Water, 5),
    (ItemKindTag::Bandage, 4),
    (ItemKindTag::Antidote, 3),
];

const STRATEGIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::KillMade, 6),
    (AudienceEventKind::AllianceFormed, 4),
    (AudienceEventKind::Cowardice, -3),
    (AudienceEventKind::BetrayalCommitted, 2),
];
const STRATEGIST_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Map, 5),
    (ItemKindTag::Signal, 4),
    (ItemKindTag::WeaponBasic, 3),
    (ItemKindTag::Shield, 3),
];

pub static ARCHETYPES: &[Archetype] = &[
    Archetype {
        id: ArchetypeId::Aesthete,
        canonical_name: "Aesthete",
        budget_band: (80, 120),
        event_weights: AESTHETE_WEIGHTS,
        gift_preferences: AESTHETE_PREFS,
    },
    Archetype {
        id: ArchetypeId::Gambler,
        canonical_name: "Gambler",
        budget_band: (60, 100),
        event_weights: GAMBLER_WEIGHTS,
        gift_preferences: GAMBLER_PREFS,
    },
    Archetype {
        id: ArchetypeId::Loyalist,
        canonical_name: "Loyalist",
        budget_band: (30, 60),
        event_weights: LOYALIST_WEIGHTS,
        gift_preferences: LOYALIST_PREFS,
    },
    Archetype {
        id: ArchetypeId::Sadist,
        canonical_name: "Sadist",
        budget_band: (50, 90),
        event_weights: SADIST_WEIGHTS,
        gift_preferences: SADIST_PREFS,
    },
    Archetype {
        id: ArchetypeId::Compassionate,
        canonical_name: "Compassionate",
        budget_band: (70, 110),
        event_weights: COMPASSIONATE_WEIGHTS,
        gift_preferences: COMPASSIONATE_PREFS,
    },
    Archetype {
        id: ArchetypeId::Strategist,
        canonical_name: "Strategist",
        budget_band: (70, 110),
        event_weights: STRATEGIST_WEIGHTS,
        gift_preferences: STRATEGIST_PREFS,
    },
];

pub fn archetype(id: ArchetypeId) -> &'static Archetype {
    ARCHETYPES
        .iter()
        .find(|a| a.id == id)
        .expect("archetype catalog missing entry")
}

pub fn weight_for(id: ArchetypeId, kind: AudienceEventKind) -> i32 {
    archetype(id)
        .event_weights
        .iter()
        .find_map(|(k, w)| (*k == kind).then_some(*w))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_six_archetypes() {
        assert_eq!(ARCHETYPES.len(), 6);
    }

    #[test]
    fn priority_order_covers_all() {
        for a in ARCHETYPES {
            assert!(priority_rank(a.id) < ARCHETYPES.len());
        }
    }

    #[test]
    fn unknown_event_weight_is_zero() {
        assert_eq!(
            weight_for(ArchetypeId::Aesthete, AudienceEventKind::UnderdogVictory),
            0
        );
    }

    #[test]
    fn loyalist_loves_district_loyalty_acts() {
        assert!(weight_for(ArchetypeId::Loyalist, AudienceEventKind::DistrictLoyaltyAct) > 0);
    }

    #[test]
    fn sadist_hates_rescues() {
        assert!(weight_for(ArchetypeId::Sadist, AudienceEventKind::RescueAlly) < 0);
    }
}
