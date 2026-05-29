use maud::html;
use shared::afflictions::AfflictionKind;
use shared::{DisplayGame, GameStatus};

use super::pages::status_color;
use super::{AuthState, base_layout, icon, narrative_icon};

/// Full game detail page with navigation tabs.
pub fn game_detail_page(auth: AuthState, game: &DisplayGame) -> maud::Markup {
    base_layout(
        &game.name,
        auth,
        html! {
            div hx-ext="sse" sse-connect=(format!("/api/games/{}/events", game.identifier)) {
                // Header section
                div class="detail-header" {
                    div {
                        h1 { (game.name) }
                        p class="detail-meta" {
                            "Day " (game.day.unwrap_or(0))
                            " · "
                            span class=(status_color(&game.status.to_string())) { (game.status) }
                            " · "
                            span id="living-count" class="num" { (game.living_count) } "/" (game.tribute_count) " alive"
                        }
                    }
                    @if game.is_mine {
                        div class="detail-actions" {
                            @if game.status == GameStatus::NotStarted {
                                button
                                    class="btn btn-primary btn-sm"
                                    hx-put=(format!("/api/games/{}/next", game.identifier))
                                    hx-target="#game-status"
                                    hx-swap="innerHTML"
                                    hx-disabled-elt="this"
                                    hx-indicator="#start-spinner" {
                                    "Start Game"
                                    span id="start-spinner" class="htmx-indicator" {
                                        (spinner_icon())
                                    }
                                }
                            } @else if game.status == GameStatus::InProgress {
                                button
                                    class="btn btn-ghost btn-sm"
                                    hx-put=(format!("/api/games/{}/next", game.identifier))
                                    hx-target="#game-status"
                                    hx-swap="innerHTML"
                                    hx-disabled-elt="this"
                                    hx-indicator="#play-spinner" {
                                    "Play Day " (game.day.unwrap_or(0))
                                    span id="play-spinner" class="htmx-indicator" {
                                        (spinner_icon())
                                    }
                                }
                            }
                        }
                    } @else {
                        p class="detail-meta" {
                            "By " (game.created_by.username)
                        }
                    }
                }

                // Status update target for HTMX
                div id="game-status" {
                    @if game.status == GameStatus::Finished {
                        @if let Some(winner) = &game.winner {
                            div class="winner-banner" {
                                (icon("trophy"))
                                span { "Winner: " (winner.name) }
                            }
                        }
                    }
                }

                // Navigation tabs
                nav class="detail-tabs" {
                    a href=(format!("/games/{}/tributes", game.identifier))
                        class="detail-tab" {
                        "Tributes"
                    }
                    a href=(format!("/games/{}/areas", game.identifier))
                        class="detail-tab" {
                        "Areas"
                    }
                    a href=(format!("/games/{}/log", game.identifier))
                        class="detail-tab" {
                        "Log"
                    }
                }

                // Content area — sub-pages load here via HTMX
                div id="detail-content" {
                    p class="empty-state" { "Select a tab above" }
                }
            }
        },
    )
}

/// Tribute list page — grid grouped by district.
pub fn tributes_page(
    auth: AuthState,
    game_id: &str,
    tributes: &[game::tributes::Tribute],
) -> maud::Markup {
    base_layout(
        "Tributes",
        auth,
        html! {
            div class="container" style="padding-block:var(--gap-lg);" {
                a href=(format!("/games/{}", game_id))
                    class="back-link" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 style="font-family:var(--font-display);font-size:var(--fs-h3);font-weight:600;margin:0 0 var(--gap-md);" { "Tributes" }

                @if tributes.is_empty() {
                    p class="empty-state" { "No tributes yet." }
                } @else {
                    // Group by district
                    @for district_num in 1..=12 {
                        @let district_tributes: Vec<_> = tributes.iter()
                            .filter(|t| t.district == district_num)
                            .collect();
                        @if !district_tributes.is_empty() {
                            div {
                                h3 class="section-header" {
                                    "District " (district_num)
                                }
                                div class="card-grid" {
                                    @for tribute in &district_tributes {
                                        (tribute_card(tribute))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// Format a trauma source for display in tribute card.
fn format_trauma_source(source: &shared::afflictions::TraumaSource) -> String {
    match source {
        shared::afflictions::TraumaSource::WitnessedAllyDeath { ally, cause: _ } => {
            format!("witnessed {ally}'s death")
        }
        shared::afflictions::TraumaSource::NearDeath { cause: _ } => "near-death".to_string(),
        shared::afflictions::TraumaSource::Betrayal { by } => {
            format!("betrayed by {by}")
        }
        shared::afflictions::TraumaSource::MassCasualty {
            cause_class: _,
            deaths_this_cycle,
        } => format!("mass casualty (\u{d7}{deaths_this_cycle})"),
    }
}

/// Single tribute card with stats strip.
fn tribute_card(tribute: &game::tributes::Tribute) -> maud::Markup {
    let is_alive = tribute.is_alive();
    let status_class = if is_alive { "alive" } else { "dead" };
    let status_icon = if is_alive { "check-circle" } else { "x-circle" };

    html! {
        div class="tribute-card" {
            div class="card-top" {
                span class="card-name" { (tribute.name) }
                span class=(format!("card-status {}", status_class)) { (icon(status_icon)) }
            }

            // Stats strip
            div class="card-stats" {
                // Health
                div {
                    (icon("heart"))
                    " HP "
                    span class="stat-val" { (tribute.attributes.health) }
                }
                // Sanity
                div {
                    (icon("brain"))
                    " SAN "
                    span class="stat-val" { (tribute.attributes.sanity) }
                }
                // Movement
                div {
                    (icon("zap"))
                    " MOV "
                    span class="stat-val" { (tribute.attributes.movement) }
                }
                // Strength
                div {
                    (icon("sword"))
                    " STR "
                    span class="stat-val" { (tribute.attributes.strength) }
                }
            }

            // Afflictions
            @if !tribute.afflictions.is_empty() {
                div class="card-afflictions" {
                    @for (_key, affliction) in &tribute.afflictions {
                        @if !matches!(affliction.kind, AfflictionKind::Addiction(_)) {
                            @let severity_class = match affliction.severity.to_string().as_str() {
                                "severe" => "severity-severe",
                                "moderate" => "severity-moderate",
                                _ => "severity-mild",
                            };
                            @let body_part = affliction.body_part.map(|bp| format!(" ({bp})")).unwrap_or_default();
                            span class=(format!("affliction-badge {}", severity_class)) {
                                (icon("bandage"))
                                " " (affliction.kind) (body_part)
                            }
                        }
                    }
                }
            }

            // Addiction state indicators
            @let addictions: Vec<_> = tribute.afflictions.values()
                .filter(|a| matches!(a.kind, AfflictionKind::Addiction(_)))
                .collect();
            @if !addictions.is_empty() {
                div class="card-addiction-state" {
                    @for affliction in &addictions {
                        @let substance = match &affliction.kind {
                            AfflictionKind::Addiction(s) => s,
                            _ => unreachable!(),
                        };
                        @if let Some(meta) = &affliction.addiction_metadata {
                            @if meta.high_cycles_remaining > 0 {
                                span class="addiction-state-high" {
                                    (icon(substance.icon_name()))
                                    " HIGH"
                                }
                            } @else {
                                @if affliction.severity >= shared::afflictions::Severity::Moderate {
                                    span class="addiction-state-withdrawal" {
                                        (icon("withdrawal"))
                                        " WITHDRAWAL"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Fears
            @let phobias: Vec<_> = tribute.afflictions.values()
                .filter(|a| matches!(a.kind, shared::afflictions::AfflictionKind::Phobia(_)))
                .collect();
            @if !phobias.is_empty() {
                div class="card-fears" {
                    span class="fears-header" { (icon("eye")) " Fears" }
                    @for affliction in &phobias {
                        @let trigger = match &affliction.kind {
                            shared::afflictions::AfflictionKind::Phobia(t) => t.to_string(),
                            _ => unreachable!(),
                        };
                        @let severity_class = match affliction.severity.to_string().as_str() {
                            "severe" => "severity-severe",
                            "moderate" => "severity-moderate",
                            _ => "severity-mild",
                        };
                        @let origin_icon = match &affliction.phobia_metadata {
                            Some(m) => match m.origin {
                                shared::afflictions::PhobiaOrigin::Innate => "dna",
                                shared::afflictions::PhobiaOrigin::Traumatic { event_ref: _ } => "zap",
                            },
                            None => "help-circle",
                        };
                        span class=(format!("affliction-badge {}", severity_class)) {
                            (icon(origin_icon))
                            " " (trigger)
                        }
                    }
                }
            }

            // Trauma
            @let traumas: Vec<_> = tribute.afflictions.values()
                .filter(|a| matches!(a.kind, shared::afflictions::AfflictionKind::Trauma))
                .collect();
            @if !traumas.is_empty() {
                div class="card-trauma" {
                    span class="trauma-header" { (icon("brain")) " Trauma" }
                    @for affliction in &traumas {
                        @let severity_class = match affliction.severity.to_string().as_str() {
                            "severe" => "severity-severe",
                            "moderate" => "severity-moderate",
                            _ => "severity-mild",
                        };
                        @let source_text = affliction.trauma_metadata.as_ref()
                            .and_then(|m| m.sources.iter().next().map(format_trauma_source))
                            .unwrap_or_default();
                        span class=(format!("affliction-badge {}", severity_class)) {
                            (icon("brain"))
                            " " (affliction.severity)
                            @if !source_text.is_empty() {
                                " \u{2014} " (source_text)
                            }
                        }
                        @if let Some(meta) = &affliction.trauma_metadata {
                            @if !meta.observed_by.is_empty() {
                                span class="trauma-observers" {
                                    (icon("eye")) " " (meta.observed_by.len()) " observer(s)"
                                }
                            }
                        }
                    }
                }
            }

            // Survival bands
            div class="card-bands" {
                span class=(hunger_color(tribute.hunger)) { "H: " (hunger_label(tribute.hunger)) }
                span class=(thirst_color(tribute.thirst)) { "T: " (thirst_label(tribute.thirst)) }
                span class=(stamina_color(tribute.stamina, tribute.max_stamina)) {
                    "S: " (stamina_label(tribute.stamina, tribute.max_stamina))
                }
            }

            // Area location
            div class="card-location" {
                (icon("map-pin"))
                " " (tribute.area)
            }

            // Inventory
            @if !tribute.items.is_empty() {
                div class="card-items" {
                    @for item in &tribute.items {
                        span class="item-tag" {
                            (icon("backpack"))
                            " " (item.name)
                        }
                    }
                }
            }
        }
    }
}

/// Areas list page.
pub fn areas_page(
    auth: AuthState,
    game_id: &str,
    areas: &[game::areas::AreaDetails],
) -> maud::Markup {
    base_layout(
        "Areas",
        auth,
        html! {
            div class="container" style="padding-block:var(--gap-lg);" {
                a href=(format!("/games/{}", game_id))
                    class="back-link" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 style="font-family:var(--font-display);font-size:var(--fs-h3);font-weight:600;margin:0 0 var(--gap-md);" { "Areas" }

                @if areas.is_empty() {
                    p class="empty-state" { "No areas yet." }
                } @else {
                    div class="card-grid" {
                        @for area in areas {
                            (area_card(area))
                        }
                    }
                }
            }
        },
    )
}

/// Single area card.
fn area_card(area: &game::areas::AreaDetails) -> maud::Markup {
    let is_open = area.events.is_empty();
    let status_text = if is_open { "Open" } else { "Active Events" };
    let status_class = if is_open { "open" } else { "active" };

    html! {
        div class="area-card" {
            div class="card-top" {
                h3 class="card-name" {
                    (icon("map-pin"))
                    " " (area.name)
                }
                span class=(format!("card-status {}", status_class)) {
                    @if is_open {
                        (icon("unlock"))
                    } @else {
                        (icon("triangle-alert"))
                    }
                    " " (status_text)
                }
            }

            // Items
            @if !area.items.is_empty() {
                div class="card-items" {
                    @for item in &area.items {
                        span class="item-tag" {
                            (icon("backpack"))
                            " " (item.name)
                        }
                    }
                }
            }

            // Events
            @if !area.events.is_empty() {
                div class="card-events" {
                    p style="font-weight:600;margin-bottom:4px;" { "Events:" }
                    ul style="list-style:none;padding:0;" {
                        @for event in &area.events {
                            li { (narrative_icon("event")) " " (event) }
                        }
                    }
                }
            }
        }
    }
}

/// Game log page — scrollable message list with SSE real-time updates.
pub fn log_page(
    auth: AuthState,
    game_id: &str,
    messages: &[shared::messages::GameMessage],
) -> maud::Markup {
    // All MessagePayload variant names for SSE event filtering
    let sse_events = "TributeKilled,TributeWounded,TributeAttacked,Combat,CombatSwing,\
        AllianceFormed,AllianceProposed,AllianceDissolved,BetrayalTriggered,TrustShockBreak,\
        TributeMoved,TributeHidden,AreaClosed,AreaEvent,\
        ItemFound,ItemUsed,ItemDropped,SponsorGift,\
        TributeRested,TributeStarved,TributeDehydrated,SanityBreak,\
        HungerBandChanged,ThirstBandChanged,StaminaBandChanged,\
        ShelterSought,Foraged,Drank,Ate,\
        CycleStart,CycleEnd,PhaseStarted,PhaseEnded,\
        TributeSlept,TributeWoke,GameEnded,\
        AfflictionAcquired,AfflictionProgressed,AfflictionHealed,AfflictionCascaded,\
        TraumaAcquired,TraumaReinforced,TraumaEscalated,TraumaFlashback,TraumaAvoidance,TraumaObserved,TraumaForgotten,TraumaHabituated,\
        PhobiaAcquired,PhobiaTriggered,        FixationAcquired,FixationEscalated,FixationFired,FixationConsummated,FixationThwarted,FixationFaded,\
        SubstanceUsed,AddictionAcquired,AddictionReinforced,AddictionEscalated,AddictionResisted,AddictionRelapse,\
        AddictionCraving,AddictionObserved,AddictionForgotten,AddictionHabituated";

    base_layout(
        "Log",
        auth,
        html! {
            div class="container" style="padding-block:var(--gap-lg);" hx-ext="sse" sse-connect=(format!("/api/games/{}/events", game_id)) {
                a href=(format!("/games/{}", game_id))
                    class="back-link" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 style="font-family:var(--font-display);font-size:var(--fs-h3);font-weight:600;margin:0 0 var(--gap-md);" { "Game Log" }

                @if messages.is_empty() {
                    p class="empty-state" { "No messages yet." }
                } @else {
                    div id="log-entries"
                        class="log-container"
                        hx-trigger=(format!("sse:{}", sse_events))
                        hx-get=(format!("/games/{}/log", game_id))
                        hx-target="#log-entries"
                        hx-swap="innerHTML" {
                        @for msg in messages {
                            (log_entry(msg))
                        }
                    }
                }
            }
        },
    )
}

/// Single log entry.
fn log_entry(msg: &shared::messages::GameMessage) -> maud::Markup {
    let kind_icon = message_kind_icon(&msg.payload);
    let phase_label = msg.phase.to_string();
    let kind_label = message_kind_label(&msg.payload);
    let kind_clr = kind_color(&msg.payload);

    html! {
        div class="log-entry" {
            div class="log-meta" {
                span { "Day " (msg.game_day) }
                span { "·" }
                span class="capitalize" { (phase_label) }
                span { "·" }
                (kind_icon)
                span class=(kind_clr) { (kind_label) }
            }
            p class="log-content" { (msg.content) }
            @if !msg.subject.is_empty() {
                p class="log-subject" { (msg.subject) }
            }
        }
    }
}

/// Human-readable label for message kind.
fn message_kind_label(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "Death",
        MessageKind::Combat => "Combat",
        MessageKind::CombatSwing => "Combat",
        MessageKind::Alliance => "Alliance",
        MessageKind::Movement => "Movement",
        MessageKind::Item => "Item",
        MessageKind::SponsorGift => "Sponsor",
        MessageKind::State => "State",
        MessageKind::Trauma => "Trauma",
        MessageKind::Affliction => "Health",
        MessageKind::Phobia => "Fear",
        MessageKind::Fixation => "Fixation",
    }
}

/// Icon for message kind.
fn message_kind_icon(payload: &shared::messages::MessagePayload) -> maud::Markup {
    use shared::messages::MessagePayload;
    let name = match payload {
        MessagePayload::TributeKilled { .. } | MessagePayload::Combat(_) => "sword",
        MessagePayload::TributeWounded { .. } | MessagePayload::TributeAttacked { .. } => "heart",
        MessagePayload::AllianceFormed { .. } | MessagePayload::AllianceProposed { .. } => "users",
        MessagePayload::AllianceDissolved { .. } | MessagePayload::BetrayalTriggered { .. } => {
            "user-x"
        }
        MessagePayload::TributeMoved { .. } | MessagePayload::TributeHidden { .. } => "map-pin",
        MessagePayload::AreaClosed { .. } | MessagePayload::AreaEvent { .. } => "triangle-alert",
        MessagePayload::ItemFound { .. }
        | MessagePayload::ItemUsed { .. }
        | MessagePayload::ItemDropped { .. } => "backpack",
        MessagePayload::SponsorGift { .. } => "gift",
        MessagePayload::TributeRested { .. } => "bed",
        MessagePayload::TributeStarved { .. } | MessagePayload::TributeDehydrated { .. } => "skull",
        MessagePayload::SanityBreak { .. } => "brain",
        MessagePayload::HungerBandChanged { .. }
        | MessagePayload::ThirstBandChanged { .. }
        | MessagePayload::StaminaBandChanged { .. } => "activity",
        MessagePayload::ShelterSought { .. } => "tent",
        MessagePayload::Foraged { .. } => "search",
        MessagePayload::Drank { .. } => "droplet",
        MessagePayload::Ate { .. } => "utensils",
        MessagePayload::CycleStart { .. } | MessagePayload::CycleEnd { .. } => "sunrise",
        MessagePayload::PhaseStarted { .. } | MessagePayload::PhaseEnded { .. } => "clock",
        MessagePayload::TributeSlept { .. } => "moon",
        MessagePayload::TributeWoke { .. } => "sun",
        MessagePayload::GameEnded { .. } => "trophy",
        MessagePayload::CombatSwing(_) => "sword",
        MessagePayload::TrustShockBreak { .. } => "heart-crack",
        MessagePayload::AfflictionAcquired { .. }
        | MessagePayload::AfflictionProgressed { .. }
        | MessagePayload::AfflictionHealed { .. }
        | MessagePayload::AfflictionCascaded { .. } => "bandage",
        MessagePayload::SubstanceUsed { .. }
        | MessagePayload::AddictionAcquired { .. }
        | MessagePayload::AddictionReinforced { .. }
        | MessagePayload::AddictionEscalated { .. }
        | MessagePayload::AddictionResisted { .. }
        | MessagePayload::AddictionRelapse { .. }
        | MessagePayload::AddictionCraving { .. }
        | MessagePayload::AddictionObserved { .. }
        | MessagePayload::AddictionForgotten { .. }
        | MessagePayload::AddictionHabituated { .. }
        | MessagePayload::TraumaAcquired { .. }
        | MessagePayload::TraumaReinforced { .. }
        | MessagePayload::TraumaEscalated { .. }
        | MessagePayload::TraumaFlashback { .. }
        | MessagePayload::TraumaAvoidance { .. }
        | MessagePayload::TraumaObserved { .. }
        | MessagePayload::TraumaForgotten { .. }
        | MessagePayload::TraumaHabituated { .. } => "brain",
        MessagePayload::PhobiaAcquired { .. }
        | MessagePayload::PhobiaTriggered { .. }
        | MessagePayload::PhobiaEscalated { .. }
        | MessagePayload::PhobiaHabituated { .. }
        | MessagePayload::PhobiaObserved { .. }
        | MessagePayload::PhobiaForgotten { .. }
        | MessagePayload::FixationAcquired { .. }
        | MessagePayload::FixationEscalated { .. }
        | MessagePayload::FixationFired { .. }
        | MessagePayload::FixationConsummated { .. }
        | MessagePayload::FixationThwarted { .. }
        | MessagePayload::FixationFaded { .. } => "eye",
    };
    icon(name)
}

/// Color class for message kind.
fn kind_color(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "kind-death",
        MessageKind::Combat | MessageKind::CombatSwing => "kind-combat",
        MessageKind::Alliance => "kind-alliance",
        MessageKind::Movement => "kind-movement",
        MessageKind::Item | MessageKind::SponsorGift => "kind-item",
        MessageKind::State => "kind-state",
        MessageKind::Trauma => "kind-trauma",
        MessageKind::Affliction => "kind-affliction",
        MessageKind::Phobia => "kind-phobia",
        MessageKind::Fixation => "kind-fixation",
    }
}

// ── Loading spinner ─────────────────────────────────────────────────

/// Inline SVG spinner for HTMX loading indicators.
fn spinner_icon() -> maud::Markup {
    maud::html! {
        svg class="spinner" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" {
            circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" stroke-opacity="0.25" {}
            path fill="currentColor" fill-opacity="0.75" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" {}
        }
    }
}

// ── Survival band helpers ─────────────────────────────────────────────

fn hunger_label(hunger: u8) -> &'static str {
    match hunger {
        0 => "Sated",
        1..=2 => "Peckish",
        3..=4 => "Hungry",
        _ => "Starving",
    }
}

fn hunger_color(hunger: u8) -> &'static str {
    match hunger {
        0 => "band-good",
        1..=2 => "band-warn",
        3..=4 => "band-warn",
        _ => "band-danger",
    }
}

fn thirst_label(thirst: u8) -> &'static str {
    match thirst {
        0 => "Sated",
        1..=2 => "Thirsty",
        3..=4 => "Parched",
        _ => "Dehydrated",
    }
}

fn thirst_color(thirst: u8) -> &'static str {
    match thirst {
        0 => "band-good",
        1..=2 => "band-warn",
        3..=4 => "band-warn",
        _ => "band-danger",
    }
}

fn stamina_label(stamina: u32, max_stamina: u32) -> &'static str {
    if max_stamina == 0 {
        return "N/A";
    }
    let ratio = stamina as f64 / max_stamina as f64;
    if ratio > 0.66 {
        "Fresh"
    } else if ratio > 0.33 {
        "Winded"
    } else {
        "Exhausted"
    }
}

fn stamina_color(stamina: u32, max_stamina: u32) -> &'static str {
    if max_stamina == 0 {
        return "band-none";
    }
    let ratio = stamina as f64 / max_stamina as f64;
    if ratio > 0.66 {
        "band-good"
    } else if ratio > 0.33 {
        "band-warn"
    } else {
        "band-danger"
    }
}
