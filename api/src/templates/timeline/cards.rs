use maud::html;
use shared::messages::{GameMessage, MessagePayload};

use super::super::icon;

/// Death event card.
pub fn death_card(msg: &GameMessage) -> maud::Markup {
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

/// Combat event card. Also handles trap events (MessageKind::Combat).
pub fn combat_card(msg: &GameMessage) -> maud::Markup {
    // Handle trap variants
    if let MessagePayload::TrapSet { tribute, trap_kind } = &msg.payload {
        return html! {
            div class="bg-gray-900 border border-orange-900/50 rounded-lg p-3" {
                div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                    (icon("skull"))
                    span class="text-orange-400 font-medium" { "Trap" }
                    span { "·" }
                    span { "Day " (msg.game_day) " " (msg.phase) }
                }
                p class="text-sm text-gray-200" {
                    (tribute.name) " set a " (trap_kind) " trap"
                }
            }
        };
    }
    if let MessagePayload::TrapTriggered { victim, trap_kind } = &msg.payload {
        return html! {
            div class="bg-gray-900 border border-orange-900/50 rounded-lg p-3" {
                div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                    (icon("skull"))
                    span class="text-orange-400 font-medium" { "Trap" }
                    span { "·" }
                    span { "Day " (msg.game_day) " " (msg.phase) }
                }
                p class="text-sm text-gray-200" {
                    (victim.name) " triggered a " (trap_kind) " trap"
                }
            }
        };
    }

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
pub fn combat_swing_card(msg: &GameMessage) -> maud::Markup {
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
pub fn alliance_card(msg: &GameMessage) -> maud::Markup {
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
pub fn movement_card(msg: &GameMessage) -> maud::Markup {
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
pub fn item_card(msg: &GameMessage) -> maud::Markup {
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
pub fn state_card(msg: &GameMessage) -> maud::Markup {
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

/// Trauma event card. Handles all 8 trauma message variants with
/// severity-colored borders and badges. Follows affliction/phobia_card pattern.
pub fn trauma_card(msg: &GameMessage) -> maud::Markup {
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
            format!("{tribute}'s trauma reinforced: {from_severity} → {to_severity}"),
        ),
        TraumaEscalated {
            tribute,
            from_severity,
            to_severity,
        } => (
            to_severity.as_str(),
            format!("{tribute}'s trauma escalated: {from_severity} → {to_severity}"),
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
            format!("{tribute}'s trauma response weakens: {from_severity} → {to}"),
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
pub fn affliction_card(msg: &GameMessage) -> maud::Markup {
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
        SubstanceUsed {
            tribute,
            item,
            substance,
        } => ("", format!("{tribute} used {item} ({substance})")),
        AddictionAcquired {
            tribute,
            substance,
            severity,
            use_count,
        } => (
            severity.as_str(),
            format!("{tribute} acquired {substance} addiction (use #{use_count})"),
        ),
        AddictionReinforced {
            tribute,
            substance,
            severity,
        } => (
            severity.as_str(),
            format!("{tribute}'s {substance} addiction reinforced (now {severity})"),
        ),
        AddictionEscalated {
            tribute,
            substance,
            from_severity,
            to_severity,
        } => (
            to_severity.as_str(),
            format!("{tribute}'s {substance} addiction escalated: {from_severity} → {to_severity}"),
        ),
        AddictionResisted {
            tribute,
            substance,
            reason,
        } => (
            "",
            format!("{tribute} resisted {substance} addiction ({reason})"),
        ),
        AddictionRelapse {
            tribute,
            substance,
            prior_uses,
        } => (
            "",
            format!("{tribute} relapsed into {substance} addiction (prior uses: {prior_uses})"),
        ),
        AddictionCraving {
            tribute,
            substance,
            severity,
        } => (
            severity.as_str(),
            format!("{tribute} craves {substance} ({severity})"),
        ),
        AddictionObserved {
            observer,
            subject,
            substance,
        } => (
            "",
            format!("{observer} noticed {subject}'s {substance} craving"),
        ),
        AddictionForgotten {
            observer,
            subject,
            substance,
        } => (
            "",
            format!("{observer} forgot {subject}'s {substance} addiction"),
        ),
        AddictionHabituated {
            tribute,
            substance: _,
            from_severity,
            to_severity: Some(to),
        } => (
            to.as_str(),
            format!("{tribute}'s addiction weakens: {from_severity} → {to}"),
        ),
        AddictionHabituated {
            tribute,
            substance: _,
            from_severity,
            to_severity: None,
        } => ("", format!("{tribute} overcomes {from_severity} addiction")),
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
pub fn phobia_card(msg: &GameMessage) -> maud::Markup {
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
            format!("fear of {trigger} deepened: {from_severity} → {to_severity}"),
        ),
        PhobiaHabituated {
            tribute: _,
            trigger,
            from_severity,
            to_severity,
        } => match to_severity {
            Some(to) => (
                to.as_str(),
                format!("fear of {trigger} faded: {from_severity} → {to}"),
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

/// Fixation event card. Handles all 6 fixation variants with severity-colored
/// borders and badges. Uses tier icon (eye) with "Fixation" label.
pub fn fixation_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessagePayload::*;
    let (severity, content) = match &msg.payload {
        FixationAcquired {
            tribute_id: _,
            target,
            severity,
            origin: _,
        } => (severity.as_str(), format!("acquired fixation on {target}")),
        FixationEscalated {
            tribute_id: _,
            target,
            old_severity,
            new_severity,
        } => (
            new_severity.as_str(),
            format!("fixation on {target} deepened: {old_severity} \u{2192} {new_severity}"),
        ),
        FixationFired {
            tribute_id: _,
            target,
            severity,
            action,
        } => (
            severity.as_str(),
            format!("fixation on {target} fired \u{2014} overriding toward {action}"),
        ),
        FixationConsummated {
            tribute_id: _,
            target,
        } => ("", format!("fixation on {target} consummated")),
        FixationThwarted {
            tribute_id: _,
            target,
            reason,
        } => (
            "",
            format!("fixation on {target} thwarted \u{2014} {reason}"),
        ),
        FixationFaded {
            tribute_id: _,
            target,
        } => ("", format!("fixation on {target} faded")),
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
                span class="text-rose-400 font-medium" { "Fixation" }
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

/// Trapped event card. Handles 4 trapped message variants.
pub fn trapped_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessagePayload::*;
    let (content, icon_name, label, label_color) = match &msg.payload {
        TributeTrapped {
            tribute,
            kind,
            severity,
        } => (
            format!("{tribute} trapped by {kind} ({severity})"),
            "alert-triangle",
            "Trapped",
            "text-orange-400",
        ),
        Struggling {
            tribute,
            kind,
            severity,
            cycles_trapped,
        } => (
            format!("{tribute} struggling in {kind} ({severity}, cycle {cycles_trapped})"),
            "alert-triangle",
            "Trapped",
            "text-orange-400",
        ),
        TrappedEscaped {
            tribute,
            kind,
            cycles_trapped,
            rescued_by,
        } => {
            let rescuer_text = if rescued_by.is_empty() {
                String::new()
            } else {
                format!(" (rescued by {})", rescued_by.join(", "))
            };
            (
                format!("{tribute} escaped {kind} after {cycles_trapped} cycles{rescuer_text}"),
                "check-circle",
                "Freed",
                "text-green-400",
            )
        }
        TributeDiedWhileTrapped { tribute, kind } => (
            format!("{tribute} died while trapped in {kind}"),
            "skull",
            "Death",
            "text-red-400",
        ),
        _ => return super::super::timeline::cards::fallback_card(msg),
    };

    let border_color = match &msg.payload {
        TributeDiedWhileTrapped { .. } => "border-red-900/50",
        TrappedEscaped { .. } => "border-green-900/50",
        _ => "border-orange-900/50",
    };

    html! {
        div class=(format!("bg-gray-900 border rounded-lg p-3 {}", border_color)) {
            div class="flex items-center gap-2 text-xs text-gray-500 mb-1" {
                (icon(icon_name))
                span class=(format!("{} font-medium", label_color)) { (label) }
                span { "\u{b7}" }
                span { "Day " (msg.game_day) " " (msg.phase) }
            }
            p class="text-sm text-gray-200" { (content) }
        }
    }
}

/// Fallback card for unrecognized payloads.
pub fn fallback_card(msg: &GameMessage) -> maud::Markup {
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
