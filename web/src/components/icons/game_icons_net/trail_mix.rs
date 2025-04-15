use dioxus::prelude::*;

#[component]
pub fn TrailMixIcon(class: String) -> Element {
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
                d: "M132.684 31.388a1.443 1.443 0 0 0-.29.004c-.396.048-.768.25-1.398.609 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
