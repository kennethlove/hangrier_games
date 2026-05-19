use maud::html;
use shared::{DisplayGame, GameStatus};

use super::pages::status_color;
use super::{base_layout, icon, narrative_icon};

/// Full game detail page with navigation tabs.
pub fn game_detail_page(game: &DisplayGame) -> maud::Markup {
    base_layout(
        &game.name,
        html! {
            div {
                // Header section
                div class="mb-6" {
                    div class="flex items-center justify-between" {
                        div {
                            h1 class="text-2xl font-bold text-amber-400" { (game.name) }
                            p class="text-sm text-gray-400 mt-1" {
                                "Day " (game.day.unwrap_or(0))
                                " · "
                                span class=(status_color(&game.status.to_string())) { (game.status) }
                                " · "
                                (game.living_count) "/" (game.tribute_count) " alive"
                            }
                        }
                        @if game.is_mine {
                            div class="flex gap-2" {
                                @if game.status == GameStatus::NotStarted {
                                    button
                                        class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded font-semibold"
                                        hx-put=(format!("/api/games/{}/next", game.identifier))
                                        hx-target="#game-status"
                                        hx-swap="innerHTML"
                                        hx-disabled-elt="this" {
                                        "Start Game"
                                    }
                                } @else if game.status == GameStatus::InProgress {
                                    button
                                        class="bg-amber-600 hover:bg-amber-700 text-white px-4 py-2 rounded font-semibold"
                                        hx-put=(format!("/api/games/{}/next", game.identifier))
                                        hx-target="#game-status"
                                        hx-swap="innerHTML"
                                        hx-disabled-elt="this" {
                                        "Play Day " (game.day.unwrap_or(0))
                                    }
                                }
                            }
                        } @else {
                            p class="text-sm text-gray-500" {
                                "By " (game.created_by.username)
                            }
                        }
                    }
                }

                // Status update target for HTMX
                div id="game-status" {
                    @if game.status == GameStatus::Finished {
                        @if let Some(winner) = &game.winner {
                            div class="mb-4 p-3 bg-amber-900/30 border border-amber-700 rounded-lg" {
                                p class="text-amber-300 font-semibold" {
                                    (icon("trophy"))
                                    " Winner: " (winner.name)
                                }
                            }
                        }
                    }
                }

                // Navigation tabs
                nav class="flex gap-4 border-b border-gray-800 mb-6" {
                    a href=(format!("/games/{}/tributes", game.identifier))
                        class="pb-2 px-1 text-sm font-medium text-gray-400 hover:text-white border-b-2 border-transparent hover:border-amber-500" {
                        "Tributes"
                    }
                    a href=(format!("/games/{}/areas", game.identifier))
                        class="pb-2 px-1 text-sm font-medium text-gray-400 hover:text-white border-b-2 border-transparent hover:border-amber-500" {
                        "Areas"
                    }
                    a href=(format!("/games/{}/log", game.identifier))
                        class="pb-2 px-1 text-sm font-medium text-gray-400 hover:text-white border-b-2 border-transparent hover:border-amber-500" {
                        "Log"
                    }
                }

                // Content area — sub-pages load here via HTMX
                div id="detail-content" {
                    p class="text-gray-500" { "Select a tab above" }
                }
            }
        },
    )
}

/// Tribute list page — grid grouped by district.
pub fn tributes_page(game_id: &str, tributes: &[game::tributes::Tribute]) -> maud::Markup {
    base_layout(
        "Tributes",
        html! {
            div {
                a href=(format!("/games/{}", game_id))
                    class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 class="text-xl font-bold text-amber-400 mb-4" { "Tributes" }

                @if tributes.is_empty() {
                    p class="text-gray-500" { "No tributes yet." }
                } @else {
                    // Group by district
                    @for district_num in 1..=12 {
                        @let district_tributes: Vec<_> = tributes.iter()
                            .filter(|t| t.district == district_num)
                            .collect();
                        @if !district_tributes.is_empty() {
                            div class="mb-6" {
                                h3 class="text-sm font-semibold text-gray-300 mb-2" {
                                    "District " (district_num)
                                }
                                div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3" {
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

/// Single tribute card with stats strip.
fn tribute_card(tribute: &game::tributes::Tribute) -> maud::Markup {
    let is_alive = tribute.is_alive();
    let status_class = if is_alive {
        "text-green-400"
    } else {
        "text-red-400"
    };
    let status_icon = if is_alive { "check-circle" } else { "x-circle" };

    html! {
        div class="bg-gray-900 border border-gray-800 rounded-lg p-3" {
            div class="flex items-center justify-between mb-2" {
                span class="font-semibold text-white text-sm" { (tribute.name) }
                span class=(status_class) { (icon(status_icon)) }
            }

            // Stats strip
            div class="space-y-1 text-xs" {
                // Health
                div class="flex items-center gap-1" {
                    (icon("heart"))
                    span class="text-gray-400" { "HP" }
                    span class="text-gray-200 ml-auto" { (tribute.attributes.health) }
                }
                // Sanity
                div class="flex items-center gap-1" {
                    (icon("brain"))
                    span class="text-gray-400" { "SAN" }
                    span class="text-gray-200 ml-auto" { (tribute.attributes.sanity) }
                }
                // Movement
                div class="flex items-center gap-1" {
                    (icon("zap"))
                    span class="text-gray-400" { "MOV" }
                    span class="text-gray-200 ml-auto" { (tribute.attributes.movement) }
                }
                // Strength
                div class="flex items-center gap-1" {
                    (icon("sword"))
                    span class="text-gray-400" { "STR" }
                    span class="text-gray-200 ml-auto" { (tribute.attributes.strength) }
                }
            }

            // Survival bands
            div class="mt-2 pt-2 border-t border-gray-800 flex gap-2 text-xs" {
                span class=(hunger_color(tribute.hunger)) { "H: " (hunger_label(tribute.hunger)) }
                span class=(thirst_color(tribute.thirst)) { "T: " (thirst_label(tribute.thirst)) }
                span class=(stamina_color(tribute.stamina, tribute.max_stamina)) {
                    "S: " (stamina_label(tribute.stamina, tribute.max_stamina))
                }
            }

            // Area location
            div class="mt-1 text-xs text-gray-500" {
                (icon("map-pin"))
                " " (tribute.area)
            }

            // Inventory
            @if !tribute.items.is_empty() {
                div class="mt-2 pt-2 border-t border-gray-800" {
                    p class="text-xs text-gray-500 mb-1" { "Items:" }
                    div class="flex flex-wrap gap-1" {
                        @for item in &tribute.items {
                            span class="text-xs bg-gray-800 text-gray-300 px-1.5 py-0.5 rounded" {
                                (icon("backpack"))
                                " " (item.name)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Areas list page.
pub fn areas_page(game_id: &str, areas: &[game::areas::AreaDetails]) -> maud::Markup {
    base_layout(
        "Areas",
        html! {
            div {
                a href=(format!("/games/{}", game_id))
                    class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 class="text-xl font-bold text-amber-400 mb-4" { "Areas" }

                @if areas.is_empty() {
                    p class="text-gray-500" { "No areas yet." }
                } @else {
                    div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3" {
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
    let status_class = if is_open {
        "text-green-400"
    } else {
        "text-orange-400"
    };

    html! {
        div class="bg-gray-900 border border-gray-800 rounded-lg p-3" {
            div class="flex items-center justify-between mb-2" {
                h3 class="font-semibold text-white text-sm" {
                    (icon("map-pin"))
                    " " (area.name)
                }
                span class=(status_class) {
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
                div class="mt-2" {
                    p class="text-xs text-gray-500 mb-1" { "Items:" }
                    div class="flex flex-wrap gap-1" {
                        @for item in &area.items {
                            span class="text-xs bg-gray-800 text-gray-300 px-1.5 py-0.5 rounded" {
                                (icon("backpack"))
                                " " (item.name)
                            }
                        }
                    }
                }
            }

            // Events
            @if !area.events.is_empty() {
                div class="mt-2 pt-2 border-t border-gray-800" {
                    p class="text-xs text-gray-500 mb-1" { "Events:" }
                    ul class="text-xs text-gray-400 space-y-0.5" {
                        @for event in &area.events {
                            li { (narrative_icon("event")) " " (event) }
                        }
                    }
                }
            }
        }
    }
}

/// Game log page — scrollable message list.
pub fn log_page(game_id: &str, messages: &[shared::messages::GameMessage]) -> maud::Markup {
    base_layout(
        "Log",
        html! {
            div {
                a href=(format!("/games/{}", game_id))
                    class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Game"
                }
                h2 class="text-xl font-bold text-amber-400 mb-4" { "Game Log" }

                @if messages.is_empty() {
                    p class="text-gray-500" { "No messages yet." }
                } @else {
                    div class="max-h-[70vh] overflow-y-auto space-y-2 pr-2" {
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
        div class="bg-gray-900 border border-gray-800 rounded p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                span { "Day " (msg.game_day) }
                span { "·" }
                span class="capitalize" { (phase_label) }
                span { "·" }
                (kind_icon)
                span class=(kind_clr) { (kind_label) }
            }
            p class="text-sm text-gray-200" { (msg.content) }
            @if !msg.subject.is_empty() {
                p class="text-xs text-gray-500 mt-1" { (msg.subject) }
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
        MessagePayload::TraumaAcquired { .. } | MessagePayload::TraumaReinforced { .. } => "brain",
        MessagePayload::PhobiaAcquired { .. } | MessagePayload::PhobiaTriggered { .. } => "eye",
    };
    icon(name)
}

/// Color class for message kind.
fn kind_color(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "text-red-400",
        MessageKind::Combat | MessageKind::CombatSwing => "text-orange-400",
        MessageKind::Alliance => "text-blue-400",
        MessageKind::Movement => "text-purple-400",
        MessageKind::Item | MessageKind::SponsorGift => "text-yellow-400",
        MessageKind::State | MessageKind::Trauma => "text-gray-400",
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
        0 => "text-green-400",
        1..=2 => "text-yellow-400",
        3..=4 => "text-orange-400",
        _ => "text-red-400",
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
        0 => "text-green-400",
        1..=2 => "text-yellow-400",
        3..=4 => "text-orange-400",
        _ => "text-red-400",
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
        return "text-gray-500";
    }
    let ratio = stamina as f64 / max_stamina as f64;
    if ratio > 0.66 {
        "text-green-400"
    } else if ratio > 0.33 {
        "text-yellow-400"
    } else {
        "text-red-400"
    }
}
