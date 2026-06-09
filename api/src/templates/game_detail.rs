use maud::html;
use shared::afflictions::AfflictionKind;
use shared::messages::Phase;
use shared::{DisplayGame, GameStatus};

use super::{AuthState, base_layout, icon, narrative_icon};

/// Determine the phase label and class from game state and messages.
fn current_broadcast_phase(
    game: &DisplayGame,
    messages: &[shared::messages::GameMessage],
) -> (&'static str, &'static str) {
    match game.status {
        GameStatus::Finished => ("finished", "FINISHED"),
        GameStatus::NotStarted => ("day", "STAGING"),
        GameStatus::InProgress => {
            if let Some(last) = messages.last() {
                match last.phase {
                    Phase::Dawn => ("dawn", "DAWN"),
                    Phase::Day => ("day", "DAY"),
                    Phase::Dusk => ("dusk", "DUSK"),
                    Phase::Night => ("night", "NIGHT"),
                }
            } else {
                ("day", "DAY")
            }
        }
    }
}

/// Full game detail page — broadcast / surveillance layout.
pub fn game_detail_page(
    auth: AuthState,
    game: &DisplayGame,
    tributes: &[game::tributes::Tribute],
    messages: &[shared::messages::GameMessage],
    segments: &[announcers::CommentarySegment],
) -> maud::Markup {
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
        TraumaAcquired,TraumaReinforced,TraumaEscalated,TraumaFlashback,TraumaAvoidance,\
        TraumaObserved,TraumaForgotten,TraumaHabituated,\
        PhobiaAcquired,PhobiaTriggered,FixationAcquired,FixationEscalated,FixationFired,\
        FixationConsummated,FixationThwarted,FixationFaded,\
        SubstanceUsed,AddictionAcquired,AddictionReinforced,AddictionEscalated,\
        AddictionResisted,AddictionRelapse,\
        AddictionCraving,AddictionObserved,AddictionForgotten,AddictionHabituated,\
        TributeTrapped,Struggling,TrappedEscaped,TributeDiedWhileTrapped,\
        TrapSet,TrapTriggered,Commentary";

    let alive = game.living_count;
    let fallen = game.tribute_count.saturating_sub(game.living_count);
    let total = game.tribute_count;
    let (phase_class, phase_label) = current_broadcast_phase(game, messages);
    let location = game.name.clone();

    // Collect unique days from messages
    let mut day_numbers: Vec<u32> = messages.iter().map(|m| m.game_day).collect();
    day_numbers.sort();
    day_numbers.dedup();
    let current_day = game.day.unwrap_or(0);

    // Sort tributes: alive first, then alphabetically
    let mut sorted_tributes: Vec<_> = tributes.iter().collect();
    sorted_tributes.sort_by(|a, b| b.is_alive().cmp(&a.is_alive()).then(a.name.cmp(&b.name)));

    base_layout(
        &game.name,
        auth,
        html! {
            div class="broadcast-theme" {
                // SSE connection for live events
                div hx-ext="sse" sse-connect=(format!("/api/games/{}/events", game.identifier)) {

                    // ══ BROADCAST HEADER ══
                    div class="broad-header" {
                        div style="display:flex;align-items:center;gap:14px;" {
                            // Emblem
                            div style="width:32px;height:32px;background:var(--broad-accent);clip-path:polygon(50% 0%,100% 25%,100% 75%,50% 100%,0% 75%,0% 25%);flex-shrink:0;display:grid;place-items:center;" {
                                div style="width:14px;height:14px;background:var(--broad-bg);clip-path:polygon(50% 0%,100% 25%,100% 75%,50% 100%,0% 75%,0% 25%);" {}
                            }
                            span class="round-id" {
                                "ROUND "
                                span class="hl" { (game.day.unwrap_or(0)) }
                            }
                            span class="location" { (location.to_uppercase()) }
                        }
                        div style="display:flex;align-items:center;gap:10px;" {
                            // Phase badge
                            div class=(format!("phase-badge {}", phase_class)) {
                                span class="dot" {}
                                span { (phase_label) }
                            }
                        }
                        div class="broad-stats" {
                            div class="stat" {
                                span class="num alive" { (alive) }
                                span class="stat-label" { "ALIVE" }
                            }
                            div class="stat" {
                                span class="num fallen" { (fallen) }
                                span class="stat-label" { "FALLEN" }
                            }
                            div class="stat" {
                                span class="num" { (total) }
                                span class="stat-label" { "TOTAL" }
                            }
                        }
                    }

                    // ══ DAY NAV ══
                    div class="day-nav" {
                        div class="day-left" {
                            span class="day-label" {
                                "DAY "
                                span class="hl" { (game.day.unwrap_or(0)) }
                            }
                            div class="day-arrows" {
                                button class="day-arrow" title="Previous day" { "<" }
                                button class="day-arrow" title="Next day" { ">" }
                            }
                            select class="day-select" id="daySelect" {
                                @for day_num in &day_numbers {
                                    @if *day_num == current_day {
                                        option value=(day_num.to_string()) selected { "Day " (day_num) }
                                    } @else {
                                        option value=(day_num.to_string()) { "Day " (day_num) }
                                    }
                                }
                            }
                        }
                        div class="day-right" {
                            @if game.is_mine && game.status == GameStatus::InProgress {
                                button
                                    class="phase-btn"
                                    hx-put=(format!("/api/games/{}/next", game.identifier))
                                    hx-target="#game-status"
                                    hx-swap="innerHTML"
                                    hx-disabled-elt="this"
                                    hx-indicator="#play-spinner" {
                                    "ADVANCE +"
                                    span id="play-spinner" class="htmx-indicator" { (spinner_icon()) }
                                }
                            } @else if game.is_mine && game.status == GameStatus::NotStarted {
                                button
                                    class="phase-btn"
                                    hx-put=(format!("/api/games/{}/next", game.identifier))
                                    hx-target="#game-status"
                                    hx-swap="innerHTML"
                                    hx-disabled-elt="this"
                                    hx-indicator="#start-spinner" {
                                    "START"
                                    span id="start-spinner" class="htmx-indicator" { (spinner_icon()) }
                                }
                            }
                        }
                    }

                    // ══ TICKER BAR ══
                    div class="ticker-bar" {
                        div class="ticker-left" {
                            span class="live-dot" {}
                            span class="live-label" { "LIVE" }
                            span class="ticker-divider" {}
                            span class="ticker-text" id="tickerText" {
                                @if let Some(last) = messages.last() {
                                    "LATEST: " (last.content)
                                } @else {
                                    "ARENA SURVEILLANCE ACTIVE — "
                                    span class="hl" { (total) " TRIBUTES" }
                                    " IN PLAY"
                                }
                            }
                        }
                        div class="ticker-right" {
                            span class="ticker-stat" {
                                "EVENTS: "
                                span class="num" { (messages.len()) }
                            }
                        }
                    }

                    @if game.status == GameStatus::Finished {
                        @if let Some(winner) = &game.winner {
                            div class="winner-banner" style="margin:0 0 10px;border-color:var(--broad-gold);color:var(--broad-gold);background:rgba(240,180,41,0.08);" {
                                span { "☠ WINNER: " (winner.name) " ☠" }
                            }
                        }
                    }

                    // ══ MAIN GRID — 50/50 ══
                    div class="broad-grid" {

                        // ── LEFT PANEL ──
                        div class="broad-left-panel" {

                            // Arena Map
                            div class="map-section" {
                                div class="section-header" {
                                    span class="title" { "ARENA SURVEILLANCE" }
                                }
                                div class="map-container" {
                                    div style="display:grid;place-items:center;height:100%;color:var(--broad-fg-muted);font-size:var(--fs-xs);font-family:var(--font-condensed);letter-spacing:2px;text-transform:uppercase;" {
                                        "MAP COMING IN PHASE 6"
                                    }
                                }
                            }

                            // Tribute Roster
                            div class="roster-section" {
                                div class="section-header" {
                                    span class="title" { "TRIBUTE ROSTER" }
                                    span class="count" { (total) " TRIBUTES" }
                                }
                                div class="roster-scroll" {
                                    @if tributes.is_empty() {
                                        div class="empty-state" style="padding:var(--gap-sm);" { "No tributes yet." }
                                    } @else {
                                        @for tribute in &sorted_tributes {
                                            (broadcast_tribute_row(tribute))
                                        }
                                    }
                                }
                            }
                        }

                        // ── RIGHT PANEL — Event Feed ──
                        div class="feed-section" {
                            div class="feed-header" {
                                span class="title" {
                                    "EVENT "
                                    span class="hl" { "FEED" }
                                }
                                div class="feed-tabs" {
                                    button class="feed-tab active" { "ALL" }
                                    button class="feed-tab" { "ACTION" }
                                    button class="feed-tab" { "DEATHS" }
                                    button class="feed-tab" { "EVENTS" }
                                    button class="feed-tab" { "COMMS" }
                                }
                            }
                            div class="feed-scroll" id="feedScroll"
                                hx-trigger=(format!("sse:{}", sse_events))
                                hx-get=(format!("/api/games/{}/log", game.identifier))
                                hx-target="#feedScroll"
                                hx-swap="innerHTML" {
                                @if messages.is_empty() && segments.is_empty() {
                                    div class="empty-state" { "No events yet." }
                                } @else {
                                    @for msg in messages {
                                        (log_entry(msg))
                                    }
                                    @for seg in segments {
                                        (commentary_segment(seg))
                                    }
                                }
                            }
                        }

                    }

                    // Hidden status target for HTMX updates
                    div id="game-status" style="display:none;" {}
                }
            }
        },
    )
}

/// Compact tribute row for broadcast roster — avatar, health bar, status pill.
fn broadcast_tribute_row(tribute: &game::tributes::Tribute) -> maud::Markup {
    let is_alive = tribute.is_alive();
    let status_text = if is_alive { "ALIVE" } else { "DEAD" };
    let status_class = if is_alive { "alive" } else { "dead" };
    let health = tribute.attributes.health;
    let health_class = if health > 60 {
        "high"
    } else if health > 20 {
        "mid"
    } else if health > 0 {
        "low"
    } else {
        "empty"
    };
    let initial = tribute.name.chars().next().unwrap_or('?');
    let avatar_color = if is_alive {
        "var(--broad-accent)"
    } else {
        "var(--broad-danger)"
    };

    html! {
        div class=(format!("tribute-card {}", if !is_alive { "dead" } else { "" })) {
            div class="tribute-avatar" style=(format!("border-color:{};color:{}", avatar_color, avatar_color)) {
                (initial.to_string())
            }
            div class="tribute-info" {
                div class="tribute-name" { (tribute.name) }
                div class="tribute-meta" {
                    div class="health-bar" {
                        div class=(format!("health-fill {}", health_class))
                            style=(format!("width:{}%;", health)) {}
                    }
                }
            }
            div class="tribute-stats" {
                span class=(format!("tribute-status {}", status_class)) { (status_text) }
            }
        }
    }
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

/// Game log page — scrollable message list with SSE real-time updates and commentary.
pub fn log_page(
    auth: AuthState,
    game_id: &str,
    messages: &[shared::messages::GameMessage],
    segments: &[announcers::CommentarySegment],
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
        AddictionCraving,AddictionObserved,AddictionForgotten,AddictionHabituated,\
        TributeTrapped,Struggling,TrappedEscaped,TributeDiedWhileTrapped,\
        TrapSet,TrapTriggered,Commentary";

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

                @if messages.is_empty() && segments.is_empty() {
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
                        @for seg in segments {
                            (commentary_segment(seg))
                        }
                    }
                }
            }
        },
    )
}

/// Render a commentary segment as an ANALYSIS card.
fn commentary_segment(seg: &announcers::CommentarySegment) -> maud::Markup {
    html! {
        div class="event-card commentary" {
            div class="card-head" {
                span class="card-badge" { "ANALYSIS" }
                span class="card-timestamp" { "Day " (seg.day) " " (seg.phase) }
            }
            div class="card-body" {
                div class="comment-text" {
                    @for line in &seg.lines {
                        div { (line.text) }
                    }
                }
                div class="comment-speaker" {
                    div class="speaker-avatar" { "C" }
                    div class="speaker-info" {
                        span class="speaker-name" {
                            (seg.lines.first().map(|l| l.speaker.as_str()).unwrap_or("COMMENTATOR"))
                        }
                        span class="speaker-role" { "ANALYSIS" }
                    }
                }
            }
        }
    }
}

/// Map message payload to broadcast card archetype.
fn message_archetype(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "death",
        MessageKind::Combat | MessageKind::CombatSwing => "action",
        MessageKind::Alliance => "commentary",
        MessageKind::Movement => "commentary",
        MessageKind::Item | MessageKind::SponsorGift => "event",
        MessageKind::State => "commentary",
        MessageKind::Trauma => "commentary",
        MessageKind::Affliction => "commentary",
        MessageKind::Phobia => "commentary",
        MessageKind::Fixation => "commentary",
        MessageKind::Trapped => "event",
        MessageKind::Sleep => "commentary",
    }
}

/// Human-readable badge label for archetype.
fn archetype_label(archetype: &str) -> &'static str {
    match archetype {
        "action" => "ACTION",
        "death" => "ELIMINATED",
        "event" => "ARENA EVENT",
        "commentary" => "ANALYSIS",
        _ => "EVENT",
    }
}

/// Timestamp string for event card — Day:Phase:Tick.
fn event_timestamp(msg: &shared::messages::GameMessage) -> String {
    format!("D{} {} T{}", msg.game_day, msg.phase, msg.tick)
}

/// Single event card — 4 archetypes with broadcast styling.
fn log_entry(msg: &shared::messages::GameMessage) -> maud::Markup {
    let archetype = message_archetype(&msg.payload);
    let badge = archetype_label(archetype);
    let ts = event_timestamp(msg);
    let icon_html = message_kind_icon(&msg.payload);
    let kind_label = message_kind_label(&msg.payload);
    let color_class = kind_color(&msg.payload);

    html! {
        div class=(format!("event-card {} {}", archetype, color_class)) {
            div class="card-head" {
                span class="card-badge" { (badge) }
                span class="card-timestamp" { (ts) }
            }
            div class="card-body" {
                (icon_html)
                " "
                span style="font-weight:600;" { (kind_label) }
                " \u{2014} "
                (msg.content)
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
        MessageKind::Trapped => "Trapped",
        MessageKind::Sleep => "Sleep",
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
        MessagePayload::SleepIncident { .. } => "moon",
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
        MessagePayload::TributeTrapped { .. } | MessagePayload::Struggling { .. } => {
            "alert-triangle"
        }
        MessagePayload::TrappedEscaped { .. } => "check-circle",
        MessagePayload::TributeDiedWhileTrapped { .. } => "skull",
        MessagePayload::RescueAttempted { .. } | MessagePayload::PartialRescueProgress { .. } => {
            "hand"
        }
        MessagePayload::TrapSet { .. } => "skull",
        MessagePayload::TrapTriggered { .. } => "skull",
        MessagePayload::Generic => "info",
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
        MessageKind::Trapped => "kind-trapped",
        MessageKind::Sleep => "kind-sleep",
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
