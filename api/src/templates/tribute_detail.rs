use maud::html;
use shared::afflictions::{AfflictionKind, FixationTarget};

use super::{AuthState, base_layout, icon};

/// Single tribute detail page showing stats, inventory, afflictions, and fixations.
pub fn tribute_detail_page(
    auth: AuthState,
    game_id: &str,
    tribute: &game::tributes::Tribute,
) -> maud::Markup {
    let is_alive = tribute.is_alive();
    let status_class = if is_alive { "alive" } else { "dead" };
    let status_icon = if is_alive { "check-circle" } else { "x-circle" };

    // Collect fixations
    let fixations: Vec<_> = tribute
        .afflictions
        .values()
        .filter(|a| matches!(a.kind, AfflictionKind::Fixation(_)))
        .collect();

    base_layout(
        &tribute.name,
        auth,
        html! {
            div class="container" style="padding-block:var(--gap-lg);" {
                a href=(format!("/games/{}", game_id))
                    class="back-link" {
                    (icon("arrow-left"))
                    " Back to Game"
                }

                // Tribute header
                div class="detail-header" {
                    div {
                        h1 { (tribute.name) }
                        span class=(format!("card-status {}", status_class)) { (icon(status_icon)) }
                        " District " (tribute.district)
                    }
                }

                // Stats strip
                div class="card-stats" {
                    div { (icon("heart")) " HP " span class="stat-val" { (tribute.attributes.health) } }
                    div { (icon("brain")) " SAN " span class="stat-val" { (tribute.attributes.sanity) } }
                    div { (icon("zap")) " MOV " span class="stat-val" { (tribute.attributes.movement) } }
                    div { (icon("sword")) " STR " span class="stat-val" { (tribute.attributes.strength) } }
                }

                // Survival bands
                div class="card-bands" {
                    span { "Hunger: " (tribute.hunger) }
                    span { "Thirst: " (tribute.thirst) }
                    span { "Stamina: " (tribute.stamina) "/" (tribute.max_stamina) }
                }

                // Area location
                div class="card-location" {
                    (icon("map-pin"))
                    " " (tribute.area)
                }

                // Inventory
                @if !tribute.items.is_empty() {
                    div class="card-items" {
                        h3 { (icon("backpack")) " Items" }
                        @for item in &tribute.items {
                            span class="item-tag" { (item.name) }
                        }
                    }
                }

                // Afflictions (non-fixation)
                @let non_fixation_afflictions: Vec<_> = tribute.afflictions.values()
                    .filter(|a| !matches!(a.kind, AfflictionKind::Fixation(_)))
                    .collect();
                @if !non_fixation_afflictions.is_empty() {
                    div class="card-afflictions" {
                        h3 { (icon("bandage")) " Afflictions" }
                        @for affliction in &non_fixation_afflictions {
                            @let severity_class = match affliction.severity.to_string().as_str() {
                                "severe" => "severity-severe",
                                "moderate" => "severity-moderate",
                                _ => "severity-mild",
                            };
                            @let body_part = affliction.body_part.map(|bp| format!(" ({bp})")).unwrap_or_default();
                            span class=(format!("affliction-badge {}", severity_class)) {
                                (icon("bandage"))
                                " " (affliction.kind.to_string()) (body_part)
                            }
                        }
                    }
                }

                // Fixations section
                @if !fixations.is_empty() {
                    div class="fixations-section" {
                        h3 { (icon("eye")) " Fixations" }
                        p class="text-sm text-gray-500" { "Always visible to you. AI severity gating applies only to other tributes." }

                        @for affliction in &fixations {
                            @let target = match &affliction.kind {
                                AfflictionKind::Fixation(t) => t,
                                _ => unreachable!(),
                            };
                            @let severity = affliction.severity.to_string();
                            @let severity_class = match severity.as_str() {
                                "severe" => "severity-severe",
                                "moderate" => "severity-moderate",
                                _ => "severity-mild",
                            };
                            @let meta = &affliction.fixation_metadata;

                            div class=(format!("fixation-card {}", severity_class)) {
                                div class="fixation-header" {
                                    span class="fixation-target" {
                                        (icon("crosshair"))
                                        " " (format_fixation_target(target))
                                    }
                                    span class=(format!("affliction-badge {}", severity_class)) { (severity) }
                                }

                                // Origin
                                @if let Some(m) = meta {
                                    div class="fixation-origin" {
                                        @match &m.origin {
                                            shared::afflictions::FixationOrigin::Innate => {
                                                (icon("dna"))
                                                " Innate"
                                            }
                                            shared::afflictions::FixationOrigin::Acquired { event_ref } => {
                                                (icon("zap"))
                                                " Acquired via " (event_ref)
                                            }
                                        }
                                    }

                                    // Contact info
                                    div class="fixation-contact" {
                                        (icon("clock"))
                                        " No contact for " (m.cycles_since_last_contact) " cycle(s)"
                                    }

                                    // Observers
                                    @if !m.observed_by.is_empty() {
                                        div class="fixation-observers" {
                                            (icon("eye"))
                                            " Observed by " (m.observed_by.len()) " tribute(s)"
                                        }
                                    } @else {
                                        div class="fixation-observers empty" {
                                            (icon("eye-off"))
                                            " Not yet observed by anyone"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } @else {
                    div class="fixations-section empty" {
                        h3 { (icon("eye")) " Fixations" }
                        p class="empty-state" { "No fixations." }
                    }
                }
            }
        },
    )
}

/// Format a FixationTarget for display — extract the inner value without prefix.
fn format_fixation_target(target: &FixationTarget) -> String {
    match target {
        FixationTarget::Tribute(id) => format!("Tribute: {id}"),
        FixationTarget::Item(id) => format!("Item: {id}"),
        FixationTarget::Area(name) => format!("Area: {name}"),
    }
}
