use dioxus::prelude::*;

#[component]
pub fn TopBar(brand: String, children: Element) -> Element {
    rsx! {
        header {
            class: "flex items-center justify-between px-6 py-3.5 \
                    bg-surface border-b-2 border-border",
            div {
                class: "flex items-center gap-6",
                div { class: "font-display text-xl tracking-wider", "{brand}" }
                {children}
            }
        }
    }
}
