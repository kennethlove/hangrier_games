# GameEvent enum — structured replacement for GameOutput

**Date:** 2026-04-26
**Bead:** [hangrier_games-mqi.1](../../../.beads) — first child of the **mqi** epic ("Migrate event log from stringly-typed GameOutput to structured GameEvent")
**Status:** Implemented (this PR introduces the type; no emission sites switched yet)
**Author:** rust-engineer agent

## Rationale

`GameOutput` (in `game/src/output.rs`) is the engine's only event vocabulary today. It has two jobs jammed into one type:

1. **Render** the player-facing log line (what the `Display` impl produces).
2. **Identify** what happened so consumers (DB, websocket clients, announcers, future analytics) can react.

Job 1 it does well. Job 2 it does badly:

- Variants borrow `&'a str`, so events cannot be stored, queued, or sent across threads.
- Names appear in payloads but UUIDs do not — a downstream consumer has no stable handle for the tribute, area, or item involved.
- Some variants (`TributeMauled`) accept stringly-typed enum members (`animal: &str`) and `FromStr` them inside `Display`, so a malformed string panics at render time.
- Adding any new field to a tuple variant is a breaking call-site change.

Downstream code (announcers in particular, plus the Game.messages buffer landed in mqi precursor PR #125) is forced to either pre-stringify everything before the engine touches it or re-parse the rendered sentence. Both options are fragile.

`GameEvent` solves this by carrying **typed, owned, serde-friendly** fields. The engine's render path stays identical (Display still produces the same bytes), so this is a *purely additive* change in this bead. Subsequent beads (mqi.2–6) will migrate emission sites and persistence layers off `GameOutput`.

## Scope of this bead (mqi.1)

-  Introduce `game::events::GameEvent` enum.
-  One variant per existing `GameOutput` variant (77 total).
-  `Display` impl producing **byte-identical** strings.
-  `Debug, Clone, PartialEq, Serialize, Deserialize` derives.
-  Parity tests asserting Display equality across all 77 variants.
-  Serde roundtrip tests covering every distinct payload shape.
-  No emission sites changed (mqi.2).
-  No persistence wiring (mqi.3).
-  No announcer integration (mqi.4+).

## Variant inventory

All 77 `GameOutput` variants have a 1:1 `GameEvent` counterpart with the same name. The parity table in `game/src/events.rs::tests::parity_table` is the authoritative mapping; `parity_table_covers_every_variant` enforces the count via assertion.

Categories (for readability — the enum itself is flat):

| Category | Variant count | Examples |
|---|---:|---|
| Day/night cycle | 11 | `GameDayStart`, `FirstDayStart`, `TributesLeft`, `TributeWins` |
| Rest/hide/movement | 12 | `TributeRest`, `TributeTravel`, `TributeTakeItem`, `TributeUseItem` |
| Status effects (single tribute) | 19 | `TributeBleeds`, `TributeMauled`, `TributeHorrified`, `TributeSuicide` |
| Combat | 12 | `TributeAttackWin`, `TributeCriticalHit`, `TributePerfectBlock` |
| Death | 6 | `TributeDiesFromStatus`, `TributeAlreadyDead`, `TributeDeath` |
| Items / equipment | 5 | `WeaponBreak`, `ShieldWear`, `SponsorGift` |
| Area events | 5 | `AreaEvent`, `AreaClose`, `TrappedInArea`, `DiedInArea` |
| Social / alliance | 7 | `AllianceFormed`, `BetrayalTriggered`, `TrustShockBreak` |

(Bead description estimated 75 variants; actual count is 77 because two alliance variants — `TributeBetrayal` and `TributeForcedBetrayal` — and the `TributeTravelNoOptions` variant slipped in after the bead was written. All are included.)

## Design decisions

### 1. Named struct variants everywhere

`GameOutput` uses positional tuple variants (`AllianceFormed(&str, &str, &str)`). `GameEvent` uses named struct variants (`AllianceFormed { tribute_a_id, tribute_a_name, tribute_b_id, tribute_b_name, factor }`).

**Why:** future fields can be added without breaking call sites, JSON output is self-describing, and pattern-match arms read like documentation. Cost is verbosity, which is acceptable for a type defined once and consumed many times.

### 2. Carry both UUID and name

Where `GameOutput` carries only a name (`TributeRest("Alice")`), `GameEvent` carries `tribute_id: Uuid` **and** `tribute_name: String`.

**Why:**
- The UUID is the only stable cross-system reference. Names can collide (two "Cato"s in different games), names can be edited, names embed in URLs poorly.
- The name is needed verbatim for `Display` to reproduce the legacy log line. We refuse to make `Display` perform a name lookup against external state — the event must be self-contained.
- Storing both is a few bytes per event. We are not in a memory-constrained environment.

### 3. Embed full `Item`, not just name

`TributeUseItem` and `SponsorGift` carry the entire `Item` struct. `Item` is already `Clone + Serialize + Deserialize + PartialEq`, and the rendered string needs `item.name`, `item.effect`, `item.attribute`, `item.current_durability`, `item.max_durability`. Embedding the whole struct is simpler than fanning fields out and keeps the event a faithful record of what happened.

### 4. Animal as enum, not string

`GameOutput::TributeMauled` carries `animal: &str` and parses it via `Animal::from_str` inside `Display` (a `.unwrap()` that would panic on a typo). `GameEvent::TributeMauled` carries `animal: Animal` directly. The parity test passes the discriminant name (`"Wolf"`) to `GameOutput` and `Animal::Wolf` to `GameEvent`, and both render `"wolves"` via `Animal::plural()`.

### 5. Serde shape: externally-tagged (default)

```json
{"GameDayStart": {"day_number": 4}}
{"AllianceFormed": {"tribute_a_id": "11111111-…", "tribute_a_name": "Alice", …}}
{"FirstDayStart": null}    // unit variant
```

**Alternatives considered:**

| Shape | Pros | Cons |
|---|---|---|
| **Externally-tagged** (chosen) | Default, no attribute noise, unambiguous, plays nicely with unit variants and named-struct variants alike, every serde adapter on every wire format already handles it. | Slightly verbose JSON; the variant name is a JSON object key (not a field). |
| Internally-tagged `#[serde(tag = "type")]` | Flatter JSON: `{"type": "GameDayStart", "day_number": 4}`. Reads nicely in DB browsers. | **Cannot represent unit variants** (`FirstDayStart`) without workaround; cannot represent newtype-of-non-struct variants. Would require splitting the enum or per-variant remediation. |
| Adjacently-tagged `#[serde(tag = "type", content = "data")]` | Round-trips every variant shape, structurally explicit. | Doubly-nested JSON for the common case; harder to query in SurrealDB without `data.*` paths everywhere. |
| Untagged | Cleanest JSON when payloads are distinguishable. | Payload shapes overlap heavily here (many variants are `{tribute_id, tribute_name}`), so deserialization would be ambiguous. Non-starter. |

Externally-tagged is the only shape that handles all 77 variants uniformly with zero attribute noise and zero ambiguity. The JSON-key-is-variant-name aspect is the cost we pay for that uniformity, and downstream consumers (SurrealDB JSON, websocket frames, announcer prompts) can handle it.

### 6. Selective `Eq + Hash`

The bead acceptance criteria suggested adding `Eq + Hash` only to UUID-only variants. After implementation, almost every variant carries a `String` (the rendered name), and `String` is `Eq + Hash`, so the discriminator could in principle apply broadly. However, two variants embed `Item`, and `Item` does not currently derive `Hash`. Rather than thread `Hash` through `Item` purely for this enum, **`GameEvent` derives `Debug + Clone + PartialEq + Serialize + Deserialize` only.** Consumers that need `Hash` should hash a stable identifier (e.g. a future `event_id: Uuid` field added in mqi.3) rather than the entire payload.

If `Eq + Hash` becomes load-bearing later, a sibling `#[derive(Hash)]` on `Item` is the right fix.

### 7. No I/O, no DB, no announcer plumbing

This bead is type design. The engine still emits `GameOutput` exclusively. The next bead (mqi.2) will switch emission sites one cluster at a time (alliance first, then combat, then status effects, etc.) and run both paths in parallel until parity is verified end-to-end.

## Migration plan

| Bead | Scope |
|---|---|
| **mqi.1** (this PR) | Introduce `GameEvent` + parity tests. |
| mqi.2 | Switch emission sites to also emit `GameEvent` alongside `GameOutput`; add `Game.event_log: Vec<GameEvent>` buffer in `game::games`. |
| mqi.3 | Persist `GameEvent` in SurrealDB; add migration; expose via API read endpoints. |
| mqi.4 | Announcer (`announcers/`) consumes structured events instead of parsing rendered sentences. |
| mqi.5 | Frontend (`web/`) consumes structured events for live tickers. |
| mqi.6 | Remove `GameOutput` and the parallel emission path. |

Each step is independently shippable; if any step regresses, the parallel `GameOutput` path remains intact until mqi.6.

## Open questions

1. **Event identity.** Should `GameEvent` carry an `event_id: Uuid` and a `timestamp: DateTime<Utc>` of its own, mirroring `GameMessage`? Likely yes, but added in mqi.3 when persistence lands so the schema is final.
2. **Severity.** PR #122 introduced event severity for messages (Info / Warning / Critical). Should `GameEvent` expose a `fn severity(&self) -> Severity` method? Probably yes, again deferred to mqi.3.
3. **`AreaEvent` typing.** Currently `area_event: String`. Long-term this should be an `AreaEventKind` enum mirroring the `MessageKind` precedent. Captured as a follow-up — not in scope for mqi.1.
4. **Removing `GameOutput`.** After mqi.6, `GameOutput` is dead code. Removal is a breaking change for any external consumer that re-exports it (none today), so it can land in mqi.6 itself.

## Verification

- `cargo test -p game --lib events::` — 7 new tests pass.
- `cargo test -p game --lib` — 475 total tests pass (was 468; delta +7).
- `cargo clippy -p game --all-targets -- -D warnings` — clean.
- `cargo check -p api` — clean.
- WASM check on `web` — clean.
- `display_matches_game_output_for_every_variant` asserts byte-identical Display output across all 77 variants.
- `parity_table_covers_every_variant` ensures the table grows in lockstep with the enum.

## Implementation deviations

### mqi.3 — `event` column is `option<string>`, not `option<object>`

The original spec called for `event option<object>` on the `message` table, holding the externally-tagged JSON form of `GameEvent` as a structured Surreal `object`. In implementation, this proved infeasible: SurrealDB's bespoke SDK serializer collapses externally-tagged Rust enums (and arbitrary `serde_json::Value::Object` payloads) to `{}` whenever they are bound into an `object` column, on every write path tested (`db.insert(...).content(...)`, `db.query("INSERT INTO ... $rows").bind(...)`, etc.). The collapse happens server-side: `RETURN type::string(event)` returns the literal string `"{  }"`.

mqi.3 therefore stores the event payload as `event option<string>` — the JSON-serialized form of `GameEvent` as a plain string. `GameMessage::with_event` / `with_event_kind` perform the JSON encoding; `GameMessage::structured_event` decodes on the read side. Wire shape (externally-tagged JSON) is preserved verbatim, all acceptance criteria from mqi.3 are met (roundtrip, legacy readable, structured decode), and the choice mirrors the existing `event_id` precedent in this repo (originally spec'd as `Uuid`, stored as `String` for the same SDK-serializer mismatch reason).

If a future SurrealDB SDK release fixes externally-tagged enum handling on `object` columns, the column type can be migrated back to `option<object>` and the JSON-string transit dropped without any change to `GameEvent`'s serde shape or `GameMessage`'s public API.
