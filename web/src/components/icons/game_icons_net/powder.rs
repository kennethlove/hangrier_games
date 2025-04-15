use dioxus::prelude::*;

#[component]
pub fn PowderIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M0 0h512v512H0z",
                fill: "#000",
                fill_opacity: "1",
            }
            path {
                d: "M260.28 71.406c-12.493.18-23.276 7.03-35.31 16.313 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
