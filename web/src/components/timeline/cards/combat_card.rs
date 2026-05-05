use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::{CombatOutcome, TributeRef};

#[derive(Props, PartialEq, Clone)]
pub struct CombatCardProps {
    pub game_identifier: String,
    pub attacker: TributeRef,
    pub target: TributeRef,
    pub outcome: CombatOutcome,
    pub detail_lines: Vec<String>,
}

#[component]
pub fn CombatCard(props: CombatCardProps) -> Element {
    let mut expanded = use_signal(|| false);
    let outcome_label = match props.outcome {
        CombatOutcome::Killed => "killed",
        CombatOutcome::Wounded => "wounded",
        CombatOutcome::TargetFled => "drove off",
        CombatOutcome::AttackerFled => "fled from",
        CombatOutcome::Stalemate => "fought to a stalemate with",
    };
    let has_details = !props.detail_lines.is_empty();
    rsx! {
        article { class: "rounded border-l-4 border-orange-500 bg-orange-50  p-3",
            header { class: "font-semibold",
                "⚔️ "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: props.attacker.identifier.clone(),
                    },
                    class: "underline",
                    "{props.attacker.name}"
                }
                " {outcome_label} "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: props.target.identifier.clone(),
                    },
                    class: "underline",
                    "{props.target.name}"
                }
            }
            if has_details {
                button {
                    class: "mt-1 text-xs underline",
                    onclick: move |_| expanded.set(!expanded()),
                    if expanded() { "hide details" } else { "show details" }
                }
                if expanded() {
                    ul { class: "mt-2 list-disc pl-5 text-sm",
                        for line in props.detail_lines.iter() { li { "{line}" } }
                    }
                }
            }
        }
    }
}
