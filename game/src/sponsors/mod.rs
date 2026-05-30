use shared::afflictions::AfflictionKind;
use shared::audience::AudienceEvent;
use shared::sponsors::ArchetypeId;

use crate::games::Game;
use crate::items::Item;
use crate::tributes::Tribute;

pub struct SponsorContext<'a> {
    pub game: &'a Game,
    pub tributes: &'a [Tribute],
}

impl<'a> SponsorContext<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self {
            game,
            tributes: &game.tributes,
        }
    }

    pub fn tribute_district(&self, identifier: &str) -> Option<u8> {
        self.tributes
            .iter()
            .find(|t| t.identifier == identifier)
            .map(|t| t.district as u8)
    }
}

pub trait ArchetypeModifiers {
    fn district_loyalty_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 {
        1.0
    }
    fn combat_style_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 {
        1.0
    }
}

pub struct DefaultModifiers;
impl ArchetypeModifiers for DefaultModifiers {}

pub fn modifiers_for(id: ArchetypeId) -> Box<dyn ArchetypeModifiers> {
    match id {
        ArchetypeId::Loyalist => Box::new(LoyalistModifiers),
        ArchetypeId::Aesthete => Box::new(AestheteModifiers),
        _ => Box::new(DefaultModifiers),
    }
}

pub struct LoyalistModifiers;
impl ArchetypeModifiers for LoyalistModifiers {
    fn district_loyalty_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 {
        // Real impl in Task 6 — stub returns 1.0 so other tasks can compile.
        1.0
    }
}

pub struct AestheteModifiers;
impl ArchetypeModifiers for AestheteModifiers {
    fn combat_style_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 {
        // Real impl in Task 6.
        1.0
    }
}

/// Translate raw payloads into 0..N audience events.
pub fn translate(
    payload: &shared::messages::MessagePayload,
    ctx: &SponsorContext,
) -> Vec<AudienceEvent> {
    use shared::messages::MessagePayload;

    let mut out = Vec::new();
    match payload {
        MessagePayload::TributeKilled { victim, killer, .. } => {
            out.push(AudienceEvent::KillReceived {
                victim: victim.clone(),
                actor: killer.clone(),
                magnitude: 5,
                modifier: 1.0,
            });
            if let Some(k) = killer {
                out.push(AudienceEvent::KillMade {
                    actor: k.clone(),
                    victim: victim.clone(),
                    magnitude: 5,
                    modifier: 1.0,
                });
            }
        }
        MessagePayload::AllianceFormed { members } => {
            out.push(AudienceEvent::AllianceFormed {
                tributes: members.clone(),
            });
        }
        MessagePayload::BetrayalTriggered { betrayer, victim } => {
            out.push(AudienceEvent::BetrayalCommitted {
                actor: betrayer.clone(),
                victim: victim.clone(),
            });
        }
        MessagePayload::TributeAttacked {
            victim,
            attacker: Some(attacker),
        } => {
            out.push(AudienceEvent::AttackTrapped {
                actor: attacker.clone(),
                victim: victim.clone(),
            });

            // Check if the victim actually has a Trapped affliction
            // (extra disapproval for attacking defenseless tributes)
            let is_victim_trapped = ctx.tributes.iter().any(|t| {
                t.identifier == victim.identifier
                    && t.afflictions
                        .values()
                        .any(|a| matches!(a.kind, AfflictionKind::Trapped(_)))
            });

            if is_victim_trapped {
                // Extra disapproval: attacking a defenseless trapped tribute
                // is considered cowardly by the audience.
                out.push(AudienceEvent::Cowardice {
                    tribute: attacker.clone(),
                });
            }
        }
        // Other variants intentionally not mapped in PR1.
        // Future affliction specs add: TrappedEscaped → RescueAlly,
        // AfflictionAcquired → AfflictionAcquired,
        // surviving-AreaEvent → SurvivedAreaEvent.
        _ => {}
    }
    out
}

/// Apply audience-event affinity deltas to all sponsors in `game`.
pub fn update_affinities(game: &mut Game, events: &[AudienceEvent]) {
    use shared::sponsors::{ArchetypeId, MAX_AFFINITY, MIN_AFFINITY, weight_for};

    // Take an owned snapshot of tributes so the sponsor loop can borrow `&mut`.
    let tributes_snapshot: Vec<crate::tributes::Tribute> = game.tributes.clone();

    for sponsor in &mut game.sponsors {
        let mods = modifiers_for(sponsor.archetype);
        for ev in events {
            let base = weight_for(sponsor.archetype, ev.kind());
            if base == 0 {
                continue;
            }

            let event_modifier = (ev.magnitude_score() as f32) / 5.0;
            let district_mod = match sponsor.archetype {
                ArchetypeId::Loyalist => {
                    loyalist_district_modifier(sponsor.bound_district, ev, &tributes_snapshot)
                }
                _ => 1.0,
            };
            let style_mod = match sponsor.archetype {
                ArchetypeId::Aesthete => aesthete_style_modifier(ev),
                _ => 1.0,
            };

            let _ = mods; // silence unused (modifiers trait used by PR2 callers)
            let delta = (base as f32 * event_modifier * district_mod * style_mod) as i32;

            for tribute in ev.affected_tributes() {
                let entry = sponsor
                    .affinity
                    .entry(tribute.identifier.clone())
                    .or_insert(0);
                *entry = (*entry + delta).clamp(MIN_AFFINITY, MAX_AFFINITY);
            }
        }
    }
}

fn loyalist_district_modifier(
    bound: Option<u8>,
    ev: &AudienceEvent,
    tributes: &[crate::tributes::Tribute],
) -> f32 {
    let Some(district) = bound else {
        return 1.0;
    };
    let actor_in_district = |tref: &shared::messages::TributeRef| -> bool {
        tributes
            .iter()
            .any(|t| t.identifier == tref.identifier && t.district as u8 == district)
    };
    match ev {
        AudienceEvent::KillMade { actor, .. }
        | AudienceEvent::DistrictLoyaltyAct { actor, .. }
        | AudienceEvent::RescueAlly { actor, .. }
            if actor_in_district(actor) =>
        {
            1.5
        }
        AudienceEvent::KillReceived { victim, .. } if actor_in_district(victim) => 1.5,
        _ => 1.0,
    }
}

fn aesthete_style_modifier(ev: &AudienceEvent) -> f32 {
    match ev {
        AudienceEvent::KillMade { modifier, .. } => modifier.max(1.0),
        _ => 1.0,
    }
}

// ---------- Gift resolution (PR2) ----------

use rand::RngExt;
use rand::prelude::*;
use shared::messages::{ItemRef, MessagePayload};
use shared::sponsors::{
    AFFINITY_FLOOR, ITEM_COSTS, ItemKindTag, TRIGGER_FLOOR, archetype, priority_rank,
};

/// Result of gift resolution: `(SponsorGift payload, gift Item)`.
pub struct GiftResult {
    pub payload: MessagePayload,
    pub item: Item,
}

/// Resolve sponsor gifts for a batch of audience events.
///
/// Returns one `GiftResult` per tribute that received a gift this cycle.
/// Max 1 gift per tribute per cycle.
pub fn resolve_gifts(
    game: &mut Game,
    events: &[AudienceEvent],
    rng: &mut impl Rng,
) -> Vec<GiftResult> {
    let mut results = Vec::new();
    let mut gifted_this_cycle: std::collections::HashSet<String> = std::collections::HashSet::new();

    for ev in events {
        if ev.magnitude_score() < TRIGGER_FLOOR {
            continue;
        }

        for tribute_ref in ev.affected_tributes() {
            if gifted_this_cycle.contains(&tribute_ref.identifier) {
                continue;
            }

            let candidates: Vec<_> = game
                .sponsors
                .iter()
                .filter(|s| s.budget_remaining > 0)
                .filter(|s| {
                    s.affinity
                        .get(&tribute_ref.identifier)
                        .copied()
                        .unwrap_or(0)
                        >= AFFINITY_FLOOR
                })
                .collect();

            if candidates.is_empty() {
                continue;
            }

            let winner = candidates.iter().max_by_key(|s| {
                let affinity = s
                    .affinity
                    .get(&tribute_ref.identifier)
                    .copied()
                    .unwrap_or(0);
                let rank = priority_rank(s.archetype);
                (affinity, usize::MAX - rank)
            });

            let Some(sponsor) = winner else {
                continue;
            };
            let sponsor_idx = game
                .sponsors
                .iter()
                .position(|s| s.id == sponsor.id)
                .unwrap();

            let Some(item) = pick_gift(&game.sponsors[sponsor_idx], rng) else {
                continue;
            };

            let cost = item_cost(&item);
            if cost > game.sponsors[sponsor_idx].budget_remaining {
                continue;
            }

            game.sponsors[sponsor_idx].budget_remaining -= cost;
            gifted_this_cycle.insert(tribute_ref.identifier.clone());

            let donor = game.sponsors[sponsor_idx].canonical_name().to_string();
            let payload = MessagePayload::SponsorGift {
                recipient: tribute_ref.clone(),
                item: ItemRef {
                    identifier: item.identifier.clone(),
                    name: item.name.clone(),
                },
                donor,
            };

            results.push(GiftResult {
                payload,
                item: item.clone(),
            });
        }
    }

    results
}

/// Pick a gift item for a sponsor, biased by archetype preferences.
/// Returns `None` if the sponsor can't afford anything.
fn pick_gift(sponsor: &shared::sponsors::Sponsor, rng: &mut impl Rng) -> Option<Item> {
    let catalog = gift_catalog();
    let affordable: Vec<_> = catalog
        .iter()
        .filter(|(_item, cost)| *cost <= sponsor.budget_remaining)
        .collect();

    if affordable.is_empty() {
        return None;
    }

    let prefs = archetype(sponsor.archetype).gift_preferences;
    let weights: Vec<u32> = affordable
        .iter()
        .map(|(item, _)| {
            let tag = item_kind_tag(item);
            prefs
                .iter()
                .find_map(|(t, w)| (*t == tag).then_some(*w))
                .unwrap_or(1)
        })
        .collect();

    let idx = weighted_index(rng, &weights)?;
    let (item, _) = affordable[idx];
    Some(item.clone())
}

/// Map an `Item` to its `ItemKindTag` for gift-preference lookup.
fn item_kind_tag(item: &Item) -> ItemKindTag {
    use crate::items::ItemType;
    match &item.item_type {
        ItemType::Food(_) => ItemKindTag::Food,
        ItemType::Water(_) => ItemKindTag::Water,
        ItemType::Consumable => match item.attribute {
            crate::items::Attribute::Health | crate::items::Attribute::Sanity => {
                ItemKindTag::Bandage
            }
            crate::items::Attribute::Defense => ItemKindTag::Antidote,
            _ => ItemKindTag::Bandage,
        },
        ItemType::Weapon => {
            if item.rarity == crate::items::ItemRarity::Rare
                || item.rarity == crate::items::ItemRarity::Legendary
            {
                ItemKindTag::WeaponRare
            } else {
                ItemKindTag::WeaponBasic
            }
        }
    }
}

/// Cost of an item, looked up from `ITEM_COSTS` by tag.
fn item_cost(item: &Item) -> u32 {
    let tag = item_kind_tag(item);
    ITEM_COSTS
        .iter()
        .find_map(|(t, c)| (*t == tag).then_some(*c))
        .unwrap_or(10)
}

/// Full gift catalog: `(Item, cost)` pairs.
#[allow(clippy::vec_init_then_push)]
fn gift_catalog() -> Vec<(Item, u32)> {
    use crate::items::{Attribute, ItemRarity};

    let mut catalog = Vec::new();

    // Food
    catalog.push((Item::new_food(None, 5), 5));
    catalog.push((Item::new_food(None, 3), 5));

    // Water
    catalog.push((Item::new_water(None, 3), 5));
    catalog.push((Item::new_water(None, 2), 5));

    // Bandage (consumable healing)
    let mut bandage = Item::new_consumable("bandage");
    bandage.attribute = Attribute::Health;
    catalog.push((bandage, 10));

    // Antidote
    let mut antidote = Item::new_consumable("antidote");
    antidote.attribute = Attribute::Defense;
    catalog.push((antidote, 18));

    // Map (signal-like consumable)
    let mut map = Item::new_consumable("map");
    map.attribute = Attribute::Movement;
    catalog.push((map, 12));

    // Signal
    let mut signal = Item::new_consumable("signal flare");
    signal.attribute = Attribute::Bravery;
    catalog.push((signal, 20));

    // Weapon basic
    let weapon = Item::new_weapon("spear");
    catalog.push((weapon, 25));

    // Weapon rare
    let mut rare_weapon = Item::new_weapon("golden spear");
    rare_weapon.rarity = ItemRarity::Rare;
    catalog.push((rare_weapon, 45));

    // Shield
    let shield = Item::new_shield("wooden shield");
    catalog.push((shield, 30));

    catalog
}

/// Weighted random index selection.
fn weighted_index(rng: &mut impl Rng, weights: &[u32]) -> Option<usize> {
    let total: u32 = weights.iter().sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.random_range(0..total);
    for (i, &w) in weights.iter().enumerate() {
        if roll < w {
            return Some(i);
        }
        roll -= w;
    }
    None
}

#[cfg(test)]
mod tests {
    use shared::audience::AudienceEvent;
    use shared::messages::{MessagePayload, TributeRef};

    use crate::games::Game;

    use super::{SponsorContext, translate, update_affinities};

    fn tref(name: &str) -> TributeRef {
        TributeRef {
            identifier: name.into(),
            name: name.into(),
        }
    }

    #[test]
    fn killed_emits_kill_made_and_kill_received() {
        let game = Game::default();
        let ctx = SponsorContext::new(&game);
        let payload = MessagePayload::TributeKilled {
            victim: tref("v"),
            killer: Some(tref("k")),
            cause: "spear".into(),
        };
        let events = translate(&payload, &ctx);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn killed_without_killer_only_emits_kill_received() {
        let game = Game::default();
        let ctx = SponsorContext::new(&game);
        let payload = MessagePayload::TributeKilled {
            victim: tref("v"),
            killer: None,
            cause: "fall".into(),
        };
        let events = translate(&payload, &ctx);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], AudienceEvent::KillReceived { .. }));
    }

    #[test]
    fn alliance_formed_passes_through() {
        let game = Game::default();
        let ctx = SponsorContext::new(&game);
        let payload = MessagePayload::AllianceFormed {
            members: vec![tref("a"), tref("b")],
        };
        let events = translate(&payload, &ctx);
        assert!(matches!(events[0], AudienceEvent::AllianceFormed { .. }));
    }

    #[test]
    fn unmapped_payload_yields_nothing() {
        let game = Game::default();
        let ctx = SponsorContext::new(&game);
        let payload = MessagePayload::TributeRested {
            tribute: tref("x"),
            hp_restored: 5,
        };
        assert!(translate(&payload, &ctx).is_empty());
    }

    #[test]
    fn alliance_increases_compassionate_affinity_for_all_members() {
        use rand::SeedableRng;
        let mut game = Game::default();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        game.spawn_sponsors(&mut rng);

        let events = vec![AudienceEvent::AllianceFormed {
            tributes: vec![tref("a"), tref("b"), tref("c")],
        }];
        update_affinities(&mut game, &events);

        let comp = game
            .sponsors
            .iter()
            .find(|s| s.archetype == shared::sponsors::ArchetypeId::Compassionate)
            .unwrap();
        assert!(comp.affinity.get("a").copied().unwrap_or(0) > 0);
        assert!(comp.affinity.get("c").copied().unwrap_or(0) > 0);
    }

    #[test]
    fn affinity_clamped_at_max() {
        use rand::SeedableRng;
        let mut game = Game::default();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        game.spawn_sponsors(&mut rng);

        // Hammer a single tribute with 200 alliance events.
        let events: Vec<_> = (0..200)
            .map(|_| AudienceEvent::AllianceFormed {
                tributes: vec![tref("a")],
            })
            .collect();
        update_affinities(&mut game, &events);

        for s in &game.sponsors {
            if let Some(v) = s.affinity.get("a") {
                assert!(*v <= shared::sponsors::MAX_AFFINITY);
                assert!(*v >= shared::sponsors::MIN_AFFINITY);
            }
        }
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn affinity_always_within_bounds(event_count in 0usize..50, magnitude in 0u32..50, modifier_x10 in 0u32..30) {
            use rand::SeedableRng;
            let mut game = Game::default();
            let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
            game.spawn_sponsors(&mut rng);

            let modifier = modifier_x10 as f32 / 10.0;
            let events: Vec<_> = (0..event_count).map(|i| {
                if i % 3 == 0 {
                    AudienceEvent::KillMade { actor: tref("a"), victim: tref("b"), magnitude, modifier }
                } else if i % 3 == 1 {
                    AudienceEvent::BetrayalCommitted { actor: tref("a"), victim: tref("b") }
                } else {
                    AudienceEvent::AllianceFormed { tributes: vec![tref("a"), tref("b")] }
                }
            }).collect();

            update_affinities(&mut game, &events);

            for s in &game.sponsors {
                for v in s.affinity.values() {
                    prop_assert!(*v >= shared::sponsors::MIN_AFFINITY);
                    prop_assert!(*v <= shared::sponsors::MAX_AFFINITY);
                }
            }
        }
    }
}
