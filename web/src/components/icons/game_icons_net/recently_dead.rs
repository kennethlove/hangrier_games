use dioxus::prelude::*;

#[component]
pub fn RecentlyDeadIcon(class: String) -> Element {
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
                d: "M266.3 30.62V397.5c20.1-1.1 37.7-5.2 51.3-11.8 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
