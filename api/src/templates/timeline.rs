use maud::html;
use shared::messages::TributeRef;
use shared::messages::{GameMessage, MessageKind, MessagePayload, PeriodSummary, TimelineSummary};

use super::{AuthState, base_layout, icon};

/// Context for rendering the timeline page.
pub struct TimelineContext<'a> {
    pub auth: AuthState,
    pub game_id: &'a str,
    pub game_name: &'a str,
    pub periods: &'a TimelineSummary,
    pub current_day: u32,
    pub current_phase: &'a shared::messages::Phase,
    pub filter: &'a str,
    pub tribute_filter: &'a str,
    pub tributes: &'a [TributeRef],
    pub events: &'a [GameMessage],
    pub selected_day: Option<u32>,
    pub selected_phase: Option<shared::messages::Phase>,
}

/// Full timeline page with period grid, filters, and event cards.
pub fn timeline_page(ctx: &TimelineContext<'_>) -> maud::Markup {
    base_layout(
        &format!("Timeline — {}", ctx.game_name),
        ctx.auth.clone(),
        html! {
            div {
                // Back link
                a href=(format!("/games/{}", ctx.game_id))
                    class="text-sm text-gray-400 hover:text-white mb-4 inline-block" {
                    (icon("arrow-left"))
                    " Back to Game"
                }

                // Header
                div class="mb-4" {
                    h1 class="text-xl font-bold text-amber-400" { "Timeline" }
                    p class="text-sm text-gray-500" {
                        "Day " (ctx.current_day) " · " (ctx.current_phase)
                    }
                }

                // Period timeline strip
                @if !ctx.periods.periods.is_empty() {
                    div class="mb-4" {
                        div class="flex gap-1.5 overflow-x-auto pb-2" {
                            @for period in &ctx.periods.periods {
                                (period_chip(ctx.game_id, period, ctx.filter, ctx.tribute_filter))
                            }
                        }
                    }
                }

                // Filter chips row
                (filter_chips(ctx.game_id, ctx.selected_day, ctx.selected_phase, ctx.filter, ctx.tribute_filter))

                // Tribute filter chips row
                @if !ctx.tributes.is_empty() {
                    (tribute_chips(ctx.game_id, ctx.selected_day, ctx.selected_phase, ctx.filter, ctx.tribute_filter, ctx.tributes))
                }

                // Event timeline or period grid
                @if let (Some(_day), Some(_phase)) = (ctx.selected_day, ctx.selected_phase) {
                    // Show filtered events for selected period
                    @if ctx.events.is_empty() {
                        (empty_state("No events for this period."))
                    } @else {
                        div class="space-y-2" {
                            @for msg in ctx.events {
                                (event_card(msg))
                            }
                        }
                    }
                } @else {
                    // Show period grid overview
                    (period_grid(&ctx.periods.periods))
                }
            }
        },
    )
}

/// Single period chip in the timeline strip.
fn period_chip(
    game_id: &str,
    period: &PeriodSummary,
    filter: &str,
    tribute_filter: &str,
) -> maud::Markup {
    let is_current = period.is_current;
    let border_class = if is_current {
        "border-amber-500 ring-1 ring-amber-500/30"
    } else {
        "border-gray-700 hover:border-gray-500"
    };
    let bg_class = if is_current {
        "bg-amber-900/30"
    } else {
        "bg-gray-900"
    };

    let deaths_indicator = if period.deaths > 0 {
        html! {
            span class="text-red-400 text-xs" {
                (icon("skull"))
                " " (period.deaths)
            }
        }
    } else {
        html! {}
    };

    let filter_param = if !filter.is_empty() {
        format!("&filter={filter}")
    } else {
        String::new()
    };
    let tribute_param = if !tribute_filter.is_empty() {
        format!("&tribute={tribute_filter}")
    } else {
        String::new()
    };

    html! {
        a href=(format!(
            "/games/{}/timeline?day={}&phase={}{}{}",
            game_id, period.day, period.phase, filter_param, tribute_param
        ))
            class=(format!("flex-shrink-0 flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs transition-colors {} {}", bg_class, border_class)) {
            span class="text-gray-300 font-medium" {
                "D" (period.day) " " (period.phase)
            }
            span class="text-gray-500" { (period.event_count) }
            (deaths_indicator)
        }
    }
}

/// Category filter chips row.
fn filter_chips(
    game_id: &str,
    selected_day: Option<u32>,
    selected_phase: Option<shared::messages::Phase>,
    active_filter: &str,
    tribute_filter: &str,
) -> maud::Markup {
    let filters = [
        ("", "All", "list"),
        ("Deaths", "Deaths", "skull"),
        ("Combat", "Combat", "sword"),
        ("Alliances", "Alliances", "users"),
        ("Movement", "Movement", "map-pin"),
        ("Items", "Items", "backpack"),
    ];

    let day_param = selected_day
        .map(|d| format!("&day={d}"))
        .unwrap_or_default();
    let phase_param = selected_phase
        .map(|p| format!("&phase={p}"))
        .unwrap_or_default();
    let tribute_param = if !tribute_filter.is_empty() {
        format!("&tribute={tribute_filter}")
    } else {
        String::new()
    };

    html! {
        div class="flex flex-wrap gap-1.5 mb-3" {
            @for (value, label, icon_name) in &filters {
                @let is_active = active_filter == *value;
                @let chip_class = if is_active {
                    "bg-amber-600 text-white"
                } else {
                    "bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200"
                };
                a href=(format!(
                    "/games/{}/timeline?filter={}{}{}{}",
                    game_id, value, day_param, phase_param, tribute_param
                ))
                    class=(format!("px-2.5 py-1 rounded-full text-xs font-medium transition-colors {}", chip_class)) {
                    (icon(icon_name))
                    " " (label)
                }
            }
        }
    }
}

/// Tribute filter chips row.
fn tribute_chips(
    game_id: &str,
    selected_day: Option<u32>,
    selected_phase: Option<shared::messages::Phase>,
    filter: &str,
    tribute_filter: &str,
    tributes: &[TributeRef],
) -> maud::Markup {
    let day_param = selected_day
        .map(|d| format!("&day={d}"))
        .unwrap_or_default();
    let phase_param = selected_phase
        .map(|p| format!("&phase={p}"))
        .unwrap_or_default();
    let filter_param = if !filter.is_empty() {
        format!("&filter={filter}")
    } else {
        String::new()
    };

    // Show first 12 tributes to avoid overwhelming the UI
    let max_tributes = tributes.len().min(12);

    html! {
        div class="flex flex-wrap gap-1.5 mb-3" {
            // "All tributes" chip
            @let is_all = tribute_filter.is_empty();
            @let all_class = if is_all {
                "bg-blue-600 text-white"
            } else {
                "bg-gray-800 text-gray-500 hover:bg-gray-700"
            };
            a href=(format!(
                "/games/{}/timeline?tribute={}{}{}{}",
                game_id, "", filter_param, day_param, phase_param
            ))
                class=(format!("px-2 py-0.5 rounded-full text-xs transition-colors {}", all_class)) {
                "All"
            }
            @for tribute in &tributes[..max_tributes] {
                @let is_active = tribute_filter == tribute.name;
                @let chip_class = if is_active {
                    "bg-blue-600 text-white"
                } else {
                    "bg-gray-800 text-gray-400 hover:bg-gray-700"
                };
                a href=(format!(
                    "/games/{}/timeline?tribute={}{}{}{}",
                    game_id, tribute.name, filter_param, day_param, phase_param
                ))
                    class=(format!("px-2 py-0.5 rounded-full text-xs transition-colors {}", chip_class)) {
                    (tribute.name)
                }
            }
        }
    }
}

/// Overview grid showing all periods at a glance.
fn period_grid(periods: &[PeriodSummary]) -> maud::Markup {
    html! {
        div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2" {
            @for period in periods {
                @let border_class = if period.is_current {
                    "border-amber-500"
                } else {
                    "border-gray-800"
                };
                @let bg_class = if period.is_current {
                    "bg-amber-900/20"
                } else {
                    "bg-gray-900"
                };
                div class=(format!("p-3 rounded-lg border {} {}", bg_class, border_class)) {
                    div class="flex items-center justify-between mb-1" {
                        span class="text-sm font-medium text-gray-300" {
                            "Day " (period.day) " " (period.phase)
                        }
                        @if period.is_current {
                            span class="text-xs text-amber-400" { "Now" }
                        }
                    }
                    div class="flex items-center gap-3 text-xs text-gray-500" {
                        span { (period.event_count) " events" }
                        @if period.deaths > 0 {
                            span class="text-red-400" {
                                (icon("skull"))
                                " " (period.deaths)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Empty state placeholder.
fn empty_state(message: &str) -> maud::Markup {
    html! {
        div class="text-center py-12" {
            (icon("ghost"))
            p class="text-gray-500 mt-2" { (message) }
        }
    }
}

/// Dispatch to kind-specific event card.
pub fn event_card(msg: &GameMessage) -> maud::Markup {
    use MessageKind::*;
    match msg.payload.kind() {
        Death => death_card(msg),
        Combat => combat_card(msg),
        CombatSwing => combat_swing_card(msg),
        Alliance => alliance_card(msg),
        Movement => movement_card(msg),
        Item | SponsorGift => item_card(msg),
        State => state_card(msg),
        Trauma => trauma_card(msg),
        Affliction => affliction_card(msg),
        Phobia => phobia_card(msg),
    }
}

/// Death event card.
fn death_card(msg: &GameMessage) -> maud::Markup {
    let (victim, killer, cause) = match &msg.payload {
        MessagePayload::TributeKilled {
            victim,
            killer,
            cause,
        } => (victim, killer.as_ref(), cause.as_str()),
        _ => return fallback_card(msg),
    };

    let killer_text = killer
        .map(|k| format!(" by {}", k.name))
        .unwrap_or_default();

    html! {
        div class="bg-gray-900 border border-red-900/50 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("skull"))
                span class="text-red-400 font-medium" { "Death" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" {
                "strong" (victim.name) (killer_text) " — " (cause)
            }
        }
    }
}

/// Combat event card.
fn combat_card(msg: &GameMessage) -> maud::Markup {
    let (attacker, target, outcome) = match &msg.payload {
        MessagePayload::Combat(engagement) => (
            &engagement.attacker,
            &engagement.target,
            &engagement.outcome,
        ),
        _ => return fallback_card(msg),
    };

    let outcome_label = match outcome {
        shared::messages::CombatOutcome::Killed => "killed",
        shared::messages::CombatOutcome::Wounded => "wounded",
        shared::messages::CombatOutcome::TargetFled => "target fled",
        shared::messages::CombatOutcome::AttackerFled => "attacker fled",
        shared::messages::CombatOutcome::Stalemate => "stalemate",
    };

    html! {
        div class="bg-gray-900 border border-orange-900/50 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("sword"))
                span class="text-orange-400 font-medium" { "Combat" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" {
                (attacker.name) " fought " (target.name) " — " (outcome_label)
            }
        }
    }
}

/// Combat swing card (typed beat).
fn combat_swing_card(msg: &GameMessage) -> maud::Markup {
    use shared::combat_beat::SwingOutcome;
    let beat = match &msg.payload {
        MessagePayload::CombatSwing(beat) => beat,
        _ => return fallback_card(msg),
    };

    let outcome_label = match &beat.outcome {
        SwingOutcome::Miss => "missed",
        SwingOutcome::Wound { .. } => "hit",
        SwingOutcome::CriticalHitWound { .. } => "critical hit",
        SwingOutcome::BlockWound { .. } => "countered",
        SwingOutcome::Kill { .. } => "killed",
        SwingOutcome::AttackerDied { .. } => "died in counter",
        SwingOutcome::FumbleSurvive { .. } => "fumbled",
        SwingOutcome::FumbleDeath { .. } => "fumbled to death",
        SwingOutcome::SelfAttackWound { .. } => "self-hit",
        SwingOutcome::Suicide { .. } => "self-killed",
    };

    let weapon_text = beat
        .weapon
        .as_ref()
        .map(|w| format!(" with {}", w.name))
        .unwrap_or_default();

    html! {
        div class="bg-gray-900 border border-orange-800/30 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("sword"))
                span class="text-orange-300 font-medium" { "Swing" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" {
                (beat.attacker.name) " → " (beat.target.name) (weapon_text) " — " (outcome_label)
            }
        }
    }
}

/// Alliance event card.
fn alliance_card(msg: &GameMessage) -> maud::Markup {
    let content = match &msg.payload {
        MessagePayload::AllianceFormed { members } => {
            let names: Vec<_> = members.iter().map(|m| m.name.as_str()).collect();
            format!("{} formed an alliance", names.join(", "))
        }
        MessagePayload::AllianceProposed { proposer, target } => {
            format!("{} proposed alliance to {}", proposer.name, target.name)
        }
        MessagePayload::AllianceDissolved { members, reason } => {
            let names: Vec<_> = members.iter().map(|m| m.name.as_str()).collect();
            format!("{} alliance dissolved — {}", names.join(", "), reason)
        }
        MessagePayload::BetrayalTriggered { betrayer, victim } => {
            format!("{} betrayed {}", betrayer.name, victim.name)
        }
        MessagePayload::TrustShockBreak { tribute, partner } => {
            format!("{} lost trust in {}", tribute.name, partner.name)
        }
        _ => return fallback_card(msg),
    };

    html! {
        div class="bg-gray-900 border border-blue-900/50 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("users"))
                span class="text-blue-400 font-medium" { "Alliance" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" { (content) }
        }
    }
}

/// Movement event card.
fn movement_card(msg: &GameMessage) -> maud::Markup {
    let content = match &msg.payload {
        MessagePayload::TributeMoved { tribute, from, to } => {
            format!("{} moved from {} to {}", tribute.name, from.name, to.name)
        }
        MessagePayload::TributeHidden { tribute, area } => {
            format!("{} hid in {}", tribute.name, area.name)
        }
        MessagePayload::AreaClosed { area } => {
            format!("Area closed: {}", area.name)
        }
        MessagePayload::AreaEvent {
            area,
            kind,
            description,
        } => {
            format!(
                "{} event in {}: {}",
                format!("{:?}", kind).to_lowercase(),
                area.name,
                description
            )
        }
        _ => return fallback_card(msg),
    };

    html! {
        div class="bg-gray-900 border border-purple-900/50 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("map-pin"))
                span class="text-purple-400 font-medium" { "Movement" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" { (content) }
        }
    }
}

/// Item event card (covers both Item and SponsorGift).
fn item_card(msg: &GameMessage) -> maud::Markup {
    let (label, color, content) = match &msg.payload {
        MessagePayload::ItemFound {
            tribute,
            item,
            area,
        } => (
            "Item",
            "text-yellow-400",
            format!("{} found {} in {}", tribute.name, item.name, area.name),
        ),
        MessagePayload::ItemUsed { tribute, item } => (
            "Item",
            "text-yellow-400",
            format!("{} used {}", tribute.name, item.name),
        ),
        MessagePayload::ItemDropped {
            tribute,
            item,
            area,
        } => (
            "Item",
            "text-yellow-400",
            format!("{} dropped {} in {}", tribute.name, item.name, area.name),
        ),
        MessagePayload::SponsorGift {
            recipient,
            item,
            donor,
        } => (
            "Sponsor",
            "text-amber-400",
            format!("{} sent {} to {}", donor, item.name, recipient.name),
        ),
        _ => return fallback_card(msg),
    };

    let icon_name = match &msg.payload {
        MessagePayload::SponsorGift { .. } => "gift",
        _ => "backpack",
    };

    html! {
        div class="bg-gray-900 border border-yellow-900/30 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon(icon_name))
                span class=(format!("{} font-medium", color)) { (label) }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" { (content) }
        }
    }
}

/// State event card (fallback for stamina, survival, cycle, phase, sleep, wake, etc.).
fn state_card(msg: &GameMessage) -> maud::Markup {
    use MessagePayload::*;
    let content = match &msg.payload {
        TributeRested {
            tribute,
            hp_restored,
        } => {
            format!("{} rested, restored {} HP", tribute.name, hp_restored)
        }
        TributeStarved { tribute, hp_lost } => {
            format!("{} starved, lost {} HP", tribute.name, hp_lost)
        }
        TributeDehydrated { tribute, hp_lost } => {
            format!("{} dehydrated, lost {} HP", tribute.name, hp_lost)
        }
        SanityBreak { tribute } => {
            format!("{} had a sanity break", tribute.name)
        }
        HungerBandChanged { tribute, from, to } => {
            format!(
                "{} hunger: {} → {}",
                tribute.name,
                from.as_str(),
                to.as_str()
            )
        }
        ThirstBandChanged { tribute, from, to } => {
            format!(
                "{} thirst: {} → {}",
                tribute.name,
                from.as_str(),
                to.as_str()
            )
        }
        StaminaBandChanged { tribute, from, to } => {
            format!(
                "{} stamina: {} → {}",
                tribute.name,
                from.as_str(),
                to.as_str()
            )
        }
        ShelterSought {
            tribute,
            area,
            success,
            roll,
        } => {
            let result = if *success {
                "successfully"
            } else {
                "failed to"
            };
            format!(
                "{} {} seek shelter in {} (roll: {})",
                tribute.name, result, area.name, roll
            )
        }
        Foraged {
            tribute,
            area,
            success,
            debt_recovered,
        } => {
            let result = if *success {
                format!("foraged {} food", debt_recovered)
            } else {
                "found nothing".to_string()
            };
            format!("{} {} in {}", tribute.name, result, area.name)
        }
        Drank {
            tribute,
            source,
            debt_recovered,
        } => {
            let source_desc = match source {
                shared::messages::DrinkSource::Terrain { area } => format!("from {}", area.name),
                shared::messages::DrinkSource::Item { item } => format!("used {}", item.name),
            };
            format!(
                "{} drank {} (recovered {})",
                tribute.name, source_desc, debt_recovered
            )
        }
        Ate {
            tribute,
            item,
            debt_recovered,
        } => {
            format!(
                "{} ate {} (recovered {})",
                tribute.name, item.name, debt_recovered
            )
        }
        CycleStart { day, phase } => {
            format!("Day {day} {phase} begins")
        }
        CycleEnd { day, phase } => {
            format!("Day {day} {phase} ends")
        }
        PhaseStarted {
            day,
            phase,
            weather_summary,
        } => {
            let weather = weather_summary
                .as_ref()
                .map(|w| format!(" — {w}"))
                .unwrap_or_default();
            format!("Phase {phase} of day {day} started{weather}")
        }
        PhaseEnded { day, phase } => {
            format!("Phase {phase} of day {day} ended")
        }
        TributeSlept {
            tribute,
            phase,
            restored_stamina,
            restored_hp,
        } => {
            format!(
                "{} slept through {} (stamina +{}, HP +{})",
                tribute.name, phase, restored_stamina, restored_hp
            )
        }
        TributeWoke {
            tribute,
            phase,
            reason,
        } => {
            let reason_text = match reason {
                shared::messages::WakeReason::Rested => "fully rested".to_string(),
                shared::messages::WakeReason::Interrupted { event } => {
                    format!("interrupted by {:?}", event)
                }
            };
            format!("{} woke during {} — {}", tribute.name, phase, reason_text)
        }
        GameEnded { winner } => match winner {
            Some(w) => format!("Game ended! {} wins!", w.name),
            None => "Game ended — no survivors".to_string(),
        },
        TributeWounded {
            victim,
            attacker,
            hp_lost,
        } => {
            let attacker_text = attacker
                .as_ref()
                .map(|a| format!(" by {}", a.name))
                .unwrap_or_default();
            format!("{}{} lost {} HP", victim.name, attacker_text, hp_lost)
        }
        TributeAttacked { victim, attacker } => {
            let attacker_text = attacker
                .as_ref()
                .map(|a| format!(" by {}", a.name))
                .unwrap_or_default();
            format!("{} attacked{}", victim.name, attacker_text)
        }
        AfflictionAcquired {
            tribute_id,
            affliction,
            severity,
        } => {
            format!(
                "Tribute {} acquired {} ({})",
                tribute_id, affliction, severity
            )
        }
        AfflictionProgressed {
            tribute_id,
            affliction,
            from_severity,
            to_severity,
        } => {
            format!(
                "Tribute {} affliction {} worsened: {} → {}",
                tribute_id, affliction, from_severity, to_severity
            )
        }
        AfflictionHealed {
            tribute_id,
            affliction,
        } => {
            format!("Tribute {} healed from {}", tribute_id, affliction)
        }
        AfflictionCascaded {
            tribute_id,
            from_affliction,
            to_affliction,
        } => {
            format!(
                "Tribute {} affliction cascaded: {} → {}",
                tribute_id, from_affliction, to_affliction
            )
        }
        _ => return fallback_card(msg),
    };

    html! {
        div class="bg-gray-900 border border-gray-800 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("activity"))
                span class="text-gray-400 font-medium" { "State" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" { (content) }
        }
    }
}

/// Trauma event card.
/// Trauma event card. Handles all 8 trauma message variants with
/// severity-colored borders and badges. Follows affliction/phobia_card pattern.
fn trauma_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessagePayload::*;
    let (severity, content) = match &msg.payload {
        TraumaAcquired {
            tribute,
            severity,
            source,
        } => (
            severity.as_str(),
            format!("{tribute} acquired trauma from {source}"),
        ),
        TraumaReinforced {
            tribute,
            from_severity,
            to_severity,
            floor_bumped: _,
        } => (
            to_severity.as_str(),
            format!("{tribute}'s trauma reinforced: {from_severity} \u{2192} {to_severity}"),
        ),
        TraumaEscalated {
            tribute,
            from_severity,
            to_severity,
        } => (
            to_severity.as_str(),
            format!("{tribute}'s trauma escalated: {from_severity} \u{2192} {to_severity}"),
        ),
        TraumaFlashback {
            tribute,
            severity,
            source,
        } => (severity.as_str(), format!("{tribute} relives {source}")),
        TraumaAvoidance {
            tribute,
            source,
            prevented_action: _,
        } => ("", format!("{tribute} avoids {source} due to trauma")),
        TraumaObserved {
            observer: _,
            subject,
            source,
        } => ("", format!("spotted {subject}'s trauma from {source}")),
        TraumaForgotten {
            observer: _,
            subject,
            source,
        } => ("", format!("forgot {subject}'s trauma from {source}")),
        TraumaHabituated {
            tribute,
            from_severity,
            to_severity: Some(to),
        } => (
            to.as_str(),
            format!("{tribute}'s trauma response weakens: {from_severity} \u{2192} {to}"),
        ),
        TraumaHabituated {
            tribute,
            from_severity,
            to_severity: None,
        } => (
            "",
            format!("{tribute} begins to heal from {from_severity} trauma"),
        ),
        _ => return fallback_card(msg),
    };

    let border_color = match severity {
        "severe" | "Severe" => "border-red-900/50",
        "moderate" | "Moderate" => "border-orange-900/50",
        _ if severity.is_empty() => "border-gray-800",
        _ => "border-yellow-900/50",
    };
    let badge_color = match severity {
        "severe" | "Severe" => "bg-red-500/20 text-red-300",
        "moderate" | "Moderate" => "bg-orange-500/20 text-orange-300",
        _ if severity.is_empty() => "",
        _ => "bg-yellow-500/20 text-yellow-300",
    };

    let badge = if !severity.is_empty() {
        html! {
            span class=(format!("text-xs px-1.5 py-0.5 rounded {}", badge_color)) { (severity) }
        }
    } else {
        html! {}
    };

    html! {
        div class=(format!("bg-gray-900 border rounded-lg p-3 {}", border_color)) {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("brain"))
                span class="text-purple-400 font-medium" { "Trauma" }
                span { "\u{b7}" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200 flex items-center gap-2" {
                (content)
                (badge)
            }
        }
    }
}

/// Affliction event card.
fn affliction_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessagePayload::*;
    let (severity, content) = match &msg.payload {
        AfflictionAcquired {
            tribute_id: _,
            affliction,
            severity,
        } => (
            severity.as_str(),
            format!("acquired {affliction} ({severity})"),
        ),
        AfflictionProgressed {
            tribute_id: _,
            affliction,
            from_severity,
            to_severity,
        } => (
            to_severity.as_str(),
            format!("{affliction} worsened: {from_severity} → {to_severity}"),
        ),
        AfflictionHealed {
            tribute_id: _,
            affliction,
        } => ("", format!("healed from {affliction}")),
        AfflictionCascaded {
            tribute_id: _,
            from_affliction,
            to_affliction,
        } => ("", format!("{from_affliction} cascaded to {to_affliction}")),
        _ => return fallback_card(msg),
    };

    let border_color = match severity {
        "severe" | "Severe" => "border-red-900/50",
        "moderate" | "Moderate" => "border-orange-900/50",
        _ => "border-yellow-900/50",
    };
    let badge_color = match severity {
        "severe" | "Severe" => "bg-red-500/20 text-red-300",
        "moderate" | "Moderate" => "bg-orange-500/20 text-orange-300",
        _ if severity.is_empty() => "",
        _ => "bg-yellow-500/20 text-yellow-300",
    };

    let badge = if !severity.is_empty() {
        html! {
            span class=(format!("text-xs px-1.5 py-0.5 rounded {}", badge_color)) { (severity) }
        }
    } else {
        html! {}
    };

    html! {
        div class=(format!("bg-gray-900 border rounded-lg p-3 {}", border_color)) {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("bandage"))
                span class="text-amber-400 font-medium" { "Health" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200 flex items-center gap-2" {
                (content)
                (badge)
            }
        }
    }
}

/// Phobia event card.
fn phobia_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessagePayload::*;
    use shared::messages::PhobiaEffect;
    let (severity, content) = match &msg.payload {
        PhobiaAcquired {
            tribute: _,
            trigger,
            severity,
            origin: _,
        } => (severity.as_str(), format!("acquired fear of {trigger}")),
        PhobiaTriggered {
            tribute: _,
            trigger,
            severity,
            effect,
        } => {
            let effect_text = match effect {
                PhobiaEffect::Penalty => " (penalty)",
                PhobiaEffect::Flee => " (fled)",
                PhobiaEffect::Freeze => " (frozen!)",
            };
            (severity.as_str(), format!("feared {trigger}{effect_text}"))
        }
        PhobiaEscalated {
            tribute: _,
            trigger,
            from_severity,
            to_severity,
        } => (
            to_severity.as_str(),
            format!("fear of {trigger} deepened: {from_severity} \u{2192} {to_severity}"),
        ),
        PhobiaHabituated {
            tribute: _,
            trigger,
            from_severity,
            to_severity,
        } => match to_severity {
            Some(to) => (
                to.as_str(),
                format!("fear of {trigger} faded: {from_severity} \u{2192} {to}"),
            ),
            None => ("", format!("overcame fear of {trigger}")),
        },
        PhobiaObserved {
            observer: _,
            subject,
            trigger,
        } => ("", format!("spotted {subject}'s fear of {trigger}")),
        PhobiaForgotten {
            observer: _,
            subject,
            trigger,
        } => ("", format!("forgot {subject}'s fear of {trigger}")),
        _ => return fallback_card(msg),
    };

    let border_color = match severity {
        "severe" | "Severe" => "border-red-900/50",
        "moderate" | "Moderate" => "border-orange-900/50",
        _ if severity.is_empty() => "border-gray-800",
        _ => "border-yellow-900/50",
    };
    let badge_color = match severity {
        "severe" | "Severe" => "bg-red-500/20 text-red-300",
        "moderate" | "Moderate" => "bg-orange-500/20 text-orange-300",
        _ if severity.is_empty() => "",
        _ => "bg-yellow-500/20 text-yellow-300",
    };

    let badge = if !severity.is_empty() {
        html! {
            span class=(format!("text-xs px-1.5 py-0.5 rounded {}", badge_color)) { (severity) }
        }
    } else {
        html! {}
    };

    html! {
        div class=(format!("bg-gray-900 border rounded-lg p-3 {}", border_color)) {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("eye"))
                span class="text-indigo-400 font-medium" { "Fear" }
                span { "\u{b7}" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200 flex items-center gap-2" {
                (content)
                (badge)
            }
        }
    }
}

/// Fallback card for unrecognized payloads.
fn fallback_card(msg: &GameMessage) -> maud::Markup {
    html! {
        div class="bg-gray-900 border border-gray-800 rounded-lg p-3" {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon("circle-help"))
                span class="text-gray-500 font-medium" { "Event" }
                span { "·" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-300" { (msg.content) }
        }
    }
}
