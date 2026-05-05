use dioxus::prelude::*;
use shared::messages::{
    AreaEventKind, GameMessage, InterruptionKind, MessagePayload, Phase, WakeReason,
};

#[derive(Props, PartialEq, Clone)]
pub struct WakeCardProps {
    pub message: GameMessage,
}

fn phase_icon(phase: Phase) -> &'static str {
    match phase {
        Phase::Dawn => "🌄",
        Phase::Day => "☀️",
        Phase::Dusk => "🌆",
        Phase::Night => "🌙",
    }
}

fn area_event_label(kind: AreaEventKind) -> &'static str {
    match kind {
        AreaEventKind::Hazard => "a hazard",
        AreaEventKind::Storm => "a storm",
        AreaEventKind::Mutts => "mutts",
        AreaEventKind::Earthquake => "an earthquake",
        AreaEventKind::Flood => "a flood",
        AreaEventKind::Fire => "a fire",
        AreaEventKind::Other => "an area event",
    }
}

#[component]
pub fn WakeCard(props: WakeCardProps) -> Element {
    let MessagePayload::TributeWoke {
        tribute,
        phase,
        reason,
    } = props.message.payload.clone()
    else {
        return rsx! {};
    };
    let icon = phase_icon(phase);
    let (glyph, accent, body) = match reason {
        WakeReason::Rested => (
            "🌅",
            "border-emerald-400 bg-emerald-50",
            format!("{} wakes rested", tribute.name),
        ),
        WakeReason::Interrupted { event } => {
            let detail = match event {
                InterruptionKind::Ambush { attacker } => {
                    format!("ambushed by {}", attacker.name)
                }
                InterruptionKind::AreaEvent { kind } => {
                    format!("woken by {}", area_event_label(kind))
                }
                InterruptionKind::AllianceSummons { ally } => {
                    format!("summoned by {}", ally.name)
                }
            };
            (
                "⚠️",
                "border-rose-500 bg-rose-50",
                format!("{} {detail}", tribute.name),
            )
        }
    };
    rsx! {
        article { class: "rounded border-l-4 {accent} p-2 text-sm",
            "{glyph} {icon} {body}"
        }
    }
}
