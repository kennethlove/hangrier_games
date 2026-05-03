use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::TributeRef;

#[derive(Props, PartialEq, Clone)]
pub struct DeathCardProps {
    pub game_identifier: String,
    pub victim: TributeRef,
    pub killer: Option<TributeRef>,
    pub cause: String,
}

#[component]
pub fn DeathCard(props: DeathCardProps) -> Element {
    let victim_route = Routes::TributeDetail {
        game_identifier: props.game_identifier.clone(),
        tribute_identifier: props.victim.identifier.clone(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-red-500 bg-red-50 theme2:bg-red-950 p-3",
            header { class: "font-semibold",
                "💀 "
                Link { to: victim_route, class: "underline", "{props.victim.name}" }
                " killed"
            }
            if let Some(k) = props.killer.as_ref() {
                p { class: "text-sm",
                    "by "
                    Link {
                        to: Routes::TributeDetail {
                            game_identifier: props.game_identifier.clone(),
                            tribute_identifier: k.identifier.clone(),
                        },
                        class: "underline",
                        "{k.name}"
                    }
                }
            }
            p { class: "text-xs {cause_class(&props.cause)}", "{props.cause}" }
        }
    }
}

fn cause_class(cause: &str) -> &'static str {
    match cause {
        "starvation" | "dehydration" => "text-amber-600 theme2:text-amber-300 italic",
        _ => "text-gray-600",
    }
}
