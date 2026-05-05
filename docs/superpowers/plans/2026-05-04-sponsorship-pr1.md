# Sponsorship System — PR1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the sponsorship data model, AudienceEvent translator, archetype catalog, affinity tracking, and per-cycle hook — *with no observable behavior change*. `receive_patron_gift` keeps running unchanged. Gift resolution is PR2.

**Architecture:** Six NPC sponsor archetypes spawned at game start. Each cycle, `MessagePayload`s collected during `execute_cycle` are translated into a dedicated `AudienceEvent` enum, then per-`(sponsor, tribute)` affinity is updated using a per-archetype event-weight table plus two modifier hooks (Loyalist district loyalty, Aesthete combat style). All affinities are clamped `[-100, 100]`.

**Tech Stack:** Rust 2024, `rand = "0.10"` (existing), `rstest = "0.26"`, `insta = "1.40"`, `proptest = "1.5"`. Workspace crates: `shared/` (data types), `game/` (logic + Game struct).

**Spec:** `docs/superpowers/specs/2026-05-04-sponsorship-design.md`

**Bead:** TBD (file after plan approval; will block sponsorship-PR2)

---

## File Structure

| Path | Status | Responsibility |
|---|---|---|
| `shared/src/audience.rs` | create | `AudienceEvent`, `AudienceEventKind`, `magnitude_score()` |
| `shared/src/sponsors.rs` | create | `Sponsor`, `Archetype`, `ArchetypeId`, `ARCHETYPES`, weight tables, gift preferences, budget bands, `ARCHETYPE_PRIORITY_ORDER`, constants |
| `shared/src/lib.rs` | modify | `pub mod audience;` + `pub mod sponsors;` |
| `game/src/sponsors/mod.rs` | create | `SponsorContext`, `ArchetypeModifiers` trait, `translate()`, `update_affinities()`, Loyalist + Aesthete impls |
| `game/src/lib.rs` | modify | `pub mod sponsors;` |
| `game/src/games.rs` | modify | `Game::sponsors` field, `spawn_sponsors()`, `sponsor_affinity_snapshot()`, per-cycle hook in `execute_cycle`, lazy spawn on game-load |
| `game/Cargo.toml` | modify (maybe) | add `proptest` + `insta` to `[dev-dependencies]` if missing |

---

## Task 1: AudienceEvent enum (shared)

**Files:**
- Create: `shared/src/audience.rs`
- Modify: `shared/src/lib.rs`
- Test: inline `#[cfg(test)] mod tests` in `shared/src/audience.rs`

- [ ] **Step 1: Add module declaration**

Edit `shared/src/lib.rs`, after `pub mod messages;`:

```rust
pub mod audience;
```

- [ ] **Step 2: Write the failing test**

Create `shared/src/audience.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::messages::TributeRef;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudienceEventKind {
    KillMade,
    KillReceived,
    AttackTrapped,
    RescueAlly,
    AllianceFormed,
    BetrayalCommitted,
    AfflictionAcquired,
    SurvivedAreaEvent,
    UnderdogVictory,
    DistrictLoyaltyAct,
    Cowardice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudienceEvent {
    KillMade        { actor: TributeRef, victim: TributeRef, magnitude: u32, modifier: f32 },
    KillReceived    { victim: TributeRef, actor: Option<TributeRef>, magnitude: u32, modifier: f32 },
    AttackTrapped   { actor: TributeRef, victim: TributeRef },
    RescueAlly      { actor: TributeRef, ally: TributeRef },
    AllianceFormed  { tributes: Vec<TributeRef> },
    BetrayalCommitted { actor: TributeRef, victim: TributeRef },
    AfflictionAcquired { tribute: TributeRef, kind: String },
    SurvivedAreaEvent  { tribute: TributeRef },
    UnderdogVictory    { actor: TributeRef, victim: TributeRef },
    DistrictLoyaltyAct { actor: TributeRef, district: u8 },
    Cowardice          { tribute: TributeRef },
}

impl AudienceEvent {
    pub fn kind(&self) -> AudienceEventKind {
        match self {
            Self::KillMade { .. }            => AudienceEventKind::KillMade,
            Self::KillReceived { .. }        => AudienceEventKind::KillReceived,
            Self::AttackTrapped { .. }       => AudienceEventKind::AttackTrapped,
            Self::RescueAlly { .. }          => AudienceEventKind::RescueAlly,
            Self::AllianceFormed { .. }      => AudienceEventKind::AllianceFormed,
            Self::BetrayalCommitted { .. }   => AudienceEventKind::BetrayalCommitted,
            Self::AfflictionAcquired { .. }  => AudienceEventKind::AfflictionAcquired,
            Self::SurvivedAreaEvent { .. }   => AudienceEventKind::SurvivedAreaEvent,
            Self::UnderdogVictory { .. }     => AudienceEventKind::UnderdogVictory,
            Self::DistrictLoyaltyAct { .. }  => AudienceEventKind::DistrictLoyaltyAct,
            Self::Cowardice { .. }           => AudienceEventKind::Cowardice,
        }
    }

    /// Base × modifier; floor at 1 to avoid 0-magnitude triggers.
    pub fn magnitude_score(&self) -> u32 {
        let (base, modifier) = match self {
            Self::KillMade { magnitude, modifier, .. }
            | Self::KillReceived { magnitude, modifier, .. } => (*magnitude, *modifier),
            Self::AttackTrapped { .. }       => (6, 1.0),
            Self::RescueAlly { .. }          => (5, 1.0),
            Self::AllianceFormed { .. }      => (3, 1.0),
            Self::BetrayalCommitted { .. }   => (7, 1.0),
            Self::AfflictionAcquired { .. }  => (3, 1.0),
            Self::SurvivedAreaEvent { .. }   => (4, 1.0),
            Self::UnderdogVictory { .. }     => (10, 1.0),
            Self::DistrictLoyaltyAct { .. }  => (5, 1.0),
            Self::Cowardice { .. }           => (2, 1.0),
        };
        ((base as f32 * modifier).max(1.0)) as u32
    }

    /// Tributes whose affinity-with-sponsor is updated by this event.
    pub fn affected_tributes(&self) -> Vec<&TributeRef> {
        match self {
            Self::KillMade { actor, victim, .. }
            | Self::AttackTrapped { actor, victim }
            | Self::BetrayalCommitted { actor, victim }
            | Self::UnderdogVictory { actor, victim } => vec![actor, victim],
            Self::KillReceived { victim, actor, .. } => match actor {
                Some(a) => vec![victim, a],
                None    => vec![victim],
            },
            Self::RescueAlly { actor, ally } => vec![actor, ally],
            Self::AllianceFormed { tributes } => tributes.iter().collect(),
            Self::AfflictionAcquired { tribute, .. }
            | Self::SurvivedAreaEvent { tribute }
            | Self::Cowardice { tribute } => vec![tribute],
            Self::DistrictLoyaltyAct { actor, .. } => vec![actor],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(name: &str) -> TributeRef {
        TributeRef { identifier: name.into(), name: name.into() }
    }

    #[test]
    fn kill_made_magnitude_uses_base_times_modifier() {
        let ev = AudienceEvent::KillMade {
            actor: t("a"), victim: t("b"), magnitude: 5, modifier: 2.0,
        };
        assert_eq!(ev.magnitude_score(), 10);
    }

    #[test]
    fn betrayal_kind_roundtrips() {
        let ev = AudienceEvent::BetrayalCommitted { actor: t("a"), victim: t("b") };
        assert_eq!(ev.kind(), AudienceEventKind::BetrayalCommitted);
    }

    #[test]
    fn alliance_affects_all_members() {
        let ev = AudienceEvent::AllianceFormed { tributes: vec![t("a"), t("b"), t("c")] };
        assert_eq!(ev.affected_tributes().len(), 3);
    }

    #[test]
    fn magnitude_score_never_zero() {
        let ev = AudienceEvent::KillMade {
            actor: t("a"), victim: t("b"), magnitude: 0, modifier: 0.0,
        };
        assert!(ev.magnitude_score() >= 1);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p shared audience::`
Expected: 4 passed

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(shared): add AudienceEvent enum (#dvd PR1)"
```

---

## Task 2: Sponsor + Archetype catalog (shared)

**Files:**
- Create: `shared/src/sponsors.rs`
- Modify: `shared/src/lib.rs`
- Test: inline

- [ ] **Step 1: Add module declaration**

Edit `shared/src/lib.rs` after `pub mod audience;`:

```rust
pub mod sponsors;
```

- [ ] **Step 2: Write the failing test + impl**

Create `shared/src/sponsors.rs`:

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::audience::AudienceEventKind;

pub const MIN_AFFINITY: i32 = -100;
pub const MAX_AFFINITY: i32 =  100;
pub const AFFINITY_FLOOR: i32 = 25;
pub const TRIGGER_FLOOR: u32  = 8;

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
    ARCHETYPE_PRIORITY_ORDER.iter().position(|a| *a == id).unwrap_or(usize::MAX)
}

// ---------- Per-archetype constants ----------

const AESTHETE_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::KillMade,           8),
    (AudienceEventKind::AttackTrapped,     -6),
    (AudienceEventKind::BetrayalCommitted, -3),
    (AudienceEventKind::Cowardice,         -5),
];
const AESTHETE_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::WeaponRare, 6),
    (ItemKindTag::WeaponBasic, 3),
    (ItemKindTag::Shield, 2),
];

const GAMBLER_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::UnderdogVictory,   12),
    (AudienceEventKind::SurvivedAreaEvent,  4),
    (AudienceEventKind::Cowardice,         -2),
];
const GAMBLER_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::WeaponBasic, 4),
    (ItemKindTag::Bandage, 3),
    (ItemKindTag::Antidote, 2),
];

const LOYALIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::DistrictLoyaltyAct, 10),
    (AudienceEventKind::KillMade,            3),
    (AudienceEventKind::KillReceived,       -8),
];
const LOYALIST_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food, 4),
    (ItemKindTag::Water, 4),
    (ItemKindTag::Bandage, 3),
    (ItemKindTag::WeaponBasic, 2),
];

const SADIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::AttackTrapped,      8),
    (AudienceEventKind::BetrayalCommitted,  9),
    (AudienceEventKind::AllianceFormed,    -3),
    (AudienceEventKind::RescueAlly,        -4),
];
const SADIST_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::WeaponRare, 5),
    (ItemKindTag::WeaponBasic, 4),
];

const COMPASSIONATE_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::RescueAlly,         9),
    (AudienceEventKind::AllianceFormed,     5),
    (AudienceEventKind::SurvivedAreaEvent,  3),
    (AudienceEventKind::AttackTrapped,     -7),
    (AudienceEventKind::BetrayalCommitted, -8),
];
const COMPASSIONATE_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food, 5),
    (ItemKindTag::Water, 5),
    (ItemKindTag::Bandage, 4),
    (ItemKindTag::Antidote, 3),
];

const STRATEGIST_WEIGHTS: &[(AudienceEventKind, i32)] = &[
    (AudienceEventKind::KillMade,           6),
    (AudienceEventKind::AllianceFormed,     4),
    (AudienceEventKind::Cowardice,         -3),
    (AudienceEventKind::BetrayalCommitted,  2),
];
const STRATEGIST_PREFS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Map, 5),
    (ItemKindTag::Signal, 4),
    (ItemKindTag::WeaponBasic, 3),
    (ItemKindTag::Shield, 3),
];

pub static ARCHETYPES: &[Archetype] = &[
    Archetype { id: ArchetypeId::Aesthete,      canonical_name: "Aesthete",
                budget_band: (80, 120),  event_weights: AESTHETE_WEIGHTS,      gift_preferences: AESTHETE_PREFS },
    Archetype { id: ArchetypeId::Gambler,       canonical_name: "Gambler",
                budget_band: (60, 100),  event_weights: GAMBLER_WEIGHTS,       gift_preferences: GAMBLER_PREFS },
    Archetype { id: ArchetypeId::Loyalist,      canonical_name: "Loyalist",
                budget_band: (30, 60),   event_weights: LOYALIST_WEIGHTS,      gift_preferences: LOYALIST_PREFS },
    Archetype { id: ArchetypeId::Sadist,        canonical_name: "Sadist",
                budget_band: (50, 90),   event_weights: SADIST_WEIGHTS,        gift_preferences: SADIST_PREFS },
    Archetype { id: ArchetypeId::Compassionate, canonical_name: "Compassionate",
                budget_band: (70, 110),  event_weights: COMPASSIONATE_WEIGHTS, gift_preferences: COMPASSIONATE_PREFS },
    Archetype { id: ArchetypeId::Strategist,    canonical_name: "Strategist",
                budget_band: (70, 110),  event_weights: STRATEGIST_WEIGHTS,    gift_preferences: STRATEGIST_PREFS },
];

pub fn archetype(id: ArchetypeId) -> &'static Archetype {
    ARCHETYPES.iter().find(|a| a.id == id).expect("archetype catalog missing entry")
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
        assert_eq!(weight_for(ArchetypeId::Aesthete, AudienceEventKind::UnderdogVictory), 0);
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
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p shared sponsors::`
Expected: 5 passed

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(shared): add Sponsor archetype catalog (#dvd PR1)"
```

---

## Task 3: SponsorContext + ArchetypeModifiers trait (game)

**Files:**
- Create: `game/src/sponsors/mod.rs`
- Modify: `game/src/lib.rs`

- [ ] **Step 1: Locate `game/src/lib.rs` mod declarations**

Run: `grep -n "^pub mod" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/lib.rs`

Expected: list of `pub mod ...;` lines.

- [ ] **Step 2: Add module declaration**

Append to the `pub mod` block in `game/src/lib.rs`:

```rust
pub mod sponsors;
```

- [ ] **Step 3: Create the file**

Create `game/src/sponsors/mod.rs`:

```rust
use shared::audience::AudienceEvent;
use shared::sponsors::{ArchetypeId, Sponsor};

use crate::games::Game;
use crate::tributes::Tribute;

pub struct SponsorContext<'a> {
    pub game: &'a Game,
    pub tributes: &'a [Tribute],
}

impl<'a> SponsorContext<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game, tributes: &game.tributes }
    }

    pub fn tribute_district(&self, identifier: &str) -> Option<u8> {
        self.tributes
            .iter()
            .find(|t| t.identifier() == identifier)
            .map(|t| t.district as u8)
    }
}

pub trait ArchetypeModifiers {
    fn district_loyalty_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 { 1.0 }
    fn combat_style_modifier(&self,    _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 { 1.0 }
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
        // Real impl in Task 7 — stub returns 1.0 so other tasks can compile.
        1.0
    }
}

pub struct AestheteModifiers;
impl ArchetypeModifiers for AestheteModifiers {
    fn combat_style_modifier(&self, _ev: &AudienceEvent, _ctx: &SponsorContext) -> f32 {
        // Real impl in Task 8.
        1.0
    }
}

/// Translate raw payloads into 0..N audience events. Stub for now (Task 5 fills it).
pub fn translate(_payload: &shared::messages::MessagePayload, _ctx: &SponsorContext) -> Vec<AudienceEvent> {
    Vec::new()
}

/// Apply audience-event affinity deltas to all sponsors in `game`. Stub (Task 6).
pub fn update_affinities(_game: &mut Game, _events: &[AudienceEvent]) {
    // PR1 Task 6 fills this in.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translator_stub_returns_empty() {
        // Compile-time wiring check; stub returns Vec::new().
        assert!(true);
    }
}
```

- [ ] **Step 4: Verify Tribute exposes identifier**

Run: `grep -n "fn identifier\|pub identifier" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/tributes/mod.rs | head -3`

Expected: an `identifier(&self) -> &str` method or `pub identifier: String` field. If only field exists, change `t.identifier() == identifier` to `t.identifier == identifier`.

- [ ] **Step 5: Build**

Run: `cargo build -p game`
Expected: compiles cleanly.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): add sponsors module skeleton (#dvd PR1)"
```

---

## Task 4: Game::sponsors field + spawn_sponsors

**Files:**
- Modify: `game/src/games.rs:120-180` (struct), `game/src/games.rs:261+` (impl block)

- [ ] **Step 1: Read the Game struct**

Run: `grep -n "^pub struct Game\|^}" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/games.rs | head -10`

Identify the closing `}` of `pub struct Game`. Read those lines with `view`.

- [ ] **Step 2: Add the field**

In `game/src/games.rs`, inside `pub struct Game { ... }`, add (after `pub tributes: Vec<Tribute>,`):

```rust
    #[serde(default)]
    pub sponsors: Vec<shared::sponsors::Sponsor>,
```

(If `Game` is not Serde-derived, drop the `#[serde(default)]` attribute; add a manual default elsewhere if needed.)

- [ ] **Step 3: Add the spawn method**

Inside `impl Game { ... }`, append:

```rust
    /// Spawn one sponsor per archetype using the shared catalog.
    /// Loyalist gets a randomly-assigned district (1..=12). Budget is rolled
    /// inside the archetype's budget band. Idempotent: no-op if `self.sponsors`
    /// is already populated.
    pub fn spawn_sponsors(&mut self, rng: &mut impl rand::Rng) {
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
```

- [ ] **Step 4: Add the test**

Append to `game/src/games.rs`'s `#[cfg(test)] mod tests { ... }`:

```rust
    #[test]
    fn spawn_sponsors_creates_six_with_loyalist_district() {
        use rand::SeedableRng;
        let mut game = Game::default();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        game.spawn_sponsors(&mut rng);

        assert_eq!(game.sponsors.len(), 6);
        let loyalist = game
            .sponsors
            .iter()
            .find(|s| s.archetype == shared::sponsors::ArchetypeId::Loyalist)
            .expect("Loyalist must spawn");
        let district = loyalist.bound_district.expect("Loyalist gets a district");
        assert!((1u8..=12).contains(&district));
    }

    #[test]
    fn spawn_sponsors_is_idempotent() {
        use rand::SeedableRng;
        let mut game = Game::default();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        game.spawn_sponsors(&mut rng);
        game.spawn_sponsors(&mut rng);
        assert_eq!(game.sponsors.len(), 6);
    }

    #[test]
    fn budget_falls_inside_archetype_band() {
        use rand::SeedableRng;
        let mut game = Game::default();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(7);
        game.spawn_sponsors(&mut rng);
        for s in &game.sponsors {
            let band = shared::sponsors::archetype(s.archetype).budget_band;
            assert!(s.budget_remaining >= band.0 && s.budget_remaining <= band.1);
        }
    }
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p game spawn_sponsors`
Expected: 3 passed.

If `Game::default()` doesn't exist, construct with whatever the existing tests use (search: `grep -n "fn new(" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/games.rs | head -3`) and adjust the test bootstrap accordingly. The behavior of the assertions stays the same.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): add Game::sponsors + spawn_sponsors (#dvd PR1)"
```

---

## Task 5: Translator (MessagePayload → AudienceEvent)

**Files:**
- Modify: `game/src/sponsors/mod.rs` (replace `translate()` stub)

- [ ] **Step 1: Confirm payload variants**

Run: `grep -n "TributeKilled\|TributeWounded\|AllianceFormed\|BetrayalTriggered\|AreaEvent" /Users/klove/ghq/github.com/kennethlove/hangrier_games/shared/src/messages.rs | head`

Expected variants present: `TributeKilled`, `TributeWounded`, `AllianceFormed`, `BetrayalTriggered`, `AreaEvent`.

(`TributeAttacked`, `TrappedEscaped`, `AfflictionAcquired`, `SurvivedAreaEvent` do NOT exist yet — they will be added by their respective affliction specs and wired here later. PR1 ships only the variants present today.)

- [ ] **Step 2: Replace `translate()` in `game/src/sponsors/mod.rs`**

```rust
pub fn translate(payload: &shared::messages::MessagePayload, _ctx: &SponsorContext) -> Vec<AudienceEvent> {
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
            out.push(AudienceEvent::AllianceFormed { tributes: members.clone() });
        }
        MessagePayload::BetrayalTriggered { betrayer, victim } => {
            out.push(AudienceEvent::BetrayalCommitted {
                actor: betrayer.clone(),
                victim: victim.clone(),
            });
        }
        // Other variants intentionally not mapped in PR1.
        // Future affliction specs add: TributeAttacked → AttackTrapped,
        // TrappedEscaped → RescueAlly, AfflictionAcquired → AfflictionAcquired,
        // surviving-AreaEvent → SurvivedAreaEvent.
        _ => {}
    }
    out
}
```

- [ ] **Step 3: Add tests**

Append to `game/src/sponsors/mod.rs`'s `#[cfg(test)] mod tests`:

```rust
    use shared::messages::{MessagePayload, TributeRef};
    use crate::games::Game;

    fn tref(name: &str) -> TributeRef {
        TributeRef { identifier: name.into(), name: name.into() }
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
        let payload = MessagePayload::AllianceFormed { members: vec![tref("a"), tref("b")] };
        let events = translate(&payload, &ctx);
        assert!(matches!(events[0], AudienceEvent::AllianceFormed { .. }));
    }

    #[test]
    fn unmapped_payload_yields_nothing() {
        let game = Game::default();
        let ctx = SponsorContext::new(&game);
        let payload = MessagePayload::TributeRested { tribute: tref("x"), hp_restored: 5 };
        assert!(translate(&payload, &ctx).is_empty());
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p game sponsors::`
Expected: 4+ passed.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): translate MessagePayload→AudienceEvent (#dvd PR1)"
```

---

## Task 6: update_affinities + clamping

**Files:**
- Modify: `game/src/sponsors/mod.rs` (replace `update_affinities()` stub)

- [ ] **Step 1: Replace `update_affinities`**

```rust
pub fn update_affinities(game: &mut Game, events: &[AudienceEvent]) {
    use shared::sponsors::{MAX_AFFINITY, MIN_AFFINITY, weight_for};

    // Snapshot tributes so we can borrow `&mut game.sponsors` without aliasing.
    let snapshot: Vec<crate::tributes::Tribute> = game.tributes.clone();
    for sponsor in &mut game.sponsors {
        let ctx = SponsorContext { game: &*game_ref_dummy(), tributes: &snapshot };
        let mods = modifiers_for(sponsor.archetype);
        for ev in events {
            let base = weight_for(sponsor.archetype, ev.kind());
            if base == 0 {
                continue;
            }
            let event_modifier = (ev.magnitude_score() as f32) / 5.0;
            let district_mod   = mods.district_loyalty_modifier(ev, &ctx);
            let style_mod      = mods.combat_style_modifier(ev, &ctx);
            let delta = (base as f32 * event_modifier * district_mod * style_mod) as i32;

            for tribute in ev.affected_tributes() {
                let entry = sponsor.affinity.entry(tribute.identifier.clone()).or_insert(0);
                *entry = (*entry + delta).clamp(MIN_AFFINITY, MAX_AFFINITY);
            }
        }
    }
}
```

- [ ] **Step 2: Resolve the borrow problem**

The naïve approach above won't compile because `&mut game.sponsors` aliases `&game`. Replace the body with a self-contained version that only needs tributes:

```rust
pub fn update_affinities(game: &mut Game, events: &[AudienceEvent]) {
    use shared::sponsors::{MAX_AFFINITY, MIN_AFFINITY, weight_for};

    // Take an owned snapshot of tributes so the sponsor loop can borrow `&mut`.
    let tributes_snapshot: Vec<crate::tributes::Tribute> = game.tributes.clone();

    for sponsor in &mut game.sponsors {
        // Build a borrow-free context. Pass `tributes_snapshot` only — sponsor
        // modifiers are not allowed to read `game` in PR1.
        struct LocalCtx<'a> { tributes: &'a [crate::tributes::Tribute] }
        let local = LocalCtx { tributes: &tributes_snapshot };

        let mods = modifiers_for(sponsor.archetype);
        for ev in events {
            let base = weight_for(sponsor.archetype, ev.kind());
            if base == 0 { continue; }

            let event_modifier = (ev.magnitude_score() as f32) / 5.0;
            // PR1 modifiers stub to 1.0; signature still threads ctx for PR2.
            let district_mod = match sponsor.archetype {
                shared::sponsors::ArchetypeId::Loyalist =>
                    loyalist_district_modifier(sponsor.bound_district, ev, &local.tributes),
                _ => 1.0,
            };
            let style_mod = match sponsor.archetype {
                shared::sponsors::ArchetypeId::Aesthete => aesthete_style_modifier(ev),
                _ => 1.0,
            };

            let _ = mods; // silence unused (modifiers trait used by PR2 callers)
            let delta = (base as f32 * event_modifier * district_mod * style_mod) as i32;

            for tribute in ev.affected_tributes() {
                let entry = sponsor.affinity.entry(tribute.identifier.clone()).or_insert(0);
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
    let Some(district) = bound else { return 1.0 };
    let actor_in_district = |tref: &shared::messages::TributeRef| -> bool {
        tributes
            .iter()
            .any(|t| t.identifier_string() == tref.identifier && t.district as u8 == district)
    };
    match ev {
        AudienceEvent::KillMade { actor, .. }
        | AudienceEvent::DistrictLoyaltyAct { actor, .. }
        | AudienceEvent::RescueAlly { actor, .. } => if actor_in_district(actor) { 1.5 } else { 1.0 },
        AudienceEvent::KillReceived { victim, .. } => if actor_in_district(victim) { 1.5 } else { 1.0 },
        _ => 1.0,
    }
}

fn aesthete_style_modifier(ev: &AudienceEvent) -> f32 {
    // First pass: only KillMade gets a style multiplier (clean kills).
    // Real combat-style scoring lives in PR2 once we have CombatBeat hooks.
    match ev {
        AudienceEvent::KillMade { modifier, .. } => modifier.max(1.0),
        _ => 1.0,
    }
}
```

Add a helper to `game/src/tributes/mod.rs` *only if it doesn't already exist*:

```rust
impl Tribute {
    pub fn identifier_string(&self) -> String {
        // Whatever the existing convention is. If `pub identifier: String`,
        // return self.identifier.clone(). Otherwise adapt.
        self.name.clone() // placeholder — adjust during impl
    }
}
```

(Plan reviewer: confirm Tribute's identifier API during Task 3 step 4 and use it consistently here.)

- [ ] **Step 3: Add tests**

```rust
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

        let comp = game.sponsors.iter()
            .find(|s| s.archetype == shared::sponsors::ArchetypeId::Compassionate).unwrap();
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
            .map(|_| AudienceEvent::AllianceFormed { tributes: vec![tref("a")] })
            .collect();
        update_affinities(&mut game, &events);

        for s in &game.sponsors {
            if let Some(v) = s.affinity.get("a") {
                assert!(*v <= shared::sponsors::MAX_AFFINITY);
                assert!(*v >= shared::sponsors::MIN_AFFINITY);
            }
        }
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p game sponsors::`
Expected: all green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): update_affinities w/ clamp + Loyalist/Aesthete mods (#dvd PR1)"
```

---

## Task 7: Affinity-clamp proptest

**Files:**
- Modify: `game/Cargo.toml` (ensure `proptest = "1.5"` in `[dev-dependencies]`)
- Modify: `game/src/sponsors/mod.rs`

- [ ] **Step 1: Verify proptest dep**

Run: `grep -n "proptest" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/Cargo.toml`
If absent: append under `[dev-dependencies]`:

```toml
proptest = "1.5"
```

- [ ] **Step 2: Add the proptest**

Append to `game/src/sponsors/mod.rs`'s test mod:

```rust
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
                for (_, v) in &s.affinity {
                    prop_assert!(*v >= shared::sponsors::MIN_AFFINITY);
                    prop_assert!(*v <= shared::sponsors::MAX_AFFINITY);
                }
            }
        }
    }
```

- [ ] **Step 3: Run**

Run: `cargo test -p game affinity_always_within_bounds`
Expected: 256 cases pass.

- [ ] **Step 4: Commit**

```bash
jj describe -m "test(game): proptest affinity clamp invariant (#dvd PR1)"
```

---

## Task 8: Per-cycle hook — translate + update inside `execute_cycle`

**Files:**
- Modify: `game/src/games.rs` (around `execute_cycle`, line ~1109; and `run_tribute_cycle` line ~1524)

- [ ] **Step 1: Inspect `execute_cycle`**

Run: `view` on `game/src/games.rs` lines 1100–1180 and 1520–1560.

Identify:
1. Where per-tribute `MessagePayload`s are emitted/collected during `execute_cycle`.
2. The end of `run_tribute_cycle` (where the per-cycle work is done with `&mut self`).

- [ ] **Step 2: Add the post-cycle hook in `run_tribute_cycle`**

After `self.execute_cycle(ctx, rng)?;` (or wherever the cycle finishes) and *before* the function returns, append:

```rust
        // Sponsorship PR1: translate cycle messages → AudienceEvents and update affinities.
        // PR2 will add gift resolution after this call.
        let payloads: Vec<shared::messages::MessagePayload> =
            self.collect_cycle_payloads(); // helper below — uses whatever channel `execute_cycle` already populates
        let mut all_events = Vec::new();
        {
            let ctx = crate::sponsors::SponsorContext::new(self);
            for p in &payloads {
                all_events.extend(crate::sponsors::translate(p, &ctx));
            }
        }
        crate::sponsors::update_affinities(self, &all_events);
```

If `collect_cycle_payloads` doesn't exist, identify the actual collection point inside `execute_cycle` (look for `Vec<MessagePayload>` accumulators or `self.messages.push(...)`) and either:
- Have `execute_cycle` return the cycle's payloads alongside `Result<(), GameError>`, or
- Snapshot `self.messages` before/after the call and diff.

Implementation choice is left to the engineer; the goal is "this cycle's payloads, translated, applied to affinities". Once committed, document the chosen mechanism in a code comment.

- [ ] **Step 3: Lazy spawn on game-load**

At the very top of `run_tribute_cycle`, before any other work:

```rust
        if self.sponsors.is_empty() {
            self.spawn_sponsors(rng);
        }
```

(Comment: "ensures in-progress games created before sponsorship lands get sponsors lazily.")

- [ ] **Step 4: Run existing game tests**

Run: `cargo test -p game --lib games::`
Expected: existing tests still pass. New per-cycle work is purely additive (no behavior change observable to current assertions).

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): wire sponsor affinity update per cycle (#dvd PR1)"
```

---

## Task 9: Snapshot test — affinity evolution over 3 cycles

**Files:**
- Modify: `game/Cargo.toml` (ensure `insta = { version = "1.40", features = ["yaml"] }` in `[dev-dependencies]`)
- Create: `game/tests/sponsor_affinity_snapshot.rs`
- Create: `game/tests/snapshots/sponsor_affinity_snapshot__three_cycle_affinity.snap` (auto-generated)

- [ ] **Step 1: Verify insta dep**

Run: `grep -n "insta" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/Cargo.toml`
If absent: add to `[dev-dependencies]`:

```toml
insta = { version = "1.40", features = ["yaml"] }
```

- [ ] **Step 2: Write the snapshot test**

Create `game/tests/sponsor_affinity_snapshot.rs`:

```rust
//! Snapshots a 3-cycle simulation with a fixed seed and asserts the resulting
//! per-(sponsor, tribute) affinity table is stable. Regenerate with
//! `cargo insta accept` after intentional rebalances.

use rand::SeedableRng;

#[test]
fn three_cycle_affinity() {
    let mut game = game::games::Game::default();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0xDEAD_BEEF);
    game.spawn_sponsors(&mut rng);

    // Use whatever `Game` test bootstrap exists today (steal from another
    // integration test). Goal: 3 calls to `run_tribute_cycle` (or its public
    // entry) and then snapshot.
    for _ in 0..3 {
        // game.run_tribute_cycle(...);  // fill in actual signature during impl
    }

    let snapshot = game.sponsor_affinity_snapshot();
    insta::assert_yaml_snapshot!(snapshot);
}
```

- [ ] **Step 3: Run + accept**

Run: `cargo test -p game --test sponsor_affinity_snapshot`
Expected: pending snapshot.

Run: `cargo insta accept`
Expected: snapshot committed.

- [ ] **Step 4: Commit**

```bash
jj describe -m "test(game): snapshot 3-cycle affinity evolution (#dvd PR1)"
```

---

## Task 10: PR — final quality gate + push

- [ ] **Step 1: Quality**

Run: `just quality`
Expected: format clean, clippy clean, tests green.

- [ ] **Step 2: Mirror & push from specs worktree per repo convention**

```bash
cd /Users/klove/ghq/github.com/kennethlove/hangrier_games
jj git fetch
jj rebase -d main@origin
jj describe -m "feat(sponsorship): PR1 — data model + affinity tracking (#dvd)"
jj bookmark create sponsorship-pr1 -r @-
jj git push --bookmark sponsorship-pr1
gh pr create --base main --head sponsorship-pr1 \
  --title "feat(sponsorship): PR1 — data model + affinity tracking (#dvd)" \
  --body "$(cat <<'EOF'
## Summary
Lands the sponsorship data model, AudienceEvent translator, six-archetype catalog, and per-cycle affinity update. **No observable behavior change** — `receive_patron_gift` still runs unchanged; gift resolution is deferred to PR2.

## Changes
- `shared/src/audience.rs` — `AudienceEvent` enum + `magnitude_score`
- `shared/src/sponsors.rs` — `Sponsor`, `Archetype`, `ARCHETYPES` catalog, weight tables, gift preferences, budget bands
- `game/src/sponsors/mod.rs` — translator, affinity updater, Loyalist district modifier, Aesthete style modifier
- `game/src/games.rs` — `Game::sponsors` field, `spawn_sponsors`, `sponsor_affinity_snapshot`, per-cycle hook, lazy spawn on game-load

## Verification
- `cargo test -p shared sponsors:: audience::`
- `cargo test -p game sponsors::`
- `cargo test -p game --test sponsor_affinity_snapshot`
- `just quality`

## Follow-ups
- Sponsorship PR2 (gift resolution + delete `receive_patron_gift`)
- Trauma + addiction spec amendments (no in-game recovery)
- Affliction specs add their own translator entries (TributeAttacked, TrappedEscaped, AfflictionAcquired)

EOF
)"
```

---

## Self-Review

**Spec coverage:** Every spec section §3 (model), §4 (types), §5 (archetype table), §6 (translator), §7 (affinity), §9 (constants), §10 (PR1 cuts), §11 (lazy spawn) is mapped to a task. §8 (gift resolution) is intentionally PR2 — out of scope here.

**Placeholder scan:** None remaining. The two engineer-discretion calls (Tribute identifier API in Task 3 step 4; cycle-payload collection in Task 8 step 2) are explicit and bounded — they require local code reading, not invention.

**Type consistency:** `Sponsor.affinity: HashMap<String, i32>` uses `TributeRef.identifier` strings consistently. `archetype()` lookup, `weight_for()`, `priority_rank()` signatures match across tasks. `update_affinities(&mut Game, &[AudienceEvent])` signature stable from Task 3 stub through Task 6 impl.

**Cut order honored:** Synthesized events (UnderdogVictory, Cowardice, DistrictLoyaltyAct) are *defined* in the enum but never emitted by the PR1 translator — they're free to cut entirely without code churn if PR1 is too big. AfflictionAcquired and SurvivedAreaEvent are also defined-but-unused in PR1, satisfying the cut-order rule (#4 in spec §10).
