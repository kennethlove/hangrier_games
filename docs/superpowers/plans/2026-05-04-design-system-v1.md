# Hangrier Games Design System v1 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the existing `theme1/2/3` ad-hoc theming with a cohesive two-mode (light/dark) design system, ship the v1 component primitives, and migrate every screen to consume the new tokens and components.

**Architecture:** CSS custom properties are the source of truth for color tokens, scoped under `.light` and `.dark` classes on a top-level container. Tailwind v4 is configured to expose those tokens as utilities (`bg-surface`, `text-muted`, `border-border`, etc.). New Dioxus component primitives live under `web/src/components/ui/` and consume only token-backed utilities — never raw hex or theme-specific classes. Screens are migrated one logical group at a time; the old `theme1/2/3` classes coexist during migration and are removed in the final task once nothing references them.

**Tech Stack:** Rust, Dioxus (WASM), Tailwind CSS v4, Google Fonts (Bebas Neue / Source Sans 3 / IBM Plex Mono), `gloo-storage` for LocalStorage persistence.

**Spec:** `docs/superpowers/specs/2026-05-04-design-system-v1-design.md`

---

## File Structure

### New files

- `web/src/components/ui/mod.rs` — re-exports for the primitive set
- `web/src/components/ui/button.rs` — `Button` with `variant: Primary | Ghost | Danger | Chrome`
- `web/src/components/ui/topbar.rs` — `TopBar` chrome component
- `web/src/components/ui/scoreboard.rs` — `Scoreboard` with team blocks + score
- `web/src/components/ui/sidebar_hud.rs` — `SidebarHud` + `StatTile`
- `web/src/components/ui/event_card.rs` — soft content card with kicker/headline/body/actions
- `web/src/components/ui/tribute_row.rs` — row primitive used inside list cards
- `web/src/components/ui/live_pill.rs` — small filled live/danger badge
- `web/src/components/ui/section_label.rs` — small uppercase muted label
- `web/src/components/ui/ticker.rs` — breaking-news footer ticker
- `web/src/theme.rs` — new `Theme::Light | Dark` enum + `use_theme()` hook (replaces old `Colorscheme`)
- `web/tests/ui_primitives.rs` — render-smoke tests for each primitive

### Modified files

- `web/assets/src/main.css` — add token variables, font imports; remove old `theme1/2/3` classes and `bg-gold-rich*` / `border-gold-rich` utilities
- `web/src/storage.rs` — replace `Colorscheme` with `Theme` (keep `AppState`, `use_persistent` shape)
- `web/src/components/app.rs` — wire `Theme` instead of `Colorscheme`; replace Google Fonts <Link> with the v1 trio; replace top-level `theme1/2/3` chrome with token utilities
- `web/src/components/navbar.rs` — rebuild as a consumer of `TopBar`, drop the entire `link_theme` block
- All 38 components currently referencing `theme1/2/3` — see Phase 4 migration task list

### Removed at end of Phase 4

- All `theme1:`, `theme2:`, `theme3:` Tailwind variant references in `web/src/**`
- `@variant theme1`, `@variant theme2`, `@variant theme3` blocks in `main.css`
- `@utility bg-gold-rich`, `@utility bg-gold-rich-reverse`, `@utility border-gold-rich`
- The `.theme1 { background-image: ... }`, `.theme2 { ... }`, `.theme3 { ... }` SVG-pattern blocks
- `web/assets/favicons/theme1.png`, `theme2.png`, `theme3.png` (replaced by `light.png` + `dark.png`)

---

## Phase 1 — Foundations (tokens, fonts, theme model)

### Task 1: Add design tokens to main.css

**Files:**
- Modify: `web/assets/src/main.css`

- [ ] **Step 1: Read current `main.css` end-to-end and confirm token block placement**

The new `:root` / `.dark` / `.light` blocks must come *before* the existing `@variant theme1/2/3` rules so the migration can run side-by-side without specificity surprises. Place them immediately after the `@source` lines.

- [ ] **Step 2: Insert token blocks**

Add this block after the existing `@source` lines and before `@variant theme1`:

```css
/* ---- Hangrier Games v1 design tokens ---- */
:root, .dark {
  --color-bg: #19121A;
  --color-surface: #241829;
  --color-surface-2: #1F1521;
  --color-border: #3A2440;
  --color-text: #F2EBE2;
  --color-text-muted: #A498A2;
  --color-primary: #00E5FF;
  --color-danger: #FF2E6E;
  --color-gold: #E8B14B;
}

.light {
  --color-bg: #FBF6E9;
  --color-surface: #FFFCF2;
  --color-surface-2: #F4EAC9;
  --color-border: #E8DCB8;
  --color-text: #1A1410;
  --color-text-muted: #6B5938;
  --color-primary: #007A99;
  --color-danger: #C8003C;
  --color-gold: #B8861B;
}

@theme inline {
  --color-bg: var(--color-bg);
  --color-surface: var(--color-surface);
  --color-surface-2: var(--color-surface-2);
  --color-border: var(--color-border);
  --color-text: var(--color-text);
  --color-text-muted: var(--color-text-muted);
  --color-primary: var(--color-primary);
  --color-danger: var(--color-danger);
  --color-gold: var(--color-gold);

  --font-display: "Bebas Neue", Impact, sans-serif;
  --font-text: "Source Sans 3", system-ui, sans-serif;
  --font-mono: "IBM Plex Mono", ui-monospace, monospace;

  --radius-card: 0.625rem;   /* 10px */
  --radius-inner: 0.375rem;  /* 6px */
}
```

- [ ] **Step 3: Build CSS and verify no warnings**

Run: `just build-css`
Expected: completes without warnings; `web/assets/dist/main.css` updated.

- [ ] **Step 4: Visual verification — drop test page**

Create `web/assets/dist/_token-check.html` (temporary, not committed) with:

```html
<!doctype html><html><head><link rel="stylesheet" href="main.css"></head>
<body class="dark"><div class="bg-surface text-text border border-border p-6 rounded-card font-display">DARK SCOREBOARD</div>
<div class="light bg-surface text-text border border-border p-6 rounded-card font-display">LIGHT SCOREBOARD</div>
</body></html>
```

Open in a browser; confirm dark = velvet/cream colors render, light = cream/teal render, Bebas font is requested (will fall back until Phase 1 Task 2 completes).

Delete the file after verifying.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(web): add design system v1 tokens to main.css"
```

---

### Task 2: Wire fonts in App component

**Files:**
- Modify: `web/src/components/app.rs:50-62`

- [ ] **Step 1: Replace the existing Google Fonts `document::Link` href**

Change line 60 from the current Cinzel/Work Sans/Orbitron/Playfair URL to:

```rust
href: "https://fonts.googleapis.com/css2?family=Bebas+Neue&family=Source+Sans+3:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;700&display=swap",
```

- [ ] **Step 2: Add a default body font on the root container**

In the same file, on the inner `div` currently using `font-[Work_Sans]` (around line 81), replace with `font-text`.

- [ ] **Step 3: Build and run web in dev mode**

Run: `just web`
Expected: app loads, body text is now Source Sans 3 (visibly different from Work Sans), no console font-load errors.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(web): swap to v1 font trio (Bebas/Source Sans 3/Plex Mono)"
```

---

### Task 3: Replace `Colorscheme` with `Theme`

**Files:**
- Create: `web/src/theme.rs`
- Modify: `web/src/storage.rs`
- Modify: `web/src/lib.rs` (add `pub mod theme;`)
- Modify: `web/src/components/app.rs`

- [ ] **Step 1: Write the failing test**

Create `web/tests/theme.rs`:

```rust
use web::theme::Theme;
use std::str::FromStr;

#[test]
fn theme_default_is_dark() {
    assert_eq!(Theme::default(), Theme::Dark);
}

#[test]
fn theme_display_matches_class_name() {
    assert_eq!(Theme::Dark.to_string(), "dark");
    assert_eq!(Theme::Light.to_string(), "light");
}

#[test]
fn theme_from_str_round_trips() {
    assert_eq!(Theme::from_str("dark").unwrap(), Theme::Dark);
    assert_eq!(Theme::from_str("LIGHT").unwrap(), Theme::Light);
    assert!(Theme::from_str("theme1").is_err());
}

#[test]
fn theme_toggle_flips_value() {
    assert_eq!(Theme::Dark.toggle(), Theme::Light);
    assert_eq!(Theme::Light.toggle(), Theme::Dark);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --package web --test theme`
Expected: FAIL — `web::theme` does not exist.

- [ ] **Step 3: Create `web/src/theme.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Dark => write!(f, "dark"),
            Theme::Light => write!(f, "light"),
        }
    }
}

impl FromStr for Theme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            other => Err(format!("invalid theme: {other}")),
        }
    }
}
```

- [ ] **Step 4: Add module to `web/src/lib.rs`**

Append `pub mod theme;` to the module list.

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --package web --test theme`
Expected: PASS — all four tests.

- [ ] **Step 6: Update `storage.rs` to use `Theme`**

Replace the `Colorscheme` enum, its `Display`/`FromStr` impls, and the three `switch_to_*` methods with this minimal version:

```rust
use crate::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub(crate) theme: Theme,
    pub(crate) username: Option<String>,
}

impl AppState {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}
```

Remove the `Colorscheme` enum entirely.

- [ ] **Step 7: Update `app.rs` to consume `Theme`**

Replace the `Colorscheme` import + signal block with:

```rust
use crate::theme::Theme;
// ...
let theme_signal: Signal<Theme> = use_signal(|| storage.get().theme);
use_context_provider(|| theme_signal);

let favicon = match *theme_signal.read() {
    Theme::Dark => asset!("/assets/favicons/dark.png"),
    Theme::Light => asset!("/assets/favicons/light.png"),
};
```

(Favicons created in Step 9 below — for now, copy the existing `theme2.png` to `dark.png` and `theme1.png` to `light.png` so the build doesn't fail.)

- [ ] **Step 8: Provisional favicon copy**

```bash
cp -f web/assets/favicons/theme2.png web/assets/favicons/dark.png
cp -f web/assets/favicons/theme1.png web/assets/favicons/light.png
```

- [ ] **Step 9: Verify the workspace builds**

Run: `cargo check --workspace`
Expected: PASS. (Many `theme1/2/3` callers will still compile because they only use `theme1:` Tailwind class strings — those are removed in Phase 4.)

- [ ] **Step 10: Commit**

```bash
jj commit -m "feat(web): replace Colorscheme with Theme (light/dark)"
```

---

### Task 4: Apply theme class at the app root

**Files:**
- Modify: `web/src/components/app.rs:71-97`

- [ ] **Step 1: Replace the `class: "{theme_signal.read()}"` div + nested chrome div**

The current outer `div` at line 71 has `class: "{theme_signal.read()}"` (which now emits `dark` or `light`). The inner chrome `div` (line 73) carries the old `theme1/2/3` background classes. Update to:

```rust
div {
    class: "{theme_signal.read()}",
    div {
        class: "grid min-h-screen frame transition duration-500 font-text bg-bg text-text",
        // ...children unchanged...
    }
}
```

Remove every `theme1:*`, `theme2:*`, `theme3:*` class on this div. Leave the `Router`, `footer`, `EditGameModal` children intact for now.

- [ ] **Step 2: Strip `theme1/2/3` from the footer block in the same file**

Around lines 107–131, replace the footer's `class:` strings with token-backed utilities:

```rust
class: "mt-4 pb-4 text-xs text-center text-text-muted",
// link class:
class: "underline text-primary",
```

- [ ] **Step 3: Build and run**

Run: `just build-css && just web`
Expected: Page loads. Body has the velvet dark background. The navbar will still look broken (Phase 4 fixes it), but the chrome around it should be coherent.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(web): apply v1 theme class + tokens at App root"
```

---

## Phase 2 — Component Primitives

### Task 5: `Button` primitive

**Files:**
- Create: `web/src/components/ui/mod.rs`
- Create: `web/src/components/ui/button.rs`
- Modify: `web/src/components/mod.rs` (add `pub mod ui;`)
- Create: `web/tests/ui_button.rs`

- [ ] **Step 1: Create `ui/mod.rs`**

```rust
pub mod button;
pub use button::{Button, ButtonVariant};
```

- [ ] **Step 2: Add `pub mod ui;` to `web/src/components/mod.rs`**

Append the line in alphabetical order with the other `pub mod` entries.

- [ ] **Step 3: Write the failing test**

`web/tests/ui_button.rs`:

```rust
use web::components::ui::ButtonVariant;

#[test]
fn variant_classes_are_distinct() {
    assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Ghost.classes());
    assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Danger.classes());
    assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Chrome.classes());
}

#[test]
fn chrome_variant_has_no_radius() {
    assert!(ButtonVariant::Chrome.classes().contains("rounded-none"));
}

#[test]
fn primary_variant_uses_primary_color() {
    assert!(ButtonVariant::Primary.classes().contains("bg-primary"));
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test --package web --test ui_button`
Expected: FAIL — module not found.

- [ ] **Step 5: Implement `button.rs`**

```rust
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Ghost,
    Danger,
    Chrome,
}

impl ButtonVariant {
    pub fn classes(self) -> &'static str {
        match self {
            ButtonVariant::Primary =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-primary text-bg",
            ButtonVariant::Ghost =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-transparent text-primary ring-1 ring-inset ring-primary",
            ButtonVariant::Danger =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-danger text-white",
            ButtonVariant::Chrome =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-none font-display text-sm tracking-wider cursor-pointer bg-surface-2 text-text",
        }
    }
}

#[component]
pub fn Button(
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = false)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "{variant.classes()}",
            disabled,
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test --package web --test ui_button`
Expected: PASS — all three tests.

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(web/ui): add Button primitive (Primary/Ghost/Danger/Chrome)"
```

---

### Task 6: `LivePill` and `SectionLabel` primitives

**Files:**
- Create: `web/src/components/ui/live_pill.rs`
- Create: `web/src/components/ui/section_label.rs`
- Modify: `web/src/components/ui/mod.rs`

- [ ] **Step 1: Implement `live_pill.rs`**

```rust
use dioxus::prelude::*;

#[component]
pub fn LivePill() -> Element {
    rsx! {
        span {
            class: "inline-flex items-center gap-1.5 px-2.5 py-0.5 bg-danger text-white \
                    font-text font-bold text-[10px] uppercase tracking-[0.16em] rounded-sm",
            span { class: "size-1.5 rounded-full bg-white", " " }
            "Live"
        }
    }
}
```

- [ ] **Step 2: Implement `section_label.rs`**

```rust
use dioxus::prelude::*;

#[component]
pub fn SectionLabel(children: Element) -> Element {
    rsx! {
        div {
            class: "font-text font-bold text-[10px] uppercase tracking-[0.18em] text-text-muted mb-3",
            {children}
        }
    }
}
```

- [ ] **Step 3: Re-export from `ui/mod.rs`**

Add:

```rust
pub mod live_pill;
pub mod section_label;
pub use live_pill::LivePill;
pub use section_label::SectionLabel;
```

- [ ] **Step 4: Verify build**

Run: `cargo check --package web`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(web/ui): add LivePill and SectionLabel primitives"
```

---

### Task 7: `Scoreboard` and `StatTile` primitives

**Files:**
- Create: `web/src/components/ui/scoreboard.rs`
- Create: `web/src/components/ui/sidebar_hud.rs`
- Modify: `web/src/components/ui/mod.rs`

- [ ] **Step 1: Implement `scoreboard.rs`**

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ScoreboardProps {
    pub home_label: String,
    pub home_meta: String,
    pub home_shield: String,
    pub away_label: String,
    pub away_meta: String,
    pub away_shield: String,
    pub score: String,
}

#[component]
pub fn Scoreboard(props: ScoreboardProps) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-[1fr_auto_1fr] items-center gap-6 px-10 py-8 \
                    bg-surface border-b border-border rounded-none",
            // home
            div {
                class: "flex items-center gap-3.5",
                Shield { code: props.home_shield.clone() }
                div {
                    div { class: "font-display text-xl tracking-wide", "{props.home_label}" }
                    div { class: "font-mono text-[10px] uppercase tracking-wider text-text-muted",
                        "{props.home_meta}"
                    }
                }
            }
            // score
            div {
                class: "font-mono font-bold text-6xl tracking-wider text-primary leading-none",
                "{props.score}"
            }
            // away (right-aligned)
            div {
                class: "flex items-center justify-end gap-3.5 opacity-75",
                div {
                    div { class: "font-display text-xl tracking-wide text-right",
                        "{props.away_label}"
                    }
                    div { class: "font-mono text-[10px] uppercase tracking-wider text-text-muted text-right",
                        "{props.away_meta}"
                    }
                }
                Shield { code: props.away_shield.clone() }
            }
        }
    }
}

#[component]
fn Shield(code: String) -> Element {
    rsx! {
        div {
            class: "size-10 bg-surface-2 flex items-center justify-center font-display text-lg",
            "{code}"
        }
    }
}
```

- [ ] **Step 2: Implement `sidebar_hud.rs`**

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct StatTileProps {
    pub label: String,
    pub value: String,
    #[props(default = "text-text".to_string())]
    pub value_class: String,
}

#[component]
pub fn StatTile(props: StatTileProps) -> Element {
    rsx! {
        div {
            class: "p-4 border-r border-b border-border last:border-r-0",
            div { class: "font-text font-bold text-[9px] uppercase tracking-[0.16em] text-text-muted mb-1.5",
                "{props.label}"
            }
            div { class: "font-mono font-bold text-2xl tracking-wider {props.value_class}",
                "{props.value}"
            }
        }
    }
}

#[component]
pub fn SidebarHud(header: String, children: Element) -> Element {
    rsx! {
        div {
            class: "bg-surface-2 border border-border rounded-none",
            div {
                class: "px-4 py-3 border-b border-border font-display text-base tracking-wider",
                "{header}"
            }
            div { class: "grid grid-cols-2", {children} }
        }
    }
}
```

- [ ] **Step 3: Re-export from `ui/mod.rs`**

```rust
pub mod scoreboard;
pub mod sidebar_hud;
pub use scoreboard::Scoreboard;
pub use sidebar_hud::{SidebarHud, StatTile};
```

- [ ] **Step 4: Build verification**

Run: `cargo check --package web`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(web/ui): add Scoreboard, SidebarHud, StatTile primitives"
```

---

### Task 8: `EventCard`, `TributeRow`, `Ticker`, `TopBar` primitives

**Files:**
- Create: `web/src/components/ui/event_card.rs`
- Create: `web/src/components/ui/tribute_row.rs`
- Create: `web/src/components/ui/ticker.rs`
- Create: `web/src/components/ui/topbar.rs`
- Modify: `web/src/components/ui/mod.rs`

- [ ] **Step 1: Implement `event_card.rs`**

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EventCardProps {
    pub kicker: String,
    pub headline: String,
    #[props(default)]
    pub body: Option<String>,
    #[props(default)]
    pub actions: Element,
}

#[component]
pub fn EventCard(props: EventCardProps) -> Element {
    rsx! {
        article {
            class: "bg-surface border border-border rounded-card p-6 mb-4",
            div {
                class: "font-text font-bold text-[10px] uppercase tracking-[0.18em] text-primary mb-2",
                "{props.kicker}"
            }
            h2 {
                class: "font-display text-3xl uppercase leading-none mb-3",
                "{props.headline}"
            }
            if let Some(body) = props.body {
                p { class: "font-text text-sm leading-relaxed text-text/85", "{body}" }
            }
            if !props.actions.as_ref().is_none_or(|_| false) {
                div { class: "flex flex-wrap gap-2 mt-4", {props.actions} }
            }
        }
    }
}
```

(Note: the `actions` slot is rendered unconditionally if provided; the wrapping condition above is illustrative — the simplest correct version is to always render the wrapper and let an empty `actions` produce no children.)

- [ ] **Step 2: Implement `tribute_row.rs`**

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TributeRowProps {
    pub district_code: String,
    pub name: String,
    pub meta: String,
    pub stat: String,
    #[props(default = false)]
    pub highlight: bool,
}

#[component]
pub fn TributeRow(props: TributeRowProps) -> Element {
    let stat_color = if props.highlight { "text-primary" } else { "text-text" };
    rsx! {
        div {
            class: "flex items-center gap-3.5 py-2.5 border-t border-border first:border-t-0",
            div {
                class: "size-9 rounded-full bg-surface-2 flex items-center justify-center font-display text-sm shrink-0",
                "{props.district_code}"
            }
            div {
                div { class: "font-text font-semibold text-sm", "{props.name}" }
                div { class: "font-mono text-xs text-text-muted", "{props.meta}" }
            }
            div { class: "ml-auto font-mono font-bold text-sm {stat_color}", "{props.stat}" }
        }
    }
}
```

- [ ] **Step 3: Implement `ticker.rs`**

```rust
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TickerItem { pub kind: String, pub message: String }

#[component]
pub fn Ticker(items: Vec<TickerItem>) -> Element {
    rsx! {
        div {
            class: "px-6 py-3.5 bg-surface border-t-2 border-border font-mono text-xs text-text/70 \
                    flex gap-8 overflow-x-auto",
            for item in items.iter() {
                span {
                    span { class: "text-primary font-bold mr-2", "{item.kind}" }
                    "{item.message}"
                }
            }
        }
    }
}
```

- [ ] **Step 4: Implement `topbar.rs`**

```rust
use dioxus::prelude::*;

#[component]
pub fn TopBar(brand: String, children: Element) -> Element {
    rsx! {
        header {
            class: "flex items-center justify-between px-6 py-3.5 \
                    bg-surface border-b-2 border-border",
            div {
                class: "flex items-center gap-6",
                div { class: "font-display text-xl tracking-wider", "{brand}" }
                {children}
            }
        }
    }
}
```

- [ ] **Step 5: Re-export from `ui/mod.rs`**

```rust
pub mod event_card;
pub mod ticker;
pub mod topbar;
pub mod tribute_row;
pub use event_card::EventCard;
pub use ticker::{Ticker, TickerItem};
pub use topbar::TopBar;
pub use tribute_row::TributeRow;
```

- [ ] **Step 6: Build verification**

Run: `cargo check --package web`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(web/ui): add EventCard, TributeRow, Ticker, TopBar primitives"
```

---

## Phase 3 — Theme switcher

### Task 9: Rebuild Navbar around `TopBar` + light/dark toggle

**Files:**
- Modify: `web/src/components/navbar.rs`

- [ ] **Step 1: Replace the entire file with this implementation**

```rust
use crate::components::ui::{Button, ButtonVariant, TopBar};
use crate::routes::Routes;
use crate::storage::{AppState, use_persistent};
use crate::theme::Theme;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);
    let mut theme_signal: Signal<Theme> = use_context();
    use_context_provider(|| Signal::new(crate::components::timeline::PeriodFilters::default()));

    let toggle_theme = move |_| {
        let next = theme_signal.read().toggle();
        theme_signal.set(next);
        let mut state = storage.get();
        state.set_theme(next);
        storage.set(state);
    };

    let signed_in = storage.get().username.is_some();
    let label = match *theme_signal.read() {
        Theme::Dark => "☀ Light",
        Theme::Light => "☾ Dark",
    };

    rsx! {
        TopBar { brand: "HANGRIER GAMES".to_string(),
            nav {
                aria_label: "Main navigation",
                class: "flex items-center gap-6 font-text font-bold text-[11px] uppercase tracking-[0.16em] text-text-muted",
                Link { class: "hover:text-text", to: Routes::Home {}, "Home" }
                if signed_in {
                    Link { class: "hover:text-text", to: Routes::GamesList {}, "Games" }
                }
                Link { class: "hover:text-text", to: Routes::AccountsPage {}, "Account" }
            }
            div { class: "ml-auto",
                Button { variant: ButtonVariant::Ghost, onclick: toggle_theme, "{label}" }
            }
        }
        main {
            class: "mx-auto max-w-3xl sm:max-w-3/4 py-6 px-2",
            Outlet::<Routes> {}
        }
    }
}
```

- [ ] **Step 2: Run the app**

Run: `just web`
Expected: Top bar renders with the new design. Clicking the theme toggle flips the page between dark velvet and light cream. Persistence works across reloads.

- [ ] **Step 3: Manual smoke check**

Verify:
- The active route does not yet have an underline indicator (deferred to a polish pass)
- Navigation between Home / Games / Account works
- The toggle button label flips after each click

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(web): rebuild Navbar around TopBar + light/dark toggle"
```

---

## Phase 4 — Screen Migration

The following components currently reference `theme1` / `theme2` / `theme3` classes. Each task migrates one logical group: replace every `theme1:*`, `theme2:*`, `theme3:*` Tailwind class with token-backed utilities (`bg-bg`, `bg-surface`, `bg-surface-2`, `text-text`, `text-text-muted`, `text-primary`, `text-danger`, `border-border`, `font-display`, `font-text`, `font-mono`, `rounded-card`, `rounded-inner`). Replace bespoke per-screen layouts with primitives from `web/src/components/ui/` wherever the existing markup matches a primitive.

**Migration ground rules — apply in every task in this phase:**

1. Read the file end-to-end before editing.
2. Replace the per-theme variants with the equivalent token utility. The mapping is:
   - any `theme*:bg-…` → `bg-surface` (cards), `bg-bg` (page), `bg-surface-2` (chrome inset)
   - any `theme*:text-…` (body) → `text-text` or `text-text-muted`
   - any `theme*:text-amber-*|yellow-*|teal-*|green-*` used as accent → `text-primary` or `text-gold` (only for ceremonial)
   - any `theme*:border-*` → `border-border` (or `border-primary` if it was the active state)
   - any `theme*:font-[Cinzel|Playfair_Display|Orbitron]` → `font-display`
   - any `theme*:font-[Work_Sans]` → `font-text` (usually deletable — it's the body default now)
   - any `theme*:bg-gold-rich`, `theme*:bg-gold-rich-reverse` → delete (no replacement; gold is reserved)
   - any `theme*:border-gold-rich` → `border-gold` (for ceremonial only) or `border-border`
3. If a component is rebuilding chrome that a primitive already provides (a stat tile, a card, a button), replace with the primitive.
4. Remove every `theme1:`, `theme2:`, `theme3:` literal — when the task is complete, `grep -n 'theme[123]:' <file>` must return zero matches.
5. After each task: `cargo check --package web` must pass.

### Task 10: Migrate `home.rs` and `games_list.rs`

**Files:**
- Modify: `web/src/components/home.rs`
- Modify: `web/src/components/games_list.rs`

- [ ] **Step 1: Migrate `home.rs`** following the migration ground rules above.
- [ ] **Step 2: Migrate `games_list.rs`** following the migration ground rules above.
- [ ] **Step 3: Verify**: `grep -n 'theme[123]:' web/src/components/home.rs web/src/components/games_list.rs` — no output.
- [ ] **Step 4: Build**: `cargo check --package web` — PASS.
- [ ] **Step 5: Visual smoke check**: load `/` and `/games` in both modes; layouts intact, no leftover gold gradients.
- [ ] **Step 6: Commit**: `jj commit -m "refactor(web): migrate Home and GamesList to v1 tokens"`

### Task 11: Migrate `accounts.rs`, `credits.rs`, `info_detail.rs`

**Files:**
- Modify: `web/src/components/accounts.rs`
- Modify: `web/src/components/credits.rs`
- Modify: `web/src/components/info_detail.rs`

- [ ] **Step 1**: Migrate each file following the ground rules.
- [ ] **Step 2**: `grep -n 'theme[123]:' web/src/components/{accounts,credits,info_detail}.rs` — no output.
- [ ] **Step 3**: `cargo check --package web` — PASS.
- [ ] **Step 4**: Smoke check `/account` and `/credits` in both modes.
- [ ] **Step 5**: Commit: `jj commit -m "refactor(web): migrate Accounts, Credits, InfoDetail to v1 tokens"`

### Task 12: Migrate game-detail screens

**Files:**
- Modify: `web/src/components/game_detail.rs`
- Modify: `web/src/components/game_areas.rs`
- Modify: `web/src/components/game_tributes.rs`
- Modify: `web/src/components/game_period_page.rs`
- Modify: `web/src/components/period_grid.rs`
- Modify: `web/src/components/period_grid_empty.rs`
- Modify: `web/src/components/period_card.rs`
- Modify: `web/src/components/recap_card.rs`

This task is the largest; the game-detail surface is the most heavily themed area.

- [ ] **Step 1**: Migrate each file following the ground rules. Where a layout matches a primitive (header band, stat block, story card), replace with `TopBar`/`SidebarHud`/`EventCard`/`StatTile`/`Scoreboard`/`SectionLabel`.
- [ ] **Step 2**: `grep -nR 'theme[123]:' web/src/components/{game_detail,game_areas,game_tributes,game_period_page,period_grid,period_grid_empty,period_card,recap_card}.rs` — no output.
- [ ] **Step 3**: `cargo check --package web` — PASS.
- [ ] **Step 4**: Smoke check `/games/<id>` in both modes — open a game with at least one period of data.
- [ ] **Step 5**: Commit: `jj commit -m "refactor(web): migrate game detail screens to v1 tokens"`

### Task 13: Migrate tribute and item detail screens

**Files:**
- Modify: `web/src/components/tribute_detail.rs`
- Modify: `web/src/components/tribute_edit.rs`
- Modify: `web/src/components/tribute_delete.rs`
- Modify: `web/src/components/tribute_filter_chips.rs`
- Modify: `web/src/components/tribute_state_strip.rs`
- Modify: `web/src/components/tribute_status_icon.rs`
- Modify: `web/src/components/tribute_survival_section.rs`
- Modify: `web/src/components/item_detail.rs`
- Modify: `web/src/components/item_icon.rs`
- Modify: `web/src/components/area_detail.rs`

- [ ] **Step 1**: Migrate each file following the ground rules. Replace the existing avatar markup in `tribute_detail.rs` with the `TributeRow` primitive's styling rules; replace the survival-section grid with `StatTile`s.
- [ ] **Step 2**: `grep -nR 'theme[123]:' <these files>` — no output.
- [ ] **Step 3**: `cargo check --package web` — PASS.
- [ ] **Step 4**: Smoke check tribute detail, tribute edit modal, item detail, area detail in both modes.
- [ ] **Step 5**: Commit: `jj commit -m "refactor(web): migrate tribute/item/area screens to v1 tokens"`

### Task 14: Migrate game create / edit / delete and remaining screens

**Files:**
- Modify: `web/src/components/create_game.rs`
- Modify: `web/src/components/game_edit.rs`
- Modify: `web/src/components/game_delete.rs`
- Modify: `web/src/components/games.rs`
- Modify: `web/src/components/icons_page.rs`
- Modify: `web/src/components/map.rs`
- Modify: `web/src/components/map_affordance_overlay.rs`
- Modify: `web/src/components/modal.rs`
- Modify: `web/src/components/loading_modal.rs`
- Modify: `web/src/components/icons/loading.rs`
- Modify: `web/src/components/input.rs`
- Modify: `web/src/components/button.rs` (note: this is the *legacy* button file — see Step 2)
- Modify: `web/src/components/filter_chips.rs`
- Modify: `web/src/components/server_version.rs`

- [ ] **Step 1**: Migrate each file following the ground rules.
- [ ] **Step 2**: For `web/src/components/button.rs`: this is a pre-existing button helper, distinct from `web/src/components/ui/button.rs`. Migrate its theme classes to tokens but leave the API alone — it can be replaced by the new `Button` in a follow-up. Add a `// TODO(beads): replace with components::ui::Button` comment at the top.
- [ ] **Step 3**: `grep -nR 'theme[123]:' web/src/components` — no output.
- [ ] **Step 4**: `cargo check --package web` — PASS.
- [ ] **Step 5**: Smoke check game creation, editing, deletion, the icons page, and the map view in both modes.
- [ ] **Step 6**: Commit: `jj commit -m "refactor(web): migrate remaining screens to v1 tokens"`

### Task 15: Migrate `timeline/` cards

**Files:**
- Modify: `web/src/components/timeline/event_card.rs`
- Modify: `web/src/components/timeline/timeline.rs`
- Modify: `web/src/components/timeline/filters.rs`
- Modify: every file under `web/src/components/timeline/cards/` that matches `grep -l 'theme[123]:' web/src/components/timeline/cards/*.rs`

- [ ] **Step 1**: Identify timeline-card files needing migration: `grep -l 'theme[123]:' web/src/components/timeline/**/*.rs`
- [ ] **Step 2**: Migrate each one following the ground rules. Where a card matches the `EventCard` primitive shape, refactor to use it.
- [ ] **Step 3**: `grep -nR 'theme[123]:' web/src/components/timeline` — no output.
- [ ] **Step 4**: `cargo check --package web` — PASS.
- [ ] **Step 5**: Smoke check the timeline view on a game with rich event data in both modes.
- [ ] **Step 6**: Commit: `jj commit -m "refactor(web): migrate timeline cards to v1 tokens"`

---

## Phase 5 — Removal

### Task 16: Strip the legacy theme system from CSS

**Files:**
- Modify: `web/assets/src/main.css`
- Delete: `web/assets/favicons/theme1.png`, `theme2.png`, `theme3.png`
- Delete: `web/src/components/codemap.md` legacy theme references (if any)

- [ ] **Step 1: Workspace-wide check**

Run: `grep -rn 'theme[123]:' web/src 2>&1 || echo "CLEAN"`
Expected: `CLEAN`. If any file still references the old variants, return to Phase 4 and migrate it before continuing.

- [ ] **Step 2: Remove legacy blocks from `main.css`**

Delete:
- The three `@variant theme1`, `@variant theme2`, `@variant theme3` lines
- `@utility bg-gold-rich`, `@utility bg-gold-rich-reverse`, `@utility border-gold-rich`
- The three `.theme1 { background-image: ... }`, `.theme2 { ... }`, `.theme3 { ... }` SVG-pattern blocks
- The `@property --angle`, `@keyframes rotate-border-angle`, `@utility border-tracer` blocks (no consumers remain after migration; verify with `grep -rn 'border-tracer' web/src` — if any matches, defer this deletion and file a beads follow-up)

Keep:
- The `@import "tailwindcss"` line
- The `@source` lines
- The new v1 token blocks added in Phase 1 Task 1
- The `.frame { grid-template-rows: auto min-content; }` rule
- The `@keyframes spinner` and `.spinner { ... }` block

- [ ] **Step 3: Replace stale favicons**

If proper light/dark favicons are not yet designed, the provisional copies from Phase 1 Task 3 Step 8 stay in place. Delete the old files:

```bash
rm -f web/assets/favicons/theme1.png web/assets/favicons/theme2.png web/assets/favicons/theme3.png
```

Open a beads issue: "Design proper light/dark favicons for design system v1." Reference its ID in the commit message.

- [ ] **Step 4: Final build + smoke**

Run: `just build-css && cargo check --package web && just web`
Expected: clean build; switching theme in the navbar works; every screen renders without leftover gold/Cinzel/Playfair artifacts.

- [ ] **Step 5: Commit**

```bash
jj commit -m "chore(web): remove legacy theme1/2/3 system (closes <beads-id>)"
```

---

## Phase 6 — Smoke verification

### Task 17: End-to-end visual sweep

**Files:** none modified

- [ ] **Step 1**: Start the stack: `just dev`.
- [ ] **Step 2**: For each route, view in **dark** then toggle to **light** and confirm the entire screen reads coherently:
  - `/` (Home)
  - `/games` (Games list)
  - `/games/<id>` (Game detail with periods)
  - `/games/<id>/tributes/<id>` (Tribute detail)
  - `/games/<id>/areas/<id>` (Area detail)
  - `/games/<id>/items/<id>` (Item detail)
  - `/account` (Account)
  - `/credits` (Credits)
  - `/icons` (Icons reference page)
- [ ] **Step 3**: For any screen that feels broken, file a beads issue with a short description and screenshot. Do not fix in this pass — Phase 7 follow-ups handle polish.
- [ ] **Step 4**: Run `just quality` (format, check, clippy, test) and resolve any failures.
- [ ] **Step 5**: Commit any quality-pass fixes: `jj commit -m "chore(web): post-migration cleanup"`.

---

## Phase 7 — Follow-ups (filed as separate beads issues, NOT part of this plan)

The following items from the spec's "Out of scope for v1" / "Open Questions" sections should be filed as beads issues at the end of Task 17 and tracked independently:

- Arena map / region color treatment
- Iconography system definition
- Motion language (transition durations, easing, reduced-motion)
- Data viz palette
- Mobile-specific layouts for cinematic surfaces
- Accessibility audit (contrast on all component states)
- Tribute portrait treatment
- Replace legacy `web/src/components/button.rs` with `web/src/components/ui/button.rs` callsite-by-callsite
- Active-route underline indicator in the new `Navbar`
- Proper light/dark favicon design

---

## Self-Review

**Spec coverage:**
- Tone & intent → captured implicitly by the chosen tokens and primitives. ✓
- Color (dark/light tokens, semantics) → Tasks 1, 4, 9. ✓
- Typography (3 roles, ALL CAPS rules, mono numerals) → Tasks 2, 5–8 use `font-display`/`font-text`/`font-mono` per the rules. ✓
- Mixed shape language → Tasks 5–8 codify hard-chrome (`rounded-none`, `border-b-2`) and soft-content (`rounded-card`) per primitive. ✓
- Density (Cinematic default, Compact for data) → Tasks 7–8 use `px-10 py-8` for hero, `py-2.5` for rows. ✓
- Component inventory → Tasks 5–8 implement every v1 component. ✓
- Tokens (CSS custom properties) → Task 1. ✓
- Migration plan → Tasks 4, 9, 10–15, 16. ✓

**Placeholder scan:** No "TBD"/"implement later"/"appropriate handling" entries. Migration ground rules are explicit and verifiable via `grep`. The `EventCard` `actions` slot has an inline note explaining the implementation intent.

**Type consistency:**
- `Theme` enum used consistently across `theme.rs`, `storage.rs`, `app.rs`, `navbar.rs`. ✓
- `ButtonVariant::{Primary, Ghost, Danger, Chrome}` consistent across primitives + tests. ✓
- `set_theme` method on `AppState` matches the call in Navbar. ✓
- `Scoreboard` props `home_label/away_label/home_meta/away_meta/home_shield/away_shield/score` consistent with rendering. ✓
- `StatTile` props `label/value/value_class` consistent. ✓
- `TickerItem.kind` / `.message` consistent between primitive + render. ✓
- `TopBar.brand` is a `String` and is interpolated as `"{props.brand}"` — consistent. ✓
