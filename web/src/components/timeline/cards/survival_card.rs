use dioxus::prelude::*;
use shared::messages::{GameMessage, HungerBand, MessagePayload, ThirstBand};

#[derive(Props, PartialEq, Clone)]
pub struct SurvivalCardProps {
    pub message: GameMessage,
}

#[component]
pub fn SurvivalCard(props: SurvivalCardProps) -> Element {
    match &props.message.payload {
        MessagePayload::HungerBandChanged { tribute, from, to } => {
            let cls = hunger_class(*to);
            rsx! {
                article { class: "rounded border-l-4 border-amber-400 bg-amber-50 theme2:bg-amber-950/40 p-2 text-sm",
                    p {
                        class: "{cls}",
                        "{tribute.name} is now "
                        strong { "{to}" }
                        span { class: "text-xs text-stone-500 ml-1", "(was {from})" }
                    }
                }
            }
        }
        MessagePayload::ThirstBandChanged { tribute, from, to } => {
            let cls = thirst_class(*to);
            rsx! {
                article { class: "rounded border-l-4 border-sky-400 bg-sky-50 theme2:bg-sky-950/40 p-2 text-sm",
                    p {
                        class: "{cls}",
                        "{tribute.name} is now "
                        strong { "{to}" }
                        span { class: "text-xs text-stone-500 ml-1", "(was {from})" }
                    }
                }
            }
        }
        MessagePayload::ShelterSought {
            tribute,
            area,
            success,
            roll: _,
        } => rsx! {
            article { class: "rounded border-l-4 border-emerald-400 bg-emerald-50 theme2:bg-emerald-950/40 p-2 text-sm",
                p {
                    class: "text-xs text-stone-600 theme2:text-stone-300",
                    if *success {
                        "🏠 {tribute.name} found shelter in {area.name}."
                    } else {
                        "🏠 {tribute.name} failed to find shelter in {area.name}."
                    }
                }
            }
        },
        MessagePayload::Foraged {
            tribute,
            area,
            success,
            debt_recovered,
        } => rsx! {
            article { class: "rounded border-l-4 border-lime-500 bg-lime-50 theme2:bg-lime-950/40 p-2 text-sm",
                p {
                    class: "text-xs text-stone-600 theme2:text-stone-300",
                    if *success {
                        "🌿 {tribute.name} foraged in {area.name} (+{debt_recovered} hunger relief)."
                    } else {
                        "🌿 {tribute.name} foraged in {area.name} but found nothing."
                    }
                }
            }
        },
        MessagePayload::Drank {
            tribute,
            source: _,
            debt_recovered,
        } => rsx! {
            article { class: "rounded border-l-4 border-sky-400 bg-sky-50 theme2:bg-sky-950/40 p-2 text-sm",
                p {
                    class: "text-xs text-stone-600 theme2:text-stone-300",
                    "💧 {tribute.name} drank (+{debt_recovered} thirst relief)."
                }
            }
        },
        MessagePayload::Ate {
            tribute,
            item: _,
            debt_recovered,
        } => rsx! {
            article { class: "rounded border-l-4 border-amber-400 bg-amber-50 theme2:bg-amber-950/40 p-2 text-sm",
                p {
                    class: "text-xs text-stone-600 theme2:text-stone-300",
                    "🍗 {tribute.name} ate (+{debt_recovered} hunger relief)."
                }
            }
        },
        _ => rsx! {},
    }
}

fn hunger_class(band: HungerBand) -> &'static str {
    match band {
        HungerBand::Hungry => "text-amber-600 theme2:text-amber-300",
        HungerBand::Starving => "text-red-600 theme2:text-red-400 font-semibold",
        _ => "text-stone-600 theme2:text-stone-300",
    }
}

fn thirst_class(band: ThirstBand) -> &'static str {
    match band {
        ThirstBand::Parched => "text-sky-600 theme2:text-sky-300",
        ThirstBand::Dehydrated => "text-red-600 theme2:text-red-400 font-semibold",
        _ => "text-stone-600 theme2:text-stone-300",
    }
}
