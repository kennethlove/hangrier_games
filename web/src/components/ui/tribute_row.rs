use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TributeRowProps {
    pub district_code: String,
    pub name: String,
    pub meta: String,
    pub stat: String,
    #[props(default = false)]
    pub highlight: bool,
}

#[component]
pub fn TributeRow(props: TributeRowProps) -> Element {
    let stat_color = if props.highlight {
        "text-primary"
    } else {
        "text-text"
    };
    rsx! {
        div {
            class: "flex items-center gap-3.5 py-2.5 border-t border-border first:border-t-0",
            div {
                class: "size-9 rounded-full bg-surface-2 flex items-center justify-center font-display text-sm shrink-0",
                "{props.district_code}"
            }
            div {
                div { class: "font-text font-semibold text-sm", "{props.name}" }
                div { class: "font-mono text-xs text-text-muted", "{props.meta}" }
            }
            div { class: "ml-auto font-mono font-bold text-sm {stat_color}", "{props.stat}" }
        }
    }
}
