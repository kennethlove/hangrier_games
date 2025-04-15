use dioxus::prelude::*;

#[component]
pub fn SwitchbladeIcon(class: String) -> Element {
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
                d: "M226.652 235.381l21.57-19.723c-21.518-19.505-39.248-5.543-42.497-.644 ...existing path data...",
                fill: "#fff",
                fill_opacity: "1",
            }
        }
    }
}
