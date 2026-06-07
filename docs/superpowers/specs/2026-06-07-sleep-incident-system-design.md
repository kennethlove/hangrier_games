# Sleep Incident System — Design

**Status:** Draft
**Date:** 2026-06-07
**Crate(s) primarily affected:** `game/` (specifically `game/src/tributes/incidents.rs`, `game/src/games/cycle.rs`)
**Related specs:** `2026-05-03-four-phase-day-design.md`, `2026-04-17-terrain-biome-system-design.md`, `2026-04-25-tribute-alliances-design.md`, `2026-05-03-shelter-hunger-thirst-design.md`
**Related beads:** `hangrier_games-pending` (this spec)

## 1. Overview

Sleep incidents are events that can occur when a sleeping tribute is vulnerable. The current implementation (`game/src/tributes/incidents.rs`) uses a flat 18% roll probability with no phase, terrain, or biome awareness. Tributes in a hot desert at night have the same incident chance as tributes in the Cornucopia at day. The incident pool is biome-agnostic — a forest tribute gets the same animal encounter candidates as a desert tribute. Ally abandonment picks randomly from a list with no relationship check.

This spec layers phase risk weighting, shelter-quality-based modifiers, biome-specific incident pools, day-based frequency scaling, and a real ally-abandonment system onto the existing incident substrate. It also formalizes trap immunity for sleeping tributes and clarifies that area events still affect them.

## 2. Design Decisions

### 2.1 Phase risk weighting

The base incident roll probability per sleeping phase follows a tiered model aligned with the four-phase day:

| Phase   | Base incident probability | Rationale                                         |
|---------|--------------------------|---------------------------------------------------|
| Day     | 8%                       | Light, activity, reduced threat                   |
| Dawn    | 12%                      | Transitional, reduced visibility                  |
| Dusk    | 12%                      | Transitional, increased nocturnal activity         |
| Night   | 22%                      | Darkness, active predators, highest vulnerability  |

**Implementation:** Replace the current `SLEEP_INCIDENT_CHANCE_PCT: u32 = 18` constant with a function or lookup from `Phase`. Roll at line 459 of `cycle.rs` passes the current phase through.

```rust
// Provisional mapping — tunable post-observability
pub fn base_incident_chance(phase: Phase) -> u32 {
    match phase {
        Phase::Day => 8,
        Phase::Dawn | Phase::Dusk => 12,
        Phase::Night => 22,
    }
}
```

The base chance is the *starting point*. Biome modifiers and shelter modifiers are applied as multiplicative factors (see §2.2–§2.3), then the day-scaling multiplier (§2.8), before the RNG check.

### 2.2 Location/terrain modifiers

A tribute's location modifies incident probability using two exclusive paths:

- **Good shelter** (tribute `sheltered_until` is active, per the shelter spec): multiplies base chance by **0.5×** (constructed shelter conceals the tribute from threats).
- **No shelter:** uses the biome's existing `shelter_quality` score (from `game/src/areas/shelter.rs`, `u8` 0–3) as a multiplicative factor. The score maps to incident probability via the shelter quality table (§2.3).

Modifier stack: shelter and biome shelter_quality are mutually exclusive. If sheltered, use 0.5× regardless of biome. If not sheltered, use the shelter-quality factor from the biome.

### 2.3 Shelter quality integration

The additive per-biome modifiers from earlier drafts are replaced by the existing `shelter_quality` system (`game/src/areas/shelter.rs`). Each biome's `shelter_quality` score (`u8`, 0–3) serves as the incident-probability multiplier for unsheltered tributes:

| Score | Biomes                                                                    | Multiplier |
|-------|---------------------------------------------------------------------------|------------|
| 3     | UrbanRuins                                                                | 0.4×       |
| 2     | Forest, Jungle, Mountains, Geothermal                                     | 0.6×       |
| 1     | Wetlands, Highlands, Clearing, Grasslands, Badlands                       | 0.8×       |
| 0     | Tundra, Desert                                                            | 1.0×       |

**Lookup function:**
```rust
pub fn biome_incident_multiplier(biome: BaseTerrain) -> f64 {
    match shelter_quality(biome) {
        3 => 0.4,
        2 => 0.6,
        1 => 0.8,
        _ => 1.0, // score 0 — no shelter bonus
    }
}
```

This removes the additive ±pp system entirely. Weather modifiers from the shelter system already handle storm-related adjustments on top of this baseline.

**Example calculation** (Night phase, Desert biome, no shelter):
- Base: 22% (Night)
- Shelter quality modifier: ×1.0 (Desert, score 0 — no shelter bonus)
- Final chance before day-scaling: 22 × 1.0 = 22%

**Example calculation** (Night phase, UrbanRuins, sheltered):
- Base: 22% (Night)
- Shelter modifier: ×0.5 (constructed shelter takes precedence over biome score)
- Final chance before day-scaling: 22 × 0.5 = 11%

### 2.4 Sleep Shelter mechanic

When a tribute decides to sleep, they roll to find or build shelter. This is separate from the terrain's inherent `shelter_quality` — it represents the tribute's active effort to create a protected sleeping spot.

**Roll:** Intelligence-based (finding natural shelter) or Strength-based (building crude shelter). Use the higher of the two, with a random factor.

**Terrain modifier:** The terrain's `shelter_quality` (0–3) sets the DC for the shelter roll. Higher `shelter_quality` means an easier roll.

**Result tiers:**

| Tier      | Description                                     | Incident modifier |
|-----------|-------------------------------------------------|------------------|
| None      | Sleeping in the open. No protection.            | 1.0x             |
| Crude     | Leaves, brush, shallow depression.              | 0.8x             |
| Natural   | Cave, hollow log, dense thicket.                | 0.5x             |
| Fortified | Reinforced position. Rare.                      | 0.3x             |

**Shelter tier → incident modifier:** None = 1.0x, Crude = 0.8x, Natural = 0.5x, Fortified = 0.3x. This modifier **replaces** the biome `shelter_quality` factor (§2.3) in the effective chance calculation when a sleep shelter is active.

**Priority order for incident modifiers:**

1. **Constructed shelter** (`sheltered_until` active) — 0.5x (§2.2). The tribute is in a pre-existing structure; no sleep shelter roll needed.
2. **Sleep shelter** (tribute rolled for shelter this session) — use tier modifier from table above.
3. **Biome shelter_quality** — fallback when neither constructed shelter nor sleep shelter exists.

**Shelter persists** until the tribute wakes or the phase changes. Must re-shelter each sleep session.

**Tribute field:** `sleep_shelter: Option<SleepShelter>` (transient, not persisted). Cleared on wake or phase change.

### 2.5 Biome-specific incident pools

#### Biome-specific animal encounters

Replace the current flat animal list (`["squirrel", "rabbit", "raccoon", "possum", "feral cat", "rat", "crow", "lizard"]`) with a per-biome lookup. Each biome gets at least two unique animals:

| Biome       | Animals                                                                    |
|-------------|---------------------------------------------------------------------------|
| Desert      | scorpion, rattlesnake, coyote, Gila monster, tarantula                   |
| Forest      | bear, wolf, wild boar, fox, owl                                          |
| Jungle      | jaguar, python, poison dart frog, spider, howler monkey                   |
| Wetlands    | alligator, snapping turtle, leeches, cottonmouth snake, bullfrog          |
| Tundra      | polar bear, wolf pack, snowy owl, arctic fox, musk ox                     |
| Grasslands  | cougar, rattlesnake, wild dog, hawk, bison                                |
| Mountains   | mountain lion, goat, golden eagle, marmot, wolverine                      |
| Badlands    | coyote, rattlesnake, vulture, scorpion, roadrunner                        |
| Highlands   | wolf, golden eagle, deer, fox, wildcat                                    |
| Geothermal  | lynx, hawk, fox, salamander, badger                                      |
| UrbanRuins  | feral dog, crow swarm, rat swarm, stray cat, raccoon                      |
| Clearing    | fox, hawk, deer, snake, rabbit                                            |

#### Terrain-specific environmental flavor incidents

Beyond animal encounters, each biome has at least 2 unique flavor incidents that can roll in the `Annoying` or `AnnoyingFlavor`-equivalent tier:

| Biome       | Environmental flavor incidents                                            |
|-------------|---------------------------------------------------------------------------|
| Desert      | "a sandstorm whips grit across their face", "the cold desert wind howls" |
| Forest      | "a branch falls nearby", "pine needles drift down with the breeze"       |
| Jungle      | "heavy rain drips through the canopy", "a howler monkey screeches nearby" |
| Wetlands    | "a chorus of frogs swells around them", "something large splashes in the murk" |
| Tundra      | "the wind screams across the frozen plain", "ice crystals form on their eyelashes" |
| Grasslands  | "wind rustles through dry grass", "a distant prairie fire glows on the horizon" |
| Mountains   | "a rockslide echoes from the peak above", "thin air makes each breath a labour" |
| Badlands    | "wind whistles through the canyons", "a rockfall tumbles nearby"          |
| Highlands   | "a thick mist rolls in", "the wind whips across the moor"                 |
| Geothermal  | "a steam vent hisses in the dark", "the ground rumbles softly"            |
| UrbanRuins  | "debris shifts in an abandoned building", "the wind moans through broken windows" |
| Clearing    | "a gentle breeze rustles the meadow", "a firefly lands on their hand"    |

### 2.6 Ally abandonment (general alliance mechanic)

Ally abandonment is a **general alliance mechanic**, not a sleep-specific incident. Any awake tribute can choose to abandon a sleeping ally, and a future feature will add a general "dissolve alliance" action for awake tributes.

The current `AllyAbandonment` sleep incident picks a random index from `tribute.allies` and removes it. This has several problems:

1. No check that the ally is a *real* ally (existing ally relationship is assumed but the random removal could target anyone in the list).
2. No check that the ally is in the same area (distance doesn't matter).
3. No event emitted for the abandoning ally — they silently lose the tie with no narrative or mechanical consequence.
4. It's possible for a dead ally to "abandon" the sleeper (no liveness check).

**New behavior (general mechanic, sleep scenario):**

- **Prerequisite (sleep scenario):** The sleeper's ally list must contain at least one living, awake tribute who is in the same area.
- **Trigger (sleep scenario):** During the sleeper's phase, if qualified allies exist, one may roll to abandon. The abandoner makes the active choice during the sleeper's turn — they slip away while the tribute is unconscious.
- **Selection:** Pick from allies that satisfy the prerequisite. Prefer the closest relationship (highest affinity, if tracked) or random among qualified candidates.
- **Effect on sleeper (abandoned):** Remove the abandoning ally from `tribute.allies`. Emit `SleepIncidentKind::AllyAbandonment` as before. **The sleeper stays asleep** — they do not wake from the abandonment itself. They discover the abandonment on natural wake or at the next phase boundary.
- **Mental effects on the abandoned:** The abandoned tribute suffers:
  - Sanity loss (betrayal trauma from the abandonment)
  - Trust damage toward future allies (lowered starting affinity for new alliances)
  - Possible Affliction seeds (paranoia, distrust) — rolled separately if that system is active
- **Effect on ally (the abandoner):** The abandoning ally receives a new transient event that the brain can score against. The event is written to the abandoning ally's pending message queue (or equivalent) as `AllyAbandonedTribute { abandoned: TributeRef }`. This can trigger:
  - Guilt/remorse (sanity cost)
  - Strategic relief (they needed to move faster alone)
  - Potential future re-alignment (lowered trust threshold for re-alliance)
- **Safety:** Only possible if the abandoning ally is alive and awake (not sleeping themselves). If no qualified ally exists, roll a different incident (re-roll or fall back to Annoying).
- **Future expansion:** A general "dissolve alliance" action (for awake tributes) will reuse the same effect structure but with different trigger conditions and potentially different mental effects (guilt on the initiator instead of betrayal on the abandoned).

**Data flow:**

```rust
// Context for the abandonment event
pub struct AllyAbandonmentContext {
    pub abandoning_ally_id: String,
    pub abandoning_ally_name: String,
}

// Emitted for the abandoning ally (game cycle integration)
MessagePayload::AllyAbandonedTribute {
    abandoning_ally: TributeRef,
    abandoned: TributeRef,
    phase: Phase,
}
```

### 2.7 Trap immunity and area event exposure

Sleeping tributes are vulnerable to *sleep incidents* — not to traps, but **area events still affect them**. This distinction is important:

**Traps:**
- Do not roll for trap triggers.
- Do not have trap-damage applied.
- Are not considered "active" for trap-detection purposes.

**Area events:**
- Environmental area events (sandstorms, floods, fire, earthquakes, temperature shifts, toxic gas, etc.) affect all living tributes regardless of sleep state.
- If an area event would hit a sleeping tribute:
  - The tribute takes the full effect of the event.
  - The tribute wakes from the event (unless the event is non-waking, such as a gradual ambient temperature change or slow toxin buildup that doesn't cross a threshold).
  - The wake reason is attributed to the area event for narrative processing.

The trap rules are already implicitly true in the cycle (sleeping tributes skip the action pipeline where trap triggers happen). The area event rule is a design clarification that prevents sleep from acting as blanket immunity against environmental threats.

> **No trap interaction:** A sleeping tribute's trap vulnerabilities are expressed through the sleep incident system exclusively. Traps themselves do not interact with sleeping tributes.
>
> **Area events affect all:** Environmental area events affect all living tributes. Sleep provides no immunity against the environment.

### 2.8 Day-based frequency scaling

As the game progresses, tributes accumulate exhaustion, gear degradation, and environmental pressure. Sleep incident probability increases with the game day to reflect rising tension.

Use the same scaling factor as other game systems (confirmed via `Game::day`):

```rust
pub fn day_scaling_multiplier(current_day: u32) -> f64 {
    match current_day {
        0..=1 => 1.0,     // Early game: baseline
        2..=3 => 1.2,     // Mid-early: slight increase
        4..=5 => 1.5,     // Mid-late: significant
        _ => 2.0,         // Late game (Day 6+): double
    }
}
```

The multiplier is applied *after* the shelter/shelter_quality factor, *before* the RNG check:

```
effective_chance = clamp(
    base_chance(phase) * shelter_factor * day_scaling_multiplier(current_day),
    0.0,
    1.0  // cap at 100% — never guaranteed
)
```

**Example** (Night, Desert, no shelter, Day 5):
- Base: 22%
- ×1.0 (Desert shelter_quality, score 0)
- ×1.5 (Day 5 multiplier)
- = 33% chance per sleeping phase

## 3. Incident Types

### 3.1 Incident variants

Eight `SleepIncident` variants, with internal changes as described:

| Variant            | Wakes tribute? | Changes                                                       |
|--------------------|----------------|---------------------------------------------------------------|
| `Annoying`         | No             | Biome-specific flavor pool added                              |
| `Nightmare`        | No             | Sanity damage (2–6). No wake. (§3.3)                         |
| `NightTerror`      | Yes            | Sanity damage (5–12), phobia trigger. Wakes. (§3.4)          |
| `Theft`            | Yes            | No change (item stolen by unseen thief)                       |
| `Relocation`       | Yes            | No change (sleepwalking)                                      |
| `AnimalEncounter`  | Yes            | Biome-specific animal pool (§2.5)                             |
| `LimbInjury`       | Yes            | No change (comedic limb-fell-asleep)                          |
| `AllyAbandonment`  | No             | General alliance mechanic (§2.6). Sleeper stays asleep.       |

### 3.2 Roll probability weights

Updated weighted distribution for `SleepIncident::random()`:

| Roll range | Incident          | Weight | Condition                     |
|------------|-------------------|--------|-------------------------------|
| 0–29       | Annoying          | 30%    | Always available              |
| 30–44      | Nightmare         | 15%    | Always available              |
| 45–49      | NightTerror       | 5%     | Only if tribute has phobia    |
| 50–61      | Theft             | 12%    | Always available              |
| 62–71      | Relocation        | 10%    | Always available              |
| 72–81      | AnimalEncounter   | 10%    | Always available              |
| 82–89      | LimbInjury        | 8%     | Always available              |
| 90–99      | AllyAbandonment   | 10%    | Only if ally exists in area   |

**Conditional roll resolution:** If a conditional variant's roll is selected but its condition is not met, fall back to `Annoying`. Specifically:
- **NightTerror** (5%): if tribute has no phobia, treat as Nightmare instead.
- **AllyAbandonment** (10%): if no qualified ally exists (living, awake, same area), reroll into the remaining pool respecting original weights.

NightTerror's effective rate is ~5% of tributes with at least one phobia, and 0% of tributes without — they get the Nightmare fallback instead.

Weights are candidates for post-observability tuning. Annoying absorbs remaining probability after all conditional variants resolve.

### 3.3 Nightmare (common)

The `Nightmare` variant replaces the previous `Hallucination` variant. Unlike hallucinations — which could be mistaken for reality — a nightmare is clearly a dream. The tribute stays asleep through the experience.

**Effects:**
- **Sanity damage:** Small loss (2–6), proportional to content severity.
- **Sleep quality reduction:** Reduces sleep quality, which feeds into a next-day fatigue mechanic if one is added.
- **No wake:** Nightmares pass without rousing the tribute. They wake normally at the next phase boundary or if interrupted by a different event.

**Content selection:**
The nightmare's theme is drawn from the tribute's experiences, weighted by priority:
1. **Recent trauma** (highest weight) — memorable event from the last 1–2 phases (combat, injury, witnessed death).
2. **Phobia content** — if the tribute has an active phobia, the nightmare centers on that fear.
3. **Fixation content** — if the tribute has an active fixation, the nightmare inverts or corrupts it.
4. **Random subconscious** (fallback) — generic fear content (heights, darkness, being chased, losing, etc.).

**Implementation note:** Content selection in v1 is narrative-only (flavor text varies by category). Phobia/fixation mechanical hooks are recognized but not wired until those systems have their own persistence model.

### 3.4 Night Terror (rare)

A rarer, more intense version of a nightmare. The dream feels viscerally real — the tribute cannot distinguish it from reality until they wake.

**Trigger condition:** Only rolls if the tribute has at least one active phobia. If no phobia exists, the NightTerror roll falls back to Nightmare (§3.3).

**Effects:**
- **Sanity damage:** Larger loss (5–12).
- **Phobia trigger/reinforcement:** If the dream involves the tribute's phobia (likely, given the trigger condition), the phobia's intensity increases.
- **Does wake the tribute:** The tribute wakes suddenly, startled. The cycle emits `TributeWoke` with `InterruptionKind::NightTerror`.

**On-wake behavior:**
```rust
// The cycle emits:
MessagePayload::TributeWoke {
    tribute: TributeRef,
    reason: WakeReason::Incident(SleepIncidentKind::NightTerror),
    interruption: Some(InterruptionKind::NightTerror),
}
```

**Content selection:**
Same priority system as Nightmare, but phobia content is elevated to highest weight:
1. **Phobia content** (highest weight) — centers on an existing phobia, reinforcing it.
2. **Recent trauma** — if no phobia (fallback scenario), treat as Nightmare instead.
3. **Random subconscious** — generic fear content.

## 4. Biome Integration

### 4.1 Data flow

The sleep incident roll currently receives only `&mut impl Rng`. To support phase-aware, biome-aware, shelter-aware, day-aware rolling, the signature expands:

```rust
// Current:
pub fn roll(rng: &mut impl Rng) -> Option<Self>

// New:
pub fn roll(
    rng: &mut impl Rng,
    phase: Phase,
    biome: BaseTerrain,
    is_sheltered: bool,
    current_day: u32,
) -> Option<Self>
```

The `random` method also expands to receive `biome`:

```rust
// Current:
pub fn random(rng: &mut impl Rng) -> Self

// New:
pub fn random(rng: &mut impl Rng, biome: BaseTerrain) -> Self
```

### 4.2 Call site changes

The roll call at `cycle.rs:459` currently reads:

```rust
if let Some(incident) = SleepIncident::roll(rng) {
```

The updated call passes context:

```rust
let tribute_area = tribute.area;
let terrain = areas[tribute_area].terrain.base;
let is_sheltered = tribute.sheltered_until.is_some_and(|p| p > now);
if let Some(incident) = SleepIncident::roll(rng, phase, terrain, is_sheltered, current_day) {
```

Where `area_details_map`, `all_areas_snapshot` (or the `areas` slice directly), and `phase`/`current_day` from `CycleContext` are available at that point in the cycle.

### 4.3 Shelter check source

The `sheltered_until` field already exists on `Tribute` (from the shelter/hunger/thirst spec). The sleep incident system reads it as a boolean: `is_sheltered` = `tribute.sheltered_until.map(|p| p > current_phase_index).unwrap_or(false)`.

If the shelter system is not yet landed at implementation time, the check defaults to `false` (all tributes considered unsheltered) — which produces slightly higher incident rates but is technically correct for that interim state.

## 5. Mechanical Details

### 5.1 Effective chance calculation (formal)

```python
def effective_incident_chance(
    phase: Phase,
    biome: BaseTerrain,
    is_sheltered: bool,
    sleep_shelter: Option[SleepShelter],
    current_day: u32
) -> f64:
    # 1. Base from phase (Table §2.1)
    base = base_incident_chance(phase)          # 8/12/12/22

    # 2. Shelter quality factor (§2.2–§2.4)
    if is_sheltered:
        factor = 0.5                            # constructed shelter (§2.2)
    elif sleep_shelter is not None:
        factor = sleep_shelter.multiplier        # 1.0 / 0.8 / 0.5 / 0.3 (§2.4)
    else:
        factor = biome_incident_multiplier(biome)  # 0.4 / 0.6 / 0.8 / 1.0 (§2.3)

    # 3. Day scaling (§2.8)
    day_scale = day_scaling_multiplier(current_day)

    # 4. Final with clamp
    effective = base * factor * day_scale
    return min(effective, 100.0)               # cap at 100% — never guaranteed
```

### 5.2 New constants

Replace the single `SLEEP_INCIDENT_CHANCE_PCT` constant with a constants block:

```rust
// Phase base rates
const SLEEP_INCIDENT_DAY_PCT: u32 = 8;
const SLEEP_INCIDENT_DAWN_PCT: u32 = 12;
const SLEEP_INCIDENT_DUSK_PCT: u32 = 12;
const SLEEP_INCIDENT_NIGHT_PCT: u32 = 22;

// Shelter multiplier (constructed shelter)
const SLEEP_INCIDENT_SHELTER_MULTIPLIER: f64 = 0.5;

// Shelter quality multipliers (per biome score, from shelter.rs)
const SHELTER_QUALITY_SCORE_3: f64 = 0.4;   // UrbanRuins
const SHELTER_QUALITY_SCORE_2: f64 = 0.6;   // Forest, Jungle, Mountains, Geothermal
const SHELTER_QUALITY_SCORE_1: f64 = 0.8;   // Wetlands, Highlands, Clearing, Grasslands, Badlands
const SHELTER_QUALITY_SCORE_0: f64 = 1.0;   // Tundra, Desert

// Sleep shelter multipliers (per tier, from §2.4)
const SLEEP_SHELTER_NONE_MULTIPLIER: f64 = 1.0;
const SLEEP_SHELTER_CRUDE_MULTIPLIER: f64 = 0.8;
const SLEEP_SHELTER_NATURAL_MULTIPLIER: f64 = 0.5;
const SLEEP_SHELTER_FORTIFIED_MULTIPLIER: f64 = 0.3;
```

### 5.3 Minor changes to `apply_sleep_incident`

The `apply_sleep_incident` function (`incidents.rs:153`) takes `(&mut Tribute, &SleepIncident, &mut impl Rng)` and returns `String`. Its signature and behavior remain largely unchanged except:

- `AllyAbandonment` branch: **removed entirely** — the cycle handles abandonment as a general alliance mechanic (§5.4).
- `AnimalEncounter` branch: animal name is pre-populated by `random()` using biome-specific pool; no change to apply itself.
- `Annoying` branch: flavor pool is biome-specific; flavor is pre-selected by `random()`.
- `Nightmare` branch: sanity damage applied (2–6), no wake. Phobia/fixation hooks stubbed in v1.
- `NightTerror` branch: new match arm. Larger sanity damage applied (5–12), phobia trigger applied. Cycle handles wake emission.

### 5.4 AllyAbandonment apply changes

The current `AllyAbandonment` match arm in `apply_sleep_incident` is removed entirely — the cycle handles abandonment as a general alliance mechanic, not a sleep incident with `apply` behavior.

Current code (to be removed from `apply_sleep_incident`):

```rust
// Current (simplified) — to be removed:
SleepIncident::AllyAbandonment => {
    if tribute.allies.is_empty() {
        // fall back to annoying
    }
    let idx = rng.random_range(0..tribute.allies.len());
    let _abandoned = tribute.allies.remove(idx);
    // flavor text only
}
```

The new version needs access to the full tribute list to validate ally qualifications and to emit the abandoning-ally event. This means moving the ally-abandonment logic out of `apply_sleep_incident` and into the cycle directly (where full game state is available):

```rust
// Pseudocode for cycle.rs integration
SleepIncident::AllyAbandonment { abandoning_ally } => {
    // 1. Remove abandoning ally from sleeper's ally list
    // 2. Queue AllyAbandonedTribute event for the abandoning ally
    // 3. Apply mental effects on sleeper (sanity loss, trust damage, affliction seeds)
    // 4. Flavor text for both
    // NOTE: No TributeWoke — the sleeper stays asleep (§2.6)
}
```

This keeps the abandonment logic where it has access to both tributes, the full ally graph, and the event pipeline. The sleeper does not wake from abandonment itself — they discover it on natural wake or at the next phase boundary.

**Future integration:** The same removal/effect logic can be called from a general "dissolve alliance" action when awake tributes initiate the split directly.

## 6. Storage

No new persistent fields on `Tribute`. The `pending_sleep_incident` field (`Option<SleepIncidentKind>`) already exists and continues to serve its purpose for non-waking incidents.

**Transient field:** `sleep_shelter: Option<SleepShelter>` on `Tribute` (see §2.4). This field is transient — set during the sleep phase, cleared on wake or phase change. Not persisted to the database.

The `AllyAbandonedTribute` event is a transient message payload — not persisted on the tribute. It flows through the existing message pipeline and is consumed by the abandoning ally on their next brain tick (or ignored if the ally is dead before processing).

## 7. Integration Points

- **Four-phase day spec:** The phase-aware roll depends on the `Phase` enum already having `Dawn`/`Dusk`/`Day`/`Night` variants. Phase values flow through `CycleContext.phase`.
- **Terrain/biome spec:** Biome lookup requires `AreaDetails.terrain.base` to be populated at game creation. Already the case.
- **Shelter spec:** The `is_sheltered` check reads `Tribute.sheltered_until`. If the shelter system is not yet landed, default to `false` (exposed) — results are higher but safe.
- **Alliance spec:** The reworked ally abandonment reads `Tribute.allies` (existing). No new alliance system changes needed.

## 8. Out of Scope (filed as follow-ups)

- **AllyAbandonment brain scoring** — the abandoning ally should react to their choice (guilt, relief). This belongs in a brain-scoring follow-up once the event pipeline for ally-to-ally events is settled.
- **Trap-sleep interaction** — explicit "night trap" variant for sleeping tributes (debated, deferred). Current rule is no trap interaction.
- **General "dissolve alliance" action** — an awake tribute initiating abandonment of an awake ally. The infrastructure (removal logic, event emission, mental effects) is designed to support this, but the trigger flow and UI are deferred.
- **Multi-phase sleep incident consequences** — e.g., an animal encounter that leaves a bleeding wound that ticks over multiple phases.
- **Sleep incident chain reactions** — e.g., a relocation incident moves a tribute into a trap or into another tribute's ambush.

## 9. Spec Self-Review

- **Grounding in existing code:** The spec is written against the live `game/src/tributes/incidents.rs` (364 lines, 7 variants, 8 tests) and `game/src/games/cycle.rs` (sleep section at lines 453–524). Every design decision maps to concrete changes in those files.
- **Placeholders:** Shelter quality factors (§2.3), day-scaling multipliers (§2.8), and animal pools (§2.5) are provisional starting values — all tunable post-observability. None are "TBD" gaps.
- **Structural growth, not creep:** The incident enum grows to 8 variants. `Hallucination` is renamed to `Nightmare` with updated mechanics (§3.3), and the new `NightTerror` variant (§3.4) splits a rarer, wake-inducing case out of the Nightmare concept. Changes are to the *roll* mechanism, the *content* of existing variants, the generalization of ally abandonment, and the addition of the sleep shelter system.
- **Alliance spec alignment:** The ally-abandonment rework (§2.6) generalizes abandonment into a non-sleep-specific mechanic. It reads existing fields (`tribute.allies`, `tribute.area`) and emits the same event structure. Future alliance system changes (trust scores, relationship depth) would upgrade this mechanic naturally. The sleep incident variant is one trigger path; a future "dissolve alliance" action is another.
- **Phase awareness:** The spec uses the existing `Phase` enum from the four-phase-day spec, which is already landed. No new phase variants needed.
- **Frequency scaling matches existing patterns:** The day-scaling curve (§2.8) mirrors the curve used by other escalating game systems (affliction rolls, environmental event frequency).
- **Area event gap closed:** The previous blanket "no double-jeopardy" rule incorrectly grouped area events with traps. The spec now distinguishes trap immunity from area event exposure (§2.7), matching the planned cycle behavior where environmental events resolve independently of sleep state.
