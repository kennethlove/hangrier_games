# Hex-tile arena map (v1)

**Bead:** hangrier_games-89z
**Date:** 2026-05-01

## Summary

Replace the hand-coded static SVG `Map` (Cornucopia + N/S/E/W lobes) with a
hex-tile renderer. v1 is a flat 7-area pointy-top hex layout: Cornucopia in
the center, six surrounding sectors numbered 1–6 clockwise starting at the
top-right. No pan/zoom, no sub-tiles, no animations.

## Layout

Pointy-top hexes, 7 tiles arranged as:

```
       [6] [1]
    [5] [0] [2]
       [4] [3]
```

- `0` — Cornucopia (center)
- `1` — top-right neighbor
- `2..6` — clockwise around the center

This is the **default orientation**. The hex math module is written so a
future orientation change (e.g. flat-top, or a rotated numbering) only
requires swapping a layout function — game logic and DB rows are unaffected.

## Area enum migration

The `game::areas::Area` enum changes from 5 cardinal variants to 7 sector
variants:

```rust
pub enum Area {
    Cornucopia,   // center, axial (0, 0)
    Sector1,      // top-right, axial (1, -1)
    Sector2,      // right,     axial (1, 0)
    Sector3,      // bottom-right, axial (0, 1)
    Sector4,      // bottom-left,  axial (-1, 1)
    Sector5,      // left,         axial (-1, 0)
    Sector6,      // top-left,     axial (0, -1)
}
```

(Cornucopia stays as `#[default]` to keep `Tribute::new()` behavior.)

### Backwards compatibility

Not preserved. Old saved games stored with the cardinal-direction variants
(`North`/`East`/`South`/`West`) are not expected to deserialize cleanly into
the new sector variants and we do not add serde aliases. Users with
existing games may need to start fresh.

### Tests update mechanically

Every reference to `Area::North`/`East`/`South`/`West` in
`game/tests/*` and `game/src/games.rs` test modules updates to the new
sector names. Mechanical rename, ~80 sites.

**Frontend**: `web/src/components/map.rs` is fully replaced. The static
SVG path and the 5-key `HashMap` go away.

### `neighbors()` regenerates

The old hand-rolled neighbor table is replaced by the hex-grid neighbor
function from the new hex math module. Center has 6 neighbors (all sectors);
each sector has 3 neighbors (Cornucopia + 2 adjacent sectors).

## Components

### `game::areas::hex` (new module)

Pure Rust, no Dioxus deps. Lives in `game/src/areas/hex.rs` so both the
frontend and any future game logic can use it.

```rust
pub struct Axial { pub q: i32, pub r: i32 }

impl Axial {
    pub fn neighbors(self) -> [Axial; 6];
    pub fn to_pixel(self, size: f64) -> (f64, f64); // pointy-top
    pub fn distance(self, other: Axial) -> i32;
}

/// The 7-tile default layout: Cornucopia + 6 sectors.
pub fn default_layout() -> [(Area, Axial); 7];
```

Tested with host-side `cargo test -p game`:
- Each axial neighbor returns 6 unique adjacent coords
- `to_pixel` round-trips for the 7 default tiles (no overlaps, expected
  spacing)
- `distance(Cornucopia, sector_n) == 1` for all 6 sectors
- `distance(opposite sectors) == 2`

### `web::components::Map` (rewritten)

```rust
#[component]
pub fn Map(areas: Vec<AreaDetails>) -> Element {
    // For each (Area, Axial) in default_layout():
    //   - look up matching AreaDetails (by .area field)
    //   - render an SVG <polygon> with class "fill-..." driven by is_open()
    //   - render a <text> label inside (0..6 from default_layout index)
    //   - on click → existing area-detail flow (TBD: today the static SVG
    //     doesn't have click handlers; for v1 we add a stub onclick that
    //     emits a tracing log; wiring to AreaDetailPanel is a follow-up
    //     since the area-detail UI doesn't exist yet either)
}
```

SVG sizing: viewBox sized so the 7-tile layout fits with reasonable padding.
Hex `size` (center→corner) ≈ 60 SVG units; total viewBox ≈ 420×360.

Theming: keep the same Tailwind class style as the old map
(`fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400`), so
the visual language is consistent.

### Render tests

- `web/tests/map_test.rs` updates: pass the new 7-area `AreaDetails` set,
  assert `VirtualDom::rebuild_to_vec` doesn't panic.
- New: `web/src/components/map.rs` inline `#[cfg(test)]` for the layout
  helper (numeric label assignment, `data-area-id` attributes).

## Out of scope

- Pan / zoom / mini-map / keyboard nav → **hangrier_games-69g** (unblocked
  by this bead)
- Sub-tiles inside each area → **hangrier_games-l57** (depends on this;
  blocks 8pq and rzy)
- Larger maps / multiple templates → 2ac, y8c, orz
- Rotated / alternate orientations → future bead if needed; the hex math
  module is structured to make this a small change

## Acceptance

- [ ] `Area` enum has 7 variants; old serialized data deserializes via
  serde aliases
- [ ] `game::areas::hex` module with axial coords, neighbors, pixel layout,
  distance — fully unit tested
- [ ] `web::components::Map` renders 7 hex tiles with numeric labels 0–6
- [ ] Open/closed tile state respected via existing `is_open()`
- [ ] Render-smoke integration test passes
- [ ] All existing game tests pass after the variant rename
- [ ] `cargo clippy -p game --tests -- -D warnings` clean
- [ ] `cargo clippy -p web --tests -- -D warnings` clean
- [ ] `cargo fmt --all` clean

## Migration / rollout

Single PR. No DB migration script — old saved games are not preserved.

## Risks

- **Test breakage volume**: ~80 test sites mechanically updated. Risk is
  typo / wrong sector mapping. Mitigation: do the rename in a single search-
  and-replace per old variant, then `cargo test -p game` to catch anything
  missed.
- **Old saved games**: not preserved. Rows storing
  `"north"`/`"east"`/`"south"`/`"west"` will fail to deserialize into the
  new `Area` variants. Users may need to delete old games. Acceptable per
  product decision during brainstorming.
- **Visual regression**: the new hex map looks nothing like the old SVG
  flower. This is intentional and was approved during brainstorming.
