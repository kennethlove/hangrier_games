//! Renders the typed `CombatBeat` payload (`MessagePayload::CombatSwing`).
//!
//! Sits alongside `CombatCard` (which still renders the legacy
//! `CombatEngagement.detail_lines`). Once consumers have migrated off
//! `detail_lines`, this card becomes the sole combat renderer.

use crate::routes::Routes;
use dioxus::prelude::*;
use shared::combat_beat::{CombatBeat, SwingOutcome, WearOutcomeReport};

#[derive(Props, PartialEq, Clone)]
pub struct CombatSwingCardProps {
    pub game_identifier: String,
    pub beat: CombatBeat,
}

/// One-line summary of the swing outcome, used as the card header.
fn outcome_summary(outcome: &SwingOutcome) -> (&'static str, String) {
    match outcome {
        SwingOutcome::Miss => ("🥊", "missed".into()),
        SwingOutcome::Wound { damage } => ("🤕", format!("wounded for {damage}")),
        SwingOutcome::CriticalHitWound { damage } => ("💥", format!("crit wounded for {damage}")),
        SwingOutcome::BlockWound { damage } => ("🛡️", format!("blocked + countered for {damage}")),
        SwingOutcome::Kill { damage } => ("💀", format!("killed (dealt {damage})")),
        SwingOutcome::AttackerDied { damage } => ("☠️", format!("died countered ({damage})")),
        SwingOutcome::FumbleSurvive { self_damage } => {
            ("🤦", format!("fumbled (-{self_damage} self)"))
        }
        SwingOutcome::FumbleDeath { self_damage } => {
            ("💀", format!("fatal fumble (-{self_damage})"))
        }
        SwingOutcome::SelfAttackWound { damage } => ("🩸", format!("self-attack ({damage})")),
        SwingOutcome::Suicide { damage } => ("🪦", format!("suicide ({damage})")),
    }
}

#[component]
pub fn CombatSwingCard(props: CombatSwingCardProps) -> Element {
    let beat = &props.beat;
    let (icon, verb) = outcome_summary(&beat.outcome);
    let weapon = beat.weapon.as_ref();
    let shield = beat.shield.as_ref();
    let has_wear = !beat.wear.is_empty();
    let has_stress = beat.stress.stress_damage > 0;

    rsx! {
        article { class: "rounded border-l-4 border-amber-500 bg-amber-50 theme2:bg-amber-950 p-3",
            header { class: "font-semibold flex flex-wrap items-center gap-1",
                "{icon} "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: beat.attacker.identifier.clone(),
                    },
                    class: "underline",
                    "{beat.attacker.name}"
                }
                " {verb} "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: beat.target.identifier.clone(),
                    },
                    class: "underline",
                    "{beat.target.name}"
                }
            }

            // Equipment line.
            if weapon.is_some() || shield.is_some() {
                div { class: "mt-1 text-xs opacity-80",
                    if let Some(w) = weapon {
                        span { class: "mr-3", "🗡️ {w.name}" }
                    }
                    if let Some(s) = shield {
                        span { "🛡️ {s.name}" }
                    }
                }
            }

            // Wear / break badges.
            if has_wear {
                ul { class: "mt-2 list-disc pl-5 text-sm",
                    for w in beat.wear.iter() {
                        li { key: "{w.owner.identifier}-{w.item.identifier}",
                            match w.outcome {
                                WearOutcomeReport::Worn => rsx! {
                                    "{w.owner.name}'s "
                                    span { class: "font-medium", "{w.item.name}" }
                                    " wore down"
                                },
                                WearOutcomeReport::Broken => rsx! {
                                    "{w.owner.name}'s "
                                    span { class: "font-medium", "{w.item.name}" }
                                    " broke"
                                    if let Some(p) = w.mid_action_penalty {
                                        " (-{p})"
                                    }
                                },
                                WearOutcomeReport::Pristine => rsx! {},
                            }
                        }
                    }
                }
            }

            // Stress badge.
            if has_stress {
                if let Some(stressed) = beat.stress.stressed.as_ref() {
                    div { class: "mt-1 text-xs italic opacity-80",
                        "😱 {stressed.name} loses {beat.stress.stress_damage} sanity"
                    }
                } else {
                    div { class: "mt-1 text-xs italic opacity-80",
                        "😱 -{beat.stress.stress_damage} sanity"
                    }
                }
            }
        }
    }
}
