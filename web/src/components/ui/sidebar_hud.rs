use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct StatTileProps {
    pub label: String,
    pub value: String,
    #[props(default = "text-text".to_string())]
    pub value_class: String,
}

#[component]
pub fn StatTile(props: StatTileProps) -> Element {
    rsx! {
        div {
            class: "p-4 border-r border-b border-border last:border-r-0",
            div { class: "font-text font-bold text-[9px] uppercase tracking-[0.16em] text-text-muted mb-1.5",
                "{props.label}"
            }
            div { class: "font-mono font-bold text-2xl tracking-wider {props.value_class}",
                "{props.value}"
            }
        }
    }
}

#[component]
pub fn SidebarHud(header: String, children: Element) -> Element {
    rsx! {
        div {
            class: "bg-surface-2 border border-border rounded-none",
            div {
                class: "px-4 py-3 border-b border-border font-display text-base tracking-wider",
                "{header}"
            }
            div { class: "grid grid-cols-2", {children} }
        }
    }
}
