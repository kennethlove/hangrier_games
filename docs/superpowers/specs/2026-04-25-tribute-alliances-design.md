# Tribute Alliances — Design Spec

**Bead:** `hangrier_games-0ug`
**Date:** 2026-04-25
**Status:** Draft v2 (awaiting user review)

## 1. Problem & Goal

Today every tribute is hostile to every other tribute. The simulation has no
mechanism for cooperation, which removes a core dynamic of the source
material: small bands of tributes that travel together, share food, watch
each other's backs, and eventually turn on one another.

This spec adds an **AI-driven alliance system** to the game engine. Tributes
form pair-wise alliances based on their traits and district, allies stop
attacking each other and pool their perception inside a shared area, and
alliances dissolve through sanity loss, betrayal, or grief.

The first version is fully autonomous: no player UI for accept/reject. Player
control over alliance decisions is a separate later feature.

## 2. Scope

**In scope (v1):**

- Replace `BrainPersonality` enum with a `traits: Vec<Trait>` system **on
  `Tribute`** (not `Brain`). A tribute with no traits behaves as the old
  `Balanced` baseline.
- A small set of physical/medical traits ship alongside personality traits
  (e.g. `Asthmatic`, `Nearsighted`, `Tough`).
- District-biased trait pools at tribute generation.
- Encounter-driven alliance formation with trait- and district-based rolls.
- **Pair-wise alliance graph** stored as `allies: Vec<TributeId>` on
  `Tribute`. No `alliance_id`. No central alliance entity. No `Uuid`s for
  alliances.
- Allies excluded from `pick_target` (filter inside `pick_target` against
  live state, not snapshot); same-area passive shared perception.
- Three break triggers: low sanity, Treacherous active betrayal, ally-death
  sanity cascade.
- Human-readable "deciding factor" surfaced in formation/break event messages.
- Hard cutover: existing `BrainPersonality` is removed; existing tribute rows
  are not migrated; dev DB resets.

**Out of scope (filed as follow-ups):**

1. Active `SeekAlly` action — tribute spends a turn explicitly looking for
   allies.
2. Cross-area ally location pings — knowing where your allies are across the
   map.
3. Duel feature — explicit 1v1 combat that bypasses ally filters.
4. Trait categories + caps (e.g. "max one combat-stance trait").
5. Expanded physical/medical trait library.
6. Full `alliance` SurrealDB entity with graph edges and history (ties to
   replays `5wt` and spectator `wxn`).
7. Player UI for accept/reject of alliance offers.

## 3. The Alliance Graph

This v2 spec drops the central-alliance model from v1 in favor of an explicit
pair-wise graph.

**Why pair-wise:** the user wants overlapping but distinct alliances —
"Peeta×Katniss does not overlap with Katniss×Rue except on Katniss." A single
`alliance_id` per tribute cannot represent Katniss being in two distinct
alliances simultaneously. A graph can.

**Storage:** `allies: Vec<TributeId>` on each `Tribute`. Symmetric: when A
allies with B, both A's and B's lists gain the other. Betrayal removes only
the symmetric pair (A from B's list, B from A's list).

**No transitive ally inference.** If Peeta×Katniss and Katniss×Rue are both
alliances, Peeta and Rue are **not** allies. Each pair is its own bond. This
matches the "subtle social politics" use case.

**Per-tribute cap:** at most **5 simultaneous direct allies**. Soft target
2–3. The cap is per-tribute; there is no group cap because there are no
groups.

**TributeId:** the existing `Tribute::id` (`uuid::Uuid` per `tributes/mod.rs`)
serves as the stable identifier inside the `allies: Vec<Uuid>` list.

## 4. Architecture

The change is centered in the `game/` crate (pure engine), with thin
extensions to `shared/` (DTOs), `api/` (response shapes + persistence), and
SurrealDB schema. The frontend is unaffected for v1 beyond receiving an extra
field on tribute DTOs.

```
game/src/tributes/
    mod.rs         Tribute gains `traits: Vec<Trait>` and
                   `allies: Vec<Uuid>`. pick_target filters allies
                   against live self.allies (not snapshot).
                   The existing `district != self.district` filter is
                   REMOVED. The existing `loyalty` field and
                   LOYALTY_BREAK_LEVEL betrayal branch are REMOVED.
    brains.rs      BrainPersonality removed. Brain.personality field
                   removed. PersonalityThresholds derived from the
                   Tribute's trait set (passed in or read via &Tribute).
                   Brain-related tests updated.
    traits.rs      NEW. Trait enum, conflict table, district pools,
                   threshold modifiers, alliance_affinity, label,
                   random trait set generator, refuser set.
    alliances.rs   NEW. Alliance formation roll, alliance break checks,
                   shared-perception helper, deciding-factor calculator.
                   All operate on `&Tribute` and `Vec<Uuid>` ally lists.
    combat.rs      No new branch. Betrayal mutates self.allies before
                   the attack runs; pick_target then naturally allows
                   the strike because the symmetric ally edge is gone.
    inventory.rs   No change.
shared/src/
    tribute DTO    Adds `allies: Vec<Uuid>` (default empty).
                   Adds `traits: Vec<Trait>` (default empty).
api/src/
    tributes.rs    Reads/writes allies + traits.
    games.rs       Day/night cycle persists allies + traits changes.
                   Cascade event-queue drain (see §7) lives here.
schemas/
    tribute.surql  Adds `allies: array<uuid>` and `traits: array<string>`
                   (or array of typed enum if SurrealDB supports it
                   cleanly; otherwise serialize traits as bare strings
                   matching Serde's default unit-variant repr).
migrations/definitions/
    _initial.json  Bumped; release note explains hard cutover.
```

The trait system is a self-contained module that the brain and alliance
modules consume. Combat resolution does not branch — betrayal is "mutate ally
list, then run normal attack." Nothing else moves.

## 5. Trait System

### 5.1 Trait enum

```rust
pub enum Trait {
    // Combat stance (former BrainPersonality territory)
    Aggressive,
    Defensive,
    Cautious,
    Reckless,
    // Social
    Friendly,
    Loyal,
    Paranoid,
    LoneWolf,
    Treacherous,
    // Mental
    Resilient,
    Fragile,
    Cunning,
    Dim,
    // Physical / medical (v1 starter set; expansion is a follow-up)
    Asthmatic,
    Nearsighted,
    Tough,
}
```

`Balanced` from the old enum is dropped — its semantics ("no strong stance")
are now expressed as **a tribute with zero traits**. Such a tribute uses the
default thresholds, which are the same numbers `BrainPersonality::Balanced`
used today.

Each `Trait` exposes:

- `threshold_modifiers(&self) -> ThresholdDelta` — additive contribution to
  `PersonalityThresholds`. Stacking multiple traits sums the deltas.
- `alliance_affinity(&self) -> f64` — multiplicative factor on the base
  ally roll. `f64` for arithmetic consistency with the rest of the codebase
  (see also `Brain::preferred_action_percentage`). Combat-stance and Mental
  traits return `1.0` (neutral on alliance affinity); Social traits carry
  the real signal; Physical traits are neutral on affinity unless a future
  trait makes social sense (e.g. a hypothetical `Charming`).
- `label(&self) -> &'static str` — short narrative label used in event
  messages ("loyal", "paranoid", "asthmatic", etc.).

### 5.2 Trait count and selection

Each tribute gets **2–6 traits** when *the district pool can support that
count*; otherwise the tribute gets as many as the pool can fit, with a hard
floor of 0 (i.e. a tribute may legitimately have zero traits if the pool is
empty after conflict-rejection — see §5.3). The 2–6 range is rolled
uniformly per tribute. Traits are drawn one at a time from a district-
weighted pool, rejecting any that conflict with already-selected traits.

If the count roll lands on a number larger than the pool can satisfy
(after conflict rejection), the generator stops at whatever it has. **No
infinite retry loops on small pools.** A "minimum 2" floor is *only*
attempted if the pool can fit two non-conflicting picks; otherwise the floor
is silently relaxed. In practice the smallest pool has more than enough
non-conflicting members; this clause exists to prevent generator hangs.

### 5.3 Conflict table

Eight pairwise conflicts. A tribute may not hold both members of a pair.

| A          | B           | Reason                       |
|------------|-------------|------------------------------|
| Friendly   | Paranoid    | trust vs distrust            |
| Loyal      | Treacherous | core opposites               |
| Loyal      | LoneWolf    | bonds vs avoids bonds        |
| Aggressive | Cautious    | rush vs retreat              |
| Aggressive | Defensive   | attack vs hold               |
| Reckless   | Cautious    | YOLO vs careful              |
| Resilient  | Fragile     | direct opposites             |
| Cunning    | Dim         | direct opposites             |

Notable allowed combos:

- **Friendly × Treacherous** — the snake-in-the-grass archetype. Friendly
  pulls them into alliances; Treacherous breaks those alliances explosively.
- **Paranoid × LoneWolf** — coherent loner who distrusts.

Conflicts live in a single `const CONFLICTS: &[(Trait, Trait)]` table for
easy editing. The `traits.rs` module exposes `conflicts_with(a, b) -> bool`
that checks both orderings.

A separate `const REFUSERS: &[Trait] = &[Paranoid, LoneWolf]` is consumed by
the alliance-formation gate (§6.2), so the gate is an O(1) membership check
rather than walking the conflict table.

### 5.4 District pools

Each district has a weighted pool reflecting its source-material flavor.
Weights are starting suggestions and explicitly subject to balance later.
Pools are exclusive (a trait not listed in a district's pool cannot appear in
that district's tributes); this is a deliberate flavor statement.

| District | Pool (trait, weight)                                                    |
|----------|--------------------------------------------------------------------------|
| 1        | Loyal 4, Aggressive 4, Paranoid 3, Tough 2                              |
| 2        | Aggressive 4, Defensive 4, Loyal 3, Tough 2                             |
| 3        | Cunning 4, Cautious 3, Dim 2, Nearsighted 2, Asthmatic 1                |
| 4        | Resilient 4, Aggressive 3, Loyal 3, Tough 2                             |
| 5        | Cunning 4, Cautious 3, Treacherous 2                                     |
| 6        | Fragile 3, Friendly 3, Asthmatic 2, Nearsighted 2                       |
| 7        | Resilient 4, Defensive 3, Tough 3                                        |
| 8        | Fragile 2, Friendly 4, Loyal 3, Asthmatic 2                             |
| 9        | Cautious 3, Friendly 3, Asthmatic 2                                      |
| 10       | Resilient 4, Defensive 3, Tough 3                                        |
| 11       | Loyal 3, Friendly 4, Resilient 3, Tough 2                                |
| 12       | Resilient 3, LoneWolf 3, Cunning 3, Asthmatic 2                          |

Pools live in `traits.rs` as 12 separate `const DISTRICT_N_POOL: &[(Trait,
u8)]` plus a `pub fn pool_for(district: u8) -> &'static [(Trait, u8)]`
lookup, mirroring the pattern `districts::assign_terrain_affinity` already
uses.

### 5.5 Threshold computation

`PersonalityThresholds` keeps its current shape (9 fields, ±20% per-tribute
variance). Base values come from the old `Balanced` numbers
(health 20/40, sanity 10/20/35, movement 10, intelligence 35/80,
psychotic-break 8). Each trait contributes additive deltas; deltas are
summed across the tribute's trait set. The final thresholds clamp to
[0, 100] for the four core attribute thresholds and [0, 20] for the
psychotic-break threshold.

A tribute with **no traits** gets the base thresholds with the standard
±20% per-tribute variance, reproducing today's `Balanced` behavior. This
makes the existing 60-test suite tractable: tests that need predictable
thresholds construct tributes with zero traits.

The exact delta values per trait are framed but not tuned in this spec —
they will be balanced in implementation against the existing baseline. The
locked schema-level constraint is: traits modify the same nine threshold
fields the old `BrainPersonality` did, additively, with `f64` arithmetic
internally and final clamp before storage as the existing integer types.

## 6. Alliance Formation

### 6.1 Trigger

Alliances form **at encounter time**: when a tribute's `pick_target` step
sees another living, non-allied tribute in the same area, the engine first
runs an **ally roll** between them. If the roll succeeds, both add the other
to their `allies` list, and the would-be attacker takes a non-attack action
this turn instead. If it fails, combat proceeds as normal.

This adds no new turn phase. The roll lives inside the existing target-
selection path in `game/src/tributes/mod.rs::pick_target`, before the
damage-roll step.

`base_chance` is **per-encounter**, not per-turn. Within a single turn, A's
turn might roll an ally check against B; later in the same turn, B's turn
will not roll again because they're now allies (B's `allies` already
contains A — see §6.4 on snapshot freshness).

### 6.2 Roll formula

```
trait_factor       = geometric_mean(self.traits.alliance_affinity())
                       defaulting to 1.0 if traits is empty
target_factor      = geometric_mean(target.traits.alliance_affinity())
                       defaulting to 1.0 if target.traits is empty
district_bonus     = if same_district { 1.5 } else { 1.0 }
self_cap_pen       = (5 - self.allies.len() as f64) / 5.0
target_cap_pen     = (5 - target.allies.len() as f64) / 5.0
base_chance        = 0.20

roll_chance        = clamp(base_chance * trait_factor * target_factor
                               * district_bonus * self_cap_pen
                               * target_cap_pen,
                               0.0, 0.95)
```

**Geometric mean instead of raw product.** Raw products of 2 vs 6 trait
counts produce wildly different magnitudes purely from count (`0.5²=0.25`
vs `0.5⁶=0.0156`). Geometric mean (`product^(1/n)`) makes counts comparable.

**Helper sketch.** Lives in `traits.rs`, pure fn, no allocations beyond the
caller's slice:

```rust
/// Geometric mean of trait affinity values. Returns 1.0 for empty input
/// so trait-less tributes neither boost nor penalize the roll.
pub fn geometric_mean_affinity(traits: &[Trait]) -> f64 {
    if traits.is_empty() {
        return 1.0;
    }
    let n = traits.len() as f64;
    let product: f64 = traits.iter().map(|t| t.alliance_affinity()).product();
    product.powf(1.0 / n)
}
```

`f64::product` over the v1 affinity range `[0.5, 1.5]` cannot overflow or
underflow at the trait counts we generate (max 6). No `NaN` guard needed
because all `alliance_affinity()` values are positive finite. Unit test
covers empty, single-trait identity, and known multi-trait cases against
hand-computed values within `f64::EPSILON * 10`.

**Affinity range.** All trait `alliance_affinity()` values live in
`[0.5, 1.5]` for the v1 set:

- Friendly: `1.5`
- Loyal: `1.4`
- Treacherous: `1.2`
- Combat / Mental / Physical traits not listed below: `1.0`
- LoneWolf: `0.6`
- Paranoid: `0.5`

With this range, a 6-trait geometric mean stays in `[0.5, 1.5]`, and
`trait_factor * target_factor` stays in `[0.25, 2.25]`. Combined with the
0.20 base, 1.5 district bonus, and cap penalties, `roll_chance` lands in a
sensible `[0, 0.95]` band before clamp.

**Both cap penalties multiplied in.** A tribute already at `allies.len() = 5`
contributes `0.0`, refusing all new alliances regardless of the other
tribute's state. A tribute at `allies.len() = 0` contributes `1.0`.

**Refuser gate.** Before the roll runs, both tributes must pass:

```rust
fn passes_gate(self_t: &Tribute, target: &Tribute) -> bool {
    let has_positive = |t: &Tribute| {
        t.traits.iter().any(|x| x.alliance_affinity() >= 1.0)
    };
    let has_refuser = |t: &Tribute| {
        t.traits.iter().any(|x| REFUSERS.contains(x))
    };
    (has_positive(self_t) && has_positive(target))
        || (!has_refuser(self_t) && !has_refuser(target))
}
```

A tribute with `[Friendly, Paranoid]` (positive + refuser) paired with
`[Loyal]` (positive, no refuser) **passes** because both have positives —
which is the intended behavior for snake-in-the-grass and other mixed
archetypes. Paranoid×Paranoid fails the gate (no positives, both refusers).

### 6.3 Pair-wise assignment

Roll succeeded between A and B:

1. `A.allies.push(B.id)` if not already present.
2. `B.allies.push(A.id)` if not already present.
3. Cap check is **already enforced by the roll formula** via `cap_pen`. If A
   was at 4 and B at 4, the formula already favored a low chance; if A or B
   was at 5, `cap_pen = 0` and the roll could not succeed. The push step
   does not need to re-check the cap.

There is no merge logic. There are no group identities to merge. Two
overlapping pairs (Peeta×Katniss and Katniss×Rue) coexist by construction.

### 6.4 Snapshot freshness

The existing turn loop in `games.rs` builds `tributes_by_area` and
`potential_targets` from cloned snapshots before iterating tributes. This
matters for alliance state:

- **Ally filtering happens inside `pick_target`** against the **live
  `&self`** (which has the current `allies` list) and against the **target
  clone** (whose `allies` list was frozen at snapshot time). The asymmetry
  is acceptable because alliance edges are symmetric: once A allies with B,
  A's live `allies` contains B, and `pick_target` will skip B without
  needing B's clone to know about A.
- **Mid-turn alliance changes are visible to the acting tribute** (their own
  list is live) but **not retroactively visible to earlier-iterated
  tributes' clones**. In practice this only matters for shared perception:
  a tribute earlier in the turn does not benefit from a perception pool
  with an ally formed later in the same turn. Acceptable for v1.

### 6.5 Deciding factor in events

Every formation event includes a one-line deciding factor based on the
dominant contributor. Examples:

- "Peeta allies with Katniss. Deciding factor: Peeta is friendly."
- "Cato allies with Glimmer. Deciding factor: same district (D1)."
- "Thresh allies with Rue. Deciding factor: Thresh is loyal."

The dominant contributor is computed by comparing `district_bonus` against
the maximum trait affinity on either side. The factor with the largest
contribution above 1.0 wins. Ties resolve by trait label sort order
(deterministic) for reproducibility. If no factor exceeds 1.0 (a roll won
purely by base chance and luck), the event omits the deciding factor:
"Peeta allies with Katniss."

## 7. Alliance Lifecycle

### 7.1 Shared perception (same-area only)

When the brain assembles its observations, if there are direct allies present
in the **same area**, their perception unions with this tribute's:

- Visible nearby tributes (`EnvironmentContext::nearby_tributes`)
- Visible items in the area
- A hidden ally in the same area still shares perception (you know they're
  there even if outsiders don't).

Cross-area allies contribute nothing in v1. Allies in the same area
effectively share a sensory fact pool for that turn's decision.

Because the perception pool is built from the snapshot
`tributes_by_area`, perception sharing is consistent with snapshot freshness
described in §6.4.

### 7.2 No friendly fire

`pick_target` filters out any tribute whose `id ∈ self.allies`. This is the
rule. There is no second guard inside combat resolution; the existing
"`district != self.district`" filter is **removed entirely** along with the
existing `loyalty` field and `LOYALTY_BREAK_LEVEL` betrayal path. Same-
district tributes are no longer auto-friendly; they have to ally like
anyone else (with a district bonus stacking the odds).

### 7.3 Break triggers

Three triggers, checked in this order each turn:

**(a) Sanity threshold.** When a tribute's sanity drops below their
`PersonalityThresholds::low_sanity_limit`, they roll once to leave **each**
of their direct alliances. The roll is `rng.random_bool(deficit_ratio)`
where `deficit_ratio = (low_sanity_limit - current_sanity) / low_sanity_limit`
clamped to `[0.0, 1.0]`. On success for a given ally, the symmetric pair is
removed from both sides. Event: "X leaves their alliance with Y — losing
their nerve."

**(b) Treacherous active betrayal.** A Treacherous tribute checks for a
betrayal opportunity every 5 turns from their **own** turn counter (per-
tribute timer stored on `Tribute` as `turns_since_last_betrayal: u8`,
initialized 0, incremented each turn, reset on betrayal or when the timer
elapses without an opportunity). When the timer elapses, if there is an ally
in the same area, they:

1. Remove the symmetric pair: `self.allies.retain(|x| *x != victim.id)`,
   and append `(self.id, victim.id)` to a per-cycle event queue so the
   victim's `allies` is updated when the queue drains (§7.5).
2. Resolve a normal attack against the victim. The attack runs through the
   standard `pick_target` → `attacks()` path; because the symmetric pair is
   already gone from `self.allies`, the filter allows it.
3. Reset the betrayal timer.

If no ally is in the same area when the timer elapses, the timer **resets**
to 0 (one missed opportunity per cycle is fine; do not let timers stack).

Event: "X betrays Y — true to their treacherous nature."

**Heaviness rule.** Per the user: *"It's not the killing that's heavy, it's
the betrayal."* The cascade in (c) below is triggered by the **betrayal
event itself**, not by a death. The cascade fires on the betrayed tribute's
**next turn** if they are still alive, regardless of whether the betrayer
dies in the counter-attack. If the betrayed tribute died during the
counter-attack, no cascade roll (they are dead).

**(c) Trust-shock cascade (renamed from "ally-death sanity cascade").**
Two sub-triggers:

1. **Betrayal trust-shock** (per the heaviness rule above). On the betrayed
   tribute's next turn, they roll a sanity check against
   `low_sanity_limit` (consistent with §7.3a, not `mid_sanity_limit`).
   Roll: `rng.random_bool(0.5 + 0.5 * deficit_ratio)` — high baseline
   because betrayal is severe. On success: that survivor removes the
   symmetric pair with **every other current direct ally** as well (the
   trust-shock ripples outward), each with event "Y is shaken by X's
   betrayal and breaks from Z." **The betrayer never rolls trust-shock
   for their own betrayal** — they chose to do it, so no shock to their
   own trust. (If the betrayer is also a betrayal *victim* of someone
   else, that separate event would still apply to them.)
2. **Ally-death cascade.** When any tribute dies, every direct ally on the
   per-cycle event queue rolls a sanity check against `low_sanity_limit`
   (consistent with §7.3a). Roll: `rng.random_bool(deficit_ratio)`. On
   success: that ally removes the symmetric pair with the deceased and
   emits "Y is shaken by X's death and breaks from the alliance." Because
   alliances are pair-wise, no group dissolution is needed — the bond
   simply ends.

Both sub-triggers drain through the same per-cycle event queue, processed
between tribute turns so they avoid the borrow-checker problem of cross-
tribute mutation during `self.tributes.iter_mut()`.

### 7.4 Combat carve-out for betrayal — none required

Betrayal mutates `self.allies` *before* `pick_target` runs, so the standard
filter naturally allows the strike. `combat.rs` does not change. The flow
lives in `alliances.rs` (the betrayal trigger and the ally-list mutation)
and in `mod.rs::pick_target` (the live filter consults `self.allies`).

### 7.5 Per-cycle event queue (mechanism)

A `Vec<AllianceEvent>` lives on the `Game` struct (or is threaded through
the cycle as a parameter — implementation choice; both work). Events:

```rust
enum AllianceEvent {
    BetrayalRecorded { betrayer: TributeId, victim: TributeId },
    DeathRecorded { deceased: TributeId, killer: Option<TributeId> },
}
```

The queue is appended to inside individual tribute turns. After each
tribute's turn completes (and the mutable borrow is released), the cycle
drains the queue in order:

- For each `BetrayalRecorded`: ensure the symmetric pair is removed on the
  victim's side (the betrayer's side was already updated in §7.3b step 1);
  schedule a trust-shock roll on the victim's next turn. **The drain never
  schedules a trust-shock for the betrayer themselves** — `BetrayalRecorded`
  identifies betrayer and victim as distinct fields, and only the `victim`
  is enqueued for trust-shock. The betrayer's other allies are unaffected
  by this event (their reactions, if any, come through `DeathRecorded`
  cascades when bodies fall, not from the betrayal act itself).
- For each `DeathRecorded`: roll the cascade for each direct ally of the
  deceased, removing pairs and emitting events. The deceased's `killer`
  field is informational only (used for event text); the cascade does
  not skip the killer if they happened to also be an ally of the deceased
  — that scenario is exactly the betrayal case, where the symmetric pair
  was already torn down in §7.3b before the kill landed, so the killer
  is no longer in the deceased's ally list at drain time.

This drains **between** tribute turns, not at end-of-cycle, so cascades
appear immediately in the message stream. Storywise indistinguishable from
"instant," and avoids any cross-tribute mutation under `iter_mut`.

## 8. Persistence

### 8.1 Schema

`schemas/tribute.surql` gains:

```surql
DEFINE FIELD allies ON tribute TYPE array<uuid> DEFAULT [];
DEFINE FIELD traits ON tribute TYPE array<string> DEFAULT [];
DEFINE FIELD turns_since_last_betrayal ON tribute TYPE int DEFAULT 0;
```

Traits serialize as bare strings via Serde's default unit-variant repr
(matching how `BrainPersonality` serialized today). No `alliance` table.
No graph edges. (The full first-class `alliance` entity is filed as a
follow-up tied to `5wt`/`wxn`.)

### 8.2 Migration

**Hard cutover, no migration script.** `migrations/definitions/_initial.json`
is bumped; the bump's release note will read approximately:

> 0ug: traits replace BrainPersonality; allies + traits +
> turns_since_last_betrayal added on tribute. Existing tribute rows are not
> migrated. Reset your dev DB.

Because there are no production users and the dev DB resets cheaply, this
is acceptable. If a future change ever needs preserved tribute data, write
a proper migration then.

### 8.3 Rust Serde details

On `Tribute`:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub allies: Vec<Uuid>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub traits: Vec<Trait>,

#[serde(default)]
pub turns_since_last_betrayal: u8,
```

`#[serde(default)]` on all alliance/trait fields tolerates older serialized
tribute snapshots (in-memory clones, test fixtures, partial DTO updates)
that lack these fields — matches the existing house style on `Tribute`
(`items`, `events`, `editable`, `terrain_affinity` already use this
pattern).

`Brain` no longer has a `personality` field. `Brain::default()` constructs
a brain whose threshold lookup will go through the tribute's trait set when
called via `Tribute::compute_thresholds(&self) -> PersonalityThresholds`
(or equivalent). The `#[serde(skip)]` on `Brain` inside `Tribute` stays —
brain has no persisted state in v1.

### 8.4 API exposure

`shared/`'s tribute DTO gains `allies: Vec<Uuid>` and `traits: Vec<Trait>`
(both default empty). API responses include them. The frontend is free to
ignore them for v1; future work can color-group allies in the tribute list
and surface trait labels in tribute detail views.

The Dioxus query cache key may depend on serialized DTO shape; bumping the
cache version (or a one-time hard reload after deploy) is the simplest path.
Both new fields are additive and `Option`/`Vec`-defaulted, so the schema is
backward-compatible at the type level.

## 9. Events & Observability

New event types (plain `GameOutput` strings for v1; structured events
arrive when `mqi` lands):

- Alliance formed (with deciding factor, both names).
- Sanity break leave (per ally, with name).
- Treacherous betrayal (both names + "treacherous" label).
- Betrayal trust-shock leave (per ally on victim's side).
- Ally-death cascade leave (per ally with the deceased's name).

Every event includes the relevant tribute names and, where applicable, the
trait label that drove the decision. This satisfies the "human-readable
deciding factor" requirement.

## 10. Testing

`game/` is the right place for the bulk of tests (rstest, parameterized).

**Pure functions in `traits.rs` (single-tribute or trait-only):**

- Trait generation: target count respected within pool capacity; no
  conflicts ever appear in the result; district pool weighting roughly
  observed across many samples. **Statistical framing.** rstest does not
  pull in a stats crate, and we do not want to. Replace formal chi-square
  with a **bucket-tolerance assertion**: draw 10 000 samples with a fixed
  RNG seed, count occurrences per trait, and assert each observed count
  falls within `±15%` of its expected count `(weight / total_weight) *
  10_000`. Tolerance band chosen to absorb single-seed variance while
  catching weight-table regressions (e.g. swapping two pool entries).
  Helper lives in test module: `fn assert_within_tolerance(observed: u32,
  expected: u32, pct: f64)`. Same approach used elsewhere in the codebase
  for randomized assertions; documents the tradeoff in test comments.
- Threshold math: stacking deltas sums correctly; clamp at 0/100 (and 0/20
  for psychotic-break); zero-trait tribute matches old `Balanced` baseline
  exactly after applying ±20% variance with a fixed seed.
- Refuser membership: O(1) lookup correctness for all 13 + physical traits.
- Conflict table symmetry: `conflicts_with(a, b) == conflicts_with(b, a)`
  for all pairs.

**Multi-tribute in `alliances.rs` (game-crate, `Vec<Tribute>` harness):**

- Alliance roll gate: Paranoid×Paranoid never allies; Friendly×Friendly
  with same district allies frequently (rate ≥ 60% over 1000 trials with
  fixed seed). Friendly×Paranoid (mixed) can ally with a Loyal partner.
- Cap penalties: tribute at `allies.len() = 5` always refuses new
  alliances; tribute at 4 allies has ≤ 0.20 base chance after cap penalty.
- Sanity break leave: sanity drop below `low_sanity_limit` triggers leave
  with probability proportional to deficit ratio; symmetric pair removed
  from both sides.
- Treacherous betrayal: timer increments per turn; resets on betrayal or
  on missed opportunity (no ally in area); symmetric pair removed before
  attack runs; victim added to attacker's valid-target set; event emitted
  with treacherous label.
- Trust-shock cascade: betrayed survivor rolls on next turn with high
  baseline; on success, removes symmetric pair with all other current
  direct allies; events emitted per ally.
- Ally-death cascade: queue-drained between turns; rolls per direct ally;
  symmetric pair removed; events emitted.
- Shared perception: same-area allies pool nearby_tributes and items;
  cross-area allies contribute nothing; hidden allies still pool.

**Integration in `api/`:**

- Persistence round-trip: `allies`, `traits`, `turns_since_last_betrayal`
  survive day/night cycle save and reload.
- DTO shape: API response includes new fields with correct types.

**Existing tests:** the ~60 game-crate tests need a `Tribute::test_default`
(or similar) helper that constructs a tribute with zero traits, reproducing
`Balanced` behavior. Tests that asserted specific personality types via
`BrainPersonality` need to migrate to checking trait sets directly.

No frontend tests for v1.

## 11. Risks & Open Questions

- **Balance is unfit by default.** Trait deltas, affinity multipliers,
  district weights, and the betrayal cadence are framed but not tuned.
  The first full game run will surface obvious problems (e.g. everyone
  allies immediately; Treacherous never triggers). Plan to iterate after a
  playtest pass.
- **Alliance discovery is invisible to the player UI.** Until follow-up
  work surfaces alliance state visually, players will only see the
  formation events scroll past in messages. Acceptable for v1.
- **Pair-wise graph means no "group identity" stories.** Events read as
  "X allies with Y" rather than "X joins the Career Pack." That's the
  intended trade-off for overlapping subtle politics. If group-style
  reporting becomes important, it can be derived from the graph (clique
  detection) without changing storage.
- **Per-tribute cap of 5 is a guess.** "Soft target 2–3" came from
  gameplay intuition, not data. Cap may need lowering to 4 or raising to
  6 after playtests.
- **Snapshot vs live state asymmetry (§6.4).** Acceptable for v1, but
  worth re-examining if any user-visible edge case surfaces.
- **Event resolution timing.** The user noted: *"I may have some questions
  on event resolution timing but that can come later."* The per-cycle
  queue drain in §7.5 is the v1 answer; revisit if it produces awkward
  ordering.

## 12. Follow-up Beads (file after spec approved)

1. Active `SeekAlly` action.
2. Cross-area ally pings.
3. Duel feature.
4. Trait categories + per-category caps.
5. Expanded physical/medical trait library.
6. Full `alliance` SurrealDB entity with history (ties to `5wt`, `wxn`).
7. Player UI for accept/reject of alliance offers.
8. Frontend visual grouping of allies in tribute list (clique-derived
   "factions" view).
9. Group-style narrative events (clique detection on the graph).
