# Spectator Skin — Visual Identity

**Status:** Draft
**Date:** 2026-05-02
**Scope:** The visual identity layer of the spectator skin — palette, typography, materiality, motion, and iconography. Companion to `2026-05-02-spectator-skin-layout-design.md`, which defines structure and behavior. Together the two specs fully define the spectator skin for v1.

## Goals

- Establish a coherent **Capitol broadcast** visual register: imperial baroque ground, broadcast-graphic energy, Art Deco geometric authority.
- Define a **palette system with four distinct roles** (ground, chrome, heraldic accent, content electrics) and a **12-district electric color system** for tribute-identity surfaces.
- Pick **typography** with three families serving three roles (display, numeric, body), each justified by the broadcast register.
- Commit to **subtle materiality** — physical-feeling chrome without slipping into skeuomorphism — with explicit guardrails.
- Keep **motion minimal** and purposeful, reserving ambient motion as a future enhancement for the HUD only.
- Define **iconography as a substrate + identity** split: a normalized treatment of the existing icon library plus a small bespoke set of Capitol identity marks.
- Bake **accessibility** in at the system level: every semantic signal carries at least two encodings, color is never the sole carrier of meaning.

## Non-goals

- Layout, behavior, panel system, HUD structure. Covered in the layout spec.
- Tribute and gamemaker role skins. Designed later as deltas.
- Existing routes outside the game view (`/account`, `/credits`, `/icons`, auth pages) — these get incidental updates as the design system rolls in but are not redesigned by this spec.
- A bespoke hand-drawn Art Deco substrate icon set. Recognized as a desirable future state; out of scope for v1.
- D13 colors / sigil / branding.
- Concrete font selection (specific font names) — this spec specifies the *flavor* and constraints; the implementing instance auditions specific faces from the shortlist.
- Concrete hex values for the 12-district palette — specified as design constraints, audited and pinned by the implementing instance.

## Conceptual frame

The spectator skin operates in the **Capitol broadcast** register: imperial baroque ground (deep blacks, oxblood, antique gold) overlaid with broadcast-graphic energy (saturated district electrics used as semantic content signals). The visual identity is *baroque-imperial-with-electricity* — explicitly not neon, not cyberpunk, not "futuristic," not generic-fantasy.

The skin sits on top of a **role-agnostic substrate** of board-game / infographic clarity: iconography, status pills, legible at-a-glance state. The substrate is the visual language all current and future role skins share.

This spec defines:

- The **shared substrate** — the typography body face, the iconography normalization rules, accessibility constraints — which carries forward to all future role skins.
- The **spectator skin proper** — palette, display and numeric typography, materiality, motion, identity icons — which is replaced or inverted by future role skins.

## Palette system

The palette has **four roles**, each with a clear job. Roles do not overlap.

### Role 1 — Ground

Very dark warm neutrals. Espresso black, deep oxblood near-black, charcoal with warm undertone. Used for:

- Page background.
- Panel bodies (the surface content sits on).
- Modal/dialog overlays.

Ground is intentionally *not pure black* — it carries a warm undertone that ties it to the chrome role and prevents the UI reading as a generic "dark mode" web app.

### Role 2 — Chrome

Antique gold and brass tones. Used for:

- Panel borders, gutters, dividers, header strips.
- Active controls (kebab/gear, transport buttons, resize handles).
- The HUD frame and structural elements.
- "Capitol speaking" affordances — system messages, gamemaker announcements (covered further in role 4 below; chrome and Capitol-voice share gold).

Chrome is the carrier of the *baroque/Deco* register. Its treatment (subtle gradients, sheen, bevels) is defined under Materiality below.

### Role 3 — Heraldic accent

Deep imperial red (oxblood-leaning rather than fire-engine). Used for:

- Danger, death, and urgent system signals.
- Critical gamemaker events.
- Error states (panel-level errors, failed loads).
- "Everyone is dead" / game-over end states.
- The recent-death ticker's accent edge.

Heraldic red is **scarce by design**. It must read as serious when it appears, which means it cannot appear casually. Hover states, tooltips, ordinary information signals all use other roles.

### Role 4 — Content electrics (district palette)

The 12-district color system. Used **only on tribute-identity surfaces**:

- Tribute cards in the Roster panel.
- Event log lines naming a tribute (Action panel).
- Map markers and tribute occupancy indicators.
- HUD recent-death ticker.
- Alliance summary chips (multi-color treatments for multi-district alliances; lone-wolves stay neutral).

District electrics are **never** used in chrome, panel headers, gutters, settings, or any non-tribute-specific surface. The Capitol baroque ground is preserved as the unifier.

#### District color sourcing (hybrid)

The 12 colors are derived from each district's industry, with the constraint that the resulting palette is **distributed evenly enough around the hue wheel** to remain individually distinguishable at a glance. Industry resonance is the *target*; visual distinguishability is the *constraint*.

Suggested industry → hue mapping (the implementing instance pins exact values):

- D1 (luxury) — pink/rose-gold or champagne pink (luxury, rose-gold accent against the Capitol's brass)
- D2 (masonry / weapons) — cool steel blue
- D3 (electronics) — electric cyan
- D4 (fishing) — deep ocean teal
- D5 (power) — chartreuse / electric yellow-green
- D6 (transportation) — burnt orange
- D7 (lumber) — forest amber / sap-green
- D8 (textiles) — magenta / rich violet
- D9 (grain) — wheat-gold (must differentiate from chrome gold — possibly a paler or greener variant)
- D10 (livestock) — terracotta / clay red
- D11 (agriculture) — olive / earthen green
- D12 (coal) — ember orange

Constraints on the final palette:

- Every color must reach **WCAG AA contrast (4.5:1)** against the ground role.
- Every color must remain visually distinct from chrome gold and from heraldic red, which are reserved for other meanings.
- Colors should be ordered in a defined sequence so that adjacent districts don't have hostile color interactions when shown together (e.g. D1 next to D2 in a roster).
- D9 (wheat-gold) is the highest collision risk with the chrome role; the implementing instance must shift it sufficiently warm-green or muted to read as "wheat" not "more chrome."

#### Sequential grouping rule

When multiple consecutive items would carry the same district color (e.g. five Action-log lines in a row mentioning D2 tributes), the items **collapse under a single shared district banner/header**. The banner carries the color and (per accessibility) the sigil + number; the items underneath are unstyled with respect to district color, or at most subtly indented.

This applies anywhere repeated district color would otherwise produce visual noise — Action panel event log primarily, and the HUD death ticker if multiple deaths from one district arrive close together.

#### Capitol's voice

The Capitol does not get a 13th electric. **Chrome gold does double duty** as both decorative chrome and "the show speaking" — Capitol announcements, gamemaker notices, system-voice content. This keeps the system to two channels (gold = the show; district color = a tribute) and avoids inventing a color whose only job is being not-a-district.

D13 is not in scope.

#### Palette reference table

| Role | Hue character | Where used |
| --- | --- | --- |
| Ground | Deep warm neutral (espresso, oxblood-near-black) | Page bg, panel bodies, modal overlays |
| Chrome | Antique gold / brass | Borders, headers, active controls, Capitol-voice content |
| Heraldic accent | Deep oxblood imperial red | Danger, death, urgent system signals — scarce |
| Content electric (12) | District palette per above | Tribute-identity surfaces only |

## Typography

Three families, three roles. Specific fonts are auditioned during implementation; this spec specifies flavor and role.

### Display family — Art Deco

Headers, panel titles, page chrome, ceremonial type. Carries the Capitol register at the largest sizes.

- **Flavor:** Art Deco / Streamline Moderne. Geometric construction, stylized authority. Examples in this register: Limelight, Poiret One, Della Respira, Forum.
- **Justification:** the Capitol's screen design language in canon is overtly Art Deco. Choosing Deco is *more on-canon* than the imperial-Roman-serif default and avoids the Trajan cliché of every fantasy game. It also justifies the chunky Deco-bevel chrome we already committed to in the layout spec.
- **Open implementation detail:** many display Deco faces don't hold up at small panel-header sizes. The implementing instance is permitted to introduce a **secondary display face** for small-header use, ideally from the same era / geometric family as the primary (a "deck" version, or a paired geometric like a quieter Forum companion). This is treated as an implementation detail, not a separate decision.

### Numeric family — Deco-coherent broadcast numerals

HUD scoreboard, day counter, living count, future timers, future sponsor-currency display.

- **Primary flavor:** numerals that match the Deco display register. Faces designed in the same era / geometry, or pairings explicitly designed to harmonize with the display family. The numeric face must feel like it came from the *same broadcast graphics package* as the display face.
- **Fallback flavor:** broadcast-style display numerals (DIN, United Sans Display, modern broadcast tabular numerics). Acceptable if a Deco-coherent option doesn't exist.
- **Ultimate fallback:** clean tabular monospace (JetBrains Mono, IBM Plex Mono). Acceptable if neither Deco-coherent nor broadcast-style options work — sacrifices "feels like a broadcast" for "data is unambiguous."
- **Required:** tabular figures (`font-variant-numeric: tabular-nums`) on every numeric surface. Day counters and tallies must align across renders.

### Body family — neutral humanist sans, accessibility-leaning

Event log prose, tooltips, settings, panel body content, tribute-card metadata. The substrate's text voice.

- **Flavor:** neutral, highly-readable, screen-optimized humanist sans. Examples: Inter, IBM Plex Sans, Atkinson Hyperlegible, Source Sans, Söhne.
- **Justification:** body face is the wrong place to spend identity budget. Display and numeric carry voice; body must disappear into legibility. Atkinson Hyperlegible specifically aids the accessibility goals already baked into the palette and iconography systems.
- **Permission to warm:** the implementing instance may audition slightly warmer alternatives (Public Sans, Source Serif at smaller sizes, etc.) if the result feels too neutral. Default starts from boring-and-legible.

## Materiality

The chrome is **subtly material** — physical-feeling without slipping into skeuomorphism. The chunky early-2000s portal aesthetic from the layout spec is executed through restrained materiality: light gradients, soft inner shadows, gold-leaf-style sheen on chrome elements, muted noise/grain on dark grounds.

Panels feel slightly *fabricated* — like printed cards or engraved metal plates — without literally rendering as such.

### Guardrails

These are spec-level constraints, not implementation suggestions:

1. **Materiality is reserved for chrome.** Panel borders, headers, gutters, controls, HUD frame. **Panel bodies and content surfaces stay flat.** Material treatment never appears under text content.
2. **One material per surface role.**
   - Chrome → gold-sheen.
   - Ground → subtle warm-noise (optional, very low intensity).
   - Heraldic accent → flat.
   - District electrics → flat.
   - Numerals on the HUD → flat.
   Materials do not layer.
3. **No drop shadows on type. Ever.** Subtle inner shadows on bevels and chrome surfaces are acceptable; type stays clean.
4. **All material treatments must be honest at 1× and 2×.** No hairline gradients that disappear on low-DPI; no textures that moiré on high-DPI; no sheen effects that become noise at retina scale.

### Why subtle, not heavy

- Pure flat under-delivers on what the layout spec promised — chunky Deco chrome, gilded affordances. Flat treatment leaves the chrome looking underspecified.
- Heavy materiality (real textures — gold leaf images, brushed brass, leather grounds) ages poorly, fights accessibility, costs performance, and risks iOS-6-skeuomorphism.
- Subtle materiality is the **broadcast-graphic register itself** — real broadcast graphics imply physicality without rendering it.
- Selectively dialing materiality up later is easy; reversing direction (paring back heavy textures) once asset pipelines exist is hard.

### Fallback path

If subtle materiality ages poorly in practice, the fallback is **flat-at-rest, chunky-on-interaction**: panels look modern when idle, chunky materiality emerges only when handles are hovered, gutters are dragged, or controls are activated. This is captured as an open implementation question, not a committed direction.

## Motion

**Minimal and purposeful only.** No idle animation, no ambient effects.

Permitted motion:

- Timeline reveal pacing (per `2026-05-02-progressive-display-design.md`).
- HUD ticker fades (recent-death ticker fades after ~10s).
- Panel collapse/expand transitions (window-shade behavior).
- Hover and active states on controls.
- Resize gutter drag tracking.

### Future ambient motion (HUD only)

Reserved as a future enhancement for the **HUD only** — never content panels (would interfere with reading the event log) or the Map (would compete with actual gameplay state):

- Slow gold-sheen drift across HUD chrome.
- Gentle pulsing on the "live ●" badge (already specified in the progressive display spec).
- Subtle parallax on HUD when expanded.

This is filed as a possible v1.x addition, not committed for v1.

### Reduced motion

`prefers-reduced-motion` must collapse all transitions to instant. Timeline reveal pacing reverts to Static mode under reduced-motion (already implied by the progressive display spec). HUD fades become instant swaps.

## Iconography

A **substrate + identity** split.

### Substrate icons (B+ treatment)

The existing game-icons.net library plus future additions form the **substrate icon vocabulary** — the role-agnostic infographic clarity layer. They are processed through a normalization pipeline so that the *finish* speaks Deco even though the underlying *forms* are generic.

#### Normalization pipeline

A scripted, repeatable process applied to every substrate SVG:

- **Single stroke weight** across the set (or two: regular and a bold for emphasis). All icons re-exported to the chosen weight.
- **Single grid size** with a defined safe area (24×24 or 32×32; pinned by the implementing instance).
- **Palette tokens, not hardcoded colors.** Icons render in `currentColor`. The token applied determines the rendered color: chrome-gold by default, district color when contextual, heraldic red for danger, muted-foreground for inactive states. **Never arbitrary colors.**
- **Metadata stripped** on export. Snap to grid.
- **Naming convention by semantic role**, not by source. `icon-tribute-injured`, not `icon-broken-arm-game-icons`. This lets implementations be swapped (B+ → future C) without touching consumer code.

#### Deco frame system

To make substrate icons feel curated rather than generic, they are presented inside one of a small number of **Deco frame variants** appropriate to context:

- **Medallion frame** — circular or octagonal Deco border, used on tribute-card status icons and other "this is a state" surfaces.
- **Stepped-border frame** — angular Deco border, used on HUD chrome icons.
- **Plain (no frame)** — used inline in event-log lines and other tight spaces where a frame would compete with prose.

The frames are rendered as Deco SVG/CSS composition, not as raster assets. They are the carrier of the Deco register; the icon inside is the glyph.

#### Future bespoke Deco substrate

A fully hand-drawn Art Deco substrate icon set is the desirable north star but **not in scope for v1**. It is filed as a future bead. The B+ treatment is foundational — its pipeline, palette tokens, naming, and frame system carry forward unchanged; future bespoke icons drop into the same system.

### Identity icons (bespoke)

A small set of high-identity-weight icons drawn bespoke from geometric primitives. These are *not* substrate; they appear in defined Capitol-skin chrome contexts only.

- **Panem seal.** Full-bleed page chrome, login screen, "broadcast off-air" empty states.
- **District sigils (12).** One per district. Used in tribute cards as the **secondary encoding pair** to district color (see Accessibility below).
- **Capitol / Gamemaker mark.** Used wherever Capitol-voice content appears.
- **Mockingjay (existing custom set).** Reserved for the future tribute-skin chrome; remains in place for the existing theme switcher in the interim.

Identity icons are designed as **composed SVG primitives** (circles, polygons, stepped borders, radial symmetry) — Art Deco identity marks are exactly the sort of thing this technique handles well. They do not go through the substrate normalization pipeline because they are not substrate.

## Accessibility

Color is never the sole carrier of semantic meaning. Every meaningful state must be encoded by **at least two channels**.

### District identity — standard triad

Every place district color appears, the standard triad is:

- **Color** (district electric, per palette).
- **Sigil** (district sigil from the bespoke identity icon set).
- **Number** (textual `D7`-style label).

Where space permits, all three appear. Where space is tight (event-log lines, narrow HUD ticker), the **textual label is mandatory**, the sigil is preferred, and color may serve as enhancement only.

### Status indicators — icon + label

Tribute status pills (injured, dehydrated, hungry, exposed, etc., per the emotion spec) are **always icon + text label paired**. Never icon-only.

### Other semantic signals

- Heraldic-red surfaces (danger, death) are paired with text and/or iconography.
- The "live ●" badge from the progressive display spec is paired with the word "Live" or equivalent.
- Gamemaker event indicators are paired with descriptive text on hover/expand.

### Reduced motion

See Motion section. `prefers-reduced-motion` is fully respected; no purely-decorative motion is ever uncancellable.

### Contrast

All foreground text on ground roles must meet **WCAG AA (4.5:1 for body, 3:1 for large)**. District electrics on ground must meet AA. Chrome gold against ground must meet at minimum the AA large-text threshold for use as control labels; ornamental chrome (borders, dividers) is exempt.

## Integration with prior specs

- **Layout spec (`2026-05-02-spectator-skin-layout-design.md`)** — this spec provides the visual treatment for panel chrome, HUD frame, headers, controls, dividers, and gutters specified there. The "chunky early-2000s portal" call is realized via Deco display type + subtle materiality + Deco frames + chrome gold.
- **Emotion spec (`2026-05-02-tribute-emotions-design.md`)** — tribute status pills use the substrate iconography (B+ treatment + medallion frame) plus body-face text labels per accessibility.
- **Weather spec (`2026-05-02-weather-system-design.md`)** — weather icons in the HUD mini-map and Map panel use the substrate iconography. Per-area weather is rendered with district-agnostic palette tokens (chrome gold or muted-foreground), since weather is environmental, not tribute-specific.
- **Progressive display spec (`2026-05-02-progressive-display-design.md`)** — the "live ●" badge uses the heraldic accent (oxblood imperial red) or chrome gold (implementing instance picks); the timeline reveal pacing is the primary motion in the Action panel.

## Testing strategy

- **Palette accessibility audit:** every district color, chrome gold, and heraldic red tested against ground for WCAG AA contrast. Automated test against the palette tokens.
- **Sequential-grouping correctness:** Action log and HUD ticker correctly collapse adjacent same-district items under a single banner.
- **Triad presence:** automated test that every tribute-identity surface in the rendered DOM includes color + sigil-or-label + number.
- **Status indicator pairing:** automated test that every status pill renders both icon and text label.
- **Reduced-motion compliance:** all animations cancelled or instant under `prefers-reduced-motion: reduce`.
- **Tabular-num correctness:** HUD numerics align across renders with different digit widths.
- **Icon pipeline:** snapshot tests on the normalized SVG output of the substrate pipeline; visual regression against the Deco frame variants.
- **Visual regression on identity icons:** snapshot tests for Panem seal, district sigils, Capitol mark.

## Migration

- The current `theme1/2/3` Tailwind theme variants (Ember/Forest/Gilded) are **superseded** by this spec. The spectator skin replaces all three. Future role skins (tribute, gamemaker) become the equivalent of "themes" in a more meaningful sense.
- The current Mockingjay custom icons remain in the theme switcher slot for now and migrate to tribute-skin chrome later.
- Existing game-icons.net SVGs go through the normalization pipeline as part of rollout; consumer code is updated to use the role-based naming convention.
- District color tokens, sigil set, identity icon set are introduced as new design system artifacts (`district_palette`, `district_sigils`, `capitol_marks`).
- Palette tokens, type tokens, materiality tokens, and motion tokens become part of the Tailwind config (or successor design-token system).

## Open questions for implementation

- Exact font selections for display, numeric, and body families (audition from shortlist).
- Whether a secondary smaller-size display face is needed and which face it should be.
- Pinned hex values for the 12-district palette satisfying the constraints listed.
- Concrete materiality treatments — gradient angles, shadow radii, sheen intensity, noise texture choice.
- Whether to ship subtle ambient motion on the HUD in v1 or defer.
- Implementing the icon normalization pipeline — chosen toolchain (svgo, custom Python, Figma plugin export, etc.).
- Whether the "live ●" badge uses chrome gold or heraldic red.
- Whether to introduce a design-token system (Style Dictionary, native CSS custom properties, Tailwind tokens) or extend the existing Tailwind theme.

## Out of scope (filed or to be filed as beads)

- Bespoke hand-drawn Art Deco substrate icon set (future bead).
- Tribute and gamemaker role skins.
- D13 colors / sigil / branding.
- Per-account synced visual preferences (theme variants within a role, if introduced).
- Sound design / audio identity.
- Sponsor-side UI surfaces and brand marks (future epic).
- Marketing site / landing page visual identity outside the game view.
- Print/export visual treatment (PDF replays, share images, etc.).
