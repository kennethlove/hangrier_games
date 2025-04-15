use dioxus::prelude::*;

#[component]
pub fn MauledIcon(class: String) -> Element {
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
                d: "M17.693 20.982v168.63c47.284 70.756 12.15 122.507 42.633 199.302 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
