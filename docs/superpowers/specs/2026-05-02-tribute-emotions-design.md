# Tribute Emotions & Outlook — Design

**Status:** Draft
**Date:** 2026-05-02
**Crate(s) primarily affected:** `game/`
**Related specs:** `2026-04-25-tribute-alliances-design.md`, `2026-04-26-game-event-enum.md`

## Goals

Add a load-bearing emotional state layer to tributes that:

1. **Mechanically influences combat and decision-making** (primary goal). Emotions modify the existing Brain decision tree and add new override states.
2. **Increases behavioral diversity** between tributes. Two tributes with similar health/sanity can act differently because their emotional state differs.
3. **Provides narrative color** for the announcer LLM, event log, and UI ("Cato is enraged after Glimmer's death").

Non-goals:

- Replace the existing `sanity` mechanic. Sanity is the long-term breakdown clock; emotions are short-term regulation and outlook.
- Replace traits. Traits are who-you-are at reaping; emotions are who-you-become in the arena.

## Model: Continuous Axes with Derived Label

Tributes carry four continuous emotional axes (the truth of their state) and one derived dominant-emotion label (the display/narrative surface). All decisions are made on the axes; the label is a pure function of them.

This is a hybrid by intent: continuous axes give nuance and clean numeric gating; the label gives the UI and announcer a single concept to render. The label can be flattened back into a simple enum later if the axis layer proves over-engineered.

### The Four Axes

All axes are `u8` in `0..=100`, neutral around `50`, clamped on every update.

| Axis | Low end | High end | Primary mechanical role |
|---|---|---|---|
| **Morale** | despair, suicidal | hope, resolve | Rest/give-up thresholds; willingness to push forward |
| **Aggression** | passive, fleeing | rage, bloodlust | Attack-vs-evade decisions; engaging when outnumbered |
| **Trust** | paranoid, betrayer | loyal, cooperative | Alliance formation, betrayal odds, defensive aid (replaces `loyalty`) |
| **Composure** | panicked, shaky | calm, focused | Combat accuracy gates; ability to use items under pressure |

### Distinction from Existing Stats

- **Sanity** (existing): long-term breakdown counter. Drains over the whole game; reaching 0 causes self-harm. Persistent damage.
- **Composure** (new): short-term emotional regulation. Recovers via rest, drops on shocks. Recovers fully day-to-day if the tribute is safe.
- **Trust** (new) **replaces** the per-tribute `loyalty: f32` scalar entirely. The old "loyalty to whom?" question is now answered by per-pair bond strength (below); Trust is the global disposition toward people in general.

## Alliance Bond

Per-alliance edge gets a `bond: u8` in `0..=100`. **Symmetric** — one number per alliance pair.

Bond starts at `25` for newly-formed alliances, or `40` if both tributes are from the same district. Drifts upward through cooperative behavior, downward through neglect or conflict (see trigger table below).

### What Bond Gates

- **Betrayal probability.** Existing `loyalty < 0.25` check becomes `betrayal_probability` scaled inversely with bond. Higher bond, lower odds.
- **Defensive aid (new mechanic).** Tributes with high bond to a nearby ally under attack may intervene on the ally's behalf.
- **Grief impact.** When a bond-partner dies, the surviving tribute's Morale drop scales with bond. The grief table differentiates `bond ≥ 50` from `bond < 50`.
- **Last-two-standing behavior.** High bond = hesitation; low bond = clean betrayal.

### One-Sided Betrayals

Bond is symmetric, but **decisions are per-tribute**. Even with shared `bond = 80`, A may roll betrayal this turn while B does not. The asymmetry lives in the decision, not the storage. The genuinely one-sided cases (gift-giving, healing) are handled as a small bond bonus to the recipient, retaining symmetry by convention.

## Starting Values

Tributes do **not** start identical. Traits provide a starting offset for each axis.

Each `Trait` gains an `emotion_bias() -> (i8, i8, i8, i8)` method returning per-axis deltas (Morale, Aggression, Trust, Composure). At tribute creation:

1. Start each axis at `50`.
2. For each trait the tribute owns, sum its `emotion_bias` into the axes.
3. Clamp final per-axis values to `20..=80` so no tribute starts in override-state territory.

Example bias intuitions (final values to be tuned during implementation):

- *Bloodthirsty:* `(0, +20, 0, 0)`
- *Cowardly:* `(-5, -20, 0, -15)`
- *Loyal:* `(0, 0, +20, 0)`
- *Paranoid:* `(0, 0, -25, -5)`

The trait→axes table is part of implementation, not this spec.

## Triggers and Axis Updates

Triggers fire during the existing `process_turn_phase` and combat-resolution paths. All deltas are signed and applied with saturating arithmetic, then clamped to `0..=100`.

### Combat Triggers

| Event | Morale | Aggression | Trust | Composure |
|---|---|---|---|---|
| Won fight (decisive) | +8 | +5 | — | +3 |
| Won fight (normal) | +3 | +2 | — | +1 |
| Lost fight (survived) | −5 | +3 | — | −8 |
| Killed someone | +5 | +10 | −3 | −5 |
| Took a wound | −3 | — | — | −5 |
| Witnessed kill nearby | −2 | +1 | −2 | −3 |

### Social Triggers

| Event | Morale | Aggression | Trust | Composure | Bond |
|---|---|---|---|---|---|
| Formed alliance | +5 | — | +5 | +2 | (init 25/40) |
| Ally died (bond ≥ 50) | −15 | +8 | −5 | −10 | n/a |
| Ally died (bond < 50) | −5 | +2 | −2 | −3 | n/a |
| Betrayed by ally | −10 | +12 | −20 | −8 | n/a (alliance breaks) |
| Betrayed an ally | −3 | — | −5 | −3 | n/a (alliance breaks) |
| District-mate died (non-ally) | −8 | +5 | — | −5 | — |
| Received sponsor gift | +10 | — | +3 | +5 | — |
| Survived turn together (same area) | — | — | — | — | +1 |
| Fought side-by-side (both attacked enemies) | — | — | — | — | +3 |
| Healed/gifted ally | +1 | — | — | — | +3 (recipient bond +5) |
| Witnessed ally take damage, did not help | — | — | — | — | −1 |
| Refused to share an item | — | — | −2 | — | −3 |
| Competed for same kill/item | — | +1 | −1 | — | −1 |

### Environmental / Passive Triggers

"Alone" below means the tribute ended the turn with no living ally in the same `Area`. "Hit by area event" fires when `apply_area_effects` applies any `AreaEvent` to the tribute (currently: Wildfire, Flood, Earthquake, Avalanche, Blizzard, Landslide, Heatwave). The "Hungry / low on supplies" trigger is contingent on a hunger/supplies system existing — if no such system is present at implementation time, omit this row and revisit when supplies tracking lands.

| Event | Morale | Aggression | Trust | Composure |
|---|---|---|---|---|
| Survived a day (alone) | −1 | — | −1 | +1 |
| Survived a day (with ally in same area) | +1 | — | +1 | +1 |
| Hungry / low on supplies *(if supported)* | −2 | +1 | — | −2 |
| Hit by area event (flood, wildfire, etc.) | −3 | — | — | −5 |
| Long rest in safe area | +2 | −1 | — | +5 |
| Night falls (alone) | −2 | — | — | −2 |
| Hidden successfully | +1 | −1 | — | +2 |

### Passive Drift

After all triggers resolve at end of turn, every axis moves toward `50` by `1`. If the axis is exactly at `50`, no change. If a `+1` step would cross `50` (e.g., from `49` to `50` or from `51` to `50`), it stops at `50` rather than overshooting. This provides a "reset" so emotional state doesn't ratchet permanently in one direction across long games. Triggers always dominate drift because trigger magnitudes are larger.

Drift is **silent** — no `TributeEmotionalEvent` is emitted unless the drift causes a label change.

## Brain Feedback (Mechanical Payoff)

The existing `Brain::act()` decision tree is modified in two places. Pure additions; the existing health/sanity/intelligence logic stays.

### Override States (Checked First)

Before any normal decision logic:

```text
if outlook.morale     < 15  → Broken    (Rest or self-harm)
if outlook.aggression > 85  → Enraged   (Attack)
if outlook.composure  < 20  → Panicked  (Hide or random Move)
if outlook.trust      < 20  → Paranoid  (refuse alliance, attack ally if cornered)
```

Multiple overrides hitting at once resolve by priority order above (Broken wins, then Enraged, etc.). The same priority order drives the derived label (next section).

### Threshold Shifting (Otherwise)

The existing decision-tree thresholds become functions of the axes rather than constants. Indicative formulas (final coefficients tuned during implementation; results clamped to sane bounds so a single axis can't make a threshold negative or exceed 100):

```text
attack_threshold = clamp(40 - (outlook.aggression - 50) * 0.4, 0, 100)
rest_threshold   = clamp(20 + (50 - outlook.morale)    * 0.3, 0, 100)
hide_threshold   = clamp(base_hide_threshold + (50 - outlook.composure) * 0.2, 0, 100)
```

A high-Aggression tribute attacks at lower health than baseline; a low-Morale tribute rests sooner; a low-Composure tribute hides more readily. Tributes near 50 on every axis behave identically to today's Brain.

## Derived Label

A pure function of the axes, evaluated whenever the axes change. Threshold-priority list, first match wins:

```text
Morale     <  15                              → Broken
Aggression >  85                              → Enraged
Composure  <  20                              → Panicked
Trust      <  20                              → Paranoid
Morale     <  30 AND Aggression > 65          → Vengeful
Morale     >  75 AND Composure  > 65          → Resolute
Morale     >  70 AND Trust      > 65          → Hopeful
Aggression >  65 AND Composure  > 65          → Focused
                                              → Steady (no special label)
```

The first four entries are the same thresholds as the override states — single source of truth.

`Steady` tributes display no special label. This is intentional: labeled tributes stand out narratively because the unlabeled majority provides contrast.

A label change is the storytelling moment. The announcer narrates it; the UI highlights it; the player notices it.

## Events

One event type covers all emotional updates. Defined in `game/src/tributes/events.rs`:

```rust
pub struct TributeEmotionalEvent {
    pub tribute: Uuid,
    pub trigger: EmotionTrigger,        // discriminant: Killed, Betrayed, AllyDied, etc.
    pub axis_changes: AxisDeltas,       // {morale: i16, aggression: i16, trust: i16, composure: i16}
    pub label_change: Option<(EmotionLabel, EmotionLabel)>,
}
```

One event per *causing moment* — not one per axis. A kill emits a single event with all four axis deltas plus the optional label transition. This gives:

- Announcer / event log: a clean unit to render per dramatic beat.
- Progressive display: enough data to show subtle motion if it wants.
- Replay / debugging: full causal chain.

Passive drift does **not** emit an event unless it crosses a label boundary. When drift causes a label transition, emit with `trigger: EmotionTrigger::Drift` and the relevant axis deltas.

## Data Model Changes

### `Tribute` struct (`game/src/tributes/mod.rs`)

```rust
pub struct Tribute {
    // ...existing fields...
    pub outlook: Outlook,            // NEW
    // pub loyalty: f32,             // REMOVED
}
```

### New `Outlook` struct (`game/src/tributes/emotions.rs`)

```rust
pub struct Outlook {
    pub morale: u8,
    pub aggression: u8,
    pub trust: u8,
    pub composure: u8,
}

impl Outlook {
    pub fn neutral() -> Self;
    pub fn from_traits(traits: &[Trait]) -> Self;
    pub fn apply(&mut self, deltas: AxisDeltas);
    pub fn drift_toward_neutral(&mut self);
    pub fn label(&self) -> EmotionLabel;
}
```

### Alliance edge

Wherever alliances are stored (per `2026-04-25-tribute-alliances-design.md`), the edge representation gains `bond: u8`. The exact storage shape is left to the alliance phase that introduces edges; this spec only specifies the field, default values, and update rules.

If the current alliances representation is `allies: Vec<Uuid>`, this becomes `allies: Vec<Ally { id: Uuid, bond: u8 }>` or an equivalent parallel map keyed by Uuid.

### `Trait` extension (`game/src/tributes/traits.rs`)

```rust
impl Trait {
    pub fn emotion_bias(&self) -> (i8, i8, i8, i8);  // (morale, aggression, trust, composure)
}
```

## Integration Points

- **`Brain::act()`** — add override-state checks at the top; replace constant thresholds with axis-derived expressions in existing branches.
- **`process_turn_phase()`** — invoke `outlook.drift_toward_neutral()` after triggers resolve; emit drift-driven label-change events.
- **Combat (`attacks`, `apply_combat_results`)** — fire combat triggers on winners/losers/witnesses; bond updates for allies fighting side-by-side.
- **Alliance code (`try_form_alliance`, betrayal paths)** — fire social triggers; replace existing `loyalty` reads with bond/Trust reads; initialize bond on formation.
- **Area-effect application (`apply_area_effects`)** — fire environmental triggers when a tribute is hit by a flood/wildfire/etc.
- **Sponsor-gift path (`receive_patron_gift`)** — fire sponsor-gift trigger.
- **Event consumers (announcer, API, web)** — handle the new `TributeEmotionalEvent` variant in whatever event-rendering plumbing exists.

## Testing Strategy

Following the project's existing rstest pattern in `game/`:

- **Unit tests for `Outlook`** — clamping, drift saturation at 50, trait-bias clamping to 20..=80, label derivation for every priority rung and the unlabeled fallback.
- **Trigger application tests** — each trigger row in the tables produces the documented deltas; combined triggers in one turn sum correctly.
- **Brain override tests** — each override state forces the documented action; threshold-shifting moves the existing decision boundaries by the predicted amount.
- **Bond tests** — bond initialization values, symmetric update through cooperative-action triggers, betrayal-probability scales inversely with bond, grief delta differs across the 50-bond cutoff.
- **Event emission tests** — one `TributeEmotionalEvent` per causing moment; passive drift silent unless label changes.
- **Loyalty removal regression** — existing tests that touch `loyalty` either are converted to read Trust/bond or are deleted with rationale in the commit message.

## Migration / Backward Compatibility

- The `loyalty` field is removed from `Tribute`. SurrealDB persisted games need migration: drop the `loyalty` field, add `outlook` with neutral values (all axes at `50`), add `bond` to alliance edges with a default of `25` (or `40` if same district). Migration lives as a new entry under `migrations/definitions/` plus accompanying `schemas/*.surql` updates, following the existing `surrealdb-migrations` pattern.
- API DTOs in `shared/` that surface tribute state need a paired field for outlook + label so the frontend can render it. Existing API consumers ignore unknown fields, so addition is non-breaking; removal of `loyalty` from any DTO is breaking and must be coordinated with the web frontend.
- In-flight games re-hydrate with neutral outlooks rather than reconstructing emotional history from event logs (out of scope for v1).

## Open Questions for Implementation

These don't need to be answered before writing the implementation plan, but the implementer will need to make calls:

- Exact `emotion_bias()` values per trait — needs a pass over the trait list with a tuning eye.
- Final coefficients in threshold-shifting formulas (the `* 0.4`, `* 0.3` placeholders).
- Drift magnitude — `±1` per turn is the starting value; may need to be `±2` if testing shows axes get stuck.
- Whether `EmotionTrigger` is one large enum or a struct-of-cause-data.

## Out of Scope

- Visual presentation of emotions in the web frontend (covered by the upcoming progressive-display spec).
- Announcer prompt changes to use the new event payload (covered separately when announcer integration lands).
- Any new traits introduced by the emotion system.
- Multi-emotion display (always exactly one label or none).
