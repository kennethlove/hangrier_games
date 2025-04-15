use dioxus::prelude::*;

#[component]
pub fn ElectrocutedIcon(class: String) -> Element {
    rsx! {
        svg {
            view_box: "0 0 512 512",
            class,
            path {
                d: "M256 0l-64 192h96l-64 192 192-256h-96l64-128z",
            }
        }
    }
}
