use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EventCardProps {
    pub kicker: String,
    pub headline: String,
    #[props(default)]
    pub body: Option<String>,
    #[props(default)]
    pub actions: Option<Element>,
}

#[component]
pub fn EventCard(props: EventCardProps) -> Element {
    rsx! {
        article {
            class: "bg-surface border border-border rounded-card p-6 mb-4",
            div {
                class: "font-text font-bold text-[10px] uppercase tracking-[0.18em] text-primary mb-2",
                "{props.kicker}"
            }
            h2 {
                class: "font-display text-3xl uppercase leading-none mb-3",
                "{props.headline}"
            }
            if let Some(body) = props.body {
                p { class: "font-text text-sm leading-relaxed text-text/85", "{body}" }
            }
            if let Some(actions) = props.actions {
                div { class: "flex flex-wrap gap-2 mt-4", {actions} }
            }
        }
    }
}
