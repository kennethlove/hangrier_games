use dioxus::prelude::*;
use game::tributes::Tribute;
use game::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band};

#[component]
pub fn TributeSurvivalSection(tribute: Tribute, current_phase: Option<u32>) -> Element {
    let h_band = hunger_band(tribute.hunger);
    let t_band = thirst_band(tribute.thirst);
    let sheltered_phases_left = match (tribute.sheltered_until, current_phase) {
        (Some(until), Some(now)) if until > now => Some(until - now),
        _ => None,
    };

    let starvation_drain_line: Option<String> = if h_band == HungerBand::Starving {
        Some(format!(
            "Starving — losing {} HP/phase (next phase: {})",
            tribute.starvation_drain_step,
            tribute.starvation_drain_step.saturating_add(1),
        ))
    } else {
        None
    };

    let dehydration_drain_line: Option<String> = if t_band == ThirstBand::Dehydrated {
        Some(format!(
            "Dehydrated — losing {} HP/phase (next phase: {})",
            tribute.dehydration_drain_step,
            tribute.dehydration_drain_step.saturating_add(1),
        ))
    } else {
        None
    };

    let h_label = format!("{:?}", h_band);
    let t_label = format!("{:?}", t_band);

    rsx! {
        section {
            class: "rounded-lg border border-stone-700/40 bg-stone-900/30 p-4 mt-4",
            h3 {
                class: "text-lg font-semibold mb-2 text-stone-100",
                "Survival"
            }
            dl {
                class: "grid grid-cols-2 gap-y-1 text-sm",
                dt { class: "text-stone-400", "Hunger" }
                dd { class: "text-stone-100", "{tribute.hunger} ({h_label})" }
                dt { class: "text-stone-400", "Thirst" }
                dd { class: "text-stone-100", "{tribute.thirst} ({t_label})" }
                dt { class: "text-stone-400", "Shelter" }
                dd {
                    class: "text-stone-100",
                    if let Some(left) = sheltered_phases_left {
                        "Sheltered for {left} more phases"
                    } else {
                        "Exposed"
                    }
                }
            }
            if let Some(line) = starvation_drain_line {
                p { class: "mt-2 text-sm text-red-400", "{line}" }
            }
            if let Some(line) = dehydration_drain_line {
                p { class: "mt-1 text-sm text-red-400", "{line}" }
            }
        }
    }
}
