# Sponsorship System v1 — Design Spec

**Bead:** `hangrier_games-dvd`
**Date:** 2026-05-04
**Status:** Draft (awaiting review)

## 1. Summary

Replace the per-cycle pure-RNG `receive_patron_gift` mechanic with a **sim-driven NPC sponsor system**. Each game spawns a fixed roster of archetype-named sponsors (Aesthete, Gambler, Loyalist, Sadist, Compassionate, Strategist). Sponsors observe in-game events through a translated `AudienceEvent` stream, accumulate per-tribute affinity, and spend a finite per-game budget to gift items when high-magnitude events fire and a tribute they like is in need.

This reframes the original `dvd` description (player-driven SCC currency, gift catalog UI, leaderboard, anti-griefing surface) into a system that fits the project's spectator/director nature: no user accounts, no UI, no real-time interaction.

## 2. Goals & Non-Goals

### Goals

- Sponsor decisions are **legible** — a viewer can guess which archetype sent a gift from its style.
- **Deterministic** under fixed RNG seeds (snapshot-testable).
- **Cheap to extend** — new affliction/event specs add reactions by appending to a translator table and weight rows, no new infrastructure.
- **Genre-correct** — finite budgets that *do* run out (Haymitch ran dry in the books).
- Each archetype has a **distinct mechanical role** (no near-duplicates).

### Non-Goals (explicit)

- No user accounts, authentication, or human sponsor identity.
- No SCC currency, purchasing, or transaction ledger.
- No gift-targeting UI, no leaderboard, no real-time push.
- No anti-griefing surface (no human actors, nothing to grief).
- **No therapy / detox / in-game recovery for trauma or addiction.** Games last days; recovery wouldn't have time to land. Trauma severity and addiction state are non-recoverable by design — the tribute carries them until death or end-of-game.
- No dual-path rollout flag — PR2 fully replaces `receive_patron_gift`.

## 3. Conceptual Model

```
MessagePayload stream  ──translator──►  AudienceEvent stream
                                              │
                                              ▼
                       per-sponsor weight table × archetype modifiers
                                              │
                                              ▼
                                       affinity delta per (sponsor, tribute)
                                              │
                                              ▼
                          if AudienceEvent.magnitude_score ≥ TRIGGER_FLOOR:
                                   gift resolution pass
                                              │
                                              ▼
                       MessagePayload::SponsorGift { donor: <archetype name>, ... }
```

- **AudienceEvent** is a dedicated enum, *not* `MessagePayload`. Decouples sponsor logic from message-stream evolution.
- **Affinity** is per `(sponsor, tribute)` pair, bounded `[-100, 100]`.
- **Gift resolution** is event-triggered (no periodic threshold polling).

## 4. Domain Types

### 4.1 `shared/src/audience.rs` (new)

```rust
pub enum AudienceEvent {
    KillMade        { actor: TributeId, victim: TributeId, magnitude: u32, modifier: f32 },
    KillReceived    { victim: TributeId, actor: TributeId, magnitude: u32, modifier: f32 },
    AttackTrapped   { actor: TributeId, victim: TributeId },        // covers gum2
    RescueAlly      { actor: TributeId, ally: TributeId },
    AllianceFormed  { tributes: Vec<TributeId> },
    BetrayalCommitted { actor: TributeId, victim: TributeId },
    AfflictionAcquired { tribute: TributeId, kind: AfflictionKindTag },
    SurvivedAreaEvent  { tribute: TributeId },
    UnderdogVictory    { actor: TributeId, victim: TributeId },     // synthesized
    DistrictLoyaltyAct { actor: TributeId, district: u8 },          // synthesized
    Cowardice          { tribute: TributeId },                      // synthesized
}

impl AudienceEvent {
    pub fn magnitude_score(&self) -> u32 { /* base × modifier */ }
}
```

### 4.2 `shared/src/sponsors.rs` (new)

```rust
pub struct Sponsor {
    pub id: SponsorId,
    pub archetype: ArchetypeId,         // reference to static catalog
    pub budget_remaining: u32,
    pub bound_district: Option<u8>,     // Some(d) for Loyalist, None for others
    pub affinity: HashMap<TributeId, i32>,
}

pub struct Archetype {
    pub id: ArchetypeId,
    pub canonical_name: &'static str,   // "Aesthete", "Loyalist", etc. — used as `donor`
    pub budget_band: (u32, u32),
    pub event_weights: &'static [(AudienceEventKind, i32)],
    pub gift_preferences: &'static [(ItemKindTag, u32)], // bias weights
}

pub static ARCHETYPES: &[Archetype] = &[ /* 6 entries */ ];

pub const ARCHETYPE_PRIORITY_ORDER: &[ArchetypeId] = &[
    ArchetypeId::Aesthete, ArchetypeId::Strategist, ArchetypeId::Compassionate,
    ArchetypeId::Gambler,  ArchetypeId::Sadist,     ArchetypeId::Loyalist,
];
```

### 4.3 `game/src/sponsors/mod.rs` (new)

```rust
pub struct SponsorContext<'a> {
    pub game: &'a Game,
    pub tributes: &'a [Tribute],
}

pub trait ArchetypeModifiers {
    fn district_loyalty_modifier(&self, ev: &AudienceEvent, ctx: &SponsorContext) -> f32 { 1.0 }
    fn combat_style_modifier(&self,    ev: &AudienceEvent, ctx: &SponsorContext) -> f32 { 1.0 }
}
```

Only Loyalist overrides `district_loyalty_modifier`; only Aesthete overrides `combat_style_modifier`.

### 4.4 `Game` field

```rust
pub struct Game {
    // ...existing...
    pub sponsors: Vec<Sponsor>,
}

impl Game {
    pub fn spawn_sponsors(&mut self, rng: &mut impl Rng) { /* one per archetype, roll district + budget */ }
    pub fn sponsor_affinity_snapshot(&self) -> /* test helper */ { ... }
}
```

## 5. The Six Archetypes

| Archetype | Role | Budget Band | Loves | Hates |
|---|---|---|---|---|
| **Aesthete** | Style scorer | 80–120 | Clean kills, critical hits, weapon kills | Sloppy fights, traps, environmental kills |
| **Gambler** | Underdog backer | 60–100 | Underdog victories, last-stand survival | Front-runners, boring outcomes |
| **Loyalist** | District devotee | 30–60 | Acts by tribute from `bound_district` | Acts against `bound_district` |
| **Sadist** | Suffering enthusiast | 50–90 | Slow deaths, betrayals, attacks-on-trapped | Mercy, alliances |
| **Compassionate** | Hero supporter | 70–110 | Rescues, alliances, surviving area events | Betrayals, attacks-on-trapped |
| **Strategist** | Skilled-play scorer | 70–110 | Multi-kill cycles, smart positioning, alliances of convenience | Reckless rushes |

Budgets and weights are **first-pass placeholders**; tuning iteration expected post-PR2.

## 6. Translator (`MessagePayload` → `AudienceEvent`)

Single function in `game/src/sponsors/mod.rs`:

```rust
pub fn translate(payload: &MessagePayload, ctx: &SponsorContext) -> Vec<AudienceEvent>
```

Returns 0..N events per payload (some payloads synthesize multiple, e.g. a kill emits both `KillMade` and possibly `UnderdogVictory`).

**Mapping table (excerpt):**

| MessagePayload variant | → AudienceEvent(s) |
|---|---|
| `TributeKilled { victim, killer }` | `KillReceived` + (`KillMade` if `killer.is_some()`) + maybe `UnderdogVictory` |
| `TributeAttacked { victim, attacker }` | `AttackTrapped` if `victim.is_trapped()` |
| `TrappedEscaped { tribute, helper }` | `RescueAlly { actor: helper, ally: tribute }` |
| `AllianceFormed { tributes }` | `AllianceFormed` |
| `BetrayalTriggered { actor, victim }` | `BetrayalCommitted` |
| `AfflictionAcquired { tribute, kind }` | `AfflictionAcquired` |
| `SurvivedAreaEvent { tribute }` | `SurvivedAreaEvent` |

Synthesized events (no direct payload) are emitted by inspecting `ctx.game` state at translation time.

## 7. Affinity Update Algorithm

```
for each sponsor in game.sponsors:
    for each event in audience_events:
        base = sponsor.archetype.event_weights[event.kind()]
        district_mod = sponsor.district_loyalty_modifier(event, ctx)
        style_mod    = sponsor.combat_style_modifier(event, ctx)
        delta        = (base as f32 * event.magnitude_modifier() * district_mod * style_mod) as i32

        for tribute_id in event.affected_tributes():
            entry = sponsor.affinity.entry(tribute_id).or_insert(0)
            *entry = (*entry + delta).clamp(MIN_AFFINITY, MAX_AFFINITY)
```

## 8. Gift Resolution (PR2)

After each cycle's events are translated and affinities updated:

```
for event in audience_events where event.magnitude_score() >= TRIGGER_FLOOR:
    for tribute_id in event.affected_tributes():
        candidates = game.sponsors
            .iter()
            .filter(|s| s.affinity[tribute_id] >= AFFINITY_FLOOR)
            .filter(|s| s.budget_remaining > 0)
            .filter(|s| !already_gifted_this_cycle(tribute_id))

        winner = candidates.max_by_key(|s| (s.affinity[tribute_id], priority_rank(s.archetype)))

        if let Some(sponsor) = winner:
            item = pick_gift(sponsor, tribute, &ctx)?
            sponsor.budget_remaining -= item.cost
            emit MessagePayload::SponsorGift { recipient: tribute, item, donor: sponsor.canonical_name }
            mark_gifted(tribute_id)
```

- **Max 1 gift per tribute per cycle.** Cross-tribute gifts unlimited.
- **No fallback to second-place** if winner can't afford anything affordable enough — keeps the rule simple.
- **`pick_gift`**: weighted-random over the full affordable catalog (cost ≤ budget), biased by `archetype.gift_preferences`. Bias, not restriction — Sadist *can* send water in an emergency.

## 9. Numeric Constants

```rust
pub const MIN_AFFINITY: i32 = -100;
pub const MAX_AFFINITY: i32 =  100;
pub const AFFINITY_FLOOR: i32 = 25;     // candidacy threshold
pub const TRIGGER_FLOOR: u32  = 8;      // event magnitude required to trigger resolution

// Budget bands per archetype: see §5 table.
// Item costs (first pass):
pub const ITEM_COSTS: &[(ItemKindTag, u32)] = &[
    (ItemKindTag::Food,        5),
    (ItemKindTag::Water,       5),
    (ItemKindTag::Bandage,    10),
    (ItemKindTag::Antidote,   18),
    (ItemKindTag::Map,        12),
    (ItemKindTag::Signal,     20),
    (ItemKindTag::WeaponBasic,25),
    (ItemKindTag::WeaponRare, 45),
    (ItemKindTag::Shield,     30),
];
```

## 10. PR Breakdown

### PR1 — Foundation (no behavior change yet)

- `shared/src/audience.rs`, `shared/src/sponsors.rs`
- `game/src/sponsors/mod.rs` (translator + `SponsorContext` + `ArchetypeModifiers`)
- `Game::sponsors` field, `spawn_sponsors`, `sponsor_affinity_snapshot`
- Per-cycle hook: translate events → update affinities (NO gifting)
- `receive_patron_gift` keeps running unchanged
- Game-load hook: `if game.sponsors.is_empty() { spawn_sponsors }`
- Loyalist + Aesthete modifier impls
- Affinity-clamp proptest (256 cases)
- Snapshot test: 3-cycle fixed-seed game, snapshot affinity evolution

### PR2 — Replacement

- Gift resolution + delivery
- Delete `receive_patron_gift`
- Update `MessagePayload::SponsorGift` `donor` to use archetype canonical name
- Re-snapshot tests that asserted `donor: "Sponsor"`
- Close `gum2` (covered by `AttackTrapped` translator + Sadist/Compassionate weights)

### Cut Order (if PR1 grows too large)

1. Drop synthesized events (UnderdogVictory, Cowardice) — add later.
2. Drop Aesthete `combat_style_modifier` — Loyalist alone proves the modifier-hook seam.
3. Drop snapshot test — keep proptest only.
4. Drop AfflictionAcquired / SurvivedAreaEvent translator entries — add when the corresponding affliction PRs land.

## 11. Migration & Compatibility

- **In-progress games:** game-load hook spawns sponsors lazily if the field is empty. No schema migration script needed (Rust struct default for the `Vec` field).
- **Existing snapshot tests** asserting `donor: "Sponsor"` will break in PR2 — re-snapshot then.
- **Trauma + addiction specs** currently reference "deferred to dvd Therapy/Detox." Those references must be amended to "no in-game recovery — tribute carries until death/end-of-game." Separate small PR after sponsorship PR1 lands.

## 12. Open Questions

1. Should `bound_district` be re-rolled if the Loyalist's whole district dies before any gift fires? (v1: no — they sit out the rest of the game with their unspent budget. Genre-correct: their tribute is dead, they go home.)
2. Should event base-weights live in the archetype struct (current plan) or in a single `EVENT_BASE_TABLE` keyed by `AudienceEventKind` × archetype? (Current plan keeps per-archetype locality; alternative centralizes tuning.)
3. Item-cost lookup: extend `Item` struct, or separate `ITEM_COSTS` table? (Current plan: separate table — avoids touching `Item`.)

## 13. Self-Review Checklist

- [x] Non-goals explicitly list "no recovery" so trauma/addiction specs know to amend.
- [x] PR1 introduces no behavior change to existing flows (`receive_patron_gift` untouched).
- [x] PR2 cleanly replaces the old path in one commit.
- [x] All numeric constants centralized in §9.
- [x] Cut order defined per PR.
- [x] `MessagePayload::SponsorGift` payload shape unchanged (only `donor` value changes).
- [x] `gum2` (spectator-disapproval for AttackTrapped) is satisfied by translator entry + weight rows — no new infra.
- [x] Deterministic under fixed seed (single RNG threaded through `spawn_sponsors` and `pick_gift`).
- [x] Affinity bounded; proptest planned.
- [x] Future affliction specs extend by appending to translator + weight rows only.

## 14. Follow-up Beads (to file after spec approval)

- Sponsorship PR1 (P2, no hard prereqs, blocks PR2)
- Sponsorship PR2 (P2, blocked on PR1, blocks closure of `gum2`)
- Spec amendment: trauma + addiction recovery language (P3, after PR1)
