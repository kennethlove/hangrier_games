//! Timeline card for stamina band-change events
//! (`MessagePayload::StaminaBandChanged`).
//! See spec `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`.

use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, StaminaBand};

#[derive(Props, PartialEq, Clone)]
pub struct StaminaCardProps {
    pub message: GameMessage,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Worsening,
    Recovery,
    Unknown,
}

fn transition_direction(from: StaminaBand, to: StaminaBand) -> Direction {
    use StaminaBand::*;
    match (from, to) {
        (Fresh, Winded) | (Fresh, Exhausted) | (Winded, Exhausted) => Direction::Worsening,
        (Winded, Fresh) | (Exhausted, Fresh) | (Exhausted, Winded) => Direction::Recovery,
        _ => Direction::Unknown,
    }
}

#[component]
pub fn StaminaCard(props: StaminaCardProps) -> Element {
    let MessagePayload::StaminaBandChanged { tribute, from, to } = &props.message.payload else {
        return rsx! {};
    };

    let direction = transition_direction(*from, *to);
    let (border_cls, bg_cls, glyph, phrase) = match (direction, to.as_str()) {
        (Direction::Worsening, "Winded") => (
            "border-amber-400",
            "bg-amber-50 ",
            "💨",
            format!("{} is winded.", tribute.name),
        ),
        (Direction::Worsening, "Exhausted") => (
            "border-red-500",
            "bg-red-50 ",
            "🥵",
            format!("{} is exhausted.", tribute.name),
        ),
        (Direction::Recovery, _) => (
            "border-emerald-400",
            "bg-emerald-50 ",
            "🌿",
            format!("{} caught their breath.", tribute.name),
        ),
        _ => (
            "border-stone-400",
            "bg-stone-50 ",
            "•",
            format!("{}: {} → {}", tribute.name, from, to),
        ),
    };

    rsx! {
        article {
            class: "rounded border-l-4 {border_cls} {bg_cls} p-2 text-sm",
            p {
                class: "text-xs text-stone-700 ",
                "{glyph} {phrase}"
                span { class: "text-[10px] text-stone-500 ml-2", "(was {from})" }
            }
        }
    }
}
