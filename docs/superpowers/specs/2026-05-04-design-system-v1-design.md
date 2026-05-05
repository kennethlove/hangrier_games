# Hangrier Games — Design System (v1)

**Date:** 2026-05-04
**Status:** Approved foundations. Special UI elements (e.g. arena map, kill-cam, victor card) deferred to follow-up specs.
**Replaces:** existing `theme1` / `theme2` / `theme3` ad-hoc themes and gold-gradient utilities in `web/assets/src/main.css`.

---

## 1. Tone & Intent

A broadcast-first design system for a Hunger Games simulator. The mix is intentional and proportional:

- **80% sports-broadcast spectacle** — telecast HUDs, scoreboards, lower-thirds, stat cards, breaking-news tickers. Energy and immediacy over ornament.
- **15% capitol-glam dystopia** — warm cream paper in light mode, deep velvet in dark mode, gold reserved for ceremonial moments.
- **5% pun / irreverence** — present as a slight wink in copy and special-case treatments; never in the chrome itself.

The system must feel like a single product across light and dark, recognizable as one brand even as the surface treatments shift.

---

## 2. Color

Two modes — both first-class. Same hue family, different application strategies. Light mode is *not* a derivation of dark mode; both were tuned independently for the broadcast feel.

### 2.1 Dark — `D2 Velvet`

Warm aubergine base. Cyan as primary signal, magenta as alert/danger.

| Token | Hex | Use |
|---|---|---|
| `--bg`         | `#19121A` | Page background |
| `--surface`    | `#241829` | Cards, scoreboard, top bar |
| `--surface-2`  | `#1F1521` | Sidebar HUD, secondary surface |
| `--border`     | `#3A2440` | All borders, dividers, rules |
| `--text`       | `#F2EBE2` | Primary text |
| `--text-muted` | `#A498A2` | Secondary text, metadata |
| `--primary`    | `#00E5FF` | Primary actions, data emphasis, scores |
| `--danger`     | `#FF2E6E` | Live, danger, kills |
| `--gold`       | `#E8B14B` | Ceremonial only (victor, reaping, end-card) |

### 2.2 Light — `L3 Capitol Cream`

Warm cream paper, deep teal primary, oxblood danger, gold accent. Reads as "official program."

| Token | Hex | Use |
|---|---|---|
| `--bg`         | `#FBF6E9` | Page background |
| `--surface`    | `#FFFCF2` | Cards, scoreboard, top bar |
| `--surface-2`  | `#F4EAC9` | Sidebar HUD, secondary surface |
| `--border`     | `#E8DCB8` | All borders, dividers, rules |
| `--text`       | `#1A1410` | Primary text |
| `--text-muted` | `#6B5938` | Secondary text, metadata |
| `--primary`    | `#007A99` | Primary actions, data emphasis, scores |
| `--danger`     | `#C8003C` | Live, danger, kills |
| `--gold`       | `#B8861B` | Ceremonial; also valid as a secondary accent (e.g. alive count) |

### 2.3 Color Semantics (mode-agnostic rules)

- **Cyan/teal (`--primary`)** — anything good, alive, or a primary data point. Active nav, primary buttons, score numerals, "alive" counts, links.
- **Magenta/oxblood (`--danger`)** — live indicator, kill counts, destructive actions, danger states. Never decorative.
- **Gold** — reserved. Use only for ceremonial moments and special-case ornamental treatment. Light mode may use it for one secondary data point on a screen (e.g. alive count) when a screen is otherwise heavy on `--primary`.
- Never introduce additional accent hues without a system update.

---

## 3. Typography

Three roles, three families. All free, all on Google Fonts.

| Role | Family | Used for |
|---|---|---|
| **Display** | Bebas Neue (400) | Brand, scoreboard team names, headlines, section banners, button labels for chrome buttons |
| **Text** | Source Sans 3 (400/500/600/700) | UI labels, body copy, navigation, metadata, kickers |
| **Mono** | IBM Plex Mono (500/700) | All numerals (scores, HP, kills, ratings), timestamps, district codes, breaking-news ticker, monospaced data |

### 3.1 Type rules

- **Numerals are always Mono.** Scores, day counters, kill counts, HP values, ratings, timestamps. The HUD "feels" right because numbers are tabular and uniform-width.
- **Display is condensed-only.** Never use Display for body text or anything below 16px.
- **ALL CAPS is for Display and for small uppercase labels in Text** (with letter-spacing 0.12–0.18em). Never set body or headlines in caps using Text.
- **Kickers** (the small uppercase label above a headline, e.g. "Day 3 · Nightfall") are Text 700 / 10–11px / 0.18em tracking, colored `--primary`.

---

## 4. Shape Language — Mixed (Hard Chrome, Soft Content)

The system uses two shape vocabularies. Each surface belongs to exactly one.

### 4.1 Hard chrome

For everything that frames or instruments the experience.

- Border-radius: `0`
- Borders: 1–2px solid `--border`. Heavier (2px) for the top bar bottom edge and the ticker top edge.
- ALL CAPS labels with broadcast-style tracking
- Tight vertical rhythm; data sits flush in cells
- Used for: top navigation bar, scoreboard, sidebar HUD, breaking-news ticker, stat grids, chrome buttons

### 4.2 Soft content

For everything that asks to be read.

- Border-radius: `10px` (cards), `6px` (buttons, inner stat tiles, badges)
- 1px solid `--border`, no hard rules between sections
- Sentence-case body, generous internal padding
- Used for: event cards, tribute profile cards, story panels, sponsor offer cards, modals, forms

### 4.3 Boundaries

When a soft card sits inside hard chrome (or vice versa), the boundary is a single `--border` line — no shadow, no glow. Shadows are not used in the system at v1.

---

## 5. Density & Spacing

### 5.1 Base scale

8px base, used as `8 / 16 / 24 / 32 / 48 / 64`. Half-step (`4px`) allowed only for inline icon/text spacing.

### 5.2 Two density modes

- **Cinematic (default)** — hero surfaces (scoreboard, headlines, tribute portraits, modals) get `32px+` padding and generous outer margins. The eye should land on one thing per screen. This is the default for player-facing screens.
- **Compact** — operational lists, logs, admin panels, and dense data tables use `8–14px` padding. Same 8px scale, just at the smaller end.

A single screen may mix the two — e.g. a cinematic scoreboard above a compact tribute list. The transition happens at the section boundary.

### 5.3 Letter-spacing scale

- ALL CAPS Display: `0.04–0.06em`
- ALL CAPS Text labels (uppercase small caps): `0.12em` for navigation, `0.16–0.18em` for tiny kickers/section labels
- Mono numerals: `0.02–0.05em`
- Body Text: default (none added)

---

## 6. Component Inventory (v1)

The following components are part of v1 and must follow the system rules above. Visual specs and prop interfaces will be defined in the implementation plan; this is the inventory only.

### 6.1 Chrome

- **Top bar** — brand, primary nav, live indicator, day/time clock
- **Scoreboard** — two team blocks (district shield + name + meta) flanking a large mono score
- **Sidebar HUD** — header + 2×N grid of stat cells, with optional chrome action button at bottom
- **Breaking-news ticker** — footer band, mono text, colored kicker tags

### 6.2 Content

- **Event card** — kicker, headline, body, action row
- **Tribute row** (in a card) — district avatar, name, mono meta line, right-aligned mono stat
- **Stat tile** — small uppercase label + large mono number, used inside the HUD and standalone

### 6.3 Interaction

- **Buttons** — `Primary` (filled cyan/teal), `Ghost` (1px cyan/teal outline, transparent fill), `Danger` (filled magenta/oxblood), `Chrome` (square-corner, surface-colored, used inside hard-chrome areas)
- **Live pill** — small filled badge in `--danger`, ALL CAPS Text, used in the top bar
- **Section label** — small uppercase muted Text label that introduces a content region

### 6.4 Out of scope for v1

These are acknowledged as future work and may require additions to the system. Each gets its own follow-up spec.

- Arena map and region overlays
- Kill-cam / replay viewer
- Victor proclamation / end-of-game card (gold-heavy, ceremonial)
- Reaping reveal animation
- Sponsor wallet / offer flow
- Data viz (charts, kill graphs, district leaderboards beyond simple lists)
- Toast / notification system
- Mobile-specific layouts (the v1 system is desktop-first; mobile is a follow-up)

---

## 7. Tokens (CSS Custom Properties)

The system is delivered as CSS custom properties scoped to a theme class. The existing `theme1` / `theme2` / `theme3` classes and `bg-gold-rich*` / `border-gold-rich` utilities in `web/assets/src/main.css` are removed.

```css
:root, .dark {
  /* color */
  --bg: #19121A;
  --surface: #241829;
  --surface-2: #1F1521;
  --border: #3A2440;
  --text: #F2EBE2;
  --text-muted: #A498A2;
  --primary: #00E5FF;
  --danger: #FF2E6E;
  --gold: #E8B14B;
}

.light {
  --bg: #FBF6E9;
  --surface: #FFFCF2;
  --surface-2: #F4EAC9;
  --border: #E8DCB8;
  --text: #1A1410;
  --text-muted: #6B5938;
  --primary: #007A99;
  --danger: #C8003C;
  --gold: #B8861B;
}

:root {
  /* type */
  --font-display: "Bebas Neue", Impact, sans-serif;
  --font-text:    "Source Sans 3", system-ui, sans-serif;
  --font-mono:    "IBM Plex Mono", ui-monospace, monospace;

  /* shape */
  --radius-card:   10px;
  --radius-inner:  6px;
  --radius-chrome: 0;

  /* spacing — Cinematic defaults */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 16px;
  --space-4: 24px;
  --space-5: 32px;
  --space-6: 48px;
  --space-7: 64px;
}
```

Mode is selected by adding `light` or `dark` to a high-level container (default `dark`). Persistence strategy (LocalStorage, system preference, etc.) inherits the existing theme persistence mechanism.

Tailwind v4 already drives the project; tokens should be exposed as a Tailwind theme so utilities like `bg-surface`, `text-muted`, `border` work consistently.

---

## 8. Migration Plan (high level)

The implementation plan will detail the steps. At a high level:

1. Add the new tokens to `web/assets/src/main.css` as CSS custom properties + Tailwind theme entries.
2. Wire Bebas Neue, Source Sans 3, IBM Plex Mono via Google Fonts (or self-hosted under `web/assets/`).
3. Build v1 component primitives (top bar, scoreboard, sidebar HUD, event card, tribute row, stat tile, button variants, live pill, section label, ticker) as Dioxus components in `web/src/components/`.
4. Replace existing `theme1` / `theme2` / `theme3` consumers with the new `light` / `dark` toggle. Migrate one screen at a time; the existing ad-hoc theming stays in place until the screen is migrated.
5. Remove `theme1` / `theme2` / `theme3` classes, the `bg-gold-rich*` and `border-gold-rich` utilities, and the SVG pattern background-images for the old themes once all consumers are migrated.

---

## 9. Open Questions (deferred)

These are explicitly *not* answered by v1 and will need follow-up work when the relevant feature is built:

- Map/region color treatment in the arena view
- Iconography system (style, weight, line vs filled, source)
- Motion language (transition durations, easing, reduced-motion fallbacks)
- Data viz palette (categorical hues, sequential ramps)
- Mobile breakpoint behavior for cinematic surfaces (does the scoreboard collapse, stack, or scale?)
- Accessibility audit (the chosen palette pairs all hit WCAG AA at body sizes, but specific component states need verification)
- Tribute portrait treatment (photograph? generated avatar? district crest?)
