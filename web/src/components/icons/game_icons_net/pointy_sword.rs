use dioxus::prelude::*;

#[component]
pub fn PointySwordIcon(class: String) -> Element {
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
                d: "M45.95 14.553c-19.38.81-30.594 11.357-30.282 30.283l19.768 30.78 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
