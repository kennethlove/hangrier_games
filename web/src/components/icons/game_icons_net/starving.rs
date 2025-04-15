use dioxus::prelude::*;

#[component]
pub fn StarvingIcon(class: String) -> Element {
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
                d: "M153.063 21.74a19.46 28.32 83.178 0 1-23.98 13.947 19.46 28.32 83.178 0 1-27.68-9.18 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
