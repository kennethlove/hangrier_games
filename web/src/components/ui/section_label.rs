use dioxus::prelude::*;

#[component]
pub fn SectionLabel(children: Element) -> Element {
    rsx! {
        div {
            class: "font-text font-bold text-[10px] uppercase tracking-[0.18em] text-text-muted mb-3",
            {children}
        }
    }
}
