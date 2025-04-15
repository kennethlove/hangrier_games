use dioxus::prelude::*;

#[component]
pub fn WoundedIcon(class: String) -> Element {
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
                d: "M383.594 20.313c-28.797 0-57.576 10.982-79.53 32.937 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
