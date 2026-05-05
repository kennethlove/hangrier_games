use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, Phase};

#[derive(Props, PartialEq, Clone)]
pub struct SleepCardProps {
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

#[component]
pub fn SleepCard(props: SleepCardProps) -> Element {
    let MessagePayload::TributeSlept {
        tribute,
        phase,
        restored_stamina,
        restored_hp,
    } = props.message.payload.clone()
    else {
        return rsx! {};
    };
    let icon = phase_icon(phase);
    let restoration = if restored_stamina == 0 && restored_hp == 0 {
        String::new()
    } else {
        let mut parts = Vec::new();
        if restored_stamina > 0 {
            parts.push(format!("+{restored_stamina} stamina"));
        }
        if restored_hp > 0 {
            parts.push(format!("+{restored_hp} HP"));
        }
        format!(" ({})", parts.join(", "))
    };
    rsx! {
        article { class: "rounded border-l-4 border-slate-400 bg-slate-50 p-2 text-sm",
            "💤 {icon} {tribute.name} sleeps{restoration}"
        }
    }
}
