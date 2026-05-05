# Trauma affliction system — design

Beads: `hangrier_games-cdu0`
Related: afflictions epic (`4o8a`), phobias spec (`2026-05-03-phobias-design.md`), fixations spec (`2026-05-03-fixations-design.md`), tribute emotions spec (`2026-05-02-tribute-emotions-design.md`)

## 1. Purpose

Model durable psychological scarring from in-game events (witnessing ally death, surviving betrayal, near-death, mass casualty). Trauma sits between transient `Emotions` (cycle-scale) and stimulus-keyed `Phobias` (specific-trigger reactions): it is generalized state, slow to acquire, slow to decay, and produces both always-on stat penalties and unpredictable mental-break behavior (flashbacks).

Trauma is both:

- a **stored affliction** (`AfflictionKind::Trauma`) carried on the tribute, observable to other tributes via flashbacks, with severity, decay, and reinforcement; and
- a **producer pipeline** that watches the per-cycle event stream for canonical patterns and acquires Trauma plus, where appropriate, co-acquires Traumatic-origin Phobias.

This is the "PTSD bucket" plus the "what events break a tribute" mapping in one spec.

## 2. Non-goals (v1)

- Self-trauma from killing (depends on morality system that does not exist).
- Trauma from witnessing affliction acquisition / mutilation (defer until afflictions stabilize).
- Trauma from sustained-source torture (needs new per-source duration tracker).
- Source-keyed decay rates or fine-grained source taxonomy (one 10-cycle rule for v1).
- Trauma-specific sponsor gifts beyond Therapy and Sedative (wait for `dvd`).
- Dedicated trauma-viewport UI (covered by `n52s`).

## 3. Relationship to neighboring systems

| System | Trauma's relationship |
|---|---|
| `Emotions` (`2026-05-02`) | Emotions are short-lived; trauma is durable. A producer event may also spike an emotion this cycle, but the durable consequence lives in trauma. |
| `Phobias` (`2026-05-03`) | Phobias are stimulus-keyed reactions; trauma is the generalized state. The trauma producer is the canonical source of Traumatic-origin phobias (phobia spec §11). |
| `Fixations` (`2026-05-03`) | Fixations are appetitive (pull-toward); trauma is aversive (push-away). Trauma producer can also produce Acquired-origin fixations (e.g. obsessive vengeance after betrayal) — see §11. |
| Health-conditions afflictions (`2026-05-03`) | Trauma is a peer affliction kind reusing the same storage, severity, cure, and visibility infrastructure. |
| Alliance system (`2026-04-25`) | Trauma firing in an ally's presence triggers a small bond-affinity *increase* (sympathetic bond). Trauma with `Betrayal { by: X }` source vetos allying with X. |

## 4. Types

```rust
// shared/src/afflictions.rs

pub enum AfflictionKind {
    // ... existing variants from health-conditions spec
    Trauma,
    // ... Phobia variant from phobias spec, etc
}

pub enum TraumaSource {
    WitnessedAllyDeath { ally: TributeId, cause: Option<DeathCause> },
    Betrayal           { by: TributeId },
    NearDeath          { cause: DeathCause },
    MassCasualty       { cause_class: CauseClass, deaths_this_cycle: u32 },
}

pub enum DeathCause {
    Tribute(TributeId),
    Fire,
    Drowning,
    Beast(BeastKind),
    Hazard(HazardKind),
    Starvation,
    Dehydration,
    Affliction(AfflictionKind),
    Gamemaker,
    Unknown,
}

pub enum CauseClass {
    Combat,
    Environmental,
    Gamemaker,
    Mixed,
}
```

`Affliction` already carries severity and source per the health-conditions spec. Trauma gets a metadata side-struct stored on `Affliction` when the kind is `Trauma`:

```rust
pub struct TraumaMetadata {
    pub sources: BTreeSet<TraumaSource>,            // accumulates as new producer events fire
    pub cycles_since_last_event: u32,               // reset on producer fire, ticked otherwise
    pub observed_by: BTreeSet<TributeId>,
    pub observer_seen_cycle: BTreeMap<TributeId, u32>,
}
```

`Affliction` gains `pub trauma_metadata: Option<TraumaMetadata>` (mirrors phobia spec's `phobia_metadata` extension). `None` for all non-Trauma kinds.

## 5. Acquisition (producers)

The trauma producer pass runs once per cycle, **after** the cycle's event stream is built and **before** the brain pipeline runs. It scans the cycle's emitted `MessagePayload` stream for the five canonical patterns and emits acquisitions.

| # | Producer | Trigger pattern | First-occurrence severity | Co-acquisition |
|---|---|---|---|---|
| a | Witness ally death | `MessagePayload::TributeDied { tribute, .. }` co-located with witness who has `Bond { other: tribute, .. }` | `Mild` | none |
| b | Witness ally killed by specific cause | (a) AND `cause` is `Tribute(X)` or one of the environmental causes | `Mild` Trauma | `Mild` Traumatic `Phobia(cause)` via `try_acquire_phobia` |
| c | Survive near-death | `MessagePayload::TributeDamaged { tribute, hp_after, .. }` where `hp_after ≤ 10%` of max AND tribute did not die this cycle | `Moderate` | none |
| d | Survive betrayal | `MessagePayload::AllianceBondShock { victim, betrayer, .. }` | `Moderate` Trauma | `Mild` `Phobia(Tribute(betrayer))` via `try_acquire_phobia` |
| f | Witness mass casualty | ≥3 `TributeDied` events in same area same cycle | `Moderate` if 3-4, `Severe` if ≥5 | none |

### 5.1 Acquisition rules (single-instance + reinforcement)

- **One Trauma per tribute.** `try_acquire_trauma(tribute, source)` is the only entrypoint. If the tribute has no Trauma, it creates one at the producer's first-occurrence severity with `sources = {source}`.
- **Reinforcement on existing Trauma.** If a Trauma already exists, the producer event is treated as a "fire" against the existing trauma (see §6.1): `cycles_since_last_event` resets to 0, escalation roll fires, and `source` is inserted into the existing `sources` set.
- **Multiple producers same cycle.** Each producer event is a separate fire (resets the counter, rolls escalation independently). Severity is capped at `Severe` regardless of how many fires land in one cycle.
- **First-occurrence severity floor.** If a producer's first-occurrence severity exceeds the existing Trauma's severity, the existing Trauma is bumped up to that floor before the escalation roll. Rationale: a `Mild` Trauma should immediately become `Severe` if the tribute then witnesses a 5-death mass casualty; the reinforcement roll alone would not get there fast enough to feel right.

### 5.2 Co-acquired phobias

Producers (b) and (d) call `try_acquire_phobia` after `try_acquire_trauma`. The phobia acquisition is independent: it follows phobia spec acquisition rules (cap of 3, severity comparison for replacement). If the phobia is rejected (cap full of stronger), the trauma still acquires.

## 6. Reinforcement, decay, cure

Trauma uses the **shared traumatic-affliction reinforcement rule** introduced in the phobia spec amendment (`hangrier_games-1uhw`) and applied to fixations (`hangrier_games-wss4`). The single helper lives alongside the affliction storage and is called by all three systems.

### 6.1 Per-cycle reinforcement

Every cycle in which a producer fires for an existing Trauma:

1. **Counter reset.** `cycles_since_last_event = 0`.
2. **Source merge.** Insert the producer's source into `sources`.
3. **Severity floor.** If the producer's first-occurrence severity exceeds current severity, bump to that floor.
4. **Sensitization roll.** Roll `firing_escalation_chance` (default **12%**, tunable in `[10%, 15%]`). On success, severity steps up one tier (`Mild → Moderate → Severe`). At `Severe`, the roll is performed but no change is recorded and no `TraumaEscalated` is emitted.
5. **Emit `TraumaReinforced`** unconditionally; emit `TraumaEscalated { from, to, source }` on tier change.

### 6.2 Decay

A cycle in which no producer fires:

1. Increment `cycles_since_last_event`.
2. At threshold (default **10 cycles**) → severity steps down one tier; counter resets to 0.
3. Off the bottom of `Mild` → cure: remove the Trauma affliction and emit `TraumaHabituated { from: Mild, to: None }`.
4. On every other tier-down → emit `TraumaHabituated { from, to: Some(new) }`.

Decay and reinforcement are mutually exclusive in a given cycle (the producer-pass either reset the counter, or it didn't).

### 6.3 Cure paths

| Path | Effect |
|---|---|
| Decay (10 trigger-free cycles) | -1 tier; off Mild = cured |
| **Sponsor gift: Therapy** (filed `dvd`) | Removes Trauma entirely; also removes co-acquired Traumatic phobias whose `Origin::Traumatic { event_ref }` matches a source in the cured Trauma's `sources` set |
| **Sponsor gift: Sedative** (already in phobia spec §7.3) | Suppresses the next flashback roll for one cycle (does not reset the decay counter) |
| **Ally aid** (`hangrier_games-e2cf`, filed) | Co-located ally spends a turn → -1 sanity stat penalty for one cycle (palliative; does not affect severity, counter, or escalation) |
| Shelter rest | Counts as trigger-free toward decay; sanity stat penalty halved during the rest cycle |

## 7. Effects (stat penalty, avoidance bias, flashback, sleep penalty)

All effects are tier-gated and compose into existing systems via the `affliction_modifiers` and brain-pipeline override patterns from the health-conditions spec.

| Effect | Mild | Moderate | Severe |
|---|---|---|---|
| **Stat penalty** (always-on, composes into `tribute.visible_modifiers`) | -1 sanity, -1 intelligence | -2 sanity, -2 intelligence, -1 strength | -3 sanity, -3 intelligence, -2 strength, -1 speed |
| **Avoidance bias** (brain pipeline) | none | weighted bias against actions co-located with any tribute or environmental feature matching a `TraumaSource` (see §7.1) | hard avoidance — refuses to enter such areas/engage such tributes unless cornered (no other safe destination available) |
| **Flashback chance** | 5%/cycle when a producer-related stimulus is co-located | 10%/cycle when stimulus co-located | 20%/cycle when stimulus co-located, plus **2%/cycle baseline** with no stimulus |
| **Sleep penalty (shelter rest)** | none | shelter rest restores 50% of normal stamina/sanity recovery | shelter rest restores 25% (nightmares) |

### 7.1 Avoidance source matching

A `TraumaSource` matches a current-cycle context as follows:

- `WitnessedAllyDeath { cause: Some(Tribute(X)), .. }` → matches when X is co-located.
- `WitnessedAllyDeath { cause: Some(Fire), .. }` → matches when area has fire terrain.
- `WitnessedAllyDeath { cause: Some(Drowning), .. }` → matches water-terrain areas.
- `WitnessedAllyDeath { cause: None }` → matches the area where the death occurred (stored at acquisition; if untracked, no match).
- `Betrayal { by: X }` → matches when X is co-located.
- `NearDeath { cause: Tribute(X) }` → matches when X is co-located.
- `NearDeath { cause: Fire | Drowning | Beast | Hazard }` → matches matching terrain/beast/hazard.
- `MassCasualty { cause_class: Combat }` → matches areas with ≥2 other tributes present.
- `MassCasualty { cause_class: Environmental | Gamemaker | Mixed }` → matches areas with active hazard.

Matching is computed per-cycle from `CycleContext`. If any source matches, avoidance bias / flashback stimulus condition is satisfied.

### 7.2 Flashback action

```rust
pub enum Action {
    // ... existing
    Flashback { trauma_source: TraumaSource },
    // ... Frozen variant from phobia spec, etc
}
```

A `Flashback` action:

- Tribute does nothing useful this cycle.
- Stat penalties from the trauma still apply.
- Cannot be interrupted; if attacked, takes damage normally with **no defense roll**.
- Emits `MessagePayload::TraumaFlashback { tribute, source, severity }`.
- **Counts as a visibility moment** (§9): co-located tributes are added to `observed_by`.

### 7.3 Severe avoidance refusal

When Severe avoidance triggers (the brain wants to enter / engage but trauma vetoes):

- The chosen alternative action (next-best non-avoiding action) is emitted normally.
- Additionally emit `MessagePayload::TraumaAvoidance { tribute, area, source }` for observability.
- **Counts as a visibility moment** (§9).
- If no alternative exists (cornered), the avoidance is overridden — the tribute acts as if Moderate (bias only, no veto). No `TraumaAvoidance` payload.

## 8. Brain pipeline placement

The unified pipeline (per phobia spec §9) becomes:

```
[psychotic, preferred, survival, stamina, fixation, phobia, trauma, affliction, gamemaker, alliance, consumable] → decide_base
```

Trauma slots **between phobia and generic affliction**:

- **Below phobia** because phobia's specific stimulus override should beat trauma's generalized avoidance. A tribute can carry both; phobia is sharper and wins the action override when both fire.
- **Above generic affliction** because trauma's flashback / avoidance should beat generic affliction stat-penalty action selection (e.g. Wounded(Severe) refusing combat does not block a Trauma flashback).

```rust
fn trauma_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    let trauma = tribute.trauma_affliction()?;
    let stim_match = trauma.metadata.sources.iter().any(|s| s.matches(tribute, ctx));

    // Flashback roll
    let chance = match (trauma.severity, stim_match) {
        (Severity::Mild, true)     => 0.05,
        (Severity::Moderate, true) => 0.10,
        (Severity::Severe, true)   => 0.20,
        (Severity::Severe, false)  => 0.02,  // baseline
        _                          => 0.0,
    };
    if ctx.rng.gen_bool(chance) {
        let source = trauma.metadata.sources.iter().next().cloned().unwrap();
        return Some(Action::Flashback { trauma_source: source });
    }

    // Severe avoidance veto
    if trauma.severity == Severity::Severe && stim_match {
        if let Some(alt) = avoidance_alternative(tribute, ctx, &trauma.metadata.sources) {
            return Some(alt);
        }
    }

    None  // Mild stat penalty + Moderate bias compose elsewhere
}
```

Stat penalties always compose via `tribute.visible_modifiers(ctx)` regardless of whether the override returns `Some`. Moderate avoidance bias is a `weight_modifier` applied to candidate actions in the brain's scoring pass, not an override.

## 9. Visibility (observer-aware)

Mirrors phobia spec §8 with one parameter difference (decay = 10 cycles vs phobia's 5).

Visibility moments (when other tributes learn about the trauma):

1. Any **flashback** (Mild/Moderate/Severe) — every co-located tribute at the start of the firing tribute's turn is added to `observed_by`; their `observer_seen_cycle` is set to current cycle.
2. **Severe avoidance refusal** that emits `TraumaAvoidance` — same observation set update.
3. The always-on stat penalty is **not** a visibility moment.
4. Moderate avoidance bias (weight modifier) is **not** a visibility moment (the bias only nudges scoring; the resulting behavior is not externally distinguishable from normal preference).

Observer decay: each cycle, for each observer in `observed_by`, if `current_cycle - observer_seen_cycle[id] > 10`, observer is removed and `TraumaForgotten` is emitted.

Brain consumer: `tribute.knows_trauma(target_id) -> bool`. Used by alliance scoring (§11) and the predator brain layer.

## 10. Messages

Eight new `MessagePayload` variants:

```rust
MessagePayload::TraumaAcquired   { tribute: TributeRef, source: TraumaSource, severity: Severity }
MessagePayload::TraumaReinforced { tribute: TributeRef, source: TraumaSource, severity: Severity }
MessagePayload::TraumaEscalated  { tribute: TributeRef, from: Severity, to: Severity, source: TraumaSource }
MessagePayload::TraumaFlashback  { tribute: TributeRef, source: TraumaSource, severity: Severity }
MessagePayload::TraumaAvoidance  { tribute: TributeRef, area: AreaRef, source: TraumaSource }
MessagePayload::TraumaObserved   { observer: TributeRef, subject: TributeRef, source: TraumaSource }
MessagePayload::TraumaForgotten  { observer: TributeRef, subject: TributeRef }
MessagePayload::TraumaHabituated { tribute: TributeRef, from: Severity, to: Option<Severity> }
```

`kind()` and `involves()` exhaustive matches gain 8 new arms (per the maintenance burden documented in `i26a`).

## 11. Alliance integration

Three rules, all gated on `knows_trauma`:

- **Soft acceptance penalty.** Tributes who know about a target's Trauma reduce their alliance-affinity score for that target by `severity_weight × alliance_trauma_penalty` (default `alliance_trauma_penalty = -0.15` per tier; tunable). Composes with existing alliance affinity calculation.
- **Hard veto on betrayer.** Severe Trauma with `Betrayal { by: X }` in `sources` imposes a hard veto on proposing alliance to X (and auto-refusal of proposals from X). Records `BondReason::Trauma`. The Mild `Phobia(Tribute(X))` co-acquired in producer (d) already provides this veto at all severities for the phobia path; the trauma-side veto is the redundant safety at Severe.
- **Sympathetic bond on observed flashback.** When an ally witnesses a flashback (visibility moment §9), the witness's bond-affinity toward the firing tribute is *increased* by `sympathetic_bond_increment` (default `+0.10`). This is the only positive social consequence and is the design payoff for trauma being observable: it makes carrying trauma matter to the alliance graph in both directions.

Acquired-origin fixations (per §3 relationship to fixation system): when producer (d) fires (betrayal), in addition to co-acquiring `Phobia(Tribute(X))`, the trauma producer may also call `try_acquire_fixation` for `Fixation(Tribute(X))` of `Acquired` origin (vengeance pull). Whether this happens is gated by the betrayed tribute's traits — `Vindictive` (if it exists) auto-acquires; otherwise probability gated on a `vengeance_fixation_chance` default 30%. This exact integration is fixation-spec territory; we mention it here so the producer's contract is clear, but the `try_acquire_fixation` call lives in the fixation PR2 (`hangrier_games-wss4`) integration with this spec.

## 12. UI

**Tribute detail (admin view):**

- "Trauma" section after Phobias (decomposed via `lzfe`).
- One row per Trauma (always 0 or 1 per tribute): severity badge, source list (chips: "Witnessed Glimmer killed by Cato", "Survived betrayal by Marvel"), `cycles_since_last_event` countdown, observer count.
- Expandable observer list (who knows and how recently).

**Timeline cards:**

- New `trauma_card.rs` consuming all 8 payloads.
- `TraumaFlashback` is the headline event: red accent at Severe, orange at Moderate, yellow at Mild.
- `TraumaAvoidance` is a distinct compact card (refusal moment).
- `TraumaReinforced` collapses with `TraumaEscalated` in the visual stack when both fire on the same cycle (escalation supersedes; reinforcement-only renders as a small "intensifies" indicator).
- Reuses `CardShell` (`t7g1`).

**Tribute state strip:**

- Compact icon when a flashback is firing this cycle.
- Distinct subtle indicator when Trauma is present but quiescent.
- Tooltip lists severity + source count.

**Spectator skin:**

- Trauma counts as "psychological" content; respects existing severity-color WCAG audit (`hangrier_games-3yb`).

**No tribute-viewport in v1.** Filed (`n52s`) and would respect observer state when built.

## 13. Testing strategy

Per `uz80` (proptest) and `yj9u` (snapshot streams).

### 13.1 Unit tests

- Acquisition severity table: each producer at first-occurrence produces correct severity.
- Single-instance rule: second producer event on existing Trauma reinforces, does not create second Trauma; sources merge.
- First-occurrence severity floor: weak existing Trauma + strong producer event → severity bumped to floor before escalation roll.
- Reinforcement: counter reset on producer fire, escalation rolls at configured rate (seeded RNG).
- Decay: counter increments each trigger-free cycle, tier-down at threshold, cure off Mild.
- Decay and reinforcement mutually exclusive in a single cycle.
- Trait modifier composition (if relevant): `Resilient` reduces effective severity tier on flashback rolls (mirrors phobia trait modifiers); `Fragile` increases.
- Source matching truth table: each `TraumaSource` × `CycleContext` pair → expected match outcome.
- Flashback action: emitted with correct source; tribute takes damage with no defense roll if attacked the same cycle.
- Severe avoidance: alternative action chosen when available; falls through to bias when cornered.
- Co-acquired phobia: producers (b)/(d) call `try_acquire_phobia` with correct trigger; phobia rejection on cap-full does not block trauma acquisition.

### 13.2 Integration tests (`game/tests/trauma_*.rs`)

- `witness_ally_death_acquires_trauma.rs`: Glimmer killed; co-located ally Marvel acquires Mild Trauma; correct payloads.
- `witness_specific_cause_co_acquires_phobia.rs`: producer (b); Marvel acquires Mild Trauma + Mild Traumatic `Phobia(Fire)`.
- `near_death_acquires_moderate.rs`: tribute drops to 8% HP and survives; Moderate Trauma acquired with `NearDeath { cause }` source.
- `betrayal_acquires_phobia_and_optional_fixation.rs`: BondShock; Moderate Trauma + Mild `Phobia(Tribute(betrayer))` co-acquired; vengeance fixation rolled per chance (seeded).
- `mass_casualty_severe.rs`: 5 deaths in same area same cycle; witness acquires Severe Trauma directly.
- `flashback_in_combat_no_defense.rs`: Severe Trauma tribute with stimulus co-located; flashback rolls (seeded); attacker hits without defense roll.
- `severe_avoidance_refusal.rs`: Severe Trauma with `Betrayal { by: X }`; X enters area; tribute refuses to engage and flees to alternative.
- `severe_avoidance_cornered.rs`: same setup, no alternative area; falls through to bias; no `TraumaAvoidance` payload.
- `decay_to_cure.rs`: Mild Trauma; 10 trigger-free cycles; cured; `TraumaHabituated { to: None }` emitted.
- `reinforcement_escalates.rs`: Mild Trauma; producer fires; seeded escalation hit; becomes Moderate; correct payload; counter reset.
- `observer_learns_then_alliance_penalty.rs`: target has Severe Trauma; flashback co-located with potential ally; ally observes; subsequent alliance proposal scored with reduced affinity.
- `sympathetic_bond_increment.rs`: ally witnesses ally's flashback; bond-affinity increases by sympathetic_bond_increment.
- `forgotten_after_decay.rs`: 11 cycles separated; observer's `knows_trauma` returns false; `TraumaForgotten` emitted.
- `therapy_cures_trauma_and_co_phobias.rs`: Therapy sponsor gift (using trapdoor for `dvd`-blocked test); Trauma removed AND Traumatic phobias whose event_ref matches a Trauma source removed.

### 13.3 Insta snapshots

- Trauma state on `Tribute` after each integration scenario (BTreeMap of Trauma metadata including sources and observer set).
- Ordered `MessagePayload` streams for acquire → reinforce → escalate → flashback → habituate paths.

### 13.4 Proptest properties (per `uz80`)

- **Single-instance invariant**: across any sequence of producer events, a tribute has at most one Trauma affliction.
- **Source-set monotonic during acquisition burst**: within the producer pass of a single cycle, `sources` only grows.
- **Reinforcement-decay exclusivity**: in any cycle, `cycles_since_last_event` is either reset to 0 (producer fired) or incremented (no producer fired), never both.
- **Reinforcement bounded**: over N seeded trials of producer fires at sub-Severe severity, escalation rate hits `firing_escalation_chance` ±tolerance.
- **Decay reaches cure**: from any starting severity, 10 × tier_count trigger-free cycles cures the trauma (assuming no producer events).
- **Severity floor monotonic**: severity never decreases as a result of a producer event (it can only stay or rise via floor + escalation).
- **Observer monotonicity within cycle**: observer set only grows during a single cycle's flashback emissions; decay only at cycle boundary.

## 14. Migration / rollout

No data migration required. New `AfflictionKind::Trauma` variant is additive; existing tributes default to no trauma. New `MessagePayload` variants are additive. SurrealDB schema unchanged (affliction object is flexible per health-conditions spec; verify in PR1).

Rollout: ship behind a `trauma_enabled: bool` config flag on `Game`, default `true` once tuning settles. Producer pass and brain layer both no-op when flag is off. Mirrors phobia + fixation rollout pattern.

## 15. PR breakdown

- **PR1** — Types: `AfflictionKind::Trauma`, `TraumaSource`, `TraumaMetadata`, `DeathCause`, `CauseClass`. Storage: `trauma_metadata` field on `Affliction`. `try_acquire_trauma` helper enforcing single-instance + source-merge + severity-floor. Unit tests for severity table, single-instance rule, source-set merging, severity floor.
- **PR2** — Producer pipeline. Five producers (a/b/c/d/f) each as a function consuming the cycle's `MessagePayload` stream and emitting acquisitions. Co-acquired `try_acquire_phobia` for (b) and (d). Vengeance-fixation hook for (d) (if fixation PR2 has landed; else stub). Integration tests per producer.
- **PR3** — Brain layer (`trauma_override`), `Action::Flashback`, `Action::Avoidance`, severity-tiered effects table (stat penalty, avoidance bias, flashback rolls, sleep penalty), alliance integration (acceptance penalty, betrayer veto, sympathetic bond), observer state, decay tick, reinforcement-on-producer (the shared rule), all 8 message payloads emitted. Integration tests for flashback, Severe avoidance, alliance veto, sympathetic bond, observer learning, decay-to-cure.
- **PR4** — Frontend: tribute-detail "Trauma" section after Phobias, timeline `trauma_card.rs` consuming all 8 payloads, state-strip flashback indicator, spectator-skin integration. Smoke tests + WCAG check on severity colors.

### Hard prerequisites

- PR1 blocked by **afflictions PR1** (`hangrier_games-lsis`) — needs `AfflictionKind` extensibility + `try_acquire_affliction` + flexible affliction object shape.
- PR2 blocked by **phobia PR1** — needs `try_acquire_phobia` for co-acquisition.
- PR3 blocked by **brain pipeline unification** (`hangrier_games-hbox`) — same reason phobia PR2 was blocked; the trauma layer slots into the unified pipeline.
- PR3 also implements (or coordinates with) the **shared traumatic-affliction reinforcement helper** introduced for phobia PR3 (`hangrier_games-qqqx`) and fixation PR2 (`hangrier_games-wss4`). Whichever of the three lands first should extract the helper into a shared location (likely `shared/src/afflictions.rs`) so the others can call it.
- PR4 lands last, decoupled from PR3.

### Soft dependencies

- **Vengeance-fixation** integration in PR2 producer (d) depends on fixation PR1 (`hangrier_games-fazp` epic). If fixation PR1 has not landed, the call is stubbed; the integration is wired in fixation PR2 (`hangrier_games-wss4`).
- **Therapy sponsor gift** cure path is filed under `dvd` and not implemented in this spec; PR3 provides the cure-removal logic so a future Therapy gift can call into it without further changes here.

## 16. Out of scope (filed as follow-ups)

- Producer (e) self-trauma from killing — needs morality system; file as `trauma: self-trauma from killing (morality dependency)`.
- Producer (g) sustained-source torture — needs new per-cycle source tracker; file as `trauma: sustained-source torture producer`.
- Producer (h) witness-mutilation — defer until afflictions stabilize; file as `trauma: witness-affliction-acquisition producer`.
- Trauma transmission across generations — N/A for this game.
- Trauma-specific sponsor gifts beyond Therapy/Sedative — wait for `dvd`.
- Fine-grained source taxonomy (e.g. `Fire { intensity }`) — start coarse.
- Per-source decay rates — single 10-cycle rule for v1.
- Dedicated trauma-viewport UI — covered by `n52s`.

## 17. Spec self-review

- **Placeholders / tunables** (called out for post-observability tuning):
  - Severity acquisition table (per-producer first-occurrence severity).
  - `firing_escalation_chance` = 12% (window `[10%, 15%]`, matches phobia + fixation).
  - Decay threshold = 10 cycles (intentionally 2× phobia's 5).
  - Observer decay = 10 cycles (matches trauma decay; intentionally 2× phobia observer decay).
  - Flashback chances: 5%/10%/20% per tier with stimulus; 2%/cycle Severe baseline.
  - Mass-casualty thresholds: ≥3 deaths Moderate, ≥5 deaths Severe.
  - Near-death threshold: ≤10% HP.
  - `alliance_trauma_penalty` = -0.15 per tier.
  - `sympathetic_bond_increment` = +0.10.
  - `vengeance_fixation_chance` = 30% (used in producer (d) integration with fixation system).

- **Internal consistency:**
  - Severity ordering, reaction-tier table, and visibility-firing rules use the same `Severity` ordering as health-conditions and phobias.
  - Single-Trauma-per-tribute is consistent throughout (acquisition §5.1, storage §4, UI §12, proptest §13.4).
  - Reinforcement rule is the same shared helper as phobia PR3 and fixation PR2 (cross-referenced explicitly).
  - Brain pipeline placement (between phobia and affliction) is consistent with phobia spec §9 placement (between stamina and affliction): phobia → trauma → affliction is a stable left-to-right gradient from "specific stimulus" to "generalized state" to "physical condition".

- **Scope check:** PR1-PR4 each ~300-500 LOC plus tests, comparable to phobia and fixation PRs. Trauma-self from killing and witness-mutilation correctly deferred.

- **Ambiguity resolved inline:**
  - "co-located" defined as `ctx.area.tributes` at the start of the firing/dying tribute's turn (consistent with phobia spec §18).
  - Mass-casualty "same cycle" means within the same `cycle_id`; the trauma producer pass runs once per cycle so all deaths in that cycle are visible to it.
  - "Cornered" for Severe avoidance falls through to bias (Moderate behavior) rather than freeze, to keep tribute progressing through the game.

- **Open questions resolved by punt to follow-up:**
  - Trait modifiers (`Resilient`/`Fragile` etc) on trauma — listed in §13.1 unit tests as "if relevant"; recommend mirroring phobia trait table but defer the exact numbers to PR3 implementation.
  - Whether `MassCasualty` should distinguish the witnessing tribute's own faction — deferred; producer scans all area deaths uniformly in v1.
