# Spectator Skin — Layout, Behavior, and Chrome

**Status:** Draft
**Date:** 2026-05-02
**Scope:** The structural and behavioral foundation of the default "spectator" view of Hangrier Games — grid layout, HUD, panel system, resize/collapse interactions, and chrome treatment. Visual identity (palette, typography, texture, iconography) is deferred to a companion spec.

## Goals

- Establish a coherent **broadcast composition** for the default view: HUD strip + Action + Map + Roster panels.
- Define a **panel system** that supports a shared substrate of role-agnostic infographic clarity, wrapped by a role-specific skin (initial role: spectator/sponsor; future roles: tribute, gamemaker).
- Provide **early-2000s-portal-style affordances** (resize, collapse) at the panel level without taking on the full complexity of a movable/closable portal.
- Keep the design **viewport-aware**: mobile, desktop, and widescreen all get an intentional layout, not a degraded one.
- Provide a **per-panel inspect drilldown** as the contextual debug surface, instead of a separate global debug route.
- Lay groundwork that future role skins (tribute, gamemaker) can extend or invert, not replace.

## Non-goals

- Visual identity: palette, typography, texture, iconography, motion language. Covered in a follow-up spec.
- Tribute-player and gamemaker role skins. Designed later as deltas from this baseline.
- Full portal-style movable/closable panels. Explicitly deferred to a possible future enhancement.
- Per-account synced layout persistence. Per-device (localStorage) only for v1.
- Multi-monitor "pop-out panel" support. Reserved as a future menu item but not implemented.
- Redesigning existing routes outside the game view (`/account`, `/credits`, `/icons`, auth pages).

## Conceptual frame

The interface is built in two layers:

- **Substrate (always-on, role-agnostic).** Board-game / infographic clarity. Iconography, status pills, legible at-a-glance state. The visual *language* the game speaks regardless of who is watching. Shared across all current and future roles.
- **Skin (role-dependent).** Wraps the substrate. Provides the framing, chrome, and tonal register appropriate to the viewer's role.
  - **Spectator/sponsor skin (this spec).** Capitol broadcast register: chunky control-panel chrome, ornate framing, "watching the show" energy.
  - **Tribute skin (future).** District grit register: scarce, weathered, first-person tension, minimal chrome.
  - **Gamemaker skin (future).** Possibly a clinical control-room variant of the spectator skin, or its own register entirely.

When roles intersect on screen (e.g., a sponsor sending a gift to a tribute), the two registers are intended to *visibly collide*. The seam is a feature.

This spec covers **only the spectator skin's structural layer.**

## Layout system

### Three breakpoints, three compositions

The view is composed of four logical regions: **HUD**, **Action**, **Map**, **Roster**. The HUD is always a top strip; the other three reflow.

- **Mobile (< ~768px):** HUD strip (sticky top), then a 1×3 vertical stack of Action / Map / Roster. A sticky **quick-jump nav** (3 icons) lets the user punch directly to a panel without scrolling.
- **Desktop (~768–1536px):** HUD strip across the full top. Below, a **2×2 grid where Action takes the full left column (tall)**, Map occupies top-right, Roster occupies bottom-right. Action gets the most pixels in the most-scanned position.
- **Widescreen (> ~1536px):** HUD strip on top, then **Action | Map | Roster** as three full-height columns left-to-right. Action remains leftmost (first-read).

Reading order is preserved across breakpoints: **HUD → Action → Map → Roster.**

### Panel sizing

Every panel has min/max constraints expressed in the grid. **No panel ever shrinks small enough to become non-useful** (excepting the explicit collapsed state below). Resize gestures (see "Behavior") cannot violate min/max.

Concrete min/max values are an implementation detail (the implementing instance can pick reasonable defaults), but the spec requires:

- A panel's minimum size must keep its primary content legible without horizontal scrolling.
- A panel's maximum size should not let it crowd siblings below their minimums.
- The HUD strip's height is governed by its summary/expanded state (see HUD section), not by user resize.

## HUD strip

The HUD is the **persistent top-of-page broadcast bar**. It is the most explicitly "Capitol spectacle" surface in the spectator skin — the equivalent of a sports broadcast's bottom-line scorebug or top-of-screen graphic.

It has **two states, user-toggleable**: *summary* and *expanded*.

### Summary state (default for first-time viewers)

A thin scoreboard strip (~48–56px desktop, similar on mobile). Contents:

- **Day + phase chip** (e.g. `Day 3 · Dusk`).
- **Living count** (e.g. `14 / 24`).
- **Recent-death ticker.** Surfaces the last 1–2 deaths briefly (~10s) and fades. Suppressed when no fresh deaths.
- **Gamemaker event indicator.** A small icon that animates/pulses when an event is active. Clickable: opens the expanded HUD (or jumps to detail).
- **Expand toggle** on the right edge.

The summary state intentionally **does not show weather** — weather is per-area and any single icon would lie. Weather lives in the expanded mini-map.

### Expanded state

Roughly 3× the summary's height on desktop/widescreen. **On mobile the expanded state takes most/all of the vertical viewport** (it's a takeover; user must collapse it back to see the panels below).

Contents:

- Everything from summary, but the **day/phase becomes a labeled phase progress indicator** showing the full phase cycle (Dawn → Morning → Day → Dusk → Night) with the current phase highlighted.
- A **single mini-map** of the arena, with an **overlay toggle** between:
  - **Weather** — areas colored/iconned by current weather state.
  - **Events** — areas marked with recent-event density (heatmap) or last-event icon.
  - **Both** — combined overlay.
  Hover/tap an area to see the full state in a tooltip/popover.
- **Gamemaker event detail card** if one is active: what it is, how long, affected areas.
- *(Optional)* **Alliance summary chips** (e.g. `Careers (4) · Lone wolves (8) · Pairs (3)`).

The mini-map is intentionally a **condensed echo** of the main Map panel, not a competitor. The Map panel is the canonical spatial view.

### HUD persistence

- Default state for a first-time visitor is **collapsed (summary)**.
- The user's expand/collapse choice is **persisted in localStorage** and remembered across page navigations and reloads.
- Persistence is **global, not per-game** — once a user expands the HUD, they keep it expanded everywhere until they collapse it. (Per-game scoping is reserved for future enhancement if user feedback warrants it.)

## Panel system

### Shared `<PanelFrame>` component

All three content panels (Action, Map, Roster) are wrapped by a single shared component (provisional name `PanelFrame`). Slots:

- `title` — short label and optional icon.
- `context` — optional center strip in the header for live state (e.g. timeline mode for Action, current overlay for Map, sort/filter state for Roster).
- `menu_items` — populated by each panel; rendered by the frame as a popover.
- `body` — the panel's main content.

Why a shared component (not per-panel implementations):

- Enforces visual consistency — all menus, headers, dividers, and affordances look identical.
- Centralizes resize/collapse behavior (see below) so each panel author doesn't reimplement it.
- Centralizes the inspect drilldown affordance.
- Makes adding a new panel in the future trivial.
- Centralizes the future "pop out" affordance when/if it lands.

### Panel header anatomy

Reading left-to-right:

- **Left:** panel title + small icon.
- **Center:** optional live context strip (panel-defined).
- **Right:** the **panel menu**, opened from a kebab/gear control. Contents:
  - **Inspect** — opens the per-panel debug drilldown (see "Inspect drilldown" below). Always present.
  - **Panel-specific actions:**
    - *Action:* timeline mode (Static/Reveal/Live), playback speed, pause, jump-to-end. (See `2026-05-02-progressive-display-design.md`.)
    - *Map:* overlay (Weather / Events / Tributes / All), zoom level.
    - *Roster:* sort (alliance / status / name / district), filter (alive only / all).
  - **Pop out** *(future, reserved)* — open this panel in a new window for second-screen / multi-monitor use.

The menu pattern keeps the panel body clean. Inline controls in the panel body are reserved for **transport-style fast actions** (e.g. timeline play/pause), not configuration.

### Panel chrome

Per the "Hybrid C" decision: panels use **subtle gutters and dividers, with prominent panel headers**. They are not fully framed boxes, nor are they fully bleed-to-edge regions. The panel header is the visually anchored element; the body bleeds to the panel's allocated grid cell.

The visual *treatment* of headers, gutters, dividers, and the resize/collapse handles is intentionally **chunky and early-2000s-portal-coded** (see "Chrome treatment" below). This spec defines structure; the visual identity spec defines exact materials.

## Behavior

### Resize

- Both **column gutters and row gutters** are resizable, on viewports where the layout has more than one panel per axis (i.e. desktop and widescreen). Mobile (1×3 stack) does not expose resize handles.
- Dragging a gutter changes the proportional sizes of the panels it separates.
- Resize is constrained by the panels' min/max sizes; the gutter cannot be dragged past a sibling's minimum.
- Resize state is **persisted per-device per-breakpoint** in localStorage. Different breakpoints get independent layouts (a desktop layout is not applied at widescreen, and vice versa).

### Collapse

- Each panel's header includes a **collapse control** (in addition to or as part of the menu). Clicking it collapses the panel to **just its header strip** (window-shade behavior).
- A collapsed panel is still on screen and still has its menu. Clicking the same control restores it.
- **Adjacent panels grow proportionally** to fill the freed space, respecting their max constraints.
- Collapse state is persisted per-device per-breakpoint alongside resize state.
- The HUD is **not collapsible to nothing** — the closest equivalent is the summary state. The HUD always occupies its top strip.

### Inspect drilldown (contextual debug)

There is **no separate global debug route**. Instead, every panel's menu includes an **Inspect** action that opens a dense data view scoped to that panel's subsystem.

- **Action's Inspect** → full event log table: timestamp, source, severity, payload, affected entities. Sortable, filterable.
- **Map's Inspect** → full area/terrain table: every area, current weather, active hazards, tribute occupancy, event density.
- **Roster's Inspect** → full tribute table: every internal field exposed, sortable/filterable. SurrealDB-query-tool flavor.

The inspect view opens as either a modal/dialog over the broadcast composition, or as a side-panel takeover (implementing instance can choose; modal is simpler). Closing the inspect view returns the user to the broadcast composition unchanged.

This pattern means we do not maintain two parallel UIs (broadcast and debug). Broadcast is the identity; debug is contextual drilldown.

### Reset layout affordance

A **"Reset layout"** action exists in the global settings popover (see below). It clears the persisted resize/collapse state for the current breakpoint and restores the spec's default layout.

### Settings popover

The same global menu hosts:

- **Theme switcher** (existing).
- **Reset layout** (new, this spec).
- Future toggles (HUD persistence scope, sound, accessibility prefs, etc.) live here as well.

The popover replaces the current ad-hoc theme dropdown. Implementation may keep CSS-only behavior or switch to a Dioxus-managed popover; behavior, not implementation, is specified.

## Chrome treatment

The spectator skin **leans into early-2000s portal aesthetics** for its panel chrome and affordances. Concretely:

- **Beveled / raised panel borders.**
- **Visible drag-grip textures** on resize handles (e.g. `:::` dot patterns or equivalent).
- **Thick header bars** with raised/inset shading.
- **Gutter "rails"** between panels are visible, not invisible whitespace.
- **Chunky controls** (kebab/gear, collapse, transport buttons) — substantial click targets, not minimal flat icons.

This is on-theme: it sells the "Capitol gamemaker control panel" register, and it provides a strong differentiator from the eventual tribute skin (which is intended to feel sparse and chrome-free).

**Fallback path:** if the chunky aesthetic ages poorly in practice, fall back to a **Hybrid C** treatment — flat at rest, chunky on interaction (handles light up with grip textures on hover/drag, gutters thicken). This is captured as an open question for visual identity, not committed here.

The visual identity spec defines exact materials (colors, gradients, textures, type for header labels).

## Persistence summary

All persisted UI state for the spectator skin lives in **localStorage, per-device**, matching the existing theme/JWT pattern. No per-account sync in v1.

Persisted keys (suggested, implementing instance can pick exact names):

- HUD expanded/collapsed state (global).
- Panel resize state (per breakpoint).
- Panel collapsed state (per breakpoint).
- Panel menu selections that should persist (e.g. Map overlay, Roster sort) — implementing instance decides which are session-only vs persisted.

Per-account sync is reserved as a future bead.

## Integration points

- **Existing Dioxus app structure.** The spectator skin replaces the current `/games/:id`, `/games/:id/day/:day/:phase` page bodies with the broadcast composition. The route structure may stay the same; the rendered content changes.
- **Theme system.** The existing `theme1/2/3` Tailwind theme variants are *not* assumed to survive — the visual identity spec will decide whether they remain, get refined, or get replaced. The spectator skin is one *role*, not one *theme*; theming within the role is an open question.
- **WebSocket event stream (`use_game_websocket`).** The Action panel consumes this. Wiring is per the progressive display spec (`2026-05-02-progressive-display-design.md`).
- **Weather and emotion systems.** Their UI surfaces (per their respective specs) live inside the appropriate panels — emotion pills on tribute cards in Roster; weather indicators on the Map panel and HUD mini-map.
- **dioxus-query.** Continues to be the data layer; no changes implied.

## Testing strategy

- **Layout integration tests** at each breakpoint: HUD + 3 panels render, occupy expected grid cells, respect min/max.
- **Resize behavior:** dragging a gutter changes sizes, respects min/max, persists to localStorage, restores on reload.
- **Collapse behavior:** collapsing a panel reduces it to header height, adjacent panels grow proportionally, restoring inverts cleanly.
- **HUD toggle:** summary ↔ expanded transitions, persistence across reload, mobile takeover behavior.
- **Inspect drilldown:** opens scoped data view per panel, closes cleanly, does not corrupt panel state.
- **Quick-jump nav (mobile):** tapping a target scrolls to the corresponding panel.
- **Reset layout:** clears persisted state, restores spec defaults at current breakpoint, does not affect other breakpoints' state.

## Migration

This is largely greenfield UI work — there is no existing broadcast composition to migrate from. The migration is from the **current card-stack page layouts** to the new composed view.

- Existing routes can stay; only their rendered bodies change.
- The current `theme1/2/3` system continues to function until the visual identity spec replaces or refines it.
- Existing components (tribute cards, etc.) are absorbed into panels — most likely Roster reuses tribute card concepts but in a denser layout suited to a panel.
- No data model or API changes are required by this spec.

## Open questions for implementation

- Exact min/max sizes per panel, per breakpoint.
- Exact pixel heights for HUD summary vs expanded states at each breakpoint.
- Resize handle hit-area dimensions (chunky implies generous; needs to feel right under a mouse).
- Whether the inspect view is a modal or a side-panel takeover.
- Whether the settings popover reuses the existing CSS-only dropdown pattern or moves to a Dioxus-managed popover (likely the latter, for the additional content it'll host).
- Whether collapse and resize handles should animate transitions, and how aggressively.
- Whether mobile resize should expose any axis (e.g. drag a panel's bottom edge to grow its max-height) or be entirely fixed.

## Out of scope (filed or to be filed as beads)

- Tribute and gamemaker role skins.
- Per-account synced layout persistence.
- Pop-out / multi-monitor panel support.
- Drag-to-rearrange / closable panels (full portal mode, the path from B → C).
- The visual identity (palette, typography, texture, iconography, motion).
- Sound design / audio cues for layout interactions.
- Accessibility audit and remediation pass for the new chrome (separate bead recommended).
