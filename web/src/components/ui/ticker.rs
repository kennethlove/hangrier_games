use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TickerItem {
    pub kind: String,
    pub message: String,
}

#[component]
pub fn Ticker(items: Vec<TickerItem>) -> Element {
    rsx! {
        div {
            class: "px-6 py-3.5 bg-surface border-t-2 border-border font-mono text-xs text-text/70 \
                    flex gap-8 overflow-x-auto",
            for item in items.iter() {
                span {
                    span { class: "text-primary font-bold mr-2", "{item.kind}" }
                    "{item.message}"
                }
            }
        }
    }
}
