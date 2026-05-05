use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ScoreboardProps {
    pub home_label: String,
    pub home_meta: String,
    pub home_shield: String,
    pub away_label: String,
    pub away_meta: String,
    pub away_shield: String,
    pub score: String,
}

#[component]
pub fn Scoreboard(props: ScoreboardProps) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-[1fr_auto_1fr] items-center gap-6 px-10 py-8 \
                    bg-surface border-b border-border rounded-none",
            div {
                class: "flex items-center gap-3.5",
                Shield { code: props.home_shield.clone() }
                div {
                    div { class: "font-display text-xl tracking-wide", "{props.home_label}" }
                    div { class: "font-mono text-[10px] uppercase tracking-wider text-text-muted",
                        "{props.home_meta}"
                    }
                }
            }
            div {
                class: "font-mono font-bold text-6xl tracking-wider text-primary leading-none",
                "{props.score}"
            }
            div {
                class: "flex items-center justify-end gap-3.5 opacity-75",
                div {
                    div { class: "font-display text-xl tracking-wide text-right",
                        "{props.away_label}"
                    }
                    div { class: "font-mono text-[10px] uppercase tracking-wider text-text-muted text-right",
                        "{props.away_meta}"
                    }
                }
                Shield { code: props.away_shield.clone() }
            }
        }
    }
}

#[component]
fn Shield(code: String) -> Element {
    rsx! {
        div {
            class: "size-10 bg-surface-2 flex items-center justify-center font-display text-lg",
            "{code}"
        }
    }
}
