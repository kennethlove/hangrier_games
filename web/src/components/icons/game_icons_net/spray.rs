use dioxus::prelude::*;

#[component]
pub fn SprayIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M265.174 45.484c-.776-.007-1.267.05-1.65.112l-27.59 47.79c.304.79 1.13 2.36 2.693 4.268 ...existing path data...",
            }
        }
    }
}
