use dioxus::prelude::*;

#[component]
pub fn VomitingIcon(class: String) -> Element {
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
                d: "M256.25 20.313c-108.64 0-196.78 90.592-196.78 202.937 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
