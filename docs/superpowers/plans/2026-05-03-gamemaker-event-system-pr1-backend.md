# Gamemaker Event System PR1 — Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the gamemaker actor (gauges + profile + 6 intervention variants + 11 typed events + active-effect lifecycle + brain integration) as a backend-only PR. No frontend changes.

**Architecture:** New `game/src/gamemaker/` module hangs off `Game.gamemaker: Gamemaker`. Per-phase decision flow runs from `run_day_night_cycle`. Each `InterventionKind` is a small file implementing a `score`/`target_pref`/`resolve` trio. `ActiveIntervention` carries persistent state for mutts/closures/convergence. All emissions are typed `MessagePayload` variants; no stringly-typed fallbacks. Independent of the pending weather refactor — uses a local `Weather` stub if shelter PR1 hasn't shipped.

**Tech Stack:** Rust 2024, rstest for parametric tests, serde for persistence, `chrono`/`uuid` already in scope, `rand::SmallRng` for determinism. No new crate dependencies.

**Spec:** `docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md`

---

## File Structure

**Created:**

- `game/src/gamemaker/mod.rs` — module root; `Gamemaker` struct + `RecentIntervention`
- `game/src/gamemaker/gauges.rs` — `Gauges` struct + tick/reaction tables
- `game/src/gamemaker/profile.rs` — `GamemakerProfile` struct + `CASSANDRA` const
- `game/src/gamemaker/decision.rs` — `should_intervene`, variant selection, targeting fallback loop
- `game/src/gamemaker/active.rs` — `ActiveIntervention` enum + per-phase tick/resolve
- `game/src/gamemaker/weather_stub.rs` — local `Weather` enum (only if shelter PR1 not merged)
- `game/src/gamemaker/interventions/mod.rs` — `InterventionKind` enum + `InterventionLogic` trait
- `game/src/gamemaker/interventions/fireball.rs`
- `game/src/gamemaker/interventions/mutt_pack.rs`
- `game/src/gamemaker/interventions/force_field.rs`
- `game/src/gamemaker/interventions/area_closure.rs`
- `game/src/gamemaker/interventions/convergence.rs`
- `game/src/gamemaker/interventions/weather_override.rs`
- `game/tests/gamemaker_integration.rs`

**Modified:**

- `game/src/lib.rs` — `pub mod gamemaker;`
- `game/src/games.rs` — `Game.gamemaker: Gamemaker` field + Default + integration into `run_day_night_cycle`
- `game/src/tributes/brain.rs` (or wherever `choose_destination` / action selection lives) — convergence pull, mutt avoidance, sealed-area filter
- `shared/src/messages.rs` — 11 new `MessagePayload` variants + `Lure` enum + `DespawnReason` enum + 3 cause constants

---

## Conventions

- **TDD throughout.** Failing test → run → minimal impl → run → commit.
- **Commit message:** `feat(gamemaker): <task summary>` for new code; `feat(shared): <summary>` for shared crate; `feat(game): <summary>` for game-crate integration touch points.
- **Test command (game crate):** `cargo test --package game gamemaker`
- **Test command (specific test):** `cargo test --package game gamemaker -- <test_name> --exact --nocapture`
- **Run after every task:** `just fmt && cargo check --workspace`
- **Use `SmallRng::seed_from_u64(N)` in tests** for determinism. Snapshot ranges (e.g., `assert!(count >= 28 && count <= 60)`) over exact equality where rng-driven.

---

## Task 1: Module skeleton + Weather stub coordination

**Files:**
- Create: `game/src/gamemaker/mod.rs`
- Create: `game/src/gamemaker/weather_stub.rs`
- Modify: `game/src/lib.rs`

**Goal:** Empty module compiles; `Weather` enum is reachable from inside `gamemaker::` regardless of whether shelter PR1 has shipped.

- [ ] **Step 1: Check whether shelter PR1's `Weather` exists.** Run:

```bash
grep -rn "pub enum Weather" game/src/ | head -3
```

If a `pub enum Weather { Clear, HeavyRain, Heatwave, Blizzard, ... }` is found in `game/src/`, **skip the stub file** and `pub use` from that location instead in Step 3 below.

If nothing is found, proceed with the stub.

- [ ] **Step 2: Create `game/src/gamemaker/weather_stub.rs`** (only if Step 1 found nothing):

```rust
//! Local `Weather` wedge. Removed when shelter PR1 (or weather spec) lands its
//! own `Weather` enum in `game/src/weather.rs` or similar.
//!
//! See `docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md`,
//! "Coexistence with current `AreaEvent`" section.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    HeavyRain,
    Heatwave,
    Blizzard,
}

impl Default for Weather {
    fn default() -> Self {
        Weather::Clear
    }
}

/// Stub: always returns `Weather::Clear`. Replaced by real producer when
/// weather spec lands.
pub fn current_weather() -> Weather {
    Weather::Clear
}
```

- [ ] **Step 3: Create `game/src/gamemaker/mod.rs`:**

```rust
//! Gamemaker / Capitol intervention system.
//!
//! See `docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md`.

pub mod active;
pub mod decision;
pub mod gauges;
pub mod interventions;
pub mod profile;

#[cfg(not(feature = "shelter_weather"))]
pub mod weather_stub;

#[cfg(not(feature = "shelter_weather"))]
pub use weather_stub::Weather;

#[cfg(feature = "shelter_weather")]
pub use crate::weather::Weather;

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

pub use active::ActiveIntervention;
pub use decision::should_intervene;
pub use gauges::Gauges;
pub use interventions::{InterventionKind, Lure};
pub use profile::{CASSANDRA, GamemakerProfile};

/// Maximum recent-interventions window the dispatcher inspects when
/// applying `recent_penalty`.
pub const RECENT_WINDOW: usize = 6;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RecentIntervention {
    pub kind: InterventionKind,
    pub phase_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gamemaker {
    pub profile: GamemakerProfile,
    pub gauges: Gauges,
    #[serde(default)]
    pub recent_interventions: VecDeque<RecentIntervention>,
    #[serde(default)]
    pub interventions_today: u8,
    #[serde(default)]
    pub active_effects: Vec<ActiveIntervention>,
}

impl Default for Gamemaker {
    fn default() -> Self {
        Self {
            profile: CASSANDRA,
            gauges: Gauges::STARTING,
            recent_interventions: VecDeque::with_capacity(RECENT_WINDOW),
            interventions_today: 0,
            active_effects: Vec::new(),
        }
    }
}

impl Gamemaker {
    pub fn new(profile: GamemakerProfile) -> Self {
        Self {
            profile,
            gauges: Gauges::STARTING,
            recent_interventions: VecDeque::with_capacity(RECENT_WINDOW),
            interventions_today: 0,
            active_effects: Vec::new(),
        }
    }

    /// Records a freshly-fired intervention and trims the window.
    pub fn record_intervention(&mut self, kind: InterventionKind, phase_index: u32) {
        self.recent_interventions.push_back(RecentIntervention { kind, phase_index });
        while self.recent_interventions.len() > RECENT_WINDOW {
            self.recent_interventions.pop_front();
        }
        self.interventions_today = self.interventions_today.saturating_add(1);
        self.gauges.patience = 0;
    }

    /// Counts how many times `kind` appears in the recent-interventions window.
    pub fn recent_count(&self, kind: InterventionKind) -> u32 {
        self.recent_interventions
            .iter()
            .filter(|r| r.kind == kind)
            .count() as u32
    }
}
```

- [ ] **Step 4: Add `pub mod gamemaker;` to `game/src/lib.rs`** (placed alphabetically with other modules).

- [ ] **Step 5: Stub the sub-modules so the workspace compiles.** Create empty `gauges.rs`, `profile.rs`, `decision.rs`, `active.rs`, and `interventions/mod.rs` with just `// task N` placeholders for now — they'll be filled in subsequent tasks. Minimal stubs:

```rust
// game/src/gamemaker/gauges.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gauges {
    pub drama_pressure: u8,
    pub audience_attention: u8,
    pub bloodthirst: u8,
    pub chaos: u8,
    pub patience: u8,
    pub body_count_debt: u8,
}

impl Gauges {
    pub const STARTING: Self = Self {
        drama_pressure: 0,
        audience_attention: 80,
        bloodthirst: 20,
        chaos: 10,
        patience: 0,
        body_count_debt: 0,
    };
}
```

```rust
// game/src/gamemaker/profile.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GamemakerProfile {
    pub name: &'static str,
    pub pressure_decay_rate: u8,
    pub attention_decay_rate: u8,
    pub bloodthirst_weight: u8,
    pub chaos_weight: u8,
    pub patience_threshold: u8,
    pub max_per_day: u8,
    pub max_concurrent_events: u8,
    pub late_game_multiplier: f32,
    pub recent_penalty: u32,
}

pub const CASSANDRA: GamemakerProfile = GamemakerProfile {
    name: "Cassandra",
    pressure_decay_rate: 8,
    attention_decay_rate: 3,
    bloodthirst_weight: 100,
    chaos_weight: 100,
    patience_threshold: 30,
    max_per_day: 2,
    max_concurrent_events: 2,
    late_game_multiplier: 1.5,
    recent_penalty: 25,
};
```

```rust
// game/src/gamemaker/active.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActiveIntervention {
    // Filled in Task 9
    Placeholder,
}
```

```rust
// game/src/gamemaker/decision.rs
use crate::gamemaker::Gamemaker;

pub fn should_intervene(_g: &Gamemaker) -> bool {
    false // Filled in Task 6
}
```

```rust
// game/src/gamemaker/interventions/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterventionKind {
    Fireball,
    MuttPack,
    ForceFieldShift,
    AreaClosure,
    ConvergencePoint,
    WeatherOverride,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Lure {
    Feast,
}
```

- [ ] **Step 6: Run `cargo check --workspace` and `just fmt`.** Expected: clean build, no warnings about unused enums (the variants are public so they're fine).

- [ ] **Step 7: Commit:**

```bash
jj describe -m "feat(gamemaker): module skeleton with Gamemaker + Weather stub (hangrier_games-5q9)"
jj new
```

---

## Task 2: Wire `Gamemaker` into `Game`

**Files:**
- Modify: `game/src/games.rs:90-125` (Game struct and Default)
- Test: inline in `game/src/gamemaker/mod.rs`

- [ ] **Step 1: Write failing test in `game/src/gamemaker/mod.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamemaker_default_uses_cassandra_starting_gauges() {
        let gm = Gamemaker::default();
        assert_eq!(gm.profile.name, "Cassandra");
        assert_eq!(gm.gauges, Gauges::STARTING);
        assert_eq!(gm.interventions_today, 0);
        assert!(gm.recent_interventions.is_empty());
        assert!(gm.active_effects.is_empty());
    }

    #[test]
    fn record_intervention_caps_window_and_resets_patience() {
        let mut gm = Gamemaker::default();
        gm.gauges.patience = 50;
        for i in 0..(RECENT_WINDOW as u32 + 3) {
            gm.record_intervention(InterventionKind::Fireball, i);
        }
        assert_eq!(gm.recent_interventions.len(), RECENT_WINDOW);
        assert_eq!(gm.interventions_today, RECENT_WINDOW as u8 + 3);
        assert_eq!(gm.gauges.patience, 0);
        assert_eq!(gm.recent_count(InterventionKind::Fireball), RECENT_WINDOW as u32);
    }

    #[test]
    fn game_has_default_gamemaker() {
        let game = crate::games::Game::default();
        assert_eq!(game.gamemaker.profile.name, "Cassandra");
    }
}
```

- [ ] **Step 2: Run test:**

```bash
cargo test --package game gamemaker:: -- --nocapture
```

Expected: third test fails — `Game` has no `gamemaker` field.

- [ ] **Step 3: Add field to `Game` struct in `game/src/games.rs`** (after `pub emit_index: u32,` near line ~125):

```rust
    /// Capitol intervention actor. See spec
    /// `2026-05-03-gamemaker-event-system-design.md`.
    #[serde(default)]
    pub gamemaker: crate::gamemaker::Gamemaker,
```

- [ ] **Step 4: Add to `Default` impl** (around line ~155, with other field initializers):

```rust
            gamemaker: crate::gamemaker::Gamemaker::default(),
```

- [ ] **Step 5: Run tests:**

```bash
cargo test --package game gamemaker:: -- --nocapture
```

Expected: all three pass.

- [ ] **Step 6: Run full game test suite to confirm nothing broke:**

```bash
just test
```

Expected: pass (allowing pre-existing unrelated failures).

- [ ] **Step 7: Commit:**

```bash
jj describe -m "feat(gamemaker): attach Gamemaker to Game with serde default"
jj new
```

---

## Task 3: Gauge tick — background per-phase rises/decays

**Files:**
- Modify: `game/src/gamemaker/gauges.rs`

Implements the per-phase background tick (rises before any per-event reaction). Late-game multiplier applied here when caller passes `alive_tributes <= 8`.

- [ ] **Step 1: Write failing test in `game/src/gamemaker/gauges.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    #[test]
    fn tick_phase_raises_pressure_and_drops_attention() {
        let mut g = Gauges::STARTING;
        g.tick_phase(&CASSANDRA, /*alive_tributes=*/ 16);
        assert_eq!(g.drama_pressure, 8);
        assert_eq!(g.audience_attention, 77); // 80 - 3
        assert_eq!(g.patience, 1);
    }

    #[test]
    fn tick_phase_clamps_pressure_at_100() {
        let mut g = Gauges::STARTING;
        g.drama_pressure = 95;
        g.tick_phase(&CASSANDRA, 16);
        assert_eq!(g.drama_pressure, 100);
    }

    #[test]
    fn tick_phase_late_game_multiplier_amplifies_pressure_rise() {
        let mut g = Gauges::STARTING;
        g.tick_phase(&CASSANDRA, /*alive_tributes=*/ 5);
        // 8 * 1.5 = 12
        assert_eq!(g.drama_pressure, 12);
    }

    #[test]
    fn tick_phase_attention_clamps_at_zero() {
        let mut g = Gauges::STARTING;
        g.audience_attention = 1;
        g.tick_phase(&CASSANDRA, 16);
        assert_eq!(g.audience_attention, 0);
        // tick again
        g.tick_phase(&CASSANDRA, 16);
        assert_eq!(g.audience_attention, 0);
    }

    #[test]
    fn tick_phase_patience_saturates() {
        let mut g = Gauges::STARTING;
        g.patience = 254;
        g.tick_phase(&CASSANDRA, 16);
        assert_eq!(g.patience, 255);
        g.tick_phase(&CASSANDRA, 16);
        assert_eq!(g.patience, 255);
    }
}
```

- [ ] **Step 2: Run:**

```bash
cargo test --package game gamemaker::gauges -- --nocapture
```

Expected: all five fail — `tick_phase` does not exist.

- [ ] **Step 3: Implement in `gauges.rs`** (append after the struct):

```rust
use crate::gamemaker::profile::GamemakerProfile;

const LATE_GAME_THRESHOLD: u32 = 8;

impl Gauges {
    /// Per-phase background tick: rises pressure, decays attention, increments patience.
    /// Applies `profile.late_game_multiplier` to pressure rise when `alive_tributes <= 8`.
    pub fn tick_phase(&mut self, profile: &GamemakerProfile, alive_tributes: u32) {
        let mult = if alive_tributes <= LATE_GAME_THRESHOLD {
            profile.late_game_multiplier
        } else {
            1.0
        };
        let rise = (profile.pressure_decay_rate as f32 * mult).round() as u16;
        self.drama_pressure = self.drama_pressure.saturating_add(rise.min(255) as u8).min(100);

        self.audience_attention = self.audience_attention.saturating_sub(profile.attention_decay_rate);
        self.patience = self.patience.saturating_add(1);
    }
}
```

- [ ] **Step 4: Run tests:**

```bash
cargo test --package game gamemaker::gauges -- --nocapture
```

Expected: all pass.

- [ ] **Step 5: Commit:**

```bash
jj describe -m "feat(gamemaker): per-phase gauge tick with late-game multiplier"
jj new
```

---

## Task 4: Gauge per-event reactions

**Files:**
- Modify: `game/src/gamemaker/gauges.rs`

Implements `react_to(...)` matching the per-event table in the spec.

- [ ] **Step 1: Write failing tests:**

```rust
#[cfg(test)]
mod react_tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    fn baseline() -> Gauges {
        Gauges {
            drama_pressure: 50,
            audience_attention: 50,
            bloodthirst: 50,
            chaos: 50,
            patience: 10,
            body_count_debt: 20,
        }
    }

    #[test]
    fn tribute_killed_decays_pressure_and_bloodthirst_boosts_attention() {
        let mut g = baseline();
        g.react_to(&GaugeReaction::TributeKilled);
        assert_eq!(g.drama_pressure, 35); // -15
        assert_eq!(g.audience_attention, 62); // +12
        assert_eq!(g.bloodthirst, 30); // -20
        assert_eq!(g.chaos, 45); // -5
        assert_eq!(g.body_count_debt, 10); // -10
    }

    #[test]
    fn hazard_no_kill_softer_drop() {
        let mut g = baseline();
        g.react_to(&GaugeReaction::HazardNoKill);
        assert_eq!(g.drama_pressure, 42); // -8
        assert_eq!(g.audience_attention, 55); // +5
        assert_eq!(g.chaos, 40); // -10
    }

    #[test]
    fn day_boundary_no_deaths_amplifies_pressure_and_bloodthirst() {
        let mut g = baseline();
        g.react_to(&GaugeReaction::DayBoundaryNoDeaths);
        assert_eq!(g.drama_pressure, 70); // +20
        assert_eq!(g.bloodthirst, 75); // +25
        assert_eq!(g.body_count_debt, 25); // +5
    }

    #[test]
    fn intervention_kill_resets_patience_and_drops_drama() {
        let mut g = baseline();
        g.react_to(&GaugeReaction::InterventionResolvedWithKill);
        assert_eq!(g.drama_pressure, 20); // -30
        assert_eq!(g.audience_attention, 70); // +20
        assert_eq!(g.bloodthirst, 20); // -30
        assert_eq!(g.chaos, 40); // -10
    }

    #[test]
    fn weather_override_reaction_is_cheap() {
        let mut g = baseline();
        g.react_to(&GaugeReaction::WeatherOverride);
        assert_eq!(g.drama_pressure, 45); // -5
        assert_eq!(g.chaos, 45); // -5
    }
}
```

- [ ] **Step 2: Run tests; expected: all fail (`GaugeReaction` undefined).**

- [ ] **Step 3: Implement in `gauges.rs`:**

```rust
/// Discrete event signals the gamemaker reacts to. Each maps to a row of
/// the per-event reaction table in the spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaugeReaction {
    TributeKilled,
    HazardNoKill,
    HazardWithKill,
    CombatRoundNoKill,
    DayBoundaryDebtAccrued { expected: u32, actual: u32 },
    DayBoundaryNoDeaths,
    InterventionResolvedWithKill,
    InterventionResolvedNoKill,
    InterventionDisruptive,        // ForceFieldShift, AreaClosure
    InterventionConvergenceAnnounce,
    WeatherOverride,
}

impl Gauges {
    pub fn react_to(&mut self, r: &GaugeReaction) {
        match r {
            GaugeReaction::TributeKilled => {
                self.drama_pressure = self.drama_pressure.saturating_sub(15);
                self.audience_attention = (self.audience_attention.saturating_add(12)).min(100);
                self.bloodthirst = self.bloodthirst.saturating_sub(20);
                self.chaos = self.chaos.saturating_sub(5);
                self.body_count_debt = self.body_count_debt.saturating_sub(10);
            }
            GaugeReaction::HazardNoKill => {
                self.drama_pressure = self.drama_pressure.saturating_sub(8);
                self.audience_attention = (self.audience_attention.saturating_add(5)).min(100);
                self.bloodthirst = self.bloodthirst.saturating_sub(2);
                self.chaos = self.chaos.saturating_sub(10);
            }
            GaugeReaction::HazardWithKill => {
                self.drama_pressure = self.drama_pressure.saturating_sub(15);
                self.audience_attention = (self.audience_attention.saturating_add(12)).min(100);
                self.bloodthirst = self.bloodthirst.saturating_sub(20);
                self.chaos = self.chaos.saturating_sub(15);
                self.body_count_debt = self.body_count_debt.saturating_sub(10);
            }
            GaugeReaction::CombatRoundNoKill => {
                self.drama_pressure = self.drama_pressure.saturating_sub(3);
                self.audience_attention = (self.audience_attention.saturating_add(2)).min(100);
                self.bloodthirst = (self.bloodthirst.saturating_add(2)).min(100);
                self.chaos = self.chaos.saturating_sub(2);
            }
            GaugeReaction::DayBoundaryDebtAccrued { expected, actual } => {
                let delta = expected.saturating_sub(*actual);
                self.bloodthirst = (self.bloodthirst.saturating_add(10)).min(100);
                let bump = (delta * 5).min(100) as u8;
                self.body_count_debt = (self.body_count_debt.saturating_add(bump)).min(100);
            }
            GaugeReaction::DayBoundaryNoDeaths => {
                self.drama_pressure = (self.drama_pressure.saturating_add(20)).min(100);
                self.bloodthirst = (self.bloodthirst.saturating_add(25)).min(100);
                self.body_count_debt = (self.body_count_debt.saturating_add(5)).min(100);
            }
            GaugeReaction::InterventionResolvedWithKill => {
                self.drama_pressure = self.drama_pressure.saturating_sub(30);
                self.audience_attention = (self.audience_attention.saturating_add(20)).min(100);
                self.bloodthirst = self.bloodthirst.saturating_sub(30);
                self.chaos = self.chaos.saturating_sub(10);
            }
            GaugeReaction::InterventionResolvedNoKill => {
                self.drama_pressure = self.drama_pressure.saturating_sub(15);
                self.audience_attention = (self.audience_attention.saturating_add(5)).min(100);
                self.bloodthirst = self.bloodthirst.saturating_sub(10);
                self.chaos = self.chaos.saturating_sub(5);
            }
            GaugeReaction::InterventionDisruptive => {
                self.drama_pressure = self.drama_pressure.saturating_sub(10);
                self.audience_attention = (self.audience_attention.saturating_add(5)).min(100);
                self.chaos = self.chaos.saturating_sub(20);
            }
            GaugeReaction::InterventionConvergenceAnnounce => {
                self.drama_pressure = self.drama_pressure.saturating_sub(5);
                self.audience_attention = (self.audience_attention.saturating_add(3)).min(100);
            }
            GaugeReaction::WeatherOverride => {
                self.drama_pressure = self.drama_pressure.saturating_sub(5);
                self.chaos = self.chaos.saturating_sub(5);
            }
        }
    }
}
```

- [ ] **Step 4: Run; expected pass.**

- [ ] **Step 5: Commit:**

```bash
jj describe -m "feat(gamemaker): per-event gauge reactions"
jj new
```

---

## Task 5: Shared payloads — Lure, DespawnReason, cause constants, 11 MessagePayload variants

**Files:**
- Modify: `shared/src/messages.rs`

This is a single fat task because all 11 variants need to land together; tests in subsequent tasks reference them.

- [ ] **Step 1: Write failing tests in `shared/src/messages.rs`** (append to existing `#[cfg(test)] mod tests` block, or create one):

```rust
#[cfg(test)]
mod gamemaker_payload_tests {
    use super::*;

    #[test]
    fn lure_feast_serializes_round_trip() {
        let l = Lure::Feast;
        let json = serde_json::to_string(&l).unwrap();
        let back: Lure = serde_json::from_str(&json).unwrap();
        assert_eq!(l, back);
    }

    #[test]
    fn despawn_reasons_round_trip() {
        for r in [DespawnReason::Morning, DespawnReason::NoTargetsNearby, DespawnReason::NoMembersLeft] {
            let json = serde_json::to_string(&r).unwrap();
            let back: DespawnReason = serde_json::from_str(&json).unwrap();
            assert_eq!(r, back);
        }
    }

    #[test]
    fn fireball_strike_round_trip() {
        let p = MessagePayload::FireballStrike {
            area: AreaRef { id: "a1".into(), name: "Forest".into() },
            severity_label: "Major".into(),
            victims: vec![],
            survivors: vec![],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: MessagePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn cause_constants_present() {
        assert_eq!(CAUSE_FIREBALL, "fireball");
        assert_eq!(CAUSE_MUTT_PACK, "mutt_pack");
        assert_eq!(CAUSE_AREA_SEAL, "area_seal");
    }
}
```

- [ ] **Step 2: Run; expected fail.**

```bash
cargo test --package shared gamemaker_payload -- --nocapture
```

- [ ] **Step 3: Add to `shared/src/messages.rs`** — insert the new types and variants. Place `Lure`, `DespawnReason`, and the cause constants near the top of the file (after the existing `use` block); insert the 11 new variants into the `MessagePayload` enum (alphabetically grouped is fine; spec section "Events / Messages" lists them):

```rust
// === Cause constants for TributeKilled.cause ===
pub const CAUSE_FIREBALL: &str = "fireball";
pub const CAUSE_MUTT_PACK: &str = "mutt_pack";
pub const CAUSE_AREA_SEAL: &str = "area_seal";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lure {
    Feast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DespawnReason {
    Morning,
    NoTargetsNearby,
    NoMembersLeft,
}
```

Then add the 11 variants to `pub enum MessagePayload`. Use `severity_label: String` (free-form, since `EventSeverity` lives in `game/` not `shared/`) and `kind_label: String` for animal kind (game crate stringifies it). Keeps shared crate independent of game crate types.

```rust
    // --- Gamemaker: lethal ---
    FireballStrike {
        area: AreaRef,
        severity_label: String,
        victims: Vec<TributeRef>,
        survivors: Vec<TributeRef>,
    },
    MuttSwarmSpawned {
        area: AreaRef,
        kind_label: String,
        members: u8,
    },
    MuttSwarmAttack {
        area: AreaRef,
        kind_label: String,
        victim: TributeRef,
        damage: u32,
        killed: bool,
    },
    MuttSwarmDespawned {
        area: AreaRef,
        kind_label: String,
        reason: DespawnReason,
    },

    // --- Gamemaker: disruptive ---
    ForceFieldShifted {
        closed: Vec<AreaRef>,
        opened: Vec<AreaRef>,
        warning_phases: u8,
    },
    AreaSealed {
        area: AreaRef,
        expires_at_phase: u32,
    },
    AreaUnsealed {
        area: AreaRef,
    },
    AreaSealEntryDamage {
        area: AreaRef,
        tribute: TributeRef,
        damage: u32,
    },

    // --- Gamemaker: convergence ---
    ConvergencePointAnnounced {
        area: AreaRef,
        lure: Lure,
        starts_at_phase: u32,
    },
    ConvergencePointExpired {
        area: AreaRef,
        lure: Lure,
        claimed_by: Vec<TributeRef>,
    },

    // --- Gamemaker: atmospheric ---
    WeatherOverridden {
        area: AreaRef,
        weather_label: String,
        duration_phases: u8,
    },
```

- [ ] **Step 4: Run shared tests:**

```bash
cargo test --package shared -- --nocapture
```

Expected: all four new tests pass.

- [ ] **Step 5: Run workspace check (the new `MessagePayload` arms may break exhaustive matches in `web/` or `api/`):**

```bash
cargo check --workspace
```

If matches break, add `MessagePayload::FireballStrike { .. } | MessagePayload::MuttSwarmSpawned { .. } | ... => { /* TODO PR2 */ }` arms to any non-exhaustive matches in `web/` or `api/`. **Do NOT delete or alter existing arms.** Locate via:

```bash
grep -rn "match.*MessagePayload" web/src api/src
```

Common touch-points: `web/src/components/timeline/cards/`, `api/src/routes/events.rs`. Add a generic fallback arm that returns nothing visible (PR2 wires real rendering).

- [ ] **Step 6: Re-run `cargo check --workspace`; expected: clean.**

- [ ] **Step 7: Commit:**

```bash
jj describe -m "feat(shared): add 11 gamemaker MessagePayload variants + Lure + DespawnReason"
jj new
```

---

## Task 6: `should_intervene` — eligibility gate

**Files:**
- Modify: `game/src/gamemaker/decision.rs`

- [ ] **Step 1: Failing tests in `decision.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::{Gamemaker, Gauges};
    use crate::gamemaker::profile::CASSANDRA;

    fn fresh() -> Gamemaker {
        Gamemaker::default()
    }

    #[test]
    fn no_intervene_when_patience_below_threshold() {
        let mut gm = fresh();
        gm.gauges = Gauges {
            patience: 10, // below 30
            drama_pressure: 99,
            bloodthirst: 99,
            chaos: 99,
            body_count_debt: 99,
            ..Gauges::STARTING
        };
        assert!(!should_intervene(&gm, /*active_count=*/ 0));
    }

    #[test]
    fn no_intervene_when_per_day_cap_hit() {
        let mut gm = fresh();
        gm.gauges.patience = 30;
        gm.gauges.drama_pressure = 99;
        gm.interventions_today = CASSANDRA.max_per_day;
        assert!(!should_intervene(&gm, 0));
    }

    #[test]
    fn no_intervene_when_concurrent_cap_hit() {
        let mut gm = fresh();
        gm.gauges.patience = 30;
        gm.gauges.drama_pressure = 99;
        assert!(!should_intervene(&gm, CASSANDRA.max_concurrent_events as usize));
    }

    #[test]
    fn no_intervene_when_no_threshold_crossed() {
        let mut gm = fresh();
        gm.gauges.patience = 30;
        // all gauges below their thresholds
        assert!(!should_intervene(&gm, 0));
    }

    #[test]
    fn intervene_when_pressure_above_60() {
        let mut gm = fresh();
        gm.gauges.patience = 30;
        gm.gauges.drama_pressure = 60;
        assert!(should_intervene(&gm, 0));
    }

    #[test]
    fn intervene_when_bloodthirst_above_70() {
        let mut gm = fresh();
        gm.gauges.patience = 30;
        gm.gauges.bloodthirst = 70;
        assert!(should_intervene(&gm, 0));
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Implement in `decision.rs`** (replacing the placeholder):

```rust
use crate::gamemaker::Gamemaker;

pub const PRESSURE_TRIGGER: u8 = 60;
pub const BLOODTHIRST_TRIGGER: u8 = 70;
pub const CHAOS_TRIGGER: u8 = 70;
pub const BODY_COUNT_DEBT_TRIGGER: u8 = 10;

pub fn should_intervene(gm: &Gamemaker, active_count: usize) -> bool {
    if gm.interventions_today >= gm.profile.max_per_day {
        return false;
    }
    if active_count >= gm.profile.max_concurrent_events as usize {
        return false;
    }
    if gm.gauges.patience < gm.profile.patience_threshold {
        return false;
    }
    let g = &gm.gauges;
    g.drama_pressure >= PRESSURE_TRIGGER
        || g.bloodthirst >= BLOODTHIRST_TRIGGER
        || g.chaos >= CHAOS_TRIGGER
        || g.body_count_debt >= BODY_COUNT_DEBT_TRIGGER
}
```

- [ ] **Step 4: Run; expected pass.**

- [ ] **Step 5: Commit:**

```bash
jj describe -m "feat(gamemaker): should_intervene eligibility gate"
jj new
```

---

## Task 7: `InterventionLogic` trait + `GameSnapshot` view

**Files:**
- Modify: `game/src/gamemaker/interventions/mod.rs`

`GameSnapshot` is a read-only view passed into scoring/targeting so per-variant logic stays a pure function. Building it from `&Game` happens in Task 14 (integration).

- [ ] **Step 1: Failing test:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_default_has_zero_alive() {
        let snap = GameSnapshot::default();
        assert_eq!(snap.alive_tributes, 0);
    }

    #[test]
    fn target_spec_single_area_eq() {
        let a = TargetSpec::SingleArea(AreaId("forest".into()));
        let b = TargetSpec::SingleArea(AreaId("forest".into()));
        assert_eq!(a, b);
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Implement in `interventions/mod.rs`:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterventionKind {
    Fireball,
    MuttPack,
    ForceFieldShift,
    AreaClosure,
    ConvergencePoint,
    WeatherOverride,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Lure {
    Feast,
}

/// Stable identifier for an area in the per-phase decision pipeline.
/// Wrapped so tests can construct one without the `Area` enum.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AreaId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSpec {
    SingleArea(AreaId),
    AreaSet { close: Vec<AreaId>, open: Vec<AreaId> },
}

/// Derived snapshot of game state passed to intervention scoring/targeting.
/// Built once per phase by the dispatcher.
#[derive(Clone, Debug, Default)]
pub struct GameSnapshot {
    pub alive_tributes: u32,
    pub current_phase_index: u32,
    /// (area_id, alive_tribute_count, adjacent_area_ids)
    pub areas: Vec<AreaSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct AreaSnapshot {
    pub id: AreaId,
    pub tribute_count: u32,
    pub adjacent_ids: Vec<AreaId>,
    pub is_open: bool,
    pub has_active_intervention: bool,
}

pub trait InterventionLogic {
    fn kind(&self) -> InterventionKind;
    fn score(
        &self,
        profile: &crate::gamemaker::profile::GamemakerProfile,
        gauges: &crate::gamemaker::gauges::Gauges,
        snap: &GameSnapshot,
    ) -> u32;
    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec>;
}

pub mod area_closure;
pub mod convergence;
pub mod fireball;
pub mod force_field;
pub mod mutt_pack;
pub mod weather_override;
```

- [ ] **Step 4: Create empty stub files** (one-line `// Task N` placeholders) for each listed sub-module, so `cargo check` passes:

```bash
for f in fireball mutt_pack force_field area_closure convergence weather_override; do
  printf '// Implemented in Task 8\n' > game/src/gamemaker/interventions/$f.rs
done
```

(Run from the `hangrier_games` crate root, not the specs worktree.)

- [ ] **Step 5: Run tests:**

```bash
cargo test --package game gamemaker::interventions -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit:**

```bash
jj describe -m "feat(gamemaker): InterventionLogic trait + GameSnapshot + AreaId"
jj new
```

---

## Task 8: Six intervention scorers + targeting

**Files:**
- Modify: `game/src/gamemaker/interventions/{fireball,mutt_pack,force_field,area_closure,convergence,weather_override}.rs`

Each variant gets a unit struct, `impl InterventionLogic`, and one or two unit tests. Numbers from the spec's "Variant Scoring & Targeting" sketch.

### 8a. Fireball

- [ ] **Step 1: Write `game/src/gamemaker/interventions/fireball.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{AreaId, GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct Fireball;

impl InterventionLogic for Fireball {
    fn kind(&self) -> InterventionKind { InterventionKind::Fireball }

    fn score(&self, profile: &GamemakerProfile, gauges: &Gauges, _snap: &GameSnapshot) -> u32 {
        let blood = (gauges.bloodthirst as u32 * profile.bloodthirst_weight as u32) / 50;
        blood + gauges.drama_pressure as u32
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let best = snap
            .areas
            .iter()
            .filter(|a| a.is_open && a.tribute_count > 0)
            .max_by_key(|a| a.tribute_count)?;
        Some(TargetSpec::SingleArea(best.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    #[test]
    fn score_zero_when_all_gauges_zero() {
        let g = Gauges {
            drama_pressure: 0, bloodthirst: 0, audience_attention: 0,
            chaos: 0, patience: 0, body_count_debt: 0,
        };
        assert_eq!(Fireball.score(&CASSANDRA, &g, &GameSnapshot::default()), 0);
    }

    #[test]
    fn score_scales_with_bloodthirst() {
        let mut g = Gauges {
            drama_pressure: 50, bloodthirst: 50, audience_attention: 50,
            chaos: 50, patience: 50, body_count_debt: 50,
        };
        let s1 = Fireball.score(&CASSANDRA, &g, &GameSnapshot::default());
        g.bloodthirst = 100;
        let s2 = Fireball.score(&CASSANDRA, &g, &GameSnapshot::default());
        assert!(s2 > s1);
    }

    #[test]
    fn target_pref_picks_area_with_most_tributes() {
        let snap = GameSnapshot {
            alive_tributes: 5,
            current_phase_index: 0,
            areas: vec![
                super::super::AreaSnapshot { id: AreaId("a".into()), tribute_count: 1, adjacent_ids: vec![], is_open: true, has_active_intervention: false },
                super::super::AreaSnapshot { id: AreaId("b".into()), tribute_count: 4, adjacent_ids: vec![], is_open: true, has_active_intervention: false },
            ],
        };
        match Fireball.target_pref(&snap) {
            Some(TargetSpec::SingleArea(id)) => assert_eq!(id, AreaId("b".into())),
            _ => panic!("expected SingleArea(b)"),
        }
    }

    #[test]
    fn target_pref_none_when_no_open_area_has_tributes() {
        assert!(Fireball.target_pref(&GameSnapshot::default()).is_none());
    }
}
```

- [ ] **Step 2: Run; expected pass.**

```bash
cargo test --package game gamemaker::interventions::fireball -- --nocapture
```

### 8b. MuttPack

- [ ] **Step 3: Write `mutt_pack.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{AreaId, GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct MuttPack;

impl InterventionLogic for MuttPack {
    fn kind(&self) -> InterventionKind { InterventionKind::MuttPack }

    fn score(&self, _profile: &GamemakerProfile, gauges: &Gauges, _snap: &GameSnapshot) -> u32 {
        gauges.bloodthirst as u32
            + (gauges.body_count_debt as u32) * 2
            + gauges.chaos as u32
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let best = snap.areas.iter()
            .filter(|a| a.is_open && a.tribute_count > 0)
            .filter(|a| {
                snap.areas.iter().any(|other| {
                    other.id != a.id && other.tribute_count > 0 && a.adjacent_ids.contains(&other.id)
                })
            })
            .max_by_key(|a| a.tribute_count)
            .or_else(|| snap.areas.iter().filter(|a| a.is_open && a.tribute_count > 0).max_by_key(|a| a.tribute_count))?;
        Some(TargetSpec::SingleArea(best.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    #[test]
    fn score_uses_body_count_debt_double() {
        let g = Gauges {
            bloodthirst: 10, body_count_debt: 20, chaos: 5,
            drama_pressure: 0, audience_attention: 0, patience: 0,
        };
        // 10 + 20*2 + 5 = 55
        assert_eq!(MuttPack.score(&CASSANDRA, &g, &GameSnapshot::default()), 55);
    }
}
```

- [ ] **Step 4: Run; expected pass.**

### 8c. ForceFieldShift

- [ ] **Step 5: Write `force_field.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{AreaId, GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct ForceFieldShift;

const MIN_OPEN_AFTER_SHIFT: usize = 3;

impl InterventionLogic for ForceFieldShift {
    fn kind(&self) -> InterventionKind { InterventionKind::ForceFieldShift }

    fn score(&self, profile: &GamemakerProfile, gauges: &Gauges, snap: &GameSnapshot) -> u32 {
        let chaos = (gauges.chaos as u32 * profile.chaos_weight as u32) / 33; // x3
        let alive_penalty = if snap.alive_tributes <= 4 { 50 } else { 0 };
        chaos.saturating_add(gauges.drama_pressure as u32).saturating_sub(alive_penalty)
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let open_low: Vec<AreaId> = snap.areas.iter()
            .filter(|a| a.is_open && a.tribute_count == 0)
            .map(|a| a.id.clone())
            .take(2)
            .collect();
        let closed_candidates: Vec<AreaId> = snap.areas.iter()
            .filter(|a| !a.is_open)
            .map(|a| a.id.clone())
            .take(open_low.len())
            .collect();
        let total_open_after = snap.areas.iter().filter(|a| a.is_open).count()
            + closed_candidates.len()
            - open_low.len();
        if open_low.is_empty() || total_open_after < MIN_OPEN_AFTER_SHIFT {
            return None;
        }
        Some(TargetSpec::AreaSet { close: open_low, open: closed_candidates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_pref_none_with_no_empty_open_areas() {
        let snap = GameSnapshot {
            alive_tributes: 4,
            current_phase_index: 0,
            areas: vec![
                super::super::AreaSnapshot { id: AreaId("a".into()), tribute_count: 4, adjacent_ids: vec![], is_open: true, has_active_intervention: false },
            ],
        };
        assert!(ForceFieldShift.target_pref(&snap).is_none());
    }
}
```

- [ ] **Step 6: Run; expected pass.**

### 8d. AreaClosure

- [ ] **Step 7: Write `area_closure.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct AreaClosure;

impl InterventionLogic for AreaClosure {
    fn kind(&self) -> InterventionKind { InterventionKind::AreaClosure }

    fn score(&self, profile: &GamemakerProfile, gauges: &Gauges, _snap: &GameSnapshot) -> u32 {
        let chaos = (gauges.chaos as u32 * profile.chaos_weight as u32) / 50;
        chaos + gauges.drama_pressure as u32
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let area = snap.areas.iter()
            .filter(|a| a.is_open && !a.has_active_intervention)
            .min_by_key(|a| a.tribute_count)?;
        Some(TargetSpec::SingleArea(area.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_pref_picks_lowest_population_open_area() {
        let snap = GameSnapshot {
            alive_tributes: 6,
            current_phase_index: 0,
            areas: vec![
                super::super::AreaSnapshot { id: super::super::AreaId("a".into()), tribute_count: 5, adjacent_ids: vec![], is_open: true, has_active_intervention: false },
                super::super::AreaSnapshot { id: super::super::AreaId("b".into()), tribute_count: 1, adjacent_ids: vec![], is_open: true, has_active_intervention: false },
            ],
        };
        match AreaClosure.target_pref(&snap) {
            Some(TargetSpec::SingleArea(id)) => assert_eq!(id, super::super::AreaId("b".into())),
            _ => panic!(),
        }
    }
}
```

- [ ] **Step 8: Run; expected pass.**

### 8e. ConvergencePoint

- [ ] **Step 9: Write `convergence.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct ConvergencePoint;

impl InterventionLogic for ConvergencePoint {
    fn kind(&self) -> InterventionKind { InterventionKind::ConvergencePoint }

    fn score(&self, _profile: &GamemakerProfile, gauges: &Gauges, snap: &GameSnapshot) -> u32 {
        let mut s = (gauges.drama_pressure as u32) * 2;
        if gauges.audience_attention < 40 { s += 30; }
        if snap.alive_tributes >= 6 { s += 15; }
        s
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let area = snap.areas.iter()
            .filter(|a| a.is_open && !a.has_active_intervention)
            .max_by_key(|a| a.adjacent_ids.len() as i64 - a.tribute_count as i64)?;
        Some(TargetSpec::SingleArea(area.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    #[test]
    fn audience_attention_low_adds_bonus() {
        let g_high = Gauges { audience_attention: 80, drama_pressure: 50, ..Gauges::STARTING };
        let g_low = Gauges { audience_attention: 20, drama_pressure: 50, ..Gauges::STARTING };
        let snap = GameSnapshot::default();
        assert!(ConvergencePoint.score(&CASSANDRA, &g_low, &snap) > ConvergencePoint.score(&CASSANDRA, &g_high, &snap));
    }
}
```

- [ ] **Step 10: Run; expected pass.**

### 8f. WeatherOverride

- [ ] **Step 11: Write `weather_override.rs`:**

```rust
use crate::gamemaker::gauges::Gauges;
use crate::gamemaker::profile::GamemakerProfile;
use super::{GameSnapshot, InterventionKind, InterventionLogic, TargetSpec};

pub struct WeatherOverride;

impl InterventionLogic for WeatherOverride {
    fn kind(&self) -> InterventionKind { InterventionKind::WeatherOverride }

    fn score(&self, _profile: &GamemakerProfile, gauges: &Gauges, _snap: &GameSnapshot) -> u32 {
        (gauges.drama_pressure as u32 + gauges.chaos as u32)
            .saturating_sub(gauges.bloodthirst as u32)
    }

    fn target_pref(&self, snap: &GameSnapshot) -> Option<TargetSpec> {
        let area = snap.areas.iter()
            .filter(|a| a.is_open)
            .max_by_key(|a| a.tribute_count)
            .or_else(|| snap.areas.iter().find(|a| a.is_open))?;
        Some(super::TargetSpec::SingleArea(area.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamemaker::profile::CASSANDRA;

    #[test]
    fn score_drops_when_bloodthirsty() {
        let g_calm = Gauges { drama_pressure: 60, chaos: 60, bloodthirst: 0, ..Gauges::STARTING };
        let g_blood = Gauges { drama_pressure: 60, chaos: 60, bloodthirst: 80, ..Gauges::STARTING };
        let snap = GameSnapshot::default();
        assert!(WeatherOverride.score(&CASSANDRA, &g_calm, &snap) > WeatherOverride.score(&CASSANDRA, &g_blood, &snap));
    }
}
```

- [ ] **Step 12: Run; expected pass.**

- [ ] **Step 13: Run all interventions tests at once + commit:**

```bash
cargo test --package game gamemaker::interventions -- --nocapture
just fmt
jj describe -m "feat(gamemaker): six intervention scorers and target_prefs"
jj new
```

---

## Task 9: `ActiveIntervention` enum + per-phase tick

**Files:**
- Modify: `game/src/gamemaker/active.rs`

- [ ] **Step 1: Failing tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::threats::animals::Animal;

    #[test]
    fn mutt_swarm_construct() {
        let s = ActiveIntervention::MuttSwarm {
            area_id: "forest".into(),
            kind: Animal::Wolf,
            members: 4,
            hp: 80,
            max_hp_per_member: 20,
            phases_since_combat: 0,
            despawn_at_morning: false,
        };
        assert_eq!(s.area_id_str(), "forest");
    }

    #[test]
    fn closure_expires_when_phase_passes_expiry() {
        let c = ActiveIntervention::AreaClosure {
            area_id: "desert".into(),
            expires_at_phase: 10,
            damage_per_phase: 5,
        };
        assert!(c.expired_at(11));
        assert!(!c.expired_at(10));
        assert!(!c.expired_at(9));
    }

    #[test]
    fn convergence_expires_when_phase_passes_expiry() {
        let c = ActiveIntervention::ConvergencePoint {
            area_id: "clearing".into(),
            lure: shared::messages::Lure::Feast,
            expires_at_phase: 4,
            payload: vec![],
        };
        assert!(c.expired_at(5));
        assert!(!c.expired_at(4));
    }

    #[test]
    fn mutt_swarm_decrement_member_drops_hp_pro_rata() {
        let mut s = ActiveIntervention::MuttSwarm {
            area_id: "a".into(), kind: Animal::Wolf, members: 4, hp: 80,
            max_hp_per_member: 20, phases_since_combat: 0, despawn_at_morning: false,
        };
        s.apply_damage(20);
        match s {
            ActiveIntervention::MuttSwarm { members, hp, .. } => {
                assert_eq!(members, 3);
                assert_eq!(hp, 60);
            }
            _ => unreachable!(),
        }
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Replace `active.rs` placeholder:**

```rust
use serde::{Deserialize, Serialize};

use crate::threats::animals::Animal;
use shared::items::Item;
use shared::messages::Lure;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActiveIntervention {
    MuttSwarm {
        area_id: String,
        kind: Animal,
        members: u8,
        hp: u32,
        max_hp_per_member: u32,
        phases_since_combat: u8,
        despawn_at_morning: bool,
    },
    AreaClosure {
        area_id: String,
        expires_at_phase: u32,
        damage_per_phase: u32,
    },
    ConvergencePoint {
        area_id: String,
        lure: Lure,
        expires_at_phase: u32,
        payload: Vec<Item>,
    },
}

impl ActiveIntervention {
    pub fn area_id_str(&self) -> &str {
        match self {
            ActiveIntervention::MuttSwarm { area_id, .. }
            | ActiveIntervention::AreaClosure { area_id, .. }
            | ActiveIntervention::ConvergencePoint { area_id, .. } => area_id,
        }
    }

    /// Returns true if this effect has expired by `current_phase`.
    /// Mutt swarms never expire by phase index alone (use `mutt_should_despawn`).
    pub fn expired_at(&self, current_phase: u32) -> bool {
        match self {
            ActiveIntervention::MuttSwarm { .. } => false,
            ActiveIntervention::AreaClosure { expires_at_phase, .. }
            | ActiveIntervention::ConvergencePoint { expires_at_phase, .. } => {
                current_phase > *expires_at_phase
            }
        }
    }

    /// Apply combat damage to a mutt swarm. No-op for other variants.
    /// Reduces HP first; when HP drops below `(members - 1) * max_hp_per_member`,
    /// reduce members by 1. Members floor at 0.
    pub fn apply_damage(&mut self, dmg: u32) {
        if let ActiveIntervention::MuttSwarm { members, hp, max_hp_per_member, .. } = self {
            *hp = hp.saturating_sub(dmg);
            let target_members = (*hp as f32 / *max_hp_per_member as f32).ceil() as u32;
            *members = target_members.min(*members as u32) as u8;
        }
    }

    /// Should this swarm despawn this phase? Returns true on next-morning rollover
    /// or when `phases_since_combat` exceeds the threshold (4 phases).
    pub fn mutt_should_despawn(&self, is_morning_rollover: bool) -> Option<shared::messages::DespawnReason> {
        if let ActiveIntervention::MuttSwarm { members, despawn_at_morning, phases_since_combat, .. } = self {
            if *members == 0 {
                return Some(shared::messages::DespawnReason::NoMembersLeft);
            }
            if is_morning_rollover && *despawn_at_morning {
                return Some(shared::messages::DespawnReason::Morning);
            }
            if *phases_since_combat >= 4 {
                return Some(shared::messages::DespawnReason::NoTargetsNearby);
            }
        }
        None
    }
}
```

- [ ] **Step 4: Run tests; pass.**

- [ ] **Step 5: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): ActiveIntervention enum with damage and despawn helpers"
jj new
```

---

## Task 10: Variant selection — score-aggregator with recent-penalty

**Files:**
- Modify: `game/src/gamemaker/decision.rs`

Picks the highest-scoring variant whose `target_pref` returns `Some`. Subtracts `profile.recent_penalty * recent_count(kind)` from each candidate's raw score before comparison. Falls through up to 3 attempts; returns the first variant whose targeting succeeds.

- [ ] **Step 1: Failing tests in `decision.rs`:**

```rust
#[cfg(test)]
mod selection_tests {
    use super::*;
    use crate::gamemaker::interventions::{
        AreaId, AreaSnapshot, GameSnapshot, InterventionKind, TargetSpec,
    };

    fn snap_with_two_areas() -> GameSnapshot {
        GameSnapshot {
            alive_tributes: 8,
            current_phase_index: 0,
            areas: vec![
                AreaSnapshot { id: AreaId("a".into()), tribute_count: 4, adjacent_ids: vec![AreaId("b".into())], is_open: true, has_active_intervention: false },
                AreaSnapshot { id: AreaId("b".into()), tribute_count: 4, adjacent_ids: vec![AreaId("a".into())], is_open: true, has_active_intervention: false },
            ],
        }
    }

    #[test]
    fn returns_none_when_no_variant_targets() {
        let mut gm = Gamemaker::default();
        gm.gauges = Gauges { drama_pressure: 90, ..Gauges::STARTING };
        let snap = GameSnapshot::default(); // no areas
        assert!(select_intervention(&gm, &snap).is_none());
    }

    #[test]
    fn highest_scoring_variant_wins() {
        let mut gm = Gamemaker::default();
        gm.gauges = Gauges { bloodthirst: 100, drama_pressure: 60, ..Gauges::STARTING };
        let snap = snap_with_two_areas();
        let pick = select_intervention(&gm, &snap).expect("should pick something");
        // bloodthirst-dominant -> Fireball or MuttPack
        assert!(matches!(pick.kind, InterventionKind::Fireball | InterventionKind::MuttPack));
    }

    #[test]
    fn recent_penalty_drops_score() {
        let mut gm = Gamemaker::default();
        gm.gauges = Gauges { bloodthirst: 100, drama_pressure: 60, ..Gauges::STARTING };
        // saturate Fireball window
        for i in 0..6 { gm.record_intervention(InterventionKind::Fireball, i); }
        gm.gauges.patience = 30;
        gm.interventions_today = 0;
        let snap = snap_with_two_areas();
        let pick = select_intervention(&gm, &snap).expect("should still pick something");
        // Fireball is heavily penalised; MuttPack should win
        assert_eq!(pick.kind, InterventionKind::MuttPack);
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Append to `decision.rs`:**

```rust
use crate::gamemaker::interventions::{
    GameSnapshot, InterventionKind, InterventionLogic, TargetSpec,
    area_closure::AreaClosure, convergence::ConvergencePoint, fireball::Fireball,
    force_field::ForceFieldShift, mutt_pack::MuttPack, weather_override::WeatherOverride,
};

pub struct PickedIntervention {
    pub kind: InterventionKind,
    pub target: TargetSpec,
}

fn all_logics() -> Vec<Box<dyn InterventionLogic>> {
    vec![
        Box::new(Fireball),
        Box::new(MuttPack),
        Box::new(ForceFieldShift),
        Box::new(AreaClosure),
        Box::new(ConvergencePoint),
        Box::new(WeatherOverride),
    ]
}

pub fn select_intervention(gm: &Gamemaker, snap: &GameSnapshot) -> Option<PickedIntervention> {
    let logics = all_logics();
    let mut scored: Vec<(InterventionKind, u32, &dyn InterventionLogic)> = logics.iter()
        .map(|l| {
            let raw = l.score(&gm.profile, &gm.gauges, snap);
            let penalty = gm.profile.recent_penalty.saturating_mul(gm.recent_count(l.kind()));
            (l.kind(), raw.saturating_sub(penalty), l.as_ref())
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    for (kind, _score, logic) in scored.into_iter().take(3) {
        if let Some(target) = logic.target_pref(snap) {
            return Some(PickedIntervention { kind, target });
        }
    }
    None
}
```

- [ ] **Step 4: Run tests; pass.**

- [ ] **Step 5: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): select_intervention with recent-penalty scoring"
jj new
```

---

## Task 11: Mutt vs. tribute combat resolution (shelter +5 / hide +2)

**Files:**
- Modify: `game/src/gamemaker/active.rs`

A small pure helper used by the per-phase tick when resolving a mutt swarm's attack.

- [ ] **Step 1: Failing tests:**

```rust
#[cfg(test)]
mod combat_tests {
    use super::*;

    #[test]
    fn evade_bonus_zero_when_neither_sheltered_nor_hidden() {
        assert_eq!(evade_bonus(false, false), 0);
    }

    #[test]
    fn evade_bonus_five_when_sheltered_only() {
        assert_eq!(evade_bonus(true, false), 5);
    }

    #[test]
    fn evade_bonus_two_when_hidden_only() {
        assert_eq!(evade_bonus(false, true), 2);
    }

    #[test]
    fn evade_bonus_seven_when_both() {
        assert_eq!(evade_bonus(true, true), 7);
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Append to `active.rs`:**

```rust
/// Defender's evade bonus against a mutt swarm attack.
/// Sheltered = +5, Hidden = +2, additive (max +7).
pub fn evade_bonus(sheltered: bool, hidden: bool) -> i32 {
    let mut b = 0;
    if sheltered { b += 5; }
    if hidden { b += 2; }
    b
}
```

- [ ] **Step 4: Run; pass.**

- [ ] **Step 5: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): mutt evade bonus helper"
jj new
```

---

## Task 12: Brain integration — convergence pull, mutt avoidance, sealed-area filter

**Files:**
- Modify: `game/src/tributes/brain.rs` (or wherever `choose_destination` lives — locate via `grep -rn "fn choose_destination" game/src/`)

The override pass runs *before* hunger/thirst overrides (which run before the standard brain), but *after* combat preempts. Order:

1. Combat preempt (existing rule, unchanged)
2. **Gamemaker overrides (new):**
   a. Mutt in current area + can flee → `Travel(adjacent_clear_area)`
   b. Active convergence point exists + tribute eligible → adds pull term to `choose_destination`
   c. Sealed areas filtered out of `choose_destination` candidates
3. Hunger/thirst overrides (from shelter PR1, if landed; otherwise no-op)
4. Standard brain logic

- [ ] **Step 1: Locate the relevant fn:**

```bash
grep -rn "fn choose_destination\|fn act\|pub fn brain" game/src/tributes/ | head -10
```

- [ ] **Step 2: Write a failing integration test in `game/tests/gamemaker_brain.rs`** (create if absent). Sketch:

```rust
//! Brain integration with gamemaker overrides.
use game::gamemaker::active::ActiveIntervention;
use game::threats::animals::Animal;

#[test]
fn tribute_flees_area_with_mutt_swarm_when_adjacent_clear_area_exists() {
    // Build a minimal Game with two adjacent areas, one with a mutt swarm,
    // one tribute in the swarm area, no enemies present.
    // Assert tribute's chosen action is Travel(adjacent area).
    // (Implementation depends on existing test helpers - see
    // game/tests/ for `Game::new_test_with_tributes` or similar.)
    //
    // SKETCH: copy pattern from existing brain tests.
}
```

- [ ] **Step 3: Inspect existing brain tests to find the test-helper pattern:**

```bash
grep -rn "Game::new\|fn new_test\|build_test_game" game/tests/ game/src/games.rs | head -10
```

- [ ] **Step 4: Use the located helper to write the actual test.** Pseudocode (adapt to real helper signatures):

```rust
let mut game = build_test_game_with_areas_and_tributes(
    /*areas*/ vec![("forest", vec!["clearing"]), ("clearing", vec!["forest"])],
    /*tributes*/ vec![("Alice", "forest")],
);
game.gamemaker.active_effects.push(ActiveIntervention::MuttSwarm {
    area_id: "forest".into(), kind: Animal::Wolf, members: 3, hp: 60,
    max_hp_per_member: 20, phases_since_combat: 0, despawn_at_morning: false,
});
let action = game.tributes[0].decide_action(&game);
assert!(matches!(action, Action::Travel(area) if area == Area::from_str("clearing").unwrap()));
```

- [ ] **Step 5: Run; expected fail (no override applied).**

- [ ] **Step 6: Add override pass to brain logic.** In the action-decision function (likely `Brain::act` or `Tribute::decide_action`), insert immediately after the combat-preempt check, before any other branch:

```rust
// Gamemaker overrides — Capitol force majeure trumps non-combat planning.
if let Some(action) = gamemaker_override(self, game) {
    return action;
}
```

Implement the helper in a new location next to the brain, e.g. `game/src/gamemaker/brain_override.rs`:

```rust
use crate::areas::Area;
use crate::games::Game;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::gamemaker::ActiveIntervention;

/// Returns an override action when gamemaker state should preempt the standard brain.
/// Order: mutt-flee > sealed-area-filter (passive — applied to choose_destination input) >
/// convergence-pull (adds to choose_destination scoring, never returns standalone).
pub fn gamemaker_override(tribute: &Tribute, game: &Game) -> Option<Action> {
    // 1. Mutt in current area: try to flee.
    let in_combat = tribute.in_combat(game); // existing helper
    if !in_combat {
        for eff in &game.gamemaker.active_effects {
            if let ActiveIntervention::MuttSwarm { area_id, .. } = eff {
                if Some(area_id.as_str()) == tribute.area.map(|a| a.to_string()).as_deref() {
                    if let Some(escape) = pick_flee_target(tribute, game) {
                        return Some(Action::Travel(escape));
                    }
                }
            }
        }
    }
    None
}

fn pick_flee_target(tribute: &Tribute, game: &Game) -> Option<Area> {
    let here = tribute.area?;
    let neighbours = here.neighbours(); // existing helper, or use AreaDetails graph
    neighbours.into_iter().find(|n| {
        // No mutt in neighbour
        !game.gamemaker.active_effects.iter().any(|eff| {
            if let ActiveIntervention::MuttSwarm { area_id, .. } = eff {
                area_id == &n.to_string()
            } else { false }
        })
        // Not sealed
        && !game.gamemaker.active_effects.iter().any(|eff| {
            if let ActiveIntervention::AreaClosure { area_id, .. } = eff {
                area_id == &n.to_string()
            } else { false }
        })
    })
}
```

(NOTE: the call sites `tribute.in_combat`, `tribute.area`, `here.neighbours` and `Action::Travel` need to be adjusted to match what actually exists in `game/src/tributes/`. Run `grep -rn "fn in_combat\|fn neighbours\|enum Action" game/src/tributes/ game/src/areas/` to locate.)

- [ ] **Step 7: Wire `gamemaker_override` into the brain's action-decision flow.**

- [ ] **Step 8: Add convergence-pull and sealed-area-filter integration to `choose_destination`** (likely a separate function in the brain module). Add a helper:

```rust
fn area_filter_and_convergence_bias(game: &Game) -> (HashSet<Area>, HashMap<Area, i32>) {
    let mut sealed = HashSet::new();
    let mut convergence_bias = HashMap::new();
    for eff in &game.gamemaker.active_effects {
        match eff {
            ActiveIntervention::AreaClosure { area_id, .. } => {
                if let Ok(a) = Area::from_str(area_id) { sealed.insert(a); }
            }
            ActiveIntervention::ConvergencePoint { area_id, .. } => {
                if let Ok(a) = Area::from_str(area_id) { convergence_bias.insert(a, 50); }
            }
            _ => {}
        }
    }
    (sealed, convergence_bias)
}
```

Inside `choose_destination` (existing fn): filter candidates by `!sealed.contains(area)` and add `convergence_bias.get(area).copied().unwrap_or(0)` to each candidate's score.

- [ ] **Step 9: Run integration test from Step 4; expected pass.**

- [ ] **Step 10: Run the full game-crate test suite to confirm no regression:**

```bash
just test
```

- [ ] **Step 11: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): brain overrides for mutt-flee, sealed-area filter, convergence pull"
jj new
```

---

## Task 13: Per-phase active-effect tick (decay, expiry, damage emission)

**Files:**
- Modify: `game/src/gamemaker/active.rs`
- Modify: `game/src/gamemaker/mod.rs` (or new `tick.rs`)

This is the per-phase routine that:

1. Decrements `phases_since_combat` counters on mutt swarms
2. Emits `MuttSwarmAttack` events for tributes still in mutt-swarm areas
3. Emits `AreaSealEntryDamage` for tributes in sealed areas
4. Emits expiry events (`MuttSwarmDespawned`, `AreaUnsealed`, `ConvergencePointExpired`) and removes expired effects
5. Returns counts of (kills, hazards_with_kill, hazards_no_kill) so the caller can apply gauge reactions in Task 14

- [ ] **Step 1: Failing test in `active.rs`:**

```rust
#[cfg(test)]
mod tick_tests {
    use super::*;

    #[test]
    fn closure_at_expiry_phase_removed_and_emits_unsealed() {
        let mut effects = vec![
            ActiveIntervention::AreaClosure {
                area_id: "a".into(), expires_at_phase: 5, damage_per_phase: 10,
            },
        ];
        let result = drain_expired(&mut effects, /*current_phase=*/ 6);
        assert!(effects.is_empty());
        assert_eq!(result.unsealed_areas, vec!["a".to_string()]);
    }

    #[test]
    fn convergence_at_expiry_collects_lure() {
        let mut effects = vec![
            ActiveIntervention::ConvergencePoint {
                area_id: "b".into(), lure: shared::messages::Lure::Feast,
                expires_at_phase: 3, payload: vec![],
            },
        ];
        let result = drain_expired(&mut effects, 4);
        assert!(effects.is_empty());
        assert_eq!(result.convergence_expired, vec![("b".to_string(), shared::messages::Lure::Feast)]);
    }

    #[test]
    fn mutt_swarm_with_zero_members_despawns() {
        let mut effects = vec![
            ActiveIntervention::MuttSwarm {
                area_id: "c".into(), kind: crate::threats::animals::Animal::Wolf,
                members: 0, hp: 0, max_hp_per_member: 20,
                phases_since_combat: 0, despawn_at_morning: false,
            },
        ];
        let result = drain_expired(&mut effects, 1);
        assert!(effects.is_empty());
        assert_eq!(result.mutt_despawns.len(), 1);
        assert_eq!(result.mutt_despawns[0].2, shared::messages::DespawnReason::NoMembersLeft);
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Append to `active.rs`:**

```rust
use shared::messages::{DespawnReason, Lure};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrainResult {
    pub unsealed_areas: Vec<String>,
    pub convergence_expired: Vec<(String, Lure)>,
    pub mutt_despawns: Vec<(String, crate::threats::animals::Animal, DespawnReason)>,
}

/// Removes expired effects from `effects` and reports what was removed.
/// `current_phase` is the phase index we have just begun ticking.
pub fn drain_expired(effects: &mut Vec<ActiveIntervention>, current_phase: u32) -> DrainResult {
    let mut out = DrainResult::default();

    effects.retain(|eff| match eff {
        ActiveIntervention::AreaClosure { area_id, expires_at_phase, .. } => {
            if current_phase > *expires_at_phase {
                out.unsealed_areas.push(area_id.clone());
                false
            } else { true }
        }
        ActiveIntervention::ConvergencePoint { area_id, lure, expires_at_phase, .. } => {
            if current_phase > *expires_at_phase {
                out.convergence_expired.push((area_id.clone(), lure.clone()));
                false
            } else { true }
        }
        ActiveIntervention::MuttSwarm { area_id, kind, members, phases_since_combat, despawn_at_morning, .. } => {
            if *members == 0 {
                out.mutt_despawns.push((area_id.clone(), *kind, DespawnReason::NoMembersLeft));
                false
            } else if *despawn_at_morning && is_morning_phase(current_phase) {
                out.mutt_despawns.push((area_id.clone(), *kind, DespawnReason::Morning));
                false
            } else if *phases_since_combat >= 4 {
                out.mutt_despawns.push((area_id.clone(), *kind, DespawnReason::NoTargetsNearby));
                false
            } else {
                true
            }
        }
    });

    out
}

/// True when this phase index is the start of a new game-day (i.e., morning).
/// Convention: even phases = Day, odd phases = Night.
fn is_morning_phase(phase: u32) -> bool {
    phase % 2 == 0
}
```

- [ ] **Step 4: Run; pass.**

- [ ] **Step 5: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): drain_expired removes finished effects and reports them"
jj new
```

---

## Task 14: Per-phase decision pipeline + `Game` integration

**Files:**
- Modify: `game/src/gamemaker/decision.rs`
- Modify: `game/src/games.rs` — call from `run_day_night_cycle`

The orchestrator `Game::tick_gamemaker(&mut self, current_phase: u32, rng: &mut impl Rng)` does the per-phase work in spec order:

1. Tick gauges (background)
2. Drain expired active effects → emit unsealed/expired/despawn payloads
3. Resolve persistent damage (mutt attacks, seal entry damage) → emit attack/damage payloads, increment `phases_since_combat`
4. Check `should_intervene`; if no, done
5. Build `GameSnapshot`
6. `select_intervention` → `PickedIntervention`
7. Resolve immediately (Fireball/ForceFieldShift/WeatherOverride) or push to `active_effects` (MuttPack/AreaClosure/ConvergencePoint)
8. Emit announcement/strike payloads
9. Apply per-event gauge reaction
10. `record_intervention` (which resets patience)
11. Call site: at start of `run_day_night_cycle`, derive `current_phase` from existing `tick_counter`/day/phase state, then call `self.tick_gamemaker(...)`. Day rollover: when `current_phase % 2 == 0` AND `current_phase > 0`, also reset `interventions_today` to 0.

This task is *long*. Break into sub-steps; commit after each runs cleanly.

### 14a. Snapshot builder

- [ ] **Step 1: Failing test:**

```rust
#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn snapshot_counts_alive_tributes_and_areas() {
        let mut game = crate::games::Game::default();
        // (Use existing helpers to populate areas + tributes; see test patterns elsewhere)
        let snap = build_snapshot(&game);
        assert_eq!(snap.alive_tributes, 0);
        assert!(snap.areas.is_empty());
    }
}
```

- [ ] **Step 2: Run; expected fail.**

- [ ] **Step 3: Implement `build_snapshot` in `decision.rs`:**

```rust
use crate::games::Game;
use crate::gamemaker::interventions::{AreaId, AreaSnapshot, GameSnapshot};

pub fn build_snapshot(game: &Game) -> GameSnapshot {
    let alive_tributes = game.tributes.iter().filter(|t| !t.is_dead()).count() as u32;
    let areas: Vec<AreaSnapshot> = game.areas.iter().filter_map(|ad| {
        let area = ad.area?;
        let id = AreaId(area.to_string());
        let tribute_count = game.tributes.iter()
            .filter(|t| !t.is_dead() && t.area == Some(area))
            .count() as u32;
        let adjacent_ids = ad.neighbours().iter().map(|n| AreaId(n.to_string())).collect();
        let has_active_intervention = game.gamemaker.active_effects.iter()
            .any(|eff| eff.area_id_str() == area.to_string());
        Some(AreaSnapshot {
            id,
            tribute_count,
            adjacent_ids,
            is_open: ad.is_open(),
            has_active_intervention,
        })
    }).collect();

    GameSnapshot {
        alive_tributes,
        current_phase_index: 0, // caller stamps
        areas,
    }
}
```

(Adjust `is_dead`, `area`, `neighbours`, `is_open` to match real method names — discover with `grep -rn "fn is_dead\|fn neighbours\|fn is_open" game/src/`.)

- [ ] **Step 4: Run; pass.**

### 14b. `tick_gamemaker` orchestrator

- [ ] **Step 5: Add to `games.rs` (impl block for `Game`):**

```rust
/// Runs the gamemaker decision pipeline for one phase.
/// Called from `run_day_night_cycle` immediately after `prepare_cycle`.
pub fn tick_gamemaker(&mut self, current_phase: u32, _rng: &mut SmallRng) {
    // 1. Day rollover bookkeeping.
    if current_phase % 2 == 0 && current_phase > 0 {
        self.gamemaker.interventions_today = 0;
    }

    // 2. Tick gauges.
    let alive = self.tributes.iter().filter(|t| !t.is_dead()).count() as u32;
    self.gamemaker.gauges.tick_phase(&self.gamemaker.profile, alive);

    // 3. Drain expired active effects.
    let drain = crate::gamemaker::active::drain_expired(
        &mut self.gamemaker.active_effects,
        current_phase,
    );
    for area_id in &drain.unsealed_areas {
        self.emit_area_unsealed(area_id);
    }
    for (area_id, lure) in &drain.convergence_expired {
        self.emit_convergence_expired(area_id, lure.clone(), /*claimed_by=*/ vec![]);
    }
    for (area_id, kind, reason) in &drain.mutt_despawns {
        self.emit_mutt_swarm_despawned(area_id, *kind, *reason);
    }

    // 4. Resolve persistent damage (mutt attacks, seal entry damage).
    self.resolve_active_effect_damage(current_phase);

    // 5. Eligibility gate.
    if !crate::gamemaker::decision::should_intervene(&self.gamemaker, self.gamemaker.active_effects.len()) {
        return;
    }

    // 6. Build snapshot + pick variant.
    let mut snap = crate::gamemaker::decision::build_snapshot(self);
    snap.current_phase_index = current_phase;
    let picked = match crate::gamemaker::decision::select_intervention(&self.gamemaker, &snap) {
        Some(p) => p,
        None => return,
    };

    // 7. Resolve.
    self.resolve_intervention(picked, current_phase);
}

// Emission helpers (private to games.rs):
fn emit_area_unsealed(&mut self, area_id: &str) {
    use shared::messages::{AreaRef, MessagePayload, MessageSource};
    self.log_payload(
        MessageSource::Game(self.identifier.clone()),
        MessagePayload::AreaUnsealed { area: AreaRef { id: area_id.into(), name: area_id.into() } },
    );
}

fn emit_convergence_expired(&mut self, area_id: &str, lure: shared::messages::Lure, claimed_by: Vec<shared::messages::TributeRef>) {
    use shared::messages::{AreaRef, MessagePayload, MessageSource};
    self.log_payload(
        MessageSource::Game(self.identifier.clone()),
        MessagePayload::ConvergencePointExpired {
            area: AreaRef { id: area_id.into(), name: area_id.into() },
            lure, claimed_by,
        },
    );
}

fn emit_mutt_swarm_despawned(&mut self, area_id: &str, kind: crate::threats::animals::Animal, reason: shared::messages::DespawnReason) {
    use shared::messages::{AreaRef, MessagePayload, MessageSource};
    self.log_payload(
        MessageSource::Game(self.identifier.clone()),
        MessagePayload::MuttSwarmDespawned {
            area: AreaRef { id: area_id.into(), name: area_id.into() },
            kind_label: kind.to_string(),
            reason,
        },
    );
}

fn resolve_active_effect_damage(&mut self, current_phase: u32) {
    // Iterate active effects, apply damage, emit attack payloads.
    // Implementation: snapshot the effect list, compute damage per affected
    // tribute, mutate tributes, then mutate the effect list (separate borrows).
    let snapshot: Vec<(usize, ActiveInterventionResolutionInput)> = self.gamemaker.active_effects.iter().enumerate().filter_map(|(i, eff)| {
        match eff {
            crate::gamemaker::ActiveIntervention::MuttSwarm { area_id, kind, members, .. } => Some((i, ActiveInterventionResolutionInput::Mutt { area_id: area_id.clone(), kind: *kind, members: *members })),
            crate::gamemaker::ActiveIntervention::AreaClosure { area_id, damage_per_phase, .. } => Some((i, ActiveInterventionResolutionInput::Seal { area_id: area_id.clone(), damage: *damage_per_phase })),
            _ => None,
        }
    }).collect();

    for (idx, input) in snapshot {
        match input {
            ActiveInterventionResolutionInput::Mutt { area_id, kind, members } => {
                // Pick first alive tribute in the mutt's area; mutts attack one per phase.
                let victim_idx = self.tributes.iter().position(|t| !t.is_dead() && t.area.map(|a| a.to_string()) == Some(area_id.clone()));
                if let Some(vi) = victim_idx {
                    let damage = (members as u32) * 5; // tunable
                    let killed = self.tributes[vi].take_damage(damage); // existing helper or apply manually
                    let victim = self.tributes[vi].as_ref();
                    self.emit_mutt_swarm_attack(&area_id, kind, &victim, damage, killed);
                    if let crate::gamemaker::ActiveIntervention::MuttSwarm { phases_since_combat, .. } = &mut self.gamemaker.active_effects[idx] {
                        *phases_since_combat = 0;
                    }
                } else if let crate::gamemaker::ActiveIntervention::MuttSwarm { phases_since_combat, .. } = &mut self.gamemaker.active_effects[idx] {
                    *phases_since_combat = phases_since_combat.saturating_add(1);
                }
            }
            ActiveInterventionResolutionInput::Seal { area_id, damage } => {
                let in_area: Vec<usize> = self.tributes.iter().enumerate().filter(|(_, t)| !t.is_dead() && t.area.map(|a| a.to_string()) == Some(area_id.clone())).map(|(i, _)| i).collect();
                for vi in in_area {
                    self.tributes[vi].take_damage(damage);
                    let victim = self.tributes[vi].as_ref();
                    self.emit_seal_entry_damage(&area_id, &victim, damage);
                }
            }
        }
    }
}

enum ActiveInterventionResolutionInput {
    Mutt { area_id: String, kind: crate::threats::animals::Animal, members: u8 },
    Seal { area_id: String, damage: u32 },
}

fn emit_mutt_swarm_attack(&mut self, area_id: &str, kind: crate::threats::animals::Animal, victim: &shared::messages::TributeRef, damage: u32, killed: bool) {
    use shared::messages::{AreaRef, MessagePayload, MessageSource};
    self.log_payload(
        MessageSource::Game(self.identifier.clone()),
        MessagePayload::MuttSwarmAttack {
            area: AreaRef { id: area_id.into(), name: area_id.into() },
            kind_label: kind.to_string(),
            victim: victim.clone(),
            damage,
            killed,
        },
    );
}

fn emit_seal_entry_damage(&mut self, area_id: &str, victim: &shared::messages::TributeRef, damage: u32) {
    use shared::messages::{AreaRef, MessagePayload, MessageSource};
    self.log_payload(
        MessageSource::Game(self.identifier.clone()),
        MessagePayload::AreaSealEntryDamage {
            area: AreaRef { id: area_id.into(), name: area_id.into() },
            tribute: victim.clone(),
            damage,
        },
    );
}

fn resolve_intervention(&mut self, picked: crate::gamemaker::decision::PickedIntervention, current_phase: u32) {
    use crate::gamemaker::interventions::{InterventionKind, TargetSpec};
    use shared::messages::{AreaRef, MessagePayload, MessageSource};

    let kind = picked.kind;
    match (picked.kind, picked.target) {
        (InterventionKind::Fireball, TargetSpec::SingleArea(area)) => {
            // Resolve damage immediately. Severity per spec — Major default for v1.
            // (TODO: derive from gauges.) Apply to tributes in area.
            let area_str = area.0;
            let victims_idx: Vec<usize> = self.tributes.iter().enumerate()
                .filter(|(_, t)| !t.is_dead() && t.area.map(|a| a.to_string()) == Some(area_str.clone()))
                .map(|(i, _)| i).collect();
            let mut victims = vec![];
            let mut survivors = vec![];
            for vi in victims_idx {
                let killed = self.tributes[vi].take_damage(40);
                let r = self.tributes[vi].as_ref();
                if killed { victims.push(r); } else { survivors.push(r); }
            }
            let any_kills = !victims.is_empty();
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::FireballStrike {
                    area: AreaRef { id: area_str.clone(), name: area_str.clone() },
                    severity_label: "Major".into(),
                    victims, survivors,
                },
            );
            self.gamemaker.gauges.react_to(if any_kills {
                &crate::gamemaker::gauges::GaugeReaction::InterventionResolvedWithKill
            } else {
                &crate::gamemaker::gauges::GaugeReaction::InterventionResolvedNoKill
            });
        }
        (InterventionKind::MuttPack, TargetSpec::SingleArea(area)) => {
            let area_str = area.0;
            let kind_animal = crate::threats::animals::Animal::Wolf; // v1 default
            let members = 4u8;
            let max_hp_per_member = 20u32;
            let hp = (members as u32) * max_hp_per_member;
            self.gamemaker.active_effects.push(crate::gamemaker::ActiveIntervention::MuttSwarm {
                area_id: area_str.clone(), kind: kind_animal, members, hp, max_hp_per_member,
                phases_since_combat: 0, despawn_at_morning: false,
            });
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::MuttSwarmSpawned {
                    area: AreaRef { id: area_str.clone(), name: area_str.clone() },
                    kind_label: kind_animal.to_string(), members,
                },
            );
            self.gamemaker.gauges.react_to(&crate::gamemaker::gauges::GaugeReaction::InterventionResolvedNoKill);
        }
        (InterventionKind::ForceFieldShift, TargetSpec::AreaSet { close, open }) => {
            // Topology change. For v1, mark areas closed/open by mutating AreaDetails.events.
            // (Apply bespoke in this match arm; consumers will see the closed/opened areas
            // via existing announce_area_events path.)
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::ForceFieldShifted {
                    closed: close.iter().map(|c| AreaRef { id: c.0.clone(), name: c.0.clone() }).collect(),
                    opened: open.iter().map(|o| AreaRef { id: o.0.clone(), name: o.0.clone() }).collect(),
                    warning_phases: 1,
                },
            );
            self.gamemaker.gauges.react_to(&crate::gamemaker::gauges::GaugeReaction::InterventionDisruptive);
        }
        (InterventionKind::AreaClosure, TargetSpec::SingleArea(area)) => {
            let area_str = area.0;
            self.gamemaker.active_effects.push(crate::gamemaker::ActiveIntervention::AreaClosure {
                area_id: area_str.clone(), expires_at_phase: current_phase + 4, damage_per_phase: 10,
            });
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::AreaSealed {
                    area: AreaRef { id: area_str.clone(), name: area_str.clone() },
                    expires_at_phase: current_phase + 4,
                },
            );
            self.gamemaker.gauges.react_to(&crate::gamemaker::gauges::GaugeReaction::InterventionDisruptive);
        }
        (InterventionKind::ConvergencePoint, TargetSpec::SingleArea(area)) => {
            let area_str = area.0;
            self.gamemaker.active_effects.push(crate::gamemaker::ActiveIntervention::ConvergencePoint {
                area_id: area_str.clone(), lure: shared::messages::Lure::Feast,
                expires_at_phase: current_phase + 4, payload: vec![],
            });
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::ConvergencePointAnnounced {
                    area: AreaRef { id: area_str.clone(), name: area_str.clone() },
                    lure: shared::messages::Lure::Feast,
                    starts_at_phase: current_phase + 1,
                },
            );
            self.gamemaker.gauges.react_to(&crate::gamemaker::gauges::GaugeReaction::InterventionConvergenceAnnounce);
        }
        (InterventionKind::WeatherOverride, TargetSpec::SingleArea(area)) => {
            let area_str = area.0;
            self.log_payload(
                MessageSource::Game(self.identifier.clone()),
                MessagePayload::WeatherOverridden {
                    area: AreaRef { id: area_str.clone(), name: area_str.clone() },
                    weather_label: "HeavyRain".into(),
                    duration_phases: 2,
                },
            );
            self.gamemaker.gauges.react_to(&crate::gamemaker::gauges::GaugeReaction::WeatherOverride);
        }
        // Mismatched (kind, target) shapes are unreachable given target_pref contracts.
        (k, t) => unreachable!("mismatched kind {:?} / target {:?}", k, t),
    }
    self.gamemaker.record_intervention(kind, current_phase);
}
```

NOTE: this references `self.log_payload(source, payload)`. Confirm the existing API or adapt — examples like `self.log_event(source, subject, GameEvent::...)` are in `games.rs` already. If a typed-payload logger doesn't exist, add one alongside `log_event`:

```rust
fn log_payload(&mut self, source: shared::messages::MessageSource, payload: shared::messages::MessagePayload) {
    self.messages.push(shared::messages::GameMessage {
        source,
        phase: self.current_phase.clone(),
        payload,
        emit_index: self.emit_index,
        // ... fill remaining fields per existing GameMessage shape
    });
    self.emit_index = self.emit_index.saturating_add(1);
}
```

Inspect the actual `GameMessage` struct first via `grep -n "pub struct GameMessage" shared/src/messages.rs` and adjust.

- [ ] **Step 6: Wire `tick_gamemaker` into `run_day_night_cycle`.** Add after `self.prepare_cycle(day)?;` and before `self.do_a_cycle(day)?;`:

```rust
let mut gm_rng = SmallRng::from_entropy();
let phase_index = self.tick_counter.global_index(); // or compute from day + day/night flag
self.tick_gamemaker(phase_index, &mut gm_rng);
```

(Use the actual phase-index source — read `tick_counter` impl or compute as `(day * 2) + if day_phase { 0 } else { 1 }`.)

- [ ] **Step 7: Run full game tests:**

```bash
just test
```

Expected: pre-existing tests still pass; gamemaker doesn't break anything.

- [ ] **Step 8: Commit:**

```bash
just fmt
jj describe -m "feat(gamemaker): tick_gamemaker pipeline + run_day_night_cycle integration"
jj new
```

---

## Task 15: Integration test — full per-phase flow with gauge transitions

**Files:**
- Create: `game/tests/gamemaker_integration.rs`

End-to-end: build a small game, force gauges high, run a phase, assert that an intervention fired and that recent_interventions/interventions_today/patience updated correctly.

- [ ] **Step 1: Write the test:**

```rust
//! End-to-end gamemaker dispatch under controlled gauge state.

use game::gamemaker::{Gauges, InterventionKind};
use game::games::Game;
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[test]
fn high_pressure_triggers_intervention_and_resets_patience() {
    let mut game = build_minimal_game(); // helper below
    game.gamemaker.gauges = Gauges {
        drama_pressure: 90,
        bloodthirst: 50,
        chaos: 40,
        audience_attention: 60,
        patience: 50,
        body_count_debt: 0,
    };
    let mut rng = SmallRng::seed_from_u64(42);
    game.tick_gamemaker(/*phase=*/ 3, &mut rng);

    assert!(!game.gamemaker.recent_interventions.is_empty(), "expected an intervention");
    assert_eq!(game.gamemaker.gauges.patience, 0);
    assert_eq!(game.gamemaker.interventions_today, 1);
}

#[test]
fn per_day_cap_blocks_third_intervention() {
    let mut game = build_minimal_game();
    game.gamemaker.interventions_today = game.gamemaker.profile.max_per_day;
    game.gamemaker.gauges = Gauges { drama_pressure: 99, patience: 99, ..Gauges::STARTING };
    let mut rng = SmallRng::seed_from_u64(0);
    game.tick_gamemaker(2, &mut rng);
    assert!(game.gamemaker.recent_interventions.is_empty(), "should be blocked");
}

#[test]
fn day_rollover_resets_interventions_today() {
    let mut game = build_minimal_game();
    game.gamemaker.interventions_today = 2;
    let mut rng = SmallRng::seed_from_u64(0);
    // Even non-zero phase index triggers rollover.
    game.tick_gamemaker(2, &mut rng);
    assert_eq!(game.gamemaker.interventions_today, 0,
        "expected day-rollover reset when current_phase is even and > 0");
}

fn build_minimal_game() -> Game {
    // Use existing helpers if present; else inline. The minimal viable game
    // has at least 2 areas (so target_pref has something to pick) and 4 tributes.
    let mut game = Game::default();
    // ... populate via test helpers; see other game/tests/*.rs files for patterns.
    game
}
```

- [ ] **Step 2: Inspect existing integration tests for population helpers:**

```bash
grep -rn "fn build_test_game\|fn make_game\|Game::default" game/tests/ | head -10
```

Adapt `build_minimal_game()` accordingly.

- [ ] **Step 3: Run tests:**

```bash
cargo test --package game --test gamemaker_integration -- --nocapture
```

Expected: pass.

- [ ] **Step 4: Commit:**

```bash
just fmt
jj describe -m "test(gamemaker): end-to-end pipeline integration tests"
jj new
```

---

## Self-Review

Run through the spec section-by-section and verify each item has a task:

- [x] `Gamemaker` struct on `Game` → Tasks 1-2
- [x] Gauges (6 fields) + `STARTING` → Task 1, 3
- [x] `GamemakerProfile` + `CASSANDRA` const → Task 1
- [x] Per-phase tick (rises/decays) + late-game multiplier → Task 3
- [x] Per-event reactions (8 reaction kinds) → Task 4
- [x] 11 `MessagePayload` variants + `Lure` + `DespawnReason` + 3 cause constants → Task 5
- [x] `should_intervene` eligibility gate → Task 6
- [x] `InterventionLogic` trait + `GameSnapshot` → Task 7
- [x] 6 intervention scorers + targeting → Task 8
- [x] `ActiveIntervention` enum → Task 9
- [x] Mutt evade bonus (shelter +5 / hide +2) → Task 11
- [x] Brain integration (mutt-flee, sealed-area filter, convergence pull) → Task 12
- [x] Active-effect tick (drain expired, resolve damage) → Tasks 13-14
- [x] `tick_gamemaker` orchestrator + `run_day_night_cycle` integration → Task 14
- [x] Per-day rollover (`interventions_today` reset) → Task 14, 15
- [x] Recent-interventions cap + recent-penalty → Tasks 1, 10
- [x] End-to-end integration test → Task 15

**Open known-gaps to flag during implementation review:**

1. Fireball severity is hard-coded to `"Major"` in Task 14; deriving from gauges + game state is a tuning task, not a structural one. File a follow-up bead if not done in PR1.
2. MuttPack always spawns `Animal::Wolf` with 4 members in Task 14. Animal selection from gauges/terrain is a tuning task; file follow-up.
3. WeatherOverride duration always 2 phases, weather always `HeavyRain` — same story. File follow-up.
4. ForceFieldShift's actual area-state mutation (closing/opening areas) is deferred to the existing `AreaDetails.events`/`is_open()` machinery; verify it ties through correctly during integration testing.
5. `area_id: String` is used throughout `ActiveIntervention` and emission helpers because `Area` is an enum and serialising as a string keeps the JSON stable. If `Area::from_str` is missing for any variant, add it (alphabetical, one match arm).
6. The Fireball→wildfire chain (Q7) is **NOT** in this plan. It's flagged in the spec as droppable; defer to a follow-up bead unless playtest demands it.

**Type consistency check:** all references to `AreaId`, `TargetSpec`, `GameSnapshot`, `InterventionKind`, `Lure`, `DespawnReason`, `Gauges`, `GamemakerProfile`, `Gamemaker`, `ActiveIntervention` use the same names defined in Task 1/5/7/9. Method names: `record_intervention`, `recent_count`, `tick_phase`, `react_to`, `should_intervene`, `select_intervention`, `build_snapshot`, `tick_gamemaker`, `drain_expired`, `apply_damage`, `mutt_should_despawn`, `evade_bonus`, `gamemaker_override`. All consistent.

---

## Hand-off

After Task 15 commits cleanly, run:

```bash
just quality
```

Expected: clean. If anything fails, fix before opening the PR.

PR1 lands gamemaker backend behind the scenes. PR2 (separate plan) wires the hex-map markers and timeline cards.
