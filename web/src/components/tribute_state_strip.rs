use dioxus::prelude::*;
use game::tributes::Tribute;
use game::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band};

#[component]
pub fn TributeStateStrip(tribute: Tribute, current_phase: Option<u32>) -> Element {
    let h_band = hunger_band(tribute.hunger);
    let t_band = thirst_band(tribute.thirst);
    let sheltered_phases_left = match (tribute.sheltered_until, current_phase) {
        (Some(until), Some(now)) if until > now => Some(until - now),
        _ => None,
    };

    let any_visible = h_band != HungerBand::Sated
        || t_band != ThirstBand::Sated
        || sheltered_phases_left.is_some();

    if !any_visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex flex-row gap-2 items-center text-sm select-none",
            if h_band != HungerBand::Sated {
                HungerPip { band: h_band, raw: tribute.hunger }
            }
            if t_band != ThirstBand::Sated {
                ThirstPip { band: t_band, raw: tribute.thirst }
            }
            if let Some(left) = sheltered_phases_left {
                ShelterPip { phases_left: left }
            }
        }
    }
}

#[component]
fn HungerPip(band: HungerBand, raw: u8) -> Element {
    let (cls, label) = match band {
        HungerBand::Peckish => ("text-amber-300/60", "Peckish"),
        HungerBand::Hungry => ("text-amber-400", "Hungry"),
        HungerBand::Starving => ("text-red-500 animate-pulse", "Starving"),
        HungerBand::Sated => return rsx! {},
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-1 {cls}",
            "aria-label": "Hunger: {label}",
            title: "Hunger {raw} — {label}",
            span { class: "text-base", "🍗" }
            span { class: "text-xs uppercase tracking-wide", "{label}" }
        }
    }
}

#[component]
fn ThirstPip(band: ThirstBand, raw: u8) -> Element {
    let (cls, label) = match band {
        ThirstBand::Thirsty => ("text-sky-300/60", "Thirsty"),
        ThirstBand::Parched => ("text-sky-400", "Parched"),
        ThirstBand::Dehydrated => ("text-red-500 animate-pulse", "Dehydrated"),
        ThirstBand::Sated => return rsx! {},
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-1 {cls}",
            "aria-label": "Thirst: {label}",
            title: "Thirst {raw} — {label}",
            span { class: "text-base", "💧" }
            span { class: "text-xs uppercase tracking-wide", "{label}" }
        }
    }
}

#[component]
fn ShelterPip(phases_left: u32) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center gap-1 text-emerald-300",
            "aria-label": "Sheltered for {phases_left} more phases",
            title: "Sheltered for {phases_left} more phases",
            span { class: "text-base", "🏠" }
            span { class: "text-xs", "{phases_left}" }
        }
    }
}
